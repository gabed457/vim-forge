//! The state machine input parser. Takes crossterm KeyEvents and produces Commands.
//!
//! # Key reference — extended core-vim coverage (factory semantics)
//!
//! Motions (all work standalone, after operators, and in visual mode):
//!   h j k l 0 ^ $ g_ g0 g$  w W b B e ge E  f/F/t/T{char} ; ,
//!   gg G H M L { } ( ) % n N  | (column)  _ (first entity)  + - (row below/above)
//!   `(` / `)` jump to the previous/next machine (non-belt building) in
//!   reading order — the factory "sentence" motion.
//!
//! Operators: d y c > < and the g-operators gU (upgrade tier), gu
//!   (downgrade tier), g~ (rotate 180). All accept motions, text objects,
//!   doubled linewise forms (dd/guu/gUU/g~~), counts and registers.
//!   Operator+motion emits Command::OperatorMotion; the input handler
//!   resolves the tile range (charwise/linewise per vim rules).
//!
//! Text objects: i/a + w p s b ( ) [ ] { } < > B " ' t
//!   - w: entity cluster, p: paragraph of rows, b/brackets/B/s: wall-enclosed
//!     block (machine cluster), "/': belt run (a" includes end machines),
//!     t: machine footprint (at adds its port tiles).
//!
//! Edits: x s S C D Y R (replace mode) J (join clusters with belts)
//!   Ctrl-a/Ctrl-x (tier up/down), ~, r{building}.
//!
//! Scrolling: zz zt zb, Ctrl-d/Ctrl-u (half page), Ctrl-f/Ctrl-b (full page).
//! Jumplist: Ctrl-o / Ctrl-i (gg/G/search/marks/paragraph/machine jumps).
//! Registers: "a-"z named, "A-"Z append, "_ black hole, "0 last yank,
//!   "1-"9 delete history (shifting).
//! Visual: counts on motions, f/t/F/T/;/,, x, r{building}, ~, o/O, gv,
//!   text objects via i/a, block-mode I/A column insert.
//! Command line: :s/old/new/[g], :%s, :N,Ms, :g/pat/d, & and @:,
//!   :noh :only :sp :vsp.

use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

use crate::commands::{Command, MotionKind, Operator};
use crate::resources::{Direction, EntityType, Facing};

/// The current mode of the vim editor.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Mode {
    Normal,
    Insert,
    Visual,
    VisualLine,
    VisualBlock,
    Command,
    Search,
}

/// What the parser is currently waiting for to complete a pending command.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Awaiting {
    Nothing,
    Operator,
    Motion,
    FChar,
    FCharBack,
    TChar,
    TCharBack,
    MarkSet,
    MarkJump,
    MarkJumpExact,
    RegisterSelect,
    SecondG,
    MacroRecord,
    MacroPlay,
    ReplaceChar,
    CtrlW,
    TextObjectInner,
    TextObjectAround,
    /// After `z`: waiting for z/t/b (viewport positioning).
    ZPrefix,
    /// After `I` in visual-block: next building key fills the left edge.
    BlockInsertLeft,
    /// After `A` in visual-block: next building key fills the right edge.
    BlockInsertRight,
}

/// Categories for the hierarchical insert sub-menu system.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum InsertCategoryKind {
    Conveyors,
    Pipes,
    ProcessingT1,
    ProcessingT2,
    ProcessingT4,
    Energy,
    Logistics,
    Transport,
    Research,
    Waste,
    Utility,
    Circuit,
    Balancers,
    FluidExtras,
    Storage,
}

impl InsertCategoryKind {
    pub fn name(&self) -> &'static str {
        match self {
            Self::Conveyors => "Conveyors",
            Self::Pipes => "Pipes",
            Self::ProcessingT1 => "Processing T1",
            Self::ProcessingT2 => "Processing T2+",
            Self::ProcessingT4 => "Processing T4",
            Self::Energy => "Energy",
            Self::Logistics => "Logistics",
            Self::Transport => "Transport",
            Self::Research => "Research",
            Self::Waste => "Waste",
            Self::Utility => "Utility",
            Self::Circuit => "Circuit",
            Self::Balancers => "Balancers",
            Self::FluidExtras => "Fluid Extras",
            Self::Storage => "Storage",
        }
    }
}

/// Resolve a key within a category to an EntityType.
fn resolve_category_key(cat: InsertCategoryKind, c: char) -> Option<EntityType> {
    match cat {
        InsertCategoryKind::Conveyors => match c {
            '1' => Some(EntityType::BasicBelt),
            '2' => Some(EntityType::FastBelt),
            '3' => Some(EntityType::ExpressBelt),
            'u' => Some(EntityType::UndergroundEntrance),
            'U' => Some(EntityType::UndergroundExit),
            _ => None,
        },
        InsertCategoryKind::Pipes => match c {
            'p' => Some(EntityType::Pipe),
            'j' => Some(EntityType::PipeJunction),
            's' => Some(EntityType::PumpStation),
            't' => Some(EntityType::FluidTank),
            'g' => Some(EntityType::GasCompressor),
            'l' => Some(EntityType::GasPipeline),  // Changed from 'n' to 'l'
            _ => None,
        },
        InsertCategoryKind::ProcessingT1 => match c {
            's' => Some(EntityType::Smelter),
            'k' => Some(EntityType::Kiln),
            'p' => Some(EntityType::Press),
            'w' => Some(EntityType::WireMill),
            'm' => Some(EntityType::PlateMachine),
            'v' => Some(EntityType::RubberVulcanizer),
            'd' => Some(EntityType::PlasticMolder),
            'e' => Some(EntityType::Electrolyzer),
            'c' => Some(EntityType::Caster),
            'f' => Some(EntityType::CokeFurnace),
            'g' => Some(EntityType::Gasifier),
            'b' => Some(EntityType::Boiler),
            'x' => Some(EntityType::WaferCutter),
            _ => None,
        },
        InsertCategoryKind::ProcessingT2 => match c {
            'a' => Some(EntityType::Assembler),
            'm' => Some(EntityType::Mixer),
            'c' => Some(EntityType::ChemicalPlant),
            'i' => Some(EntityType::CircuitFabricator),
            'o' => Some(EntityType::MotorAssembly),
            'r' => Some(EntityType::CrushingMill),
            'd' => Some(EntityType::AdvancedAssembler),
            'f' => Some(EntityType::Refinery),
            't' => Some(EntityType::CrackingTower),
            'n' => Some(EntityType::Cleanroom),
            'e' => Some(EntityType::EnrichmentCascade),
            'p' => Some(EntityType::CoolantProcessor),
            _ => None,
        },
        InsertCategoryKind::ProcessingT4 => match c {
            'p' => Some(EntityType::PrecisionAssembler),
            'q' => Some(EntityType::QuantumLab),
            'r' => Some(EntityType::RocketAssembly),
            'm' => Some(EntityType::Megassembler),
            's' => Some(EntityType::SingularityLab),
            _ => None,
        },
        InsertCategoryKind::Energy => match c {
            'c' => Some(EntityType::CoalGenerator),
            'g' => Some(EntityType::GasGenerator),
            's' => Some(EntityType::SolarArray),
            'w' => Some(EntityType::WindTurbine),
            't' => Some(EntityType::GeothermalPlant),
            'n' => Some(EntityType::NuclearReactor),
            'f' => Some(EntityType::FusionReactor),
            'x' => Some(EntityType::Transformer),
            'p' => Some(EntityType::PowerPole),
            'u' => Some(EntityType::Substation),
            'b' => Some(EntityType::BatteryBank),
            'a' => Some(EntityType::Accumulator),
            _ => None,
        },
        InsertCategoryKind::Storage => match c {
            'w' => Some(EntityType::Warehouse),
            's' => Some(EntityType::SiloHopper),
            'c' => Some(EntityType::CryoTank),
            'v' => Some(EntityType::ContainmentVault),
            _ => None,
        },
        InsertCategoryKind::Logistics => match c {
            'd' => Some(EntityType::DronePort),
            _ => None,
        },
        InsertCategoryKind::Transport => match c {
            'r' => Some(EntityType::RailTrack),
            's' => Some(EntityType::TrainStation),
            'd' => Some(EntityType::DronePort),
            _ => None,
        },
        InsertCategoryKind::Research => match c {
            '1' => Some(EntityType::ResearchLab),
            '2' => Some(EntityType::AdvancedLab),
            _ => None,
        },
        InsertCategoryKind::Waste => match c {
            'd' => Some(EntityType::WasteDump),
            'r' => Some(EntityType::RecyclingPlant),
            'i' => Some(EntityType::IncinerationPlant),
            'f' => Some(EntityType::FilterStack),
            's' => Some(EntityType::ScrubberUnit),
            'c' => Some(EntityType::ContainmentField),
            _ => None,
        },
        InsertCategoryKind::Utility => match c {
            'w' => Some(EntityType::Wall),
            'r' => Some(EntityType::ReinforcedWall),
            't' => Some(EntityType::Turret),
            's' => Some(EntityType::ShieldGenerator),
            _ => None,
        },
        InsertCategoryKind::Circuit => {
            // No circuit entities exist yet
            None
        },
        InsertCategoryKind::Balancers => match c {
            's' => Some(EntityType::Splitter),
            'm' => Some(EntityType::Merger),
            _ => None,
        },
        InsertCategoryKind::FluidExtras => match c {
            'p' => Some(EntityType::PumpStation),
            _ => None,
        },
    }
}

/// The insert-mode stage-1 quick-place key map, also used by the
/// visual-block I/A column insert. Mirrors `handle_insert_inner` stage 1.
pub fn resolve_quickplace_key(c: char) -> Option<EntityType> {
    match c {
        'c' | '1' => Some(EntityType::BasicBelt),
        '2' => Some(EntityType::FastBelt),
        '3' => Some(EntityType::ExpressBelt),
        's' => Some(EntityType::Smelter),
        'a' => Some(EntityType::Assembler),
        'w' => Some(EntityType::Wall),
        'p' => Some(EntityType::Pipe),
        'b' => Some(EntityType::Splitter),
        'm' => Some(EntityType::Merger),
        'u' => Some(EntityType::UndergroundEntrance),
        'U' => Some(EntityType::UndergroundExit),
        'e' => Some(EntityType::CoalGenerator),
        'r' => Some(EntityType::ResearchLab),
        't' => Some(EntityType::RailTrack),
        'd' => Some(EntityType::WasteDump),
        'g' => Some(EntityType::Warehouse),
        'f' => Some(EntityType::PumpStation),
        'o' => Some(EntityType::DronePort),
        'q' => Some(EntityType::PrecisionAssembler),
        'x' => Some(EntityType::RecyclingPlant),
        _ => None,
    }
}

/// The main vim input parser state machine.
pub struct VimParser {
    pub mode: Mode,
    register: Option<char>,
    count1: Option<usize>,
    operator: Option<Operator>,
    count2: Option<usize>,
    awaiting: Awaiting,
    pub insert_facing: Facing,
    pub insert_category: Option<InsertCategoryKind>,
    /// True while in R (replace) mode — a flavor of Insert mode where each
    /// building key replaces the tile under the cursor and advances.
    pub replace_mode: bool,
    pub recording_macro: Option<char>,
    pub macro_keystrokes: Vec<KeyEvent>,
    pub command_buffer: String,
    pub command_line: String,
    pub search_line: String,
    pub search_forward: bool,
    count_buffer: String,
    pub insert_count: usize,
    command_history: Vec<String>,
    command_history_idx: Option<usize>,
    search_history: Vec<String>,
    search_history_idx: Option<usize>,
}

impl VimParser {
    pub fn new() -> Self {
        VimParser {
            mode: Mode::Normal,
            register: None,
            count1: None,
            operator: None,
            count2: None,
            awaiting: Awaiting::Nothing,
            insert_facing: Facing::Right,
            insert_category: None,
            replace_mode: false,
            recording_macro: None,
            macro_keystrokes: Vec::new(),
            command_buffer: String::new(),
            command_line: String::new(),
            search_line: String::new(),
            search_forward: true,
            count_buffer: String::new(),
            insert_count: 1,
            command_history: Vec::new(),
            command_history_idx: None,
            search_history: Vec::new(),
            search_history_idx: None,
        }
    }

    /// Get the effective count (count1 * count2, defaulting each to 1).
    fn effective_count(&self) -> usize {
        let c1 = self.count1.unwrap_or(1);
        let c2 = self.count2.unwrap_or(1);
        c1 * c2
    }

    /// Reset the pending command state.
    fn reset_pending(&mut self) {
        self.register = None;
        self.count1 = None;
        self.operator = None;
        self.count2 = None;
        self.awaiting = Awaiting::Nothing;
        self.count_buffer.clear();
        self.command_buffer.clear();
    }

    /// Handle a key event and return any commands produced.
    pub fn handle_key_event(&mut self, key: KeyEvent) -> Vec<Command> {
        // If recording a macro, record the key (unless it's q to stop)
        if self.recording_macro.is_some() {
            let is_stop = self.mode == Mode::Normal
                && key.code == KeyCode::Char('q')
                && key.modifiers.is_empty();
            if !is_stop {
                self.macro_keystrokes.push(key);
            }
        }

        match self.mode {
            Mode::Normal => self.handle_normal(key),
            Mode::Insert => self.handle_insert(key),
            Mode::Visual | Mode::VisualLine | Mode::VisualBlock => self.handle_visual(key),
            Mode::Command => self.handle_command_mode(key),
            Mode::Search => self.handle_search_mode(key),
        }
    }

    // -----------------------------------------------------------------------
    // Normal mode
    // -----------------------------------------------------------------------

    fn handle_normal(&mut self, key: KeyEvent) -> Vec<Command> {
        // First handle awaiting states that consume exactly one more key
        match self.awaiting {
            Awaiting::FChar => return self.handle_find_char(key, true, false),
            Awaiting::FCharBack => return self.handle_find_char(key, false, false),
            Awaiting::TChar => return self.handle_find_char(key, true, true),
            Awaiting::TCharBack => return self.handle_find_char(key, false, true),
            Awaiting::MarkSet => return self.handle_mark_set(key),
            Awaiting::MarkJump => return self.handle_mark_jump(key, false),
            Awaiting::MarkJumpExact => return self.handle_mark_jump(key, true),
            Awaiting::RegisterSelect => return self.handle_register_select(key),
            Awaiting::SecondG => return self.handle_second_g(key),
            Awaiting::MacroRecord => return self.handle_macro_record_start(key),
            Awaiting::MacroPlay => return self.handle_macro_play(key),
            Awaiting::ReplaceChar => return self.handle_replace_char(key),
            Awaiting::CtrlW => return self.handle_ctrl_w(key),
            Awaiting::TextObjectInner => return self.handle_text_object_resolve(key, true),
            Awaiting::TextObjectAround => return self.handle_text_object_resolve(key, false),
            Awaiting::ZPrefix => return self.handle_z_prefix(key),
            Awaiting::BlockInsertLeft | Awaiting::BlockInsertRight => {
                // Only meaningful in visual-block; stray state — reset.
                self.reset_pending();
                return vec![];
            }
            Awaiting::Nothing | Awaiting::Operator | Awaiting::Motion => {}
        }

        let code = key.code;
        let mods = key.modifiers;

        // Handle digit accumulation for counts
        if let KeyCode::Char(c) = code {
            if c.is_ascii_digit() {
                // '0' as first char is line-start, not count
                let is_line_start = c == '0'
                    && self.count_buffer.is_empty()
                    && self.operator.is_none();
                if !is_line_start {
                    self.count_buffer.push(c);
                    self.command_buffer.push(c);
                    return vec![];
                }
            }
        }

        // Flush count buffer into the appropriate count slot
        self.flush_count();

        match code {
            // Movement keys
            KeyCode::Char('h') if mods.is_empty() => self.motion(MotionKind::Left),
            KeyCode::Char('j') if mods.is_empty() => self.motion(MotionKind::Down),
            KeyCode::Char('k') if mods.is_empty() => self.motion(MotionKind::Up),
            KeyCode::Char('l') if mods.is_empty() => self.motion(MotionKind::Right),

            // Word/entity navigation
            KeyCode::Char('w') if mods.is_empty() => self.motion(MotionKind::WordForward),
            KeyCode::Char('W') if !mods.contains(KeyModifiers::CONTROL) => {
                self.motion(MotionKind::BigWordForward)
            }
            KeyCode::Char('b') if mods.is_empty() => self.motion(MotionKind::WordBackward),
            KeyCode::Char('B') if !mods.contains(KeyModifiers::CONTROL) => {
                self.motion(MotionKind::BigWordBackward)
            }
            KeyCode::Char('e') if mods.is_empty() => self.motion(MotionKind::WordEnd),
            KeyCode::Char('E') => self.motion(MotionKind::BigWordEnd),

            // Line start/end
            KeyCode::Char('0') if mods.is_empty() => self.motion(MotionKind::LineStart),
            KeyCode::Char('$') | KeyCode::End => self.motion(MotionKind::LineEnd),
            KeyCode::Char('^') => self.motion(MotionKind::FirstEntity),
            KeyCode::Char('_') => self.motion(MotionKind::FirstEntity),
            KeyCode::Char('|') => self.motion(MotionKind::Column),
            KeyCode::Char('+') => self.motion(MotionKind::RowDown),
            KeyCode::Char('-') => self.motion(MotionKind::RowUp),
            KeyCode::Home => self.motion(MotionKind::LineStart),

            // Map start/end
            KeyCode::Char('G') => {
                let row = self.count1.or(self.count2);
                self.motion(MotionKind::MapEnd(row))
            }

            // Viewport positions
            KeyCode::Char('H') if !mods.contains(KeyModifiers::CONTROL) => {
                self.motion(MotionKind::ViewportTop)
            }
            KeyCode::Char('M') if !mods.contains(KeyModifiers::CONTROL) => {
                self.motion(MotionKind::ViewportMiddle)
            }
            KeyCode::Char('L') if !mods.contains(KeyModifiers::CONTROL) => {
                self.motion(MotionKind::ViewportBottom)
            }

            // Sentence motions: previous/next machine (non-belt building)
            KeyCode::Char('(') => self.motion(MotionKind::PrevMachine),
            KeyCode::Char(')') => self.motion(MotionKind::NextMachine),

            // Find entity
            KeyCode::Char('f') if mods.is_empty() => {
                self.awaiting = Awaiting::FChar;
                self.command_buffer.push('f');
                vec![]
            }
            KeyCode::Char('F') if mods.is_empty() => {
                self.awaiting = Awaiting::FCharBack;
                self.command_buffer.push('F');
                vec![]
            }
            KeyCode::Char('t') if mods.is_empty() => {
                self.awaiting = Awaiting::TChar;
                self.command_buffer.push('t');
                vec![]
            }
            KeyCode::Char('T') if mods.is_empty() => {
                self.awaiting = Awaiting::TCharBack;
                self.command_buffer.push('T');
                vec![]
            }

            // Repeat find
            KeyCode::Char(';') => self.motion(MotionKind::RepeatFind(true)),
            KeyCode::Char(',') => self.motion(MotionKind::RepeatFind(false)),

            // Paragraph navigation
            KeyCode::Char('}') => self.motion(MotionKind::NextParagraph),
            KeyCode::Char('{') => self.motion(MotionKind::PrevParagraph),

            // Match connection
            KeyCode::Char('%') => self.motion(MotionKind::MatchConnection),

            // Operators
            KeyCode::Char('d') if mods.is_empty() => self.handle_operator_key(Operator::Delete, 'd'),
            KeyCode::Char('y') if mods.is_empty() => self.handle_operator_key(Operator::Yank, 'y'),
            KeyCode::Char('c') if mods.is_empty() => {
                match self.operator {
                    // cc doubles to change-line; no operator starts change
                    None | Some(Operator::Change) => {
                        self.handle_operator_key(Operator::Change, 'c')
                    }
                    // 'c' after a different operator (dc, yc): invalid, reset
                    Some(_) => {
                        self.reset_pending();
                        vec![]
                    }
                }
            }
            KeyCode::Char('>') => self.handle_operator_key(Operator::RotateCW, '>'),
            KeyCode::Char('<') => self.handle_operator_key(Operator::RotateCCW, '<'),

            // Text objects (only valid after an operator)
            KeyCode::Char('i') if mods.is_empty() && self.operator.is_some() => {
                self.awaiting = Awaiting::TextObjectInner;
                self.command_buffer.push('i');
                vec![]
            }
            KeyCode::Char('a') if mods.is_empty() && self.operator.is_some() => {
                self.awaiting = Awaiting::TextObjectAround;
                self.command_buffer.push('a');
                vec![]
            }

            // Register selection
            KeyCode::Char('"') => {
                self.awaiting = Awaiting::RegisterSelect;
                self.command_buffer.push('"');
                vec![]
            }

            // Paste
            KeyCode::Char('p') if mods.is_empty() => {
                let reg = self.register;
                let count = self.effective_count();
                let cmds = vec![Command::Paste(reg, count, false)];
                self.reset_pending();
                cmds
            }
            KeyCode::Char('P') => {
                let reg = self.register;
                let count = self.effective_count();
                let cmds = vec![Command::Paste(reg, count, true)];
                self.reset_pending();
                cmds
            }

            // Marks
            KeyCode::Char('m') if mods.is_empty() => {
                self.awaiting = Awaiting::MarkSet;
                self.command_buffer.push('m');
                vec![]
            }
            KeyCode::Char('\'') => {
                self.awaiting = Awaiting::MarkJump;
                self.command_buffer.push('\'');
                vec![]
            }
            KeyCode::Char('`') => {
                self.awaiting = Awaiting::MarkJumpExact;
                self.command_buffer.push('`');
                vec![]
            }

            // Macros
            KeyCode::Char('q') if mods.is_empty() => {
                if self.recording_macro.is_some() {
                    // Stop recording
                    let cmds = vec![Command::StopMacro];
                    self.reset_pending();
                    cmds
                } else {
                    self.awaiting = Awaiting::MacroRecord;
                    self.command_buffer.push('q');
                    vec![]
                }
            }
            KeyCode::Char('@') => {
                self.awaiting = Awaiting::MacroPlay;
                self.command_buffer.push('@');
                vec![]
            }

            // Undo / Redo (also `guu` linewise when gu is pending)
            KeyCode::Char('u') if mods.is_empty() => {
                if self.operator == Some(Operator::Downgrade) {
                    let count = self.effective_count();
                    let reg = self.register;
                    self.reset_pending();
                    return vec![Command::OperatorLines(Operator::Downgrade, count, reg)];
                }
                self.reset_pending();
                vec![Command::Undo]
            }
            KeyCode::Char('U') if self.operator == Some(Operator::Upgrade) => {
                let count = self.effective_count();
                let reg = self.register;
                self.reset_pending();
                vec![Command::OperatorLines(Operator::Upgrade, count, reg)]
            }
            KeyCode::Char('r') if mods.contains(KeyModifiers::CONTROL) => {
                self.reset_pending();
                vec![Command::Redo]
            }

            // Dot repeat
            KeyCode::Char('.') if mods.is_empty() => {
                self.reset_pending();
                vec![Command::DotRepeat]
            }

            // Search
            KeyCode::Char('/') if mods.is_empty() => {
                self.reset_pending();
                self.search_forward = true;
                self.search_line.clear();
                self.mode = Mode::Search;
                self.search_history_idx = None;
                vec![Command::EnterSearch(true)]
            }
            KeyCode::Char('?') if mods.is_empty() => {
                self.reset_pending();
                self.search_forward = false;
                self.search_line.clear();
                self.mode = Mode::Search;
                self.search_history_idx = None;
                vec![Command::EnterSearch(false)]
            }
            KeyCode::Char('n') if mods.is_empty() => self.motion(MotionKind::SearchNext),
            KeyCode::Char('N') => self.motion(MotionKind::SearchPrev),
            KeyCode::Char('*') => {
                self.reset_pending();
                vec![Command::SearchWordUnderCursor(true)]
            }
            KeyCode::Char('#') => {
                self.reset_pending();
                vec![Command::SearchWordUnderCursor(false)]
            }

            // Enter insert mode
            KeyCode::Char('i') if mods.is_empty() && self.operator.is_none() => {
                let count = self.effective_count();
                self.insert_count = count;
                self.reset_pending();
                self.mode = Mode::Insert;
                vec![Command::EnterInsert(count)]
            }
            // Insert variants: a (append), A (append at cluster end),
            // I (insert at first entity), o (open row below), O (open above).
            KeyCode::Char('a') if mods.is_empty() && self.operator.is_none() => {
                let count = self.effective_count();
                self.insert_count = count;
                self.reset_pending();
                self.mode = Mode::Insert;
                vec![
                    Command::Move(Direction::Right, 1),
                    Command::EnterInsert(count),
                ]
            }
            KeyCode::Char('A') if !mods.contains(KeyModifiers::CONTROL)
                && self.operator.is_none() =>
            {
                let count = self.effective_count();
                self.insert_count = count;
                self.reset_pending();
                self.mode = Mode::Insert;
                vec![
                    Command::LastEntityInRow,
                    Command::Move(Direction::Right, 1),
                    Command::EnterInsert(count),
                ]
            }
            KeyCode::Char('I') if !mods.contains(KeyModifiers::CONTROL)
                && self.operator.is_none() =>
            {
                let count = self.effective_count();
                self.insert_count = count;
                self.reset_pending();
                self.mode = Mode::Insert;
                vec![Command::FirstEntityInRow, Command::EnterInsert(count)]
            }
            KeyCode::Char('o') if mods.is_empty() && self.operator.is_none() => {
                let count = self.effective_count();
                self.insert_count = count;
                self.reset_pending();
                self.mode = Mode::Insert;
                vec![
                    Command::Move(Direction::Down, 1),
                    Command::EnterInsert(count),
                ]
            }
            KeyCode::Char('O') if !mods.contains(KeyModifiers::CONTROL)
                && self.operator.is_none() =>
            {
                let count = self.effective_count();
                self.insert_count = count;
                self.reset_pending();
                self.mode = Mode::Insert;
                vec![
                    Command::Move(Direction::Up, 1),
                    Command::EnterInsert(count),
                ]
            }

            // Visual modes
            KeyCode::Char('v') if mods.is_empty() => {
                self.reset_pending();
                self.mode = Mode::Visual;
                vec![Command::EnterVisual]
            }
            KeyCode::Char('V') => {
                self.reset_pending();
                self.mode = Mode::VisualLine;
                vec![Command::EnterVisualLine]
            }
            KeyCode::Char('v') if mods.contains(KeyModifiers::CONTROL) => {
                self.reset_pending();
                self.mode = Mode::VisualBlock;
                vec![Command::EnterVisualBlock]
            }

            // Command mode
            KeyCode::Char(':') if mods.is_empty() => {
                self.reset_pending();
                self.command_line.clear();
                self.mode = Mode::Command;
                self.command_history_idx = None;
                vec![Command::EnterCommand]
            }

            // Replace
            KeyCode::Char('r') if mods.is_empty() && self.operator.is_none() => {
                self.awaiting = Awaiting::ReplaceChar;
                self.command_buffer.push('r');
                vec![]
            }

            // Delete under cursor
            KeyCode::Char('x') if mods.is_empty() => {
                let count = self.effective_count();
                self.reset_pending();
                vec![Command::DeleteUnderCursor(count)]
            }

            // Rotate entity under cursor (or `g~~` linewise when g~ pending)
            KeyCode::Char('~') => {
                if self.operator == Some(Operator::Rotate180) {
                    let count = self.effective_count();
                    let reg = self.register;
                    self.reset_pending();
                    return vec![Command::OperatorLines(Operator::Rotate180, count, reg)];
                }
                self.reset_pending();
                vec![Command::RotateEntityUnderCursor]
            }

            // s: substitute — delete N tiles under cursor and enter insert
            KeyCode::Char('s') if mods.is_empty() && self.operator.is_none() => {
                let count = self.effective_count();
                self.reset_pending();
                self.mode = Mode::Insert;
                self.insert_count = 1;
                vec![Command::DeleteUnderCursor(count), Command::EnterInsert(1)]
            }
            // S: change whole line (cc synonym)
            KeyCode::Char('S') => {
                let count = self.effective_count();
                self.reset_pending();
                vec![Command::ChangeLine(count)]
            }
            // C: change to end of line
            KeyCode::Char('C') => {
                let reg = self.register;
                self.reset_pending();
                vec![Command::OperatorMotion(
                    Operator::Change,
                    MotionKind::LineEnd,
                    1,
                    reg,
                )]
            }
            // D: delete to end of line
            KeyCode::Char('D') => {
                let reg = self.register;
                self.reset_pending();
                vec![Command::OperatorMotion(
                    Operator::Delete,
                    MotionKind::LineEnd,
                    1,
                    reg,
                )]
            }
            // Y: yank line (yy synonym)
            KeyCode::Char('Y') => {
                let count = self.effective_count();
                let reg = self.register;
                self.reset_pending();
                vec![Command::YankLine(count, reg)]
            }
            // R: replace mode (insert flavor; each key replaces + advances)
            KeyCode::Char('R') => {
                self.reset_pending();
                self.mode = Mode::Insert;
                self.replace_mode = true;
                self.insert_count = 1;
                vec![Command::EnterInsert(1)]
            }
            // J: join clusters — fill the gap to the next cluster with belts
            KeyCode::Char('J') if !mods.contains(KeyModifiers::CONTROL) => {
                let count = self.effective_count();
                self.reset_pending();
                vec![Command::JoinClusters(count)]
            }
            // &: repeat last :s substitution on the current row
            KeyCode::Char('&') => {
                self.reset_pending();
                vec![Command::RepeatSubstitute]
            }

            // Ctrl-a / Ctrl-x: tier upgrade/downgrade
            KeyCode::Char('a') if mods.contains(KeyModifiers::CONTROL) => {
                let count = self.effective_count();
                self.reset_pending();
                vec![Command::TierUp(count)]
            }
            KeyCode::Char('x') if mods.contains(KeyModifiers::CONTROL) => {
                let count = self.effective_count();
                self.reset_pending();
                vec![Command::TierDown(count)]
            }

            // Scrolling: Ctrl-d/u (half page), Ctrl-f/b (full page)
            KeyCode::Char('d') if mods.contains(KeyModifiers::CONTROL) => {
                let count = self.effective_count();
                self.reset_pending();
                vec![Command::ScrollHalfPageDown(count)]
            }
            KeyCode::Char('u') if mods.contains(KeyModifiers::CONTROL) => {
                let count = self.effective_count();
                self.reset_pending();
                vec![Command::ScrollHalfPageUp(count)]
            }
            KeyCode::Char('f') if mods.contains(KeyModifiers::CONTROL) => {
                let count = self.effective_count();
                self.reset_pending();
                vec![Command::ScrollFullPageDown(count)]
            }
            KeyCode::Char('b') if mods.contains(KeyModifiers::CONTROL) => {
                let count = self.effective_count();
                self.reset_pending();
                vec![Command::ScrollFullPageUp(count)]
            }

            // Jumplist: Ctrl-o back, Ctrl-i / Tab forward
            KeyCode::Char('o') if mods.contains(KeyModifiers::CONTROL) => {
                let count = self.effective_count();
                self.reset_pending();
                vec![Command::JumpListBack(count)]
            }
            KeyCode::Char('i') if mods.contains(KeyModifiers::CONTROL) => {
                let count = self.effective_count();
                self.reset_pending();
                vec![Command::JumpListForward(count)]
            }
            KeyCode::Tab => {
                let count = self.effective_count();
                self.reset_pending();
                vec![Command::JumpListForward(count)]
            }

            // z prefix: zz / zt / zb viewport positioning
            KeyCode::Char('z') if mods.is_empty() => {
                self.awaiting = Awaiting::ZPrefix;
                self.command_buffer.push('z');
                vec![]
            }

            // Toggle sidebar
            KeyCode::Char('g') if mods.contains(KeyModifiers::CONTROL) => {
                self.reset_pending();
                vec![Command::ToggleSidebar]
            }

            // gg (first g)
            KeyCode::Char('g') if mods.is_empty() => {
                self.awaiting = Awaiting::SecondG;
                self.command_buffer.push('g');
                vec![]
            }

            // Ctrl-w (window commands)
            KeyCode::Char('w') if mods.contains(KeyModifiers::CONTROL) => {
                self.awaiting = Awaiting::CtrlW;
                self.command_buffer.push_str("^W");
                vec![]
            }

            // ZZ / ZQ (second Z matches this arm again, so resolve here)
            KeyCode::Char('Z') => {
                if self.command_buffer == "Z" {
                    self.reset_pending();
                    return vec![Command::SaveAndQuit];
                }
                self.command_buffer.push('Z');
                vec![]
            }
            KeyCode::Char('Q') if self.command_buffer == "Z" => {
                self.reset_pending();
                vec![Command::QuitNoSave]
            }

            // Arrow keys for movement
            KeyCode::Left => self.motion(MotionKind::Left),
            KeyCode::Right => self.motion(MotionKind::Right),
            KeyCode::Up => self.motion(MotionKind::Up),
            KeyCode::Down => self.motion(MotionKind::Down),

            // Escape always resets
            KeyCode::Esc => {
                self.reset_pending();
                vec![]
            }

            _ => {
                // Check for ZZ / ZQ continuation
                if self.command_buffer == "Z" {
                    if let KeyCode::Char('Z') = code {
                        self.reset_pending();
                        return vec![Command::SaveAndQuit];
                    } else if let KeyCode::Char('Q') = code {
                        self.reset_pending();
                        return vec![Command::QuitNoSave];
                    }
                }
                // Unrecognized key, reset
                self.reset_pending();
                vec![]
            }
        }
    }

    /// Flush the count buffer into the appropriate count slot (count1 or count2).
    fn flush_count(&mut self) {
        if self.count_buffer.is_empty() {
            return;
        }
        let val: usize = self.count_buffer.parse().unwrap_or(1);
        if self.operator.is_some() {
            self.count2 = Some(val);
        } else {
            self.count1 = Some(val);
        }
        self.count_buffer.clear();
    }

    /// Handle an operator key (d, y, c, >, <).
    /// If no operator is pending, set it and wait for a motion.
    /// If the same operator is already pending, it's a line-wise operation.
    fn handle_operator_key(&mut self, op: Operator, ch: char) -> Vec<Command> {
        if let Some(ref pending_op) = self.operator {
            if *pending_op == op {
                // Double operator: dd, yy, cc, >>, <<
                let count = self.effective_count();
                let reg = self.register;
                let cmd = match op {
                    // A named register on a line delete/change (e.g. "add,
                    // "_dd) must reach the register store — the legacy
                    // *Line commands don't carry one, so those go through
                    // OperatorLines instead.
                    Operator::Delete if reg.is_none() => Command::DemolishLine(count),
                    Operator::Change if reg.is_none() => Command::ChangeLine(count),
                    Operator::Yank => Command::YankLine(count, reg),
                    Operator::RotateCW => Command::RotateCWLine(count),
                    Operator::RotateCCW => Command::RotateCCWLine(count),
                    _ => Command::OperatorLines(op, count, reg),
                };
                self.reset_pending();
                return vec![cmd];
            }
        }
        // Set operator and wait for motion
        self.operator = Some(op);
        self.awaiting = Awaiting::Motion;
        self.command_buffer.push(ch);
        vec![]
    }

    /// When a motion key is pressed:
    /// - If an operator is pending, emit OperatorMotion: the input handler
    ///   (which owns the cursor and map) resolves the tile range and applies
    ///   the operator to it.
    /// - Otherwise, produce the plain motion command.
    fn motion(&mut self, kind: MotionKind) -> Vec<Command> {
        let count = self.effective_count();
        if let Some(op) = self.operator.take() {
            let reg = self.register;
            self.reset_pending();
            vec![Command::OperatorMotion(op, kind, count, reg)]
        } else {
            self.reset_pending();
            vec![Self::plain_motion(kind, count)]
        }
    }

    /// Map a MotionKind to its standalone cursor-movement Command.
    fn plain_motion(kind: MotionKind, count: usize) -> Command {
        match kind {
            MotionKind::Left => Command::Move(Direction::Left, count),
            MotionKind::Right => Command::Move(Direction::Right, count),
            MotionKind::Up => Command::Move(Direction::Up, count),
            MotionKind::Down => Command::Move(Direction::Down, count),
            MotionKind::WordForward => Command::JumpNextEntity(count),
            MotionKind::WordBackward => Command::JumpPrevEntity(count),
            MotionKind::BigWordForward => Command::JumpNextEntityBig(count),
            MotionKind::BigWordBackward => Command::JumpPrevEntityBig(count),
            MotionKind::WordEnd => Command::JumpEndCluster(count),
            MotionKind::WordEndBack => Command::JumpEndClusterBack(count),
            MotionKind::BigWordEnd => Command::JumpEndClusterBig(count),
            MotionKind::LineStart => Command::LineStart,
            MotionKind::LineEnd => Command::LineEnd,
            MotionKind::FirstEntity => Command::FirstEntityInRow,
            MotionKind::LastEntity => Command::LastEntityInRow,
            MotionKind::Column => Command::JumpColumn(count),
            MotionKind::MapStart(row) => Command::MapStart(row),
            MotionKind::MapEnd(row) => Command::MapEnd(row),
            MotionKind::ViewportTop => Command::ViewportTop,
            MotionKind::ViewportMiddle => Command::ViewportMiddle,
            MotionKind::ViewportBottom => Command::ViewportBottom,
            MotionKind::Find(et, forward) => Command::FindEntity(et, count, forward),
            MotionKind::Til(et, forward) => Command::TilEntity(et, count, forward),
            MotionKind::RepeatFind(same_dir) => Command::RepeatFind(same_dir),
            MotionKind::NextParagraph => Command::NextParagraph(count),
            MotionKind::PrevParagraph => Command::PrevParagraph(count),
            MotionKind::NextMachine => Command::JumpNextMachine(count),
            MotionKind::PrevMachine => Command::JumpPrevMachine(count),
            MotionKind::RowDown => Command::FirstEntityRowDown(count),
            MotionKind::RowUp => Command::FirstEntityRowUp(count),
            MotionKind::MatchConnection => Command::MatchConnection,
            MotionKind::SearchNext => Command::SearchNext(count),
            MotionKind::SearchPrev => Command::SearchPrev(count),
        }
    }

    /// Handle the key after `z`: viewport positioning (zz/zt/zb) and
    /// tile zoom (zi = zoom in, zo = zoom out, zf = re-fit whole map).
    fn handle_z_prefix(&mut self, key: KeyEvent) -> Vec<Command> {
        let cmd = match key.code {
            KeyCode::Char('z') => Some(Command::ScrollCenterCursor),
            KeyCode::Char('t') => Some(Command::ScrollCursorTop),
            KeyCode::Char('b') => Some(Command::ScrollCursorBottom),
            KeyCode::Char('i') => Some(Command::ZoomIn),
            KeyCode::Char('o') => Some(Command::ZoomOut),
            KeyCode::Char('f') => Some(Command::ZoomFit),
            _ => None,
        };
        self.reset_pending();
        cmd.into_iter().collect()
    }

    /// Handle the character after f/F/t/T.
    fn handle_find_char(&mut self, key: KeyEvent, forward: bool, til: bool) -> Vec<Command> {
        let ch = match key.code {
            KeyCode::Char(c) => c,
            KeyCode::Esc => {
                self.reset_pending();
                return vec![];
            }
            _ => {
                self.reset_pending();
                return vec![];
            }
        };

        self.command_buffer.push(ch);

        if let Some(entity_type) = EntityType::from_find_char(ch) {
            let kind = if til {
                MotionKind::Til(entity_type, forward)
            } else {
                MotionKind::Find(entity_type, forward)
            };
            self.motion(kind)
        } else {
            self.reset_pending();
            vec![]
        }
    }

    /// Handle the character after m (set mark).
    fn handle_mark_set(&mut self, key: KeyEvent) -> Vec<Command> {
        match key.code {
            KeyCode::Char(c) if c.is_ascii_alphabetic() => {
                self.reset_pending();
                vec![Command::SetMark(c)]
            }
            KeyCode::Esc => {
                self.reset_pending();
                vec![]
            }
            _ => {
                self.reset_pending();
                vec![]
            }
        }
    }

    /// Handle the character after ' or ` (jump to mark).
    fn handle_mark_jump(&mut self, key: KeyEvent, exact: bool) -> Vec<Command> {
        match key.code {
            KeyCode::Char(c) if c.is_ascii_alphabetic() => {
                self.reset_pending();
                if exact {
                    vec![Command::JumpMarkExact(c)]
                } else {
                    vec![Command::JumpMarkRow(c)]
                }
            }
            KeyCode::Char('\'') if !exact => {
                self.reset_pending();
                vec![Command::JumpPrevJumpRow]
            }
            KeyCode::Char('`') if exact => {
                self.reset_pending();
                vec![Command::JumpPrevJumpExact]
            }
            KeyCode::Esc => {
                self.reset_pending();
                vec![]
            }
            _ => {
                self.reset_pending();
                vec![]
            }
        }
    }

    /// Handle the character after " (register select).
    fn handle_register_select(&mut self, key: KeyEvent) -> Vec<Command> {
        match key.code {
            KeyCode::Char(c) if c.is_ascii_alphanumeric() || c == '"' || c == '_' => {
                self.register = Some(c);
                self.awaiting = Awaiting::Nothing;
                self.command_buffer.push(c);
                vec![]
            }
            KeyCode::Esc => {
                self.reset_pending();
                vec![]
            }
            _ => {
                self.reset_pending();
                vec![]
            }
        }
    }

    /// Handle the key after g: gg, ge, gE, gu, gU, g~, gi, gv, g0, g$, g_.
    fn handle_second_g(&mut self, key: KeyEvent) -> Vec<Command> {
        match key.code {
            KeyCode::Char('g') => {
                let row = self.count1.or(self.count2);
                self.awaiting = Awaiting::Nothing;
                self.motion(MotionKind::MapStart(row))
            }
            KeyCode::Char('e') => {
                self.awaiting = Awaiting::Nothing;
                self.motion(MotionKind::WordEndBack)
            }
            KeyCode::Char('E') => {
                self.awaiting = Awaiting::Nothing;
                self.motion(MotionKind::WordEndBack)
            }
            KeyCode::Char('0') => {
                self.awaiting = Awaiting::Nothing;
                self.motion(MotionKind::LineStart)
            }
            KeyCode::Char('$') => {
                self.awaiting = Awaiting::Nothing;
                self.motion(MotionKind::LineEnd)
            }
            KeyCode::Char('_') => {
                self.awaiting = Awaiting::Nothing;
                self.motion(MotionKind::LastEntity)
            }
            // g-operators: gu (downgrade), gU (upgrade), g~ (rotate 180).
            // These behave like d/y/c: they wait for a motion, a text
            // object, or their doubled linewise form (guu/gUU/g~~).
            KeyCode::Char('u') if self.mode == Mode::Normal => {
                self.operator = Some(Operator::Downgrade);
                self.awaiting = Awaiting::Motion;
                self.command_buffer.push('u');
                vec![]
            }
            KeyCode::Char('U') if self.mode == Mode::Normal => {
                self.operator = Some(Operator::Upgrade);
                self.awaiting = Awaiting::Motion;
                self.command_buffer.push('U');
                vec![]
            }
            KeyCode::Char('~') if self.mode == Mode::Normal => {
                self.operator = Some(Operator::Rotate180);
                self.awaiting = Awaiting::Motion;
                self.command_buffer.push('~');
                vec![]
            }
            // gi: re-enter insert at the last insert position
            KeyCode::Char('i') if self.mode == Mode::Normal => {
                self.reset_pending();
                self.mode = Mode::Insert;
                self.insert_count = 1;
                vec![Command::JumpLastInsert, Command::EnterInsert(1)]
            }
            // gv: reselect last visual selection (handler restores kind/mode)
            KeyCode::Char('v') if self.mode == Mode::Normal => {
                self.reset_pending();
                vec![Command::ReselectVisual]
            }
            KeyCode::Esc => {
                self.reset_pending();
                vec![]
            }
            _ => {
                self.reset_pending();
                vec![]
            }
        }
    }

    /// Handle the character after q (start macro recording).
    fn handle_macro_record_start(&mut self, key: KeyEvent) -> Vec<Command> {
        match key.code {
            KeyCode::Char(c) if c.is_ascii_lowercase() => {
                self.reset_pending();
                self.recording_macro = Some(c);
                self.macro_keystrokes.clear();
                vec![Command::StartMacro(c)]
            }
            KeyCode::Esc => {
                self.reset_pending();
                vec![]
            }
            _ => {
                self.reset_pending();
                vec![]
            }
        }
    }

    /// Handle the character after @ (play macro).
    fn handle_macro_play(&mut self, key: KeyEvent) -> Vec<Command> {
        match key.code {
            KeyCode::Char(c) if c.is_ascii_lowercase() => {
                let count = self.effective_count();
                self.reset_pending();
                vec![Command::PlayMacro(c, count)]
            }
            KeyCode::Char('@') => {
                let count = self.effective_count();
                self.reset_pending();
                vec![Command::PlayLastMacro(count)]
            }
            // @: — repeat the last command-line command
            KeyCode::Char(':') => {
                self.reset_pending();
                if let Some(last) = self.command_history.last().cloned() {
                    self.parse_command_line(&last)
                } else {
                    vec![]
                }
            }
            KeyCode::Esc => {
                self.reset_pending();
                vec![]
            }
            _ => {
                self.reset_pending();
                vec![]
            }
        }
    }

    /// Handle the character after r (replace entity).
    fn handle_replace_char(&mut self, key: KeyEvent) -> Vec<Command> {
        match key.code {
            KeyCode::Char(c) => {
                self.reset_pending();
                if let Some(entity_type) = EntityType::from_find_char(c) {
                    vec![Command::ReplaceEntity(entity_type)]
                } else {
                    vec![]
                }
            }
            KeyCode::Esc => {
                self.reset_pending();
                vec![]
            }
            _ => {
                self.reset_pending();
                vec![]
            }
        }
    }

    /// Handle the character after Ctrl-w (window/split commands).
    fn handle_ctrl_w(&mut self, key: KeyEvent) -> Vec<Command> {
        let cmd = match key.code {
            KeyCode::Char('v') => Some(Command::SplitVertical),
            KeyCode::Char('s') => Some(Command::SplitHorizontal),
            KeyCode::Char('h') => Some(Command::FocusPane(Direction::Left)),
            KeyCode::Char('j') => Some(Command::FocusPane(Direction::Down)),
            KeyCode::Char('k') => Some(Command::FocusPane(Direction::Up)),
            KeyCode::Char('l') => Some(Command::FocusPane(Direction::Right)),
            KeyCode::Char('q') => Some(Command::ClosePane),
            KeyCode::Char('c') => Some(Command::ClosePane),
            KeyCode::Char('o') => Some(Command::CloseOtherPanes),
            KeyCode::Char('=') => Some(Command::EqualizePanes),
            KeyCode::Char('w') => Some(Command::CyclePane),
            KeyCode::Char('x') => Some(Command::SwapPanes),
            KeyCode::Char('r') => Some(Command::RotatePanes),
            _ => None,
        };
        self.reset_pending();
        cmd.into_iter().collect()
    }

    /// Handle text object resolution after operator + i/a + object key.
    /// Produces a TextObjectOp command encoding the operator, inner/around, and object char.
    fn handle_text_object_resolve(&mut self, key: KeyEvent, inner: bool) -> Vec<Command> {
        let obj_char = match key.code {
            KeyCode::Char(c) => c,
            KeyCode::Esc => {
                self.reset_pending();
                return vec![];
            }
            _ => {
                self.reset_pending();
                return vec![];
            }
        };

        // Valid text objects:
        //   w (entity cluster), p (paragraph of rows),
        //   b/B/(/)/[/]/{/}/</> (wall-enclosed block — all bracket flavors),
        //   s (sentence = machine cluster, block alias),
        //   " / ' (belt run), t (machine footprint / + ports)
        if !Self::is_text_object_char(obj_char) {
            self.reset_pending();
            return vec![];
        }

        if let Some(op) = self.operator.take() {
            let reg = self.register;
            self.reset_pending();
            vec![Command::TextObjectOp(op, inner, obj_char, reg)]
        } else {
            self.reset_pending();
            vec![]
        }
    }

    /// The set of supported text-object keys.
    fn is_text_object_char(c: char) -> bool {
        matches!(
            c,
            'w' | 'p' | 's' | 'b' | 'B' | '(' | ')' | '[' | ']' | '{' | '}' | '<' | '>'
                | '"' | '\'' | 't'
        )
    }

    // -----------------------------------------------------------------------
    // Insert mode
    // -----------------------------------------------------------------------

    fn handle_insert(&mut self, key: KeyEvent) -> Vec<Command> {
        let cmds = self.handle_insert_inner(key);
        if self.replace_mode {
            // R mode: every placement becomes replace-and-advance.
            cmds.into_iter()
                .map(|c| match c {
                    Command::PlaceEntity(et) => Command::ReplaceTile(et),
                    other => other,
                })
                .collect()
        } else {
            cmds
        }
    }

    fn handle_insert_inner(&mut self, key: KeyEvent) -> Vec<Command> {
        let mods = key.modifiers;

        // --- Universal keys (work in both stages) ---
        match key.code {
            KeyCode::Esc => {
                if self.insert_category.is_some() {
                    // Stage 2 → back to stage 1
                    self.insert_category = None;
                    return vec![];
                }
                // Stage 1 → normal mode
                self.mode = Mode::Normal;
                self.insert_category = None;
                self.replace_mode = false;
                self.reset_pending();
                return vec![Command::ExitToNormal];
            }
            KeyCode::Backspace => return vec![Command::InsertBackspace],
            // Shift+HJKL: change facing + move (always available)
            KeyCode::Char('H') => {
                return vec![
                    Command::SetInsertFacing(Facing::Left),
                    Command::InsertMoveOnly(Direction::Left),
                ];
            }
            KeyCode::Char('J') => {
                return vec![
                    Command::SetInsertFacing(Facing::Down),
                    Command::InsertMoveOnly(Direction::Down),
                ];
            }
            KeyCode::Char('K') => {
                return vec![
                    Command::SetInsertFacing(Facing::Up),
                    Command::InsertMoveOnly(Direction::Up),
                ];
            }
            KeyCode::Char('L') => {
                return vec![
                    Command::SetInsertFacing(Facing::Right),
                    Command::InsertMoveOnly(Direction::Right),
                ];
            }
            // Arrow keys: change facing + move (always available)
            KeyCode::Left => {
                return vec![
                    Command::SetInsertFacing(Facing::Left),
                    Command::InsertMoveOnly(Direction::Left),
                ];
            }
            KeyCode::Right => {
                return vec![
                    Command::SetInsertFacing(Facing::Right),
                    Command::InsertMoveOnly(Direction::Right),
                ];
            }
            KeyCode::Up => {
                return vec![
                    Command::SetInsertFacing(Facing::Up),
                    Command::InsertMoveOnly(Direction::Up),
                ];
            }
            KeyCode::Down => {
                return vec![
                    Command::SetInsertFacing(Facing::Down),
                    Command::InsertMoveOnly(Direction::Down),
                ];
            }
            _ => {}
        }

        // --- Stage 2: category selected → resolve building from key ---
        if let Some(cat) = self.insert_category {
            if let KeyCode::Char(c) = key.code {
                // hjkl are still movement in category mode
                if mods.is_empty() {
                    match c {
                        'h' => return vec![Command::InsertMoveOnly(Direction::Left)],
                        'j' => return vec![Command::InsertMoveOnly(Direction::Down)],
                        'k' => return vec![Command::InsertMoveOnly(Direction::Up)],
                        'l' => return vec![Command::InsertMoveOnly(Direction::Right)],
                        _ => {}
                    }
                }
                if let Some(entity_type) = resolve_category_key(cat, c) {
                    self.insert_category = None;
                    return vec![Command::PlaceEntity(entity_type)];
                }
            }
            // Unrecognized key in category mode
            return vec![];
        }

        // --- Stage 1: no category → quick-place, category select, or movement ---
        match key.code {
            // hjkl movement
            KeyCode::Char('h') if mods.is_empty() => {
                vec![Command::InsertMoveOnly(Direction::Left)]
            }
            KeyCode::Char('j') if mods.is_empty() => {
                vec![Command::InsertMoveOnly(Direction::Down)]
            }
            KeyCode::Char('k') if mods.is_empty() => {
                vec![Command::InsertMoveOnly(Direction::Up)]
            }
            KeyCode::Char('l') if mods.is_empty() => {
                vec![Command::InsertMoveOnly(Direction::Right)]
            }

            // --- Direct quick-place (lowercase) ---
            KeyCode::Char('c') if mods.is_empty() => {
                vec![Command::PlaceEntity(EntityType::BasicBelt)]
            }
            KeyCode::Char('s') if mods.is_empty() => {
                vec![Command::PlaceEntity(EntityType::Smelter)]
            }
            KeyCode::Char('a') if mods.is_empty() => {
                vec![Command::PlaceEntity(EntityType::Assembler)]
            }
            KeyCode::Char('w') if mods.is_empty() => {
                vec![Command::PlaceEntity(EntityType::Wall)]
            }
            KeyCode::Char('p') if mods.is_empty() => {
                vec![Command::PlaceEntity(EntityType::Pipe)]
            }
            KeyCode::Char('b') if mods.is_empty() => {
                vec![Command::PlaceEntity(EntityType::Splitter)]
            }
            KeyCode::Char('m') if mods.is_empty() => {
                vec![Command::PlaceEntity(EntityType::Merger)]
            }
            KeyCode::Char('u') if mods.is_empty() => {
                vec![Command::PlaceEntity(EntityType::UndergroundEntrance)]
            }
            KeyCode::Char('e') if mods.is_empty() => {
                vec![Command::PlaceEntity(EntityType::CoalGenerator)]
            }
            KeyCode::Char('r') if mods.is_empty() => {
                vec![Command::PlaceEntity(EntityType::ResearchLab)]
            }
            KeyCode::Char('t') if mods.is_empty() => {
                vec![Command::PlaceEntity(EntityType::RailTrack)]
            }
            KeyCode::Char('d') if mods.is_empty() => {
                vec![Command::PlaceEntity(EntityType::WasteDump)]
            }
            KeyCode::Char('g') if mods.is_empty() => {
                vec![Command::PlaceEntity(EntityType::Warehouse)]
            }
            KeyCode::Char('f') if mods.is_empty() => {
                vec![Command::PlaceEntity(EntityType::PumpStation)]
            }
            KeyCode::Char('o') if mods.is_empty() => {
                vec![Command::PlaceEntity(EntityType::DronePort)]
            }
            KeyCode::Char('q') if mods.is_empty() => {
                vec![Command::PlaceEntity(EntityType::PrecisionAssembler)]
            }
            KeyCode::Char('x') if mods.is_empty() => {
                vec![Command::PlaceEntity(EntityType::RecyclingPlant)]
            }
            KeyCode::Char('n') if mods.is_empty() => {
                // Circuit reserved — no-op
                vec![]
            }
            KeyCode::Char('1') if mods.is_empty() => {
                vec![Command::PlaceEntity(EntityType::BasicBelt)]
            }
            KeyCode::Char('2') if mods.is_empty() => {
                vec![Command::PlaceEntity(EntityType::FastBelt)]
            }
            KeyCode::Char('3') if mods.is_empty() => {
                vec![Command::PlaceEntity(EntityType::ExpressBelt)]
            }

            // --- Category browser (uppercase / Shift+letter) ---
            KeyCode::Char('C') => {
                self.insert_category = Some(InsertCategoryKind::Conveyors);
                vec![]
            }
            KeyCode::Char('S') => {
                self.insert_category = Some(InsertCategoryKind::ProcessingT1);
                vec![]
            }
            KeyCode::Char('A') => {
                self.insert_category = Some(InsertCategoryKind::ProcessingT2);
                vec![]
            }
            KeyCode::Char('Q') => {
                self.insert_category = Some(InsertCategoryKind::ProcessingT4);
                vec![]
            }
            KeyCode::Char('E') => {
                self.insert_category = Some(InsertCategoryKind::Energy);
                vec![]
            }
            KeyCode::Char('G') => {
                self.insert_category = Some(InsertCategoryKind::Storage);
                vec![]
            }
            KeyCode::Char('O') => {
                self.insert_category = Some(InsertCategoryKind::Logistics);
                vec![]
            }
            KeyCode::Char('T') => {
                self.insert_category = Some(InsertCategoryKind::Transport);
                vec![]
            }
            KeyCode::Char('R') => {
                self.insert_category = Some(InsertCategoryKind::Research);
                vec![]
            }
            KeyCode::Char('X') => {
                self.insert_category = Some(InsertCategoryKind::Waste);
                vec![]
            }
            KeyCode::Char('D') => {
                self.insert_category = Some(InsertCategoryKind::Utility);
                vec![]
            }
            KeyCode::Char('N') => {
                self.insert_category = Some(InsertCategoryKind::Circuit);
                vec![]
            }
            KeyCode::Char('B') => {
                self.insert_category = Some(InsertCategoryKind::Balancers);
                vec![]
            }
            KeyCode::Char('F') => {
                self.insert_category = Some(InsertCategoryKind::FluidExtras);
                vec![]
            }
            KeyCode::Char('P') => {
                self.insert_category = Some(InsertCategoryKind::Pipes);
                vec![]
            }
            KeyCode::Char('U') => {
                // Direct-place UndergroundExit (not a category)
                vec![Command::PlaceEntity(EntityType::UndergroundExit)]
            }

            _ => vec![],
        }
    }

    // -----------------------------------------------------------------------
    // Visual mode
    // -----------------------------------------------------------------------

    fn handle_visual(&mut self, key: KeyEvent) -> Vec<Command> {
        // Multi-key sequences in visual mode (f/t, gg, text objects, r,
        // block I/A) are dispatched through the same awaiting states.
        match self.awaiting {
            Awaiting::FChar => return self.handle_find_char(key, true, false),
            Awaiting::FCharBack => return self.handle_find_char(key, false, false),
            Awaiting::TChar => return self.handle_find_char(key, true, true),
            Awaiting::TCharBack => return self.handle_find_char(key, false, true),
            Awaiting::SecondG => return self.handle_visual_second_g(key),
            Awaiting::TextObjectInner => return self.handle_visual_text_object(key, true),
            Awaiting::TextObjectAround => return self.handle_visual_text_object(key, false),
            Awaiting::ReplaceChar => return self.handle_visual_replace(key),
            Awaiting::BlockInsertLeft => return self.handle_block_insert(key, false),
            Awaiting::BlockInsertRight => return self.handle_block_insert(key, true),
            Awaiting::RegisterSelect => return self.handle_register_select(key),
            _ => {}
        }

        let mods = key.modifiers;

        // Count accumulation ('0' with empty buffer is the line-start motion)
        if let KeyCode::Char(c) = key.code {
            if c.is_ascii_digit() && !(c == '0' && self.count_buffer.is_empty()) {
                self.count_buffer.push(c);
                return vec![];
            }
        }
        self.flush_count();
        let count = self.effective_count();
        let take_count = |p: &mut Self| {
            p.count1 = None;
            p.count2 = None;
        };

        let motion = |p: &mut Self, cmd: Command| -> Vec<Command> {
            take_count(p);
            vec![cmd]
        };

        match key.code {
            // Motions extend the selection (with counts)
            KeyCode::Char('h') if mods.is_empty() => {
                motion(self, Command::Move(Direction::Left, count))
            }
            KeyCode::Char('j') if mods.is_empty() => {
                motion(self, Command::Move(Direction::Down, count))
            }
            KeyCode::Char('k') if mods.is_empty() => {
                motion(self, Command::Move(Direction::Up, count))
            }
            KeyCode::Char('l') if mods.is_empty() => {
                motion(self, Command::Move(Direction::Right, count))
            }

            KeyCode::Char('w') if mods.is_empty() => {
                motion(self, Command::JumpNextEntity(count))
            }
            KeyCode::Char('W') => motion(self, Command::JumpNextEntityBig(count)),
            KeyCode::Char('b') if mods.is_empty() => {
                motion(self, Command::JumpPrevEntity(count))
            }
            KeyCode::Char('B') => motion(self, Command::JumpPrevEntityBig(count)),
            KeyCode::Char('e') if mods.is_empty() => {
                motion(self, Command::JumpEndCluster(count))
            }
            KeyCode::Char('E') => motion(self, Command::JumpEndClusterBig(count)),

            KeyCode::Char('0') if mods.is_empty() => motion(self, Command::LineStart),
            KeyCode::Char('$') => motion(self, Command::LineEnd),
            KeyCode::Char('^') => motion(self, Command::FirstEntityInRow),
            KeyCode::Char('_') => motion(self, Command::FirstEntityInRow),
            KeyCode::Char('|') => motion(self, Command::JumpColumn(count)),
            KeyCode::Char('+') => motion(self, Command::FirstEntityRowDown(count)),
            KeyCode::Char('(') => motion(self, Command::JumpPrevMachine(count)),
            KeyCode::Char(')') => motion(self, Command::JumpNextMachine(count)),

            KeyCode::Char('G') => {
                let row = self.count1.take();
                self.count2 = None;
                vec![Command::MapEnd(row)]
            }

            KeyCode::Char('}') => motion(self, Command::NextParagraph(count)),
            KeyCode::Char('{') => motion(self, Command::PrevParagraph(count)),
            KeyCode::Char('%') => motion(self, Command::MatchConnection),

            KeyCode::Char('H') if !mods.contains(KeyModifiers::CONTROL) => {
                motion(self, Command::ViewportTop)
            }
            KeyCode::Char('M') if !mods.contains(KeyModifiers::CONTROL) => {
                motion(self, Command::ViewportMiddle)
            }
            KeyCode::Char('L') if !mods.contains(KeyModifiers::CONTROL) => {
                motion(self, Command::ViewportBottom)
            }

            KeyCode::Char('n') if mods.is_empty() => motion(self, Command::SearchNext(count)),
            KeyCode::Char('N') => motion(self, Command::SearchPrev(count)),

            // f/t/F/T and ;/, work in visual mode
            KeyCode::Char('f') if mods.is_empty() => {
                self.awaiting = Awaiting::FChar;
                self.command_buffer.push('f');
                vec![]
            }
            KeyCode::Char('F') => {
                self.awaiting = Awaiting::FCharBack;
                self.command_buffer.push('F');
                vec![]
            }
            KeyCode::Char('t') if mods.is_empty() => {
                self.awaiting = Awaiting::TChar;
                self.command_buffer.push('t');
                vec![]
            }
            KeyCode::Char('T') => {
                self.awaiting = Awaiting::TCharBack;
                self.command_buffer.push('T');
                vec![]
            }
            KeyCode::Char(';') => motion(self, Command::RepeatFind(true)),
            KeyCode::Char(',') => motion(self, Command::RepeatFind(false)),

            KeyCode::Left => motion(self, Command::Move(Direction::Left, count)),
            KeyCode::Right => motion(self, Command::Move(Direction::Right, count)),
            KeyCode::Up => motion(self, Command::Move(Direction::Up, count)),
            KeyCode::Down => motion(self, Command::Move(Direction::Down, count)),

            // Text objects set the selection (viw, vib, va", ...)
            KeyCode::Char('i') if mods.is_empty() && self.mode != Mode::VisualBlock => {
                self.awaiting = Awaiting::TextObjectInner;
                self.command_buffer.push('i');
                vec![]
            }
            KeyCode::Char('a') if mods.is_empty() && self.mode != Mode::VisualBlock => {
                self.awaiting = Awaiting::TextObjectAround;
                self.command_buffer.push('a');
                vec![]
            }
            // Visual-block column insert: I (left edge) / A (right edge)
            KeyCode::Char('I') if self.mode == Mode::VisualBlock => {
                self.awaiting = Awaiting::BlockInsertLeft;
                self.command_buffer.push('I');
                vec![]
            }
            KeyCode::Char('A') if self.mode == Mode::VisualBlock => {
                self.awaiting = Awaiting::BlockInsertRight;
                self.command_buffer.push('A');
                vec![]
            }
            // In non-block visual, i/a are still text-object prefixes; in
            // block mode plain i/a fall through to text objects too.
            KeyCode::Char('i') if mods.is_empty() => {
                self.awaiting = Awaiting::TextObjectInner;
                self.command_buffer.push('i');
                vec![]
            }
            KeyCode::Char('a') if mods.is_empty() => {
                self.awaiting = Awaiting::TextObjectAround;
                self.command_buffer.push('a');
                vec![]
            }

            // Operators on the selection
            KeyCode::Char('d') if mods.is_empty() => {
                self.mode = Mode::Normal;
                self.reset_pending();
                vec![Command::VisualOperator(Operator::Delete)]
            }
            KeyCode::Char('x') if mods.is_empty() => {
                self.mode = Mode::Normal;
                self.reset_pending();
                vec![Command::VisualOperator(Operator::Delete)]
            }
            KeyCode::Char('y') if mods.is_empty() => {
                self.mode = Mode::Normal;
                self.reset_pending();
                vec![Command::VisualOperator(Operator::Yank)]
            }
            KeyCode::Char('c') if mods.is_empty() => {
                self.mode = Mode::Insert;
                self.reset_pending();
                vec![Command::VisualOperator(Operator::Change)]
            }
            KeyCode::Char('>') => {
                self.mode = Mode::Normal;
                self.reset_pending();
                vec![Command::VisualOperator(Operator::RotateCW)]
            }
            KeyCode::Char('<') => {
                self.mode = Mode::Normal;
                self.reset_pending();
                vec![Command::VisualOperator(Operator::RotateCCW)]
            }
            // ~ rotates every selected entity 180 degrees
            KeyCode::Char('~') => {
                self.mode = Mode::Normal;
                self.reset_pending();
                vec![Command::VisualOperator(Operator::Rotate180)]
            }
            // U / u upgrade/downgrade every selected entity one tier
            KeyCode::Char('U') => {
                self.mode = Mode::Normal;
                self.reset_pending();
                vec![Command::VisualOperator(Operator::Upgrade)]
            }
            KeyCode::Char('u') if mods.is_empty() => {
                self.mode = Mode::Normal;
                self.reset_pending();
                vec![Command::VisualOperator(Operator::Downgrade)]
            }

            // r{building}: replace all selected tiles with the keyed building
            KeyCode::Char('r') if mods.is_empty() => {
                self.awaiting = Awaiting::ReplaceChar;
                self.command_buffer.push('r');
                vec![]
            }

            // Swap anchor (O in block mode swaps the corner horizontally)
            KeyCode::Char('o') if mods.is_empty() => {
                vec![Command::VisualSwapAnchor]
            }
            KeyCode::Char('O') => {
                if self.mode == Mode::VisualBlock {
                    vec![Command::VisualSwapCorner]
                } else {
                    vec![Command::VisualSwapAnchor]
                }
            }

            // Paste over selection
            KeyCode::Char('p') if mods.is_empty() => {
                self.mode = Mode::Normal;
                vec![Command::VisualPaste(self.register)]
            }
            // Register select (for "ay etc.)
            KeyCode::Char('"') => {
                self.awaiting = Awaiting::RegisterSelect;
                self.command_buffer.push('"');
                vec![]
            }

            // Switch visual modes
            KeyCode::Char('v') if mods.is_empty() => {
                if self.mode == Mode::Visual {
                    self.mode = Mode::Normal;
                    vec![Command::ExitToNormal]
                } else {
                    self.mode = Mode::Visual;
                    vec![Command::EnterVisual]
                }
            }
            KeyCode::Char('V') => {
                if self.mode == Mode::VisualLine {
                    self.mode = Mode::Normal;
                    vec![Command::ExitToNormal]
                } else {
                    self.mode = Mode::VisualLine;
                    vec![Command::EnterVisualLine]
                }
            }
            KeyCode::Char('v') if mods.contains(KeyModifiers::CONTROL) => {
                if self.mode == Mode::VisualBlock {
                    self.mode = Mode::Normal;
                    vec![Command::ExitToNormal]
                } else {
                    self.mode = Mode::VisualBlock;
                    vec![Command::EnterVisualBlock]
                }
            }

            // Escape cancels visual mode
            KeyCode::Esc => {
                self.mode = Mode::Normal;
                self.reset_pending();
                vec![Command::ExitToNormal]
            }

            // gg in visual
            KeyCode::Char('g') if mods.is_empty() => {
                self.awaiting = Awaiting::SecondG;
                self.command_buffer.push('g');
                vec![]
            }

            _ => vec![],
        }
    }

    /// gg (and g-family) inside visual mode.
    fn handle_visual_second_g(&mut self, key: KeyEvent) -> Vec<Command> {
        self.awaiting = Awaiting::Nothing;
        match key.code {
            KeyCode::Char('g') => {
                let row = self.count1.take();
                self.count_buffer.clear();
                vec![Command::MapStart(row)]
            }
            KeyCode::Char('e') | KeyCode::Char('E') => {
                vec![Command::JumpEndClusterBack(1)]
            }
            KeyCode::Char('_') => vec![Command::LastEntityInRow],
            KeyCode::Char('0') => vec![Command::LineStart],
            KeyCode::Char('$') => vec![Command::LineEnd],
            _ => vec![],
        }
    }

    /// Text objects in visual mode (viw, vab, vi", ...): the selection is
    /// set to the object's range by the handler.
    fn handle_visual_text_object(&mut self, key: KeyEvent, inner: bool) -> Vec<Command> {
        self.awaiting = Awaiting::Nothing;
        let obj_char = match key.code {
            KeyCode::Char(c) if Self::is_text_object_char(c) => c,
            _ => {
                self.command_buffer.clear();
                return vec![];
            }
        };
        self.command_buffer.clear();
        vec![Command::VisualTextObject(inner, obj_char)]
    }

    /// r{building} in visual mode: replace every selected tile.
    fn handle_visual_replace(&mut self, key: KeyEvent) -> Vec<Command> {
        self.awaiting = Awaiting::Nothing;
        self.command_buffer.clear();
        match key.code {
            KeyCode::Char(c) => {
                if let Some(entity_type) = EntityType::from_find_char(c) {
                    self.mode = Mode::Normal;
                    self.reset_pending();
                    vec![Command::VisualReplace(entity_type)]
                } else {
                    vec![]
                }
            }
            _ => vec![],
        }
    }

    /// Building key after I/A in visual-block: classic column insert.
    fn handle_block_insert(&mut self, key: KeyEvent, append: bool) -> Vec<Command> {
        self.awaiting = Awaiting::Nothing;
        self.command_buffer.clear();
        match key.code {
            KeyCode::Char(c) => {
                if let Some(entity_type) = resolve_quickplace_key(c) {
                    self.mode = Mode::Normal;
                    self.reset_pending();
                    vec![Command::VisualBlockInsert(entity_type, append)]
                } else {
                    vec![]
                }
            }
            KeyCode::Esc => {
                self.reset_pending();
                vec![]
            }
            _ => vec![],
        }
    }

    // -----------------------------------------------------------------------
    // Command mode (:)
    // -----------------------------------------------------------------------

    fn handle_command_mode(&mut self, key: KeyEvent) -> Vec<Command> {
        match key.code {
            KeyCode::Enter => {
                let line = self.command_line.clone();
                if !line.is_empty() {
                    self.command_history.push(line.clone());
                }
                self.mode = Mode::Normal;
                self.command_history_idx = None;
                self.parse_command_line(&line)
            }
            KeyCode::Esc => {
                self.mode = Mode::Normal;
                self.command_line.clear();
                self.command_history_idx = None;
                vec![Command::ExitToNormal]
            }
            KeyCode::Backspace => {
                if self.command_line.is_empty() {
                    self.mode = Mode::Normal;
                    vec![Command::ExitToNormal]
                } else {
                    self.command_line.pop();
                    vec![]
                }
            }
            KeyCode::Char(c) => {
                self.command_line.push(c);
                vec![]
            }
            KeyCode::Up => {
                // Command history navigation
                if self.command_history.is_empty() {
                    return vec![];
                }
                let idx = match self.command_history_idx {
                    None => self.command_history.len().saturating_sub(1),
                    Some(i) => i.saturating_sub(1),
                };
                self.command_history_idx = Some(idx);
                if let Some(entry) = self.command_history.get(idx) {
                    self.command_line = entry.clone();
                }
                vec![]
            }
            KeyCode::Down => {
                if let Some(idx) = self.command_history_idx {
                    let new_idx = idx + 1;
                    if new_idx < self.command_history.len() {
                        self.command_history_idx = Some(new_idx);
                        self.command_line = self.command_history[new_idx].clone();
                    } else {
                        self.command_history_idx = None;
                        self.command_line.clear();
                    }
                }
                vec![]
            }
            KeyCode::Tab => {
                // Tab completion for command names
                self.tab_complete_command();
                vec![]
            }
            _ => vec![],
        }
    }

    /// Parse a command-line string (after `:`) into Commands.
    fn parse_command_line(&self, line: &str) -> Vec<Command> {
        // Substitution / global-delete forms first (:s/.., :%s/.., :N,Ms/..,
        // :g/pat/d) — they don't follow the word+argument shape.
        if let Some(cmds) = Self::parse_substitute_command(line.trim()) {
            return cmds;
        }

        let parts: Vec<&str> = line.trim().splitn(2, ' ').collect();
        let cmd = parts[0];
        let arg = parts.get(1).map(|s| s.trim().to_string());

        match cmd {
            "w" | "write" => vec![Command::CmdSave(arg)],
            "q" | "quit" => vec![Command::CmdQuit(false)],
            "q!" | "quit!" => vec![Command::CmdQuit(true)],
            "wq" | "x" => vec![Command::CmdSaveQuit],
            "e" | "edit" => {
                if let Some(path) = arg {
                    vec![Command::CmdLoad(path)]
                } else {
                    vec![]
                }
            }
            "speed" => {
                if let Some(ref a) = arg {
                    if let Ok(n) = a.parse::<u32>() {
                        return vec![Command::CmdSetSpeed(n)];
                    }
                }
                vec![]
            }
            "pause" => vec![Command::CmdPause],
            "resume" | "run" => vec![Command::CmdResume],
            "step" => vec![Command::CmdStep],
            "stats" => vec![Command::CmdStats],
            "registers" | "reg" => vec![Command::CmdRegisters],
            "marks" => vec![Command::CmdMarks],
            "map" | "mapinfo" => vec![Command::CmdMapInfo],
            "help" | "h" => vec![Command::CmdHelp(arg)],
            "level" => {
                let lvl = arg.as_ref().and_then(|a| a.parse::<usize>().ok());
                vec![Command::CmdLevel(lvl)]
            }
            "restart" => vec![Command::CmdRestart],
            "freeplay" => vec![Command::CmdFreeplay],
            "menu" => vec![Command::CmdMenu],
            "noh" | "nohlsearch" => vec![Command::CmdNoHighlight],
            "version" | "ver" => vec![Command::CmdVersion],
            // Split aliases
            "only" | "on" => vec![Command::CloseOtherPanes],
            "sp" | "split" => vec![Command::SplitHorizontal],
            "vsp" | "vsplit" => vec![Command::SplitVertical],
            // Economy / expansion commands
            "contracts" => vec![Command::CmdContracts],
            "market" => vec![Command::CmdMarket],
            "finance" => vec![Command::CmdFinance],
            "loan" => vec![Command::CmdLoan],
            "recipe" => {
                vec![Command::CmdRecipe(arg.as_ref().and_then(|a| a.parse().ok()))]
            }
            "research" => vec![Command::CmdResearch],
            "sell" => vec![Command::CmdSell],
            "campaign" => vec![Command::CmdCampaign],
            "prestige" => vec![Command::CmdPrestige],
            "seed" => vec![Command::CmdSeed],
            "" => vec![],
            other => vec![Command::CmdUnknown(other.to_string())],
        }
    }

    /// Try to parse the line as a substitution-style command.
    /// Returns None when the line is not substitution-shaped so ordinary
    /// command parsing can proceed.
    fn parse_substitute_command(line: &str) -> Option<Vec<Command>> {
        use crate::commands::SubstScope;

        // :g/pattern/d — delete every entity matching pattern
        if let Some(rest) = line.strip_prefix("g/") {
            let mut parts = rest.splitn(2, '/');
            let pattern = parts.next().unwrap_or("").to_string();
            let action = parts.next().unwrap_or("").trim();
            if !pattern.is_empty() && action == "d" {
                return Some(vec![Command::CmdGlobalDelete(pattern)]);
            }
            return Some(vec![]);
        }

        // Determine scope prefix
        let (scope, rest) = if let Some(r) = line.strip_prefix("%s/") {
            (SubstScope::WholeMap, r)
        } else if let Some(r) = line.strip_prefix("s/") {
            (SubstScope::CurrentRow, r)
        } else {
            // :N,Ms/old/new/[g] — row-range substitution (1-indexed rows)
            let idx = line.find("s/")?;
            let range_part = &line[..idx];
            if range_part.is_empty()
                || !range_part
                    .chars()
                    .all(|c| c.is_ascii_digit() || c == ',' || c == ' ')
            {
                return None;
            }
            let mut nums = range_part.splitn(2, ',');
            let a = nums.next()?.trim().parse::<usize>().ok()?;
            let b = nums.next()?.trim().parse::<usize>().ok()?;
            (
                SubstScope::Rows(a.saturating_sub(1), b.saturating_sub(1)),
                &line[idx + 2..],
            )
        };

        let mut parts = rest.split('/');
        let pattern = parts.next().unwrap_or("").to_string();
        let replacement = parts.next().unwrap_or("").to_string();
        let flags = parts.next().unwrap_or("");
        if pattern.is_empty() || replacement.is_empty() {
            return Some(vec![]);
        }
        Some(vec![Command::CmdSubstitute {
            scope,
            pattern,
            replacement,
            global: flags.contains('g'),
        }])
    }

    /// Simple tab completion for command names.
    fn tab_complete_command(&mut self) {
        let candidates = [
            "write", "quit", "wq", "edit", "speed", "pause", "resume", "step", "stats",
            "registers", "marks", "mapinfo", "help", "level", "restart", "freeplay", "menu",
            "nohlsearch", "version",
            "contracts", "market", "finance", "loan", "recipe", "research", "sell",
            "campaign", "prestige", "seed",
        ];
        let prefix = &self.command_line;
        if prefix.is_empty() {
            return;
        }
        let matches: Vec<&&str> = candidates
            .iter()
            .filter(|c| c.starts_with(prefix.as_str()))
            .collect();
        if matches.len() == 1 {
            self.command_line = matches[0].to_string();
        }
    }

    // -----------------------------------------------------------------------
    // Search mode (/ ?)
    // -----------------------------------------------------------------------

    fn handle_search_mode(&mut self, key: KeyEvent) -> Vec<Command> {
        match key.code {
            KeyCode::Enter => {
                let pattern = self.search_line.clone();
                if !pattern.is_empty() {
                    self.search_history.push(pattern.clone());
                }
                self.mode = Mode::Normal;
                self.search_history_idx = None;
                self.search_line.clear();
                if pattern.is_empty() {
                    vec![Command::ExitToNormal]
                } else {
                    // Hand the typed pattern to the handler, which resolves it
                    // to an entity type, collects matches, and jumps to the
                    // first one (n/N then step through the same match list).
                    vec![Command::ExecuteSearch(pattern, self.search_forward)]
                }
            }
            KeyCode::Esc => {
                self.mode = Mode::Normal;
                self.search_line.clear();
                self.search_history_idx = None;
                vec![Command::ExitToNormal]
            }
            KeyCode::Backspace => {
                if self.search_line.is_empty() {
                    self.mode = Mode::Normal;
                    vec![Command::ExitToNormal]
                } else {
                    self.search_line.pop();
                    vec![]
                }
            }
            KeyCode::Char(c) => {
                self.search_line.push(c);
                vec![]
            }
            KeyCode::Up => {
                if self.search_history.is_empty() {
                    return vec![];
                }
                let idx = match self.search_history_idx {
                    None => self.search_history.len().saturating_sub(1),
                    Some(i) => i.saturating_sub(1),
                };
                self.search_history_idx = Some(idx);
                if let Some(entry) = self.search_history.get(idx) {
                    self.search_line = entry.clone();
                }
                vec![]
            }
            KeyCode::Down => {
                if let Some(idx) = self.search_history_idx {
                    let new_idx = idx + 1;
                    if new_idx < self.search_history.len() {
                        self.search_history_idx = Some(new_idx);
                        self.search_line = self.search_history[new_idx].clone();
                    } else {
                        self.search_history_idx = None;
                        self.search_line.clear();
                    }
                }
                vec![]
            }
            _ => vec![],
        }
    }

    // -----------------------------------------------------------------------
    // Public accessors
    // -----------------------------------------------------------------------

    /// Get the current mode.
    pub fn current_mode(&self) -> Mode {
        self.mode
    }

    /// Get the pending register.
    pub fn pending_register(&self) -> Option<char> {
        self.register
    }

    /// Get the pending operator.
    pub fn pending_operator(&self) -> Option<&Operator> {
        self.operator.as_ref()
    }

    /// Whether the parser is mid-way through a multi-key sequence
    /// (pending operator, count, register, or awaiting state). Used by the
    /// input handler's dot-repeat keystroke recorder.
    pub fn has_pending(&self) -> bool {
        self.awaiting != Awaiting::Nothing
            || self.operator.is_some()
            || self.register.is_some()
            || self.count1.is_some()
            || !self.count_buffer.is_empty()
            || self.command_buffer.starts_with('Z')
    }

    /// Check if we are recording a macro.
    pub fn is_recording(&self) -> bool {
        self.recording_macro.is_some()
    }

    /// Stop macro recording and return the collected keystrokes.
    pub fn stop_recording(&mut self) -> Option<(char, Vec<KeyEvent>)> {
        if let Some(reg) = self.recording_macro.take() {
            let keys = std::mem::take(&mut self.macro_keystrokes);
            Some((reg, keys))
        } else {
            None
        }
    }

    /// Get the current insert category name (for status bar display).
    pub fn insert_category_name(&self) -> Option<&'static str> {
        self.insert_category.map(|c| c.name())
    }

    /// Get a string describing the awaiting state for status bar display.
    pub fn status_info(&self) -> String {
        let mut s = String::new();
        if let Some(ref reg) = self.register {
            s.push('"');
            s.push(*reg);
        }
        if let Some(c) = self.count1 {
            s.push_str(&c.to_string());
        }
        if let Some(ref op) = self.operator {
            s.push_str(match op {
                Operator::Delete => "d",
                Operator::Yank => "y",
                Operator::Change => "c",
                Operator::RotateCW => ">",
                Operator::RotateCCW => "<",
                Operator::Upgrade => "gU",
                Operator::Downgrade => "gu",
                Operator::Rotate180 => "g~",
            });
        }
        if !self.count_buffer.is_empty() {
            s.push_str(&self.count_buffer);
        }
        s
    }
}
