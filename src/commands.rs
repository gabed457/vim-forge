use serde::{Deserialize, Serialize};

use crate::resources::{Direction, EntityType, Facing};

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Operator {
    Delete,
    Yank,
    Change,
    RotateCW,
    RotateCCW,
    /// gU — upgrade every upgradable entity in range one tier.
    Upgrade,
    /// gu — downgrade every downgradable entity in range one tier.
    Downgrade,
    /// g~ — rotate every entity in range 180 degrees.
    Rotate180,
}

/// Every motion that can follow an operator (or stand alone).
/// The parser classifies the keystroke; the input handler (which owns the
/// cursor and the map) resolves the actual destination tile.
///
/// Range semantics when combined with an operator:
/// - Linewise motions (Up/Down/MapStart/MapEnd/paragraphs/viewport rows/
///   RowDown/RowUp): whole rows between cursor row and destination row.
/// - Charwise motions: the 2D reading-order span between cursor and
///   destination, exactly like a visual-char selection between the two
///   points. Inclusive motions (e/E/f/t/$/%/g_) include the far endpoint;
///   exclusive motions (w/b/h/l/0/^/|/F/T/(/)/n/N/ge) exclude the later
///   endpoint in reading order — matching vim's inclusive/exclusive rules.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MotionKind {
    Left,
    Right,
    Up,
    Down,
    WordForward,
    WordBackward,
    BigWordForward,
    BigWordBackward,
    WordEnd,
    WordEndBack,
    BigWordEnd,
    LineStart,
    LineEnd,
    FirstEntity,
    LastEntity,
    Column,
    MapStart(Option<usize>),
    MapEnd(Option<usize>),
    ViewportTop,
    ViewportMiddle,
    ViewportBottom,
    Find(EntityType, bool),
    Til(EntityType, bool),
    RepeatFind(bool),
    NextParagraph,
    PrevParagraph,
    NextMachine,
    PrevMachine,
    RowDown,
    RowUp,
    MatchConnection,
    SearchNext,
    SearchPrev,
}

/// Row scope for :s substitution commands.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SubstScope {
    /// `:s/old/new/` — cursor row only.
    CurrentRow,
    /// `:%s/old/new/` — every row.
    WholeMap,
    /// `:N,Ms/old/new/` — rows N..=M (0-indexed, already converted).
    Rows(usize, usize),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Range {
    pub tiles: Vec<(usize, usize)>,
    pub linewise: bool,
}

impl Range {
    pub fn empty() -> Self {
        Range {
            tiles: vec![],
            linewise: false,
        }
    }

    pub fn single(x: usize, y: usize) -> Self {
        Range {
            tiles: vec![(x, y)],
            linewise: false,
        }
    }

    pub fn horizontal(y: usize, x_start: usize, x_end: usize) -> Self {
        let (lo, hi) = if x_start <= x_end {
            (x_start, x_end)
        } else {
            (x_end, x_start)
        };
        Range {
            tiles: (lo..=hi).map(|x| (x, y)).collect(),
            linewise: false,
        }
    }

    pub fn linewise_rows(y_start: usize, y_end: usize, map_width: usize) -> Self {
        let (lo, hi) = if y_start <= y_end {
            (y_start, y_end)
        } else {
            (y_end, y_start)
        };
        let mut tiles = Vec::new();
        for y in lo..=hi {
            for x in 0..map_width {
                tiles.push((x, y));
            }
        }
        Range {
            tiles,
            linewise: true,
        }
    }

    pub fn block(x1: usize, y1: usize, x2: usize, y2: usize) -> Self {
        let (lx, hx) = if x1 <= x2 { (x1, x2) } else { (x2, x1) };
        let (ly, hy) = if y1 <= y2 { (y1, y2) } else { (y2, y1) };
        let mut tiles = Vec::new();
        for y in ly..=hy {
            for x in lx..=hx {
                tiles.push((x, y));
            }
        }
        Range {
            tiles,
            linewise: false,
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct BlueprintEntity {
    pub offset_x: usize,
    pub offset_y: usize,
    pub entity_type: EntityType,
    pub facing: Facing,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Blueprint {
    pub entities: Vec<BlueprintEntity>,
    pub width: usize,
    pub height: usize,
    pub linewise: bool,
}

impl Blueprint {
    pub fn empty() -> Self {
        Blueprint {
            entities: vec![],
            width: 0,
            height: 0,
            linewise: false,
        }
    }

    pub fn is_empty(&self) -> bool {
        self.entities.is_empty()
    }

    pub fn summary(&self) -> String {
        format!("{}x{} ({})", self.width, self.height, self.entities.len())
    }
}

#[derive(Clone, Debug)]
pub enum RegisterContent {
    Blueprint(Blueprint),
    Macro(Vec<crossterm::event::KeyEvent>),
}

#[derive(Clone, Debug, PartialEq)]
pub enum Command {
    // Movement
    Move(Direction, usize),
    JumpNextEntity(usize),
    JumpNextEntityBig(usize),
    JumpPrevEntity(usize),
    JumpPrevEntityBig(usize),
    JumpEndCluster(usize),
    JumpEndClusterBack(usize),
    JumpEndClusterBig(usize),
    JumpNextMachine(usize),
    JumpPrevMachine(usize),
    JumpColumn(usize),
    FirstEntityRowDown(usize),
    FirstEntityRowUp(usize),
    LastEntityInRow,
    LineStart,
    LineEnd,
    FirstEntityInRow,
    MapStart(Option<usize>),
    MapEnd(Option<usize>),
    ViewportTop,
    ViewportMiddle,
    ViewportBottom,
    FindEntity(EntityType, usize, bool),
    TilEntity(EntityType, usize, bool),
    NextParagraph(usize),
    PrevParagraph(usize),
    MatchConnection,
    RepeatFind(bool),

    /// Operator + motion (e.g. `dw`, `d3l`, `dfs`, `c$`, `gUw`).
    /// The handler resolves the motion into a concrete Range and applies
    /// the operator: (operator, motion, count, register).
    OperatorMotion(Operator, MotionKind, usize, Option<char>),
    /// Linewise operator over `count` rows starting at the cursor row,
    /// used by the g-operators (`guu`, `gUU`, `g~~`).
    OperatorLines(Operator, usize, Option<char>),

    // Operators applied to ranges
    Demolish(Range),
    Yank(Range, Option<char>),
    Change(Range),
    RotateCW(Range),
    RotateCCW(Range),
    DemolishLine(usize),
    YankLine(usize, Option<char>),
    ChangeLine(usize),
    RotateCWLine(usize),
    RotateCCWLine(usize),

    // Paste
    Paste(Option<char>, usize, bool),

    // Marks
    SetMark(char),
    JumpMarkRow(char),
    JumpMarkExact(char),
    JumpPrevJumpRow,
    JumpPrevJumpExact,

    // Macros
    StartMacro(char),
    StopMacro,
    PlayMacro(char, usize),
    PlayLastMacro(usize),

    // Mode changes
    EnterInsert(usize),
    EnterVisual,
    EnterVisualLine,
    EnterVisualBlock,
    EnterCommand,
    EnterSearch(bool),
    ExitToNormal,

    // Insert mode actions
    PlaceEntity(EntityType),
    SetInsertFacing(Facing),
    InsertBackspace,
    InsertMoveOnly(Direction),

    // Visual mode
    VisualOperator(Operator),
    VisualSwapAnchor,
    VisualPaste(Option<char>),

    // Single-key edits
    ReplaceEntity(EntityType),
    DeleteUnderCursor(usize),
    ToggleFacing,
    RotateEntityUnderCursor,
    /// J — fill the gap between the entity cluster at/right of the cursor
    /// and the next cluster on the same row with right-facing belts.
    JoinClusters(usize),
    /// Ctrl-a — upgrade the tier of the entity under the cursor N times.
    TierUp(usize),
    /// Ctrl-x — downgrade the tier of the entity under the cursor N times.
    TierDown(usize),
    /// Replace-mode placement: replace the tile under the cursor with the
    /// given building and advance the cursor (R mode).
    ReplaceTile(EntityType),
    /// gi — jump to the position of the last insert-mode session.
    JumpLastInsert,

    // Viewport scrolling (zz/zt/zb, Ctrl-d/u/f/b)
    ScrollCenterCursor,
    ScrollCursorTop,
    ScrollCursorBottom,
    ScrollHalfPageDown(usize),
    ScrollHalfPageUp(usize),
    ScrollFullPageDown(usize),
    ScrollFullPageUp(usize),

    // Jumplist (Ctrl-o / Ctrl-i)
    JumpListBack(usize),
    JumpListForward(usize),

    // Visual-mode extensions
    /// Replace every selected tile with the keyed building (visual `r`).
    VisualReplace(EntityType),
    /// Set the visual selection to a text object (`viw`, `vib`, ...).
    VisualTextObject(bool, char),
    /// Visual-block column insert: place the building down the block's
    /// left (I, false) or right (A, true) edge column, one per row.
    VisualBlockInsert(EntityType, bool),
    /// O in visual-block: swap the anchor corner horizontally.
    VisualSwapCorner,
    /// gv — reselect the last visual selection.
    ReselectVisual,

    // Extra pane commands (Ctrl-w w/c/x/r)
    CyclePane,
    SwapPanes,
    RotatePanes,

    // Undo/Redo
    Undo,
    Redo,
    DotRepeat,

    // Search
    SearchNext(usize),
    SearchPrev(usize),
    SearchWordUnderCursor(bool),
    /// Execute a search typed on the `/` or `?` line: pattern text + forward flag.
    ExecuteSearch(String, bool),

    // Splits
    SplitVertical,
    SplitHorizontal,
    FocusPane(Direction),
    ClosePane,
    CloseOtherPanes,
    EqualizePanes,

    // Sidebar
    ToggleSidebar,

    // Save/Quit shortcuts
    SaveAndQuit,
    QuitNoSave,

    // Text object operations (operator + inner/around + object char)
    TextObjectOp(Operator, bool, char, Option<char>),

    // Command mode commands
    CmdSave(Option<String>),
    CmdQuit(bool),
    CmdSaveQuit,
    CmdLoad(String),
    CmdSetSpeed(u32),
    CmdPause,
    CmdResume,
    CmdStep,
    CmdStats,
    CmdRegisters,
    CmdMarks,
    CmdMapInfo,
    CmdHelp(Option<String>),
    CmdLevel(Option<usize>),
    CmdRestart,
    CmdFreeplay,
    CmdMenu,
    CmdNoHighlight,
    CmdVersion,
    /// :s/old/new/[g], :%s/..., :N,Ms/... — entity-type substitution.
    CmdSubstitute {
        scope: SubstScope,
        pattern: String,
        replacement: String,
        global: bool,
    },
    /// :g/pattern/d — delete every entity matching the pattern.
    CmdGlobalDelete(String),
    /// & — repeat the last :s on the current row.
    RepeatSubstitute,

    // Economy / expansion commands
    CmdContracts,
    CmdMarket,
    CmdFinance,
    CmdLoan,
    CmdRecipe(Option<u16>),
    CmdResearch,
    CmdSell,
    CmdCampaign,
    CmdPrestige,
    CmdSeed,
}

impl Command {
    /// Whether this command mutates the factory (used for tutorial
    /// edit-counting and move-budget scoring).
    pub fn is_edit(&self) -> bool {
        matches!(
            self,
            Command::Demolish(_)
                | Command::Change(_)
                | Command::RotateCW(_)
                | Command::RotateCCW(_)
                | Command::DemolishLine(_)
                | Command::ChangeLine(_)
                | Command::RotateCWLine(_)
                | Command::RotateCCWLine(_)
                | Command::Paste(..)
                | Command::PlaceEntity(_)
                | Command::ReplaceEntity(_)
                | Command::DeleteUnderCursor(_)
                | Command::RotateEntityUnderCursor
                | Command::VisualOperator(_)
                | Command::VisualPaste(_)
                | Command::TextObjectOp(..)
                | Command::JoinClusters(_)
                | Command::TierUp(_)
                | Command::TierDown(_)
                | Command::ReplaceTile(_)
                | Command::VisualReplace(_)
                | Command::VisualBlockInsert(..)
                | Command::CmdSubstitute { .. }
                | Command::CmdGlobalDelete(_)
                | Command::RepeatSubstitute
        ) || matches!(
            self,
            Command::OperatorMotion(op, ..) | Command::OperatorLines(op, ..)
                if !matches!(op, Operator::Yank)
        )
    }

    /// Short name used by UseCommands tutorial objectives, if this command
    /// corresponds to a recognizable vim action.
    pub fn tutorial_name(&self) -> Option<&'static str> {
        Some(match self {
            Command::Move(crate::resources::Direction::Left, _) => "h",
            Command::Move(crate::resources::Direction::Down, _) => "j",
            Command::Move(crate::resources::Direction::Up, _) => "k",
            Command::Move(crate::resources::Direction::Right, _) => "l",
            Command::JumpNextEntity(_) => "w",
            Command::JumpNextEntityBig(_) => "W",
            Command::JumpPrevEntity(_) => "b",
            Command::JumpPrevEntityBig(_) => "B",
            Command::JumpEndCluster(_) => "e",
            Command::JumpEndClusterBack(_) => "ge",
            Command::JumpEndClusterBig(_) => "E",
            Command::JumpNextMachine(_) => ")",
            Command::JumpPrevMachine(_) => "(",
            Command::LineStart => "0",
            Command::LineEnd => "$",
            Command::FirstEntityInRow => "^",
            Command::MapStart(_) => "gg",
            Command::MapEnd(_) => "G",
            Command::ViewportTop => "H",
            Command::ViewportMiddle => "M",
            Command::ViewportBottom => "L",
            Command::FindEntity(_, _, true) => "f",
            Command::FindEntity(_, _, false) => "F",
            Command::TilEntity(_, _, true) => "t",
            Command::TilEntity(_, _, false) => "T",
            Command::RepeatFind(true) => ";",
            Command::RepeatFind(false) => ",",
            Command::NextParagraph(_) => "}",
            Command::PrevParagraph(_) => "{",
            Command::MatchConnection => "%",
            Command::Demolish(_) | Command::DemolishLine(_) => "d",
            Command::Yank(..) | Command::YankLine(..) => "y",
            Command::Change(_) | Command::ChangeLine(_) => "c",
            Command::RotateCW(_) | Command::RotateCWLine(_) => ">",
            Command::RotateCCW(_) | Command::RotateCCWLine(_) => "<",
            Command::OperatorMotion(op, ..) | Command::OperatorLines(op, ..) => match op {
                Operator::Delete => "d",
                Operator::Yank => "y",
                Operator::Change => "c",
                Operator::RotateCW => ">",
                Operator::RotateCCW => "<",
                Operator::Upgrade => "gU",
                Operator::Downgrade => "gu",
                Operator::Rotate180 => "g~",
            },
            Command::Paste(_, _, false) => "p",
            Command::Paste(_, _, true) => "P",
            Command::SetMark(_) => "m",
            Command::JumpMarkRow(_) => "'",
            Command::JumpMarkExact(_) => "`",
            Command::StartMacro(_) => "q",
            Command::PlayMacro(..) | Command::PlayLastMacro(_) => "@",
            Command::EnterInsert(_) => "i",
            Command::EnterVisual => "v",
            Command::EnterVisualLine => "V",
            Command::EnterVisualBlock => "ctrl-v",
            Command::DeleteUnderCursor(_) => "x",
            Command::ReplaceEntity(_) => "r",
            Command::RotateEntityUnderCursor => "~",
            Command::Undo => "u",
            Command::Redo => "ctrl-r",
            Command::DotRepeat => ".",
            Command::SearchNext(_) => "n",
            Command::SearchPrev(_) => "N",
            Command::SearchWordUnderCursor(true) => "*",
            Command::SearchWordUnderCursor(false) => "#",
            Command::EnterSearch(true) => "/",
            Command::EnterSearch(false) => "?",
            Command::SplitVertical => "ctrl-w v",
            Command::SplitHorizontal => "ctrl-w s",
            Command::JoinClusters(_) => "J",
            Command::TierUp(_) => "ctrl-a",
            Command::TierDown(_) => "ctrl-x",
            Command::ReplaceTile(_) => "R",
            Command::JumpLastInsert => "gi",
            Command::ScrollCenterCursor => "zz",
            Command::ScrollCursorTop => "zt",
            Command::ScrollCursorBottom => "zb",
            Command::ScrollHalfPageDown(_) => "ctrl-d",
            Command::ScrollHalfPageUp(_) => "ctrl-u",
            Command::ScrollFullPageDown(_) => "ctrl-f",
            Command::ScrollFullPageUp(_) => "ctrl-b",
            Command::JumpColumn(_) => "|",
            Command::FirstEntityRowDown(_) => "+",
            Command::FirstEntityRowUp(_) => "-",
            Command::LastEntityInRow => "g_",
            Command::JumpListBack(_) => "ctrl-o",
            Command::JumpListForward(_) => "ctrl-i",
            Command::VisualReplace(_) => "r",
            Command::VisualBlockInsert(_, true) => "I",
            Command::VisualBlockInsert(_, false) => "A",
            Command::VisualTextObject(true, _) => "i-object",
            Command::VisualTextObject(false, _) => "a-object",
            Command::TextObjectOp(op, ..) => match op {
                Operator::Delete => "d",
                Operator::Yank => "y",
                Operator::Change => "c",
                Operator::RotateCW => ">",
                Operator::RotateCCW => "<",
                Operator::Upgrade => "gU",
                Operator::Downgrade => "gu",
                Operator::Rotate180 => "g~",
            },
            Command::CmdSubstitute { .. } => ":s",
            Command::CmdGlobalDelete(_) => ":g",
            _ => return None,
        })
    }
}
