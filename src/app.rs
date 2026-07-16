use hecs::World;
use serde::{Deserialize, Serialize};

use crate::game::inventory::Inventory;
use crate::game::simulation::Simulation;
use crate::game::undo::UndoStack;
use crate::map::grid::Map;
use crate::render::animations::AnimationManager;
use crate::render::particles::ParticleSystem;
use crate::render::splits::SplitManager;
use crate::render::trails::TrailSystem;
use crate::resources::Facing;
use crate::vim::macros::MacroSystem;
use crate::vim::marks::MarkStore;
use crate::vim::registers::RegisterStore;
use crate::vim::search::SearchState;

/// The mode the editor/game is currently in.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Mode {
    Normal,
    Insert,
    Replace,
    Visual,
    VisualLine,
    VisualBlock,
    Command,
    Search,
    Menu,
}

/// Which popup is currently displayed, if any.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum PopupKind {
    Help(Option<String>),
    Stats,
    Registers,
    Marks,
    Contracts,
    Market,
    Finance,
    Research,
    Prestige,
    Recipes,
}

/// Game mode: tutorial levels, campaign, or freeplay sandbox.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum GameMode {
    Tutorial,
    Campaign,
    Freeplay,
}

/// Which screen the menu is showing while `mode == Mode::Menu`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MenuScreen {
    Title,
    LevelSelect,
    Help,
}

/// What a title-screen entry does when confirmed.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TitleAction {
    Campaign,
    Sandbox,
    Continue,
    Help,
    Quit,
}

/// One selectable entry on the title screen.
pub struct TitleEntry {
    pub key: char,
    pub name: &'static str,
    pub sub: String,
    pub action: TitleAction,
}

/// The kind of full-screen overlay card being shown.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum OverlayKind {
    LevelIntro,
    LevelComplete,
    CampaignFinale,
}

/// A full-screen overlay card (level intro / level complete / finale).
///
/// FALL-THROUGH semantics: any key dismisses the card AND still executes
/// normally, so overlays never eat input (see GameSession::handle_key).
/// Cards also expire on their own after `ticks_left` simulation ticks.
#[derive(Clone, Debug)]
pub struct OverlayCard {
    pub kind: OverlayKind,
    /// The level the card is about (the completed level for LevelComplete).
    pub level: usize,
    /// Edits made during the completed level (LevelComplete/Finale).
    pub edits: usize,
    /// Total output delivered when the level completed.
    pub output: u64,
    /// Auto-expiry countdown in simulation ticks.
    pub ticks_left: u32,
    /// Card to show after this one is dismissed (complete -> next intro).
    pub followup: Option<Box<OverlayCard>>,
}

/// Default overlay lifetime in simulation ticks (~300 per the design spec).
pub const OVERLAY_TICKS: u32 = 300;

impl OverlayCard {
    pub fn intro(level: usize) -> Self {
        OverlayCard {
            kind: OverlayKind::LevelIntro,
            level,
            edits: 0,
            output: 0,
            ticks_left: OVERLAY_TICKS,
            followup: None,
        }
    }

    pub fn complete(level: usize, edits: usize, output: u64, followup: Option<OverlayCard>) -> Self {
        OverlayCard {
            kind: OverlayKind::LevelComplete,
            level,
            edits,
            output,
            ticks_left: OVERLAY_TICKS,
            followup: followup.map(Box::new),
        }
    }

    pub fn finale(edits: usize, output: u64) -> Self {
        OverlayCard {
            kind: OverlayKind::CampaignFinale,
            level: 30,
            edits,
            output,
            ticks_left: OVERLAY_TICKS,
            followup: None,
        }
    }
}

/// Central application state.
pub struct AppState {
    // -- Core game data --
    pub world: World,
    pub map: Map,
    pub simulation: Simulation,
    pub undo_stack: UndoStack,
    pub inventory: Inventory,

    // -- Cursor and selection --
    pub cursor_x: usize,
    pub cursor_y: usize,
    pub visual_anchor_x: usize,
    pub visual_anchor_y: usize,

    // -- Mode --
    pub mode: Mode,
    pub insert_facing: Facing,
    /// Insert-mode category submenu currently open (e.g. "Power"), if any.
    pub insert_category: Option<&'static str>,

    // -- Vim state --
    pub registers: RegisterStore,
    pub marks: MarkStore,
    pub search: SearchState,
    pub macro_system: MacroSystem,
    pub recording_macro: Option<char>,

    // -- Command / search input --
    pub command_buffer: String,
    pub pending_keys: String,

    // -- UI state --
    pub show_sidebar: bool,
    pub show_tutorial: bool,
    pub split_manager: SplitManager,
    pub animations: AnimationManager,
    pub popup: Option<PopupKind>,
    pub popup_scroll: usize,
    /// Selection index inside interactive popups (research/contracts).
    pub popup_cursor: usize,

    // -- Status / messaging --
    pub status_message: String,
    pub status_error: bool,

    // -- Game progression --
    pub current_level: Option<usize>,
    /// Kept for save compatibility; Sandbox is ALWAYS available regardless.
    pub freeplay_unlocked: bool,
    pub has_save: bool,
    /// Campaign levels ever completed (union of live progress and the save
    /// on disk); drives the menu's N/30 readout and level-select locks.
    pub campaign_completed: Vec<usize>,
    /// The level the save on disk was parked on (Continue readout).
    pub saved_level: Option<usize>,

    // -- Menu state --
    pub menu_screen: MenuScreen,
    pub menu_cursor: usize,

    // -- Overlay cards (level intro / complete / finale) --
    pub overlay: Option<OverlayCard>,

    // -- Economy --
    pub economy: crate::economy::ledger::Economy,
    pub loans: crate::economy::loans::LoanManager,
    /// Per-resource output-bin totals at the last post_tick (delta detection
    /// for auto-sell + contract deliveries).
    pub delivered_snapshot: std::collections::HashMap<crate::resources::Resource, u64>,
    /// Lifetime deliveries per resource (drives contract generation,
    /// auto-accept, and scaling).
    pub delivered_lifetime: std::collections::HashMap<crate::resources::Resource, u64>,
    /// Sale income accumulated during the current economy cycle.
    pub income_this_cycle: f64,
    /// Sale income of the last completed cycle (finance popup).
    pub income_last_cycle: f64,
    /// Expense report of the last completed cycle (finance popup).
    pub last_expense_report: crate::economy::expenses::ExpenseReport,

    // -- Contracts --
    pub contract_board: crate::contracts::board::ContractBoard,

    // -- Market --
    pub market: crate::market::prices::MarketState,

    // -- Research --
    pub research: crate::research::tree::ResearchState,

    // -- Scaling --
    pub scaling: crate::scaling::difficulty::ScalingState,
    pub regulations: crate::scaling::regulations::Regulations,

    // -- Pollution --
    pub pollution: crate::waste::pollution::PollutionState,

    // -- Particles & trails --
    pub particles: ParticleSystem,
    pub trails: TrailSystem,

    // -- Day/night cycle --
    pub day_tick: u32,

    // -- Prestige --
    pub prestige: crate::scaling::prestige::PrestigeState,

    // -- Game mode --
    pub game_mode: GameMode,
    pub seed: u64,

    // -- Running state --
    pub should_quit: bool,
}

impl AppState {
    pub fn new(map_width: usize, map_height: usize) -> Self {
        AppState {
            world: World::new(),
            map: Map::new(map_width, map_height),
            simulation: Simulation::new(),
            undo_stack: UndoStack::new(),
            inventory: Inventory::new(),

            cursor_x: 0,
            cursor_y: 0,
            visual_anchor_x: 0,
            visual_anchor_y: 0,

            mode: Mode::Menu,
            insert_facing: Facing::Right,
            insert_category: None,

            registers: RegisterStore::new(),
            marks: MarkStore::new(),
            search: SearchState::new(),
            macro_system: MacroSystem::new(),
            recording_macro: None,

            command_buffer: String::new(),
            pending_keys: String::new(),

            show_sidebar: true,
            show_tutorial: false,
            split_manager: SplitManager::new(),
            animations: AnimationManager::new(),
            popup: None,
            popup_scroll: 0,
            popup_cursor: 0,

            status_message: String::new(),
            status_error: false,

            current_level: None,
            freeplay_unlocked: false,
            has_save: false,
            campaign_completed: Vec::new(),
            saved_level: None,

            menu_screen: MenuScreen::Title,
            menu_cursor: 0,

            overlay: None,

            economy: crate::economy::ledger::Economy::new(crate::economy::ledger::Difficulty::Normal),
            loans: crate::economy::loans::LoanManager::new(crate::economy::ledger::Difficulty::Normal),
            delivered_snapshot: std::collections::HashMap::new(),
            delivered_lifetime: std::collections::HashMap::new(),
            income_this_cycle: 0.0,
            income_last_cycle: 0.0,
            last_expense_report: crate::economy::expenses::ExpenseReport::default(),
            contract_board: crate::contracts::board::ContractBoard::new(),
            market: crate::market::prices::MarketState::new(),
            research: crate::research::tree::ResearchState::new(),
            scaling: crate::scaling::difficulty::ScalingState::new(),
            regulations: crate::scaling::regulations::Regulations::new(),
            pollution: crate::waste::pollution::PollutionState::new(),
            particles: ParticleSystem::new(),
            trails: TrailSystem::new(),
            day_tick: 0,
            prestige: crate::scaling::prestige::PrestigeState::new(),
            game_mode: GameMode::Tutorial,
            seed: 42,

            should_quit: false,
        }
    }

    /// Number of campaign levels (1..=30) completed, for the menu readouts.
    pub fn campaign_progress(&self) -> usize {
        self.campaign_completed
            .iter()
            .filter(|l| (1..=30).contains(*l))
            .count()
    }

    /// Furthest campaign level ever completed (0 if none). Levels up to
    /// furthest+1 are unlocked in the level-select grid.
    pub fn furthest_campaign_level(&self) -> usize {
        self.campaign_completed
            .iter()
            .filter(|l| (1..=30).contains(*l))
            .max()
            .copied()
            .unwrap_or(0)
    }

    /// A campaign level is selectable when it is not beyond furthest+1.
    pub fn is_level_unlocked(&self, level: usize) -> bool {
        level >= 1 && level <= self.furthest_campaign_level() + 1
    }

    /// The title-screen entries in display order. Shared by the renderer
    /// and the menu key handler so cursor indices always agree.
    pub fn title_entries(&self) -> Vec<TitleEntry> {
        let done = self.campaign_progress();
        let mut entries = vec![
            TitleEntry {
                key: '1',
                name: "Campaign",
                sub: if done == 0 {
                    "30-level vim curriculum".to_string()
                } else {
                    format!("{done}/30 complete")
                },
                action: TitleAction::Campaign,
            },
            TitleEntry {
                key: '2',
                name: "Sandbox",
                sub: "open factory · economy · research".to_string(),
                action: TitleAction::Sandbox,
            },
        ];
        if self.has_save {
            entries.push(TitleEntry {
                key: '3',
                name: "Continue",
                sub: match self.saved_level {
                    Some(l) if (1..=30).contains(&l) => format!("resume at level {l}"),
                    _ => "resume your save".to_string(),
                },
                action: TitleAction::Continue,
            });
        }
        entries.push(TitleEntry {
            key: 'h',
            name: "Help",
            sub: String::new(),
            action: TitleAction::Help,
        });
        entries.push(TitleEntry {
            key: 'q',
            name: "Quit",
            sub: String::new(),
            action: TitleAction::Quit,
        });
        entries
    }

    /// Returns the list of visual-mode selected tiles (if in a visual mode).
    pub fn visual_selection(&self) -> Vec<(usize, usize)> {
        match self.mode {
            Mode::Visual => {
                let x1 = self.visual_anchor_x.min(self.cursor_x);
                let x2 = self.visual_anchor_x.max(self.cursor_x);
                let y1 = self.visual_anchor_y.min(self.cursor_y);
                let y2 = self.visual_anchor_y.max(self.cursor_y);
                // Character-wise: from anchor to cursor in reading order
                let mut tiles = Vec::new();
                for y in y1..=y2 {
                    let sx = if y == y1 { x1 } else { 0 };
                    let ex = if y == y2 {
                        x2
                    } else {
                        self.map.width.saturating_sub(1)
                    };
                    for x in sx..=ex {
                        tiles.push((x, y));
                    }
                }
                tiles
            }
            Mode::VisualLine => {
                let y1 = self.visual_anchor_y.min(self.cursor_y);
                let y2 = self.visual_anchor_y.max(self.cursor_y);
                let mut tiles = Vec::new();
                for y in y1..=y2 {
                    for x in 0..self.map.width {
                        tiles.push((x, y));
                    }
                }
                tiles
            }
            Mode::VisualBlock => {
                let x1 = self.visual_anchor_x.min(self.cursor_x);
                let x2 = self.visual_anchor_x.max(self.cursor_x);
                let y1 = self.visual_anchor_y.min(self.cursor_y);
                let y2 = self.visual_anchor_y.max(self.cursor_y);
                let mut tiles = Vec::new();
                for y in y1..=y2 {
                    for x in x1..=x2 {
                        tiles.push((x, y));
                    }
                }
                tiles
            }
            _ => Vec::new(),
        }
    }
}
