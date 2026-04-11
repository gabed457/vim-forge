use serde::{Deserialize, Serialize};

use crate::resources::{Direction, EntityType, Facing};

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Operator {
    Delete,
    Yank,
    Change,
    RotateCW,
    RotateCCW,
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
    JumpEndCluster,
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

    // Undo/Redo
    Undo,
    Redo,
    DotRepeat,

    // Search
    SearchNext(usize),
    SearchPrev(usize),
    SearchWordUnderCursor(bool),

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
}
