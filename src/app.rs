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
}

/// Game mode: tutorial levels, campaign, or freeplay sandbox.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum GameMode {
    Tutorial,
    Campaign,
    Freeplay,
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

    // -- Status / messaging --
    pub status_message: String,
    pub status_error: bool,

    // -- Game progression --
    pub current_level: Option<usize>,
    pub freeplay_unlocked: bool,
    pub has_save: bool,

    // -- Economy --
    pub economy: crate::economy::ledger::Economy,
    pub loans: crate::economy::loans::LoanManager,

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

            status_message: String::new(),
            status_error: false,

            current_level: None,
            freeplay_unlocked: false,
            has_save: false,

            economy: crate::economy::ledger::Economy::new(crate::economy::ledger::Difficulty::Normal),
            loans: crate::economy::loans::LoanManager::new(crate::economy::ledger::Difficulty::Normal),
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
