//! Top-level input handler that:
//! - Takes a KeyEvent and the full app state
//! - Delegates to the VimParser
//! - Executes the resulting Commands against the game state
//! - Handles macro playback loop

use crossterm::event::KeyEvent;

use crate::commands::{Command, MotionKind, Operator, Range, SubstScope};
use crate::game::inventory::Inventory;
use crate::resources::Facing;
use crate::game::undo::UndoStack;
use crate::map::grid::Map;
use crate::resources::EntityType;
use crate::vim::dot::DotRepeat;
use crate::vim::macros::MacroSystem;
use crate::vim::marks::MarkStore;
use crate::vim::parser::{Mode, VimParser};
use crate::vim::registers::RegisterStore;
use crate::vim::search::SearchState;

/// Which kind of visual selection is active.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum VisualKind {
    Char,
    Line,
    Block,
}

/// A motion resolved to a concrete destination plus its vim range class.
#[derive(Clone, Copy, Debug)]
struct ResolvedMotion {
    dest: (usize, usize),
    linewise: bool,
    inclusive: bool,
}

/// Resolve an entity name typed on the command line (:s, :g) to a type.
/// Extends `EntityType::from_search_prefix` with the tier names and other
/// buildings that the shared resolver does not know about.
pub fn resolve_entity_name(name: &str) -> Option<EntityType> {
    let n: String = name
        .trim()
        .to_lowercase()
        .chars()
        .filter(|c| !matches!(c, ' ' | '_' | '-'))
        .collect();
    let mapped = match n.as_str() {
        "belt" | "basicbelt" | "conveyor" => Some(EntityType::BasicBelt),
        "fastbelt" => Some(EntityType::FastBelt),
        "expressbelt" => Some(EntityType::ExpressBelt),
        "smelter" => Some(EntityType::Smelter),
        "kiln" => Some(EntityType::Kiln),
        "press" => Some(EntityType::Press),
        "wall" => Some(EntityType::Wall),
        "reinforcedwall" => Some(EntityType::ReinforcedWall),
        "pipe" => Some(EntityType::Pipe),
        "splitter" => Some(EntityType::Splitter),
        "merger" => Some(EntityType::Merger),
        "assembler" => Some(EntityType::Assembler),
        "wiremill" => Some(EntityType::WireMill),
        "warehouse" => Some(EntityType::Warehouse),
        "turret" => Some(EntityType::Turret),
        "lab" | "researchlab" => Some(EntityType::ResearchLab),
        "advancedlab" => Some(EntityType::AdvancedLab),
        "pumpstation" | "pump" => Some(EntityType::PumpStation),
        "undergroundentrance" => Some(EntityType::UndergroundEntrance),
        "undergroundexit" => Some(EntityType::UndergroundExit),
        _ => None,
    };
    mapped.or_else(|| EntityType::from_search_prefix(name.trim()))
}

/// All the mutable state the input handler needs access to.
pub struct InputState {
    pub parser: VimParser,
    pub registers: RegisterStore,
    pub marks: MarkStore,
    pub macros: MacroSystem,
    pub search: SearchState,
    pub dot: DotRepeat,
    pub cursor_x: usize,
    pub cursor_y: usize,
    pub visual_anchor: Option<(usize, usize)>,
    pub visual_kind: Option<VisualKind>,
    pub sidebar_visible: bool,
    pub status_message: String,
    pub viewport_top: usize,
    pub viewport_height: usize,
    // Last find/til parameters for ; and , repeat
    pub last_find_entity: Option<EntityType>,
    pub last_find_forward: bool,
    pub last_find_til: bool,
    /// Jumplist for Ctrl-o / Ctrl-i (positions before big jumps).
    pub jumplist: Vec<(usize, usize)>,
    pub jump_idx: usize,
    /// Last visual selection (anchor, cursor, kind) for gv.
    pub last_visual: Option<((usize, usize), (usize, usize), VisualKind)>,
    /// Exact tile range set by a visual text object (viw etc.); overrides
    /// the rectangular anchor..cursor selection until the cursor moves.
    pub visual_override: Option<Range>,
    /// Where the last insert session happened (for gi).
    pub last_insert_pos: Option<(usize, usize)>,
    /// Last :s parameters (pattern, replacement, global) for &.
    pub last_subst: Option<(String, String, bool)>,
    // Dot-repeat keystroke recorder: accumulates the keys of the command
    // currently being typed; when the completed command turns out to be an
    // edit, the buffer becomes the new dot-repeat sequence.
    dot_buffer: Vec<KeyEvent>,
    dot_pending_edit: bool,
}

impl InputState {
    pub fn new() -> Self {
        InputState {
            parser: VimParser::new(),
            registers: RegisterStore::new(),
            marks: MarkStore::new(),
            macros: MacroSystem::new(),
            search: SearchState::new(),
            dot: DotRepeat::new(),
            cursor_x: 0,
            cursor_y: 0,
            visual_anchor: None,
            visual_kind: None,
            sidebar_visible: true,
            status_message: String::new(),
            viewport_top: 0,
            viewport_height: 24,
            last_find_entity: None,
            last_find_forward: true,
            last_find_til: false,
            jumplist: Vec::new(),
            jump_idx: 0,
            last_visual: None,
            visual_override: None,
            last_insert_pos: None,
            last_subst: None,
            dot_buffer: Vec::new(),
            dot_pending_edit: false,
        }
    }

    /// Process a single key event, producing and executing commands.
    /// Returns a list of commands that were executed (for external consumers).
    pub fn handle_key(
        &mut self,
        key: KeyEvent,
        map: &mut Map,
        world: &mut hecs::World,
        undo: &mut UndoStack,
        inventory: &mut Inventory,
    ) -> Vec<Command> {
        // Dot-repeat keystroke recorder: collect the keys of the command
        // being typed. Skipped during macro playback and dot replay (those
        // feed the parser directly, not through handle_key).
        self.dot_buffer.push(key);

        let commands = self.parser.handle_key_event(key);
        let mut executed = Vec::new();

        let mut saw_dot_repeat = false;
        for cmd in commands {
            if matches!(cmd, Command::DotRepeat) {
                saw_dot_repeat = true;
            }
            // Command-line edits (:s, :g/d) are repeated with & / @:, not
            // with `.` — vim behaves the same way.
            if cmd.is_edit()
                && !matches!(
                    cmd,
                    Command::CmdSubstitute { .. } | Command::CmdGlobalDelete(_)
                )
            {
                self.dot_pending_edit = true;
            }
            self.execute_command(cmd.clone(), map, world, undo, inventory);
            executed.push(cmd);
        }

        // Decide whether the just-completed key sequence is a repeatable
        // edit. We keep accumulating while the parser is mid-sequence
        // (counts, pending operator, f/t, ...) or while an insert/visual
        // session is open; once we're back in plain normal mode the buffer
        // either becomes the new dot-repeat sequence (if it edited the
        // factory) or is discarded.
        let accumulating = self.parser.has_pending()
            || !matches!(self.parser.mode, Mode::Normal);
        if !accumulating {
            if self.dot_pending_edit && !saw_dot_repeat && !self.dot_buffer.is_empty() {
                self.dot.last_edit = Some(crate::vim::dot::ReplayableCommand::NormalEdit {
                    keystrokes: self.dot_buffer.clone(),
                });
            }
            self.dot_buffer.clear();
            self.dot_pending_edit = false;
        }

        executed
    }

    /// Process a macro playback: feed stored keystrokes back through the parser.
    pub fn play_macro(
        &mut self,
        reg: char,
        count: usize,
        map: &mut Map,
        world: &mut hecs::World,
        undo: &mut UndoStack,
        inventory: &mut Inventory,
    ) {
        let keys = match self.macros.get_playback_keys(reg, &self.registers) {
            Some(k) => k,
            None => return,
        };

        for _ in 0..count {
            if self.macros.recursion_exceeded() {
                self.status_message = "Macro recursion limit reached".to_string();
                break;
            }
            self.macros.enter_playback();
            for key in &keys {
                let cmds = self.parser.handle_key_event(*key);
                for cmd in cmds {
                    self.execute_command(cmd, map, world, undo, inventory);
                }
            }
            self.macros.exit_playback();
        }
        self.macros.reset_recursion();
    }

    /// Execute a single command against the game state.
    fn execute_command(
        &mut self,
        cmd: Command,
        map: &mut Map,
        world: &mut hecs::World,
        undo: &mut UndoStack,
        inventory: &mut Inventory,
    ) {
        // A visual text-object selection is pinned until anything other
        // than a visual operation happens (any motion re-shapes the
        // selection back to anchor..cursor).
        if !matches!(
            cmd,
            Command::VisualOperator(_)
                | Command::VisualPaste(_)
                | Command::VisualReplace(_)
                | Command::VisualTextObject(..)
                | Command::VisualBlockInsert(..)
                | Command::VisualSwapAnchor
                | Command::VisualSwapCorner
        ) {
            self.visual_override = None;
        }

        match cmd {
            // Movement
            Command::Move(dir, count) => {
                let (nx, ny) = crate::vim::motions::move_direction(
                    self.cursor_x,
                    self.cursor_y,
                    dir,
                    count,
                    map.width,
                    map.height,
                );
                self.cursor_x = nx;
                self.cursor_y = ny;
            }
            Command::JumpNextEntity(count) => {
                for _ in 0..count {
                    if let Some((x, y)) = map.find_next_entity(self.cursor_x, self.cursor_y) {
                        self.cursor_x = x;
                        self.cursor_y = y;
                    }
                }
            }
            Command::JumpNextEntityBig(count) => {
                for _ in 0..count {
                    if let Some((x, y)) =
                        map.find_next_entity_big(world, self.cursor_x, self.cursor_y)
                    {
                        self.cursor_x = x;
                        self.cursor_y = y;
                    }
                }
            }
            Command::JumpPrevEntity(count) => {
                for _ in 0..count {
                    if let Some((x, y)) = map.find_prev_entity(self.cursor_x, self.cursor_y) {
                        self.cursor_x = x;
                        self.cursor_y = y;
                    }
                }
            }
            Command::JumpPrevEntityBig(count) => {
                for _ in 0..count {
                    if let Some((x, y)) =
                        map.find_prev_entity_big(world, self.cursor_x, self.cursor_y)
                    {
                        self.cursor_x = x;
                        self.cursor_y = y;
                    }
                }
            }
            Command::JumpEndCluster(count) => {
                let (x, y) = self.end_cluster_dest(map, count, false);
                self.cursor_x = x;
                self.cursor_y = y;
            }
            Command::JumpEndClusterBig(count) => {
                let (x, y) = self.end_cluster_dest(map, count, true);
                self.cursor_x = x;
                self.cursor_y = y;
            }
            Command::JumpEndClusterBack(count) => {
                let (x, y) = self.prev_cluster_end_dest(map, count);
                self.cursor_x = x;
                self.cursor_y = y;
            }
            Command::JumpNextMachine(count) => {
                if let Some((x, y)) = self.machine_dest(map, world, count, true) {
                    self.push_jump();
                    self.cursor_x = x;
                    self.cursor_y = y;
                }
            }
            Command::JumpPrevMachine(count) => {
                if let Some((x, y)) = self.machine_dest(map, world, count, false) {
                    self.push_jump();
                    self.cursor_x = x;
                    self.cursor_y = y;
                }
            }
            Command::JumpColumn(col) => {
                self.cursor_x = col.saturating_sub(1).min(map.width.saturating_sub(1));
            }
            Command::FirstEntityRowDown(count) => {
                self.cursor_y = (self.cursor_y + count).min(map.height.saturating_sub(1));
                self.cursor_x = map.first_entity_in_row(self.cursor_y).unwrap_or(0);
            }
            Command::FirstEntityRowUp(count) => {
                self.cursor_y = self.cursor_y.saturating_sub(count);
                self.cursor_x = map.first_entity_in_row(self.cursor_y).unwrap_or(0);
            }
            Command::LastEntityInRow => {
                if let Some(x) = self.last_entity_in_row(map, self.cursor_y) {
                    self.cursor_x = x;
                }
            }
            Command::LineStart => {
                self.cursor_x = 0;
            }
            Command::LineEnd => {
                self.cursor_x = map.width.saturating_sub(1);
            }
            Command::FirstEntityInRow => {
                if let Some(x) = map.first_entity_in_row(self.cursor_y) {
                    self.cursor_x = x;
                }
            }
            Command::MapStart(row) => {
                let (x, y) = crate::vim::motions::map_start(row, map.height);
                self.marks
                    .set_prev_jump(self.cursor_x, self.cursor_y);
                self.push_jump();
                self.cursor_x = x;
                self.cursor_y = y;
            }
            Command::MapEnd(row) => {
                let (x, y) = crate::vim::motions::map_end(row, map.height);
                self.marks
                    .set_prev_jump(self.cursor_x, self.cursor_y);
                self.push_jump();
                self.cursor_x = x;
                self.cursor_y = y;
            }
            Command::ViewportTop => {
                self.cursor_y = self.viewport_top;
            }
            Command::ViewportMiddle => {
                self.cursor_y = self.viewport_top + self.viewport_height / 2;
                self.cursor_y = self.cursor_y.min(map.height.saturating_sub(1));
            }
            Command::ViewportBottom => {
                self.cursor_y =
                    (self.viewport_top + self.viewport_height.saturating_sub(1))
                        .min(map.height.saturating_sub(1));
            }
            Command::FindEntity(entity_type, count, forward) => {
                // Store for repeat-find (;/,)
                self.last_find_entity = Some(entity_type);
                self.last_find_forward = forward;
                self.last_find_til = false;
                for _ in 0..count {
                    let found = if forward {
                        map.find_entity_type_forward(
                            world,
                            self.cursor_x,
                            self.cursor_y,
                            entity_type,
                        )
                    } else {
                        map.find_entity_type_backward(
                            world,
                            self.cursor_x,
                            self.cursor_y,
                            entity_type,
                        )
                    };
                    if let Some((x, y)) = found {
                        self.cursor_x = x;
                        self.cursor_y = y;
                    }
                }
            }
            Command::TilEntity(entity_type, count, forward) => {
                // Store for repeat-find (;/,)
                self.last_find_entity = Some(entity_type);
                self.last_find_forward = forward;
                self.last_find_til = true;
                // Same as FindEntity but stop one tile before
                for _ in 0..count {
                    let found = if forward {
                        map.find_entity_type_forward(
                            world,
                            self.cursor_x,
                            self.cursor_y,
                            entity_type,
                        )
                    } else {
                        map.find_entity_type_backward(
                            world,
                            self.cursor_x,
                            self.cursor_y,
                            entity_type,
                        )
                    };
                    if let Some((x, y)) = found {
                        // Move one tile back toward the original position
                        if forward {
                            if x > 0 && (x, y) != (self.cursor_x, self.cursor_y) {
                                self.cursor_x = x.saturating_sub(1);
                                self.cursor_y = y;
                            }
                        } else if x + 1 < map.width {
                            self.cursor_x = x + 1;
                            self.cursor_y = y;
                        }
                    }
                }
            }
            Command::NextParagraph(count) => {
                self.push_jump();
                for _ in 0..count {
                    self.cursor_y = map.find_next_paragraph(self.cursor_y);
                }
                self.cursor_x = 0;
            }
            Command::PrevParagraph(count) => {
                self.push_jump();
                for _ in 0..count {
                    self.cursor_y = map.find_prev_paragraph(self.cursor_y);
                }
                self.cursor_x = 0;
            }
            Command::MatchConnection => {
                if let Some((fx, fy)) = self.match_connection_dest(map, world) {
                    self.cursor_x = fx;
                    self.cursor_y = fy;
                }
            }
            Command::RepeatFind(same_dir) => {
                if let Some(entity_type) = self.last_find_entity {
                    let forward = if same_dir {
                        self.last_find_forward
                    } else {
                        !self.last_find_forward
                    };
                    let found = if forward {
                        map.find_entity_type_forward(world, self.cursor_x, self.cursor_y, entity_type)
                    } else {
                        map.find_entity_type_backward(world, self.cursor_x, self.cursor_y, entity_type)
                    };
                    if let Some((x, y)) = found {
                        if self.last_find_til {
                            // Stop one tile before
                            if forward {
                                if x > 0 && (x, y) != (self.cursor_x, self.cursor_y) {
                                    self.cursor_x = x.saturating_sub(1);
                                    self.cursor_y = y;
                                }
                            } else if x + 1 < map.width {
                                self.cursor_x = x + 1;
                                self.cursor_y = y;
                            }
                        } else {
                            self.cursor_x = x;
                            self.cursor_y = y;
                        }
                    }
                }
            }

            // Operators with empty range (from parser; the range must be computed by
            // examining the preceding motion command in the executed list).
            // In practice these are handled as part of motion+operator pairs.
            Command::Demolish(ref range) if range.tiles.is_empty() => {
                // No-op: the actual demolish happens when range is non-empty
            }
            Command::Yank(ref range, _) if range.tiles.is_empty() => {}
            Command::Change(ref range) if range.tiles.is_empty() => {}
            Command::RotateCW(ref range) if range.tiles.is_empty() => {}
            Command::RotateCCW(ref range) if range.tiles.is_empty() => {}

            // Operator + motion: resolve the motion into a concrete tile
            // range (charwise or linewise per vim rules) and apply the
            // operator to it. Cursor moves to the range start for mutating
            // operators; yank leaves it in place.
            Command::OperatorMotion(ref op, kind, count, reg) => {
                let origin = (self.cursor_x, self.cursor_y);
                if let Some(resolved) = self.motion_destination(kind, count, map, world) {
                    let range = if resolved.linewise {
                        let (top, bottom) =
                            crate::vim::motions::linewise_range(origin.1, resolved.dest.1);
                        Range::linewise_rows(top, bottom, map.width)
                    } else {
                        Range {
                            tiles: crate::vim::motions::charwise_span(
                                origin,
                                resolved.dest,
                                resolved.inclusive,
                                map.width,
                            ),
                            linewise: false,
                        }
                    };
                    if !range.tiles.is_empty() {
                        self.apply_operator_to_range(op, &range, reg, map, world, undo, inventory);
                        // Cursor placement per vim: mutating operators move
                        // to the range start; yank stays where it was.
                        if !matches!(op, Operator::Yank) {
                            if resolved.linewise {
                                self.cursor_y = origin.1.min(resolved.dest.1);
                                self.cursor_x = origin.0;
                            } else if let Some(&(sx, sy)) = range.tiles.first() {
                                self.cursor_x = sx;
                                self.cursor_y = sy;
                            }
                        }
                    }
                }
            }

            // Doubled g-operators (guu / gUU / g~~): linewise over count rows.
            Command::OperatorLines(ref op, count, reg) => {
                let range = Range::linewise_rows(
                    self.cursor_y,
                    (self.cursor_y + count)
                        .saturating_sub(1)
                        .min(map.height.saturating_sub(1)),
                    map.width,
                );
                self.apply_operator_to_range(op, &range, reg, map, world, undo, inventory);
            }

            // Operators with actual ranges
            Command::Demolish(ref range) => {
                undo.push_snapshot(world, map, inventory);
                let bp =
                    crate::vim::operators::delete_range(world, map, inventory, range);
                self.registers.set_blueprint(None, bp, false);
            }
            Command::Yank(ref range, reg) => {
                let bp = crate::vim::operators::yank_range(world, map, range);
                self.registers.set_blueprint(reg, bp, true);
            }
            Command::Change(ref range) => {
                undo.push_snapshot(world, map, inventory);
                let bp =
                    crate::vim::operators::delete_range(world, map, inventory, range);
                self.registers.set_blueprint(None, bp, false);
                self.parser.mode = Mode::Insert;
            }
            Command::RotateCW(ref range) => {
                undo.push_snapshot(world, map, inventory);
                crate::vim::operators::rotate_cw_range(world, map, range);
            }
            Command::RotateCCW(ref range) => {
                undo.push_snapshot(world, map, inventory);
                crate::vim::operators::rotate_ccw_range(world, map, range);
            }

            // Line operators
            Command::DemolishLine(count) => {
                undo.push_snapshot(world, map, inventory);
                let range = crate::commands::Range::linewise_rows(
                    self.cursor_y,
                    (self.cursor_y + count).saturating_sub(1).min(map.height.saturating_sub(1)),
                    map.width,
                );
                let bp =
                    crate::vim::operators::delete_range(world, map, inventory, &range);
                self.registers.set_blueprint(None, bp, false);
            }
            Command::YankLine(count, reg) => {
                let range = crate::commands::Range::linewise_rows(
                    self.cursor_y,
                    (self.cursor_y + count).saturating_sub(1).min(map.height.saturating_sub(1)),
                    map.width,
                );
                let bp = crate::vim::operators::yank_range(world, map, &range);
                self.registers.set_blueprint(reg, bp, true);
            }
            Command::ChangeLine(count) => {
                undo.push_snapshot(world, map, inventory);
                let range = crate::commands::Range::linewise_rows(
                    self.cursor_y,
                    (self.cursor_y + count).saturating_sub(1).min(map.height.saturating_sub(1)),
                    map.width,
                );
                let bp =
                    crate::vim::operators::delete_range(world, map, inventory, &range);
                self.registers.set_blueprint(None, bp, false);
                self.parser.mode = Mode::Insert;
            }
            Command::RotateCWLine(count) => {
                undo.push_snapshot(world, map, inventory);
                let range = crate::commands::Range::linewise_rows(
                    self.cursor_y,
                    (self.cursor_y + count).saturating_sub(1).min(map.height.saturating_sub(1)),
                    map.width,
                );
                crate::vim::operators::rotate_cw_range(world, map, &range);
            }
            Command::RotateCCWLine(count) => {
                undo.push_snapshot(world, map, inventory);
                let range = crate::commands::Range::linewise_rows(
                    self.cursor_y,
                    (self.cursor_y + count).saturating_sub(1).min(map.height.saturating_sub(1)),
                    map.width,
                );
                crate::vim::operators::rotate_ccw_range(world, map, &range);
            }

            // Paste
            Command::Paste(reg, count, before) => {
                if let Some(bp) = self.registers.get_blueprint(reg).cloned() {
                    undo.push_snapshot(world, map, inventory);
                    for _ in 0..count {
                        self.paste_blueprint(map, world, &bp, before);
                    }
                }
            }

            // Marks
            Command::SetMark(c) => {
                self.marks.set(c, self.cursor_x, self.cursor_y);
            }
            Command::JumpMarkRow(c) => {
                if let Some((_, y)) = self.marks.get(c) {
                    self.marks
                        .set_prev_jump(self.cursor_x, self.cursor_y);
                    self.push_jump();
                    self.cursor_y = y;
                    // Jump to first entity in that row
                    if let Some(x) = map.first_entity_in_row(y) {
                        self.cursor_x = x;
                    } else {
                        self.cursor_x = 0;
                    }
                }
            }
            Command::JumpMarkExact(c) => {
                if let Some((x, y)) = self.marks.get(c) {
                    self.marks
                        .set_prev_jump(self.cursor_x, self.cursor_y);
                    self.push_jump();
                    self.cursor_x = x;
                    self.cursor_y = y;
                }
            }
            Command::JumpListBack(count) => {
                for _ in 0..count {
                    self.jump_back();
                }
            }
            Command::JumpListForward(count) => {
                for _ in 0..count {
                    self.jump_forward();
                }
            }
            Command::JumpPrevJumpRow => {
                if let Some((_, y)) = self.marks.get_prev_jump() {
                    let old_x = self.cursor_x;
                    let old_y = self.cursor_y;
                    self.cursor_y = y;
                    if let Some(x) = map.first_entity_in_row(y) {
                        self.cursor_x = x;
                    } else {
                        self.cursor_x = 0;
                    }
                    self.marks.set_prev_jump(old_x, old_y);
                }
            }
            Command::JumpPrevJumpExact => {
                if let Some((x, y)) = self.marks.get_prev_jump() {
                    let old_x = self.cursor_x;
                    let old_y = self.cursor_y;
                    self.cursor_x = x;
                    self.cursor_y = y;
                    self.marks.set_prev_jump(old_x, old_y);
                }
            }

            // Macros
            Command::StartMacro(_c) => {
                // Recording already started in the parser
            }
            Command::StopMacro => {
                if let Some((reg, keys)) = self.parser.stop_recording() {
                    self.registers.set_macro(reg, keys);
                }
            }
            Command::PlayMacro(reg, count) => {
                self.play_macro(reg, count, map, world, undo, inventory);
            }
            Command::PlayLastMacro(count) => {
                if let Some(reg) = self.macros.last_played() {
                    self.play_macro(reg, count, map, world, undo, inventory);
                }
            }

            // Mode changes
            Command::EnterInsert(_count) => {
                // Mode already set by parser
                self.dot.start_recording();
                self.last_insert_pos = Some((self.cursor_x, self.cursor_y));
            }
            Command::EnterVisual => {
                self.visual_anchor = Some((self.cursor_x, self.cursor_y));
                self.visual_kind = Some(VisualKind::Char);
            }
            Command::EnterVisualLine => {
                self.visual_anchor = Some((0, self.cursor_y));
                self.visual_kind = Some(VisualKind::Line);
            }
            Command::EnterVisualBlock => {
                self.visual_anchor = Some((self.cursor_x, self.cursor_y));
                self.visual_kind = Some(VisualKind::Block);
            }
            Command::EnterCommand => {
                // Mode already set by parser
            }
            Command::EnterSearch(_forward) => {
                // Mode already set by parser
            }
            Command::ExitToNormal => {
                // Remember the selection for gv before dropping it.
                if let (Some(anchor), Some(kind)) = (self.visual_anchor, self.visual_kind) {
                    self.last_visual =
                        Some((anchor, (self.cursor_x, self.cursor_y), kind));
                }
                self.visual_anchor = None;
                self.visual_kind = None;
                self.visual_override = None;
                if self.dot.is_recording() {
                    self.dot.finish_insert_session();
                }
            }

            // Insert mode actions
            Command::PlaceEntity(entity_type) => {
                let facing = self.parser.insert_facing;
                undo.push_snapshot(world, map, inventory);
                if map.place_entity_on_map(
                    world,
                    self.cursor_x,
                    self.cursor_y,
                    entity_type,
                    facing,
                    true,
                ).is_some() {
                    self.last_insert_pos = Some((self.cursor_x, self.cursor_y));
                    // Advance cursor past the footprint in the facing direction
                    let fp = crate::map::multitile::building_footprint(entity_type)
                        .rotate_to(facing);
                    let (dx, dy) = facing.offset();
                    // Advance by footprint size in the facing direction
                    let advance = match facing {
                        crate::resources::Facing::Right | crate::resources::Facing::Left => fp.width,
                        crate::resources::Facing::Down | crate::resources::Facing::Up => fp.height,
                    };
                    let nx = self.cursor_x as isize + dx * advance as isize;
                    let ny = self.cursor_y as isize + dy * advance as isize;
                    if map.in_bounds_signed(nx, ny) {
                        self.cursor_x = nx as usize;
                        self.cursor_y = ny as usize;
                    }
                }
            }
            Command::SetInsertFacing(facing) => {
                self.parser.insert_facing = facing;
            }
            Command::InsertBackspace => {
                // Undo the last placement (simple undo)
                undo.undo(world, map, inventory);
            }
            Command::InsertMoveOnly(dir) => {
                let (nx, ny) = crate::vim::motions::move_direction(
                    self.cursor_x,
                    self.cursor_y,
                    dir,
                    1,
                    map.width,
                    map.height,
                );
                self.cursor_x = nx;
                self.cursor_y = ny;
            }

            // Visual mode
            Command::VisualOperator(ref op) => {
                if let Some((ax, ay)) = self.visual_anchor {
                    let range = self.compute_visual_range(ax, ay, map);
                    undo.push_snapshot(world, map, inventory);
                    match op {
                        Operator::Delete => {
                            let bp = crate::vim::operators::delete_range(world, map, inventory, &range);
                            self.registers.set_blueprint(None, bp, false);
                        }
                        Operator::Yank => {
                            let bp = crate::vim::operators::yank_range(world, map, &range);
                            self.registers.set_blueprint(None, bp, true);
                        }
                        Operator::Change => {
                            let bp = crate::vim::operators::delete_range(world, map, inventory, &range);
                            self.registers.set_blueprint(None, bp, false);
                        }
                        Operator::RotateCW => {
                            crate::vim::operators::rotate_cw_range(world, map, &range);
                        }
                        Operator::RotateCCW => {
                            crate::vim::operators::rotate_ccw_range(world, map, &range);
                        }
                        Operator::Rotate180 => {
                            crate::vim::operators::rotate_180_range(world, map, &range);
                        }
                        Operator::Upgrade => {
                            crate::vim::operators::tier_change_range(world, map, &range, true);
                        }
                        Operator::Downgrade => {
                            crate::vim::operators::tier_change_range(world, map, &range, false);
                        }
                    }
                    self.end_visual_selection();
                }
            }
            Command::VisualSwapAnchor => {
                if let Some((ax, ay)) = self.visual_anchor {
                    self.visual_anchor = Some((self.cursor_x, self.cursor_y));
                    self.cursor_x = ax;
                    self.cursor_y = ay;
                }
            }
            Command::VisualSwapCorner => {
                // O in block mode: swap the corner horizontally.
                if let Some((ax, ay)) = self.visual_anchor {
                    let cx = self.cursor_x;
                    self.visual_anchor = Some((cx, ay));
                    self.cursor_x = ax;
                }
            }
            Command::VisualPaste(reg) => {
                if let Some((ax, ay)) = self.visual_anchor {
                    let range = self.compute_visual_range(ax, ay, map);
                    undo.push_snapshot(world, map, inventory);
                    // Delete the selection first
                    crate::vim::operators::delete_range(world, map, inventory, &range);
                    // Then paste
                    if let Some(bp) = self.registers.get_blueprint(reg).cloned() {
                        self.paste_blueprint(map, world, &bp, false);
                    }
                    self.end_visual_selection();
                }
            }
            // r{building} in visual: replace every selected tile.
            Command::VisualReplace(entity_type) => {
                if let Some((ax, ay)) = self.visual_anchor {
                    let range = self.compute_visual_range(ax, ay, map);
                    undo.push_snapshot(world, map, inventory);
                    let facing = self.parser.insert_facing;
                    for &(x, y) in &range.tiles {
                        if map.entity_at(x, y).is_some() {
                            crate::vim::operators::collect_resources_at(
                                world, map, inventory, x, y,
                            );
                            map.remove_multitile_entity(world, x, y);
                        }
                    }
                    for &(x, y) in &range.tiles {
                        if map.entity_at(x, y).is_none() {
                            let _ = map.place_multitile_entity(
                                world, x, y, entity_type, facing, true,
                            );
                        }
                    }
                    self.end_visual_selection();
                }
            }
            // viw / vab / vi" ...: pin the selection to the text object.
            Command::VisualTextObject(inner, obj_char) => {
                let range = self.text_object_range(inner, obj_char, map, world);
                if !range.tiles.is_empty() {
                    if let (Some(&min), Some(&max)) =
                        (range.tiles.first(), range.tiles.last())
                    {
                        self.visual_anchor = Some(min);
                        self.cursor_x = max.0;
                        self.cursor_y = max.1;
                    }
                    self.visual_override = Some(range);
                }
            }
            // Visual-block I/A column insert: one building per block row,
            // down the left (I) or right (A) edge column.
            Command::VisualBlockInsert(entity_type, append) => {
                if let Some((ax, ay)) = self.visual_anchor {
                    let min_x = ax.min(self.cursor_x);
                    let max_x = ax.max(self.cursor_x);
                    let min_y = ay.min(self.cursor_y);
                    let max_y = ay.max(self.cursor_y);
                    let col = if append { max_x } else { min_x };
                    undo.push_snapshot(world, map, inventory);
                    let facing = self.parser.insert_facing;
                    for y in min_y..=max_y {
                        if map.in_bounds(col, y) && map.entity_at(col, y).is_none() {
                            let _ = map.place_multitile_entity(
                                world, col, y, entity_type, facing, true,
                            );
                        }
                    }
                    self.end_visual_selection();
                }
            }
            // gv: reselect the last visual selection.
            Command::ReselectVisual => {
                if let Some((anchor, cursor, kind)) = self.last_visual {
                    self.visual_anchor = Some(anchor);
                    self.visual_kind = Some(kind);
                    self.cursor_x = cursor.0;
                    self.cursor_y = cursor.1;
                    self.parser.mode = match kind {
                        VisualKind::Char => Mode::Visual,
                        VisualKind::Line => Mode::VisualLine,
                        VisualKind::Block => Mode::VisualBlock,
                    };
                }
            }

            // Single-key edits
            Command::ReplaceEntity(entity_type) => {
                if map.entity_at(self.cursor_x, self.cursor_y).is_some() {
                    undo.push_snapshot(world, map, inventory);
                    crate::vim::operators::collect_resources_at(world, map, inventory, self.cursor_x, self.cursor_y);
                    map.remove_multitile_entity(world, self.cursor_x, self.cursor_y);
                    let facing = self.parser.insert_facing;
                    let _ = map.place_multitile_entity(
                        world,
                        self.cursor_x,
                        self.cursor_y,
                        entity_type,
                        facing,
                        true,
                    );
                }
            }
            Command::DeleteUnderCursor(count) => {
                undo.push_snapshot(world, map, inventory);
                for i in 0..count {
                    let x = self.cursor_x + i;
                    if x < map.width {
                        crate::vim::operators::collect_resources_at(world, map, inventory, x, self.cursor_y);
                        map.remove_multitile_entity(world, x, self.cursor_y);
                    }
                }
            }
            Command::ToggleFacing => {
                self.parser.insert_facing = self.parser.insert_facing.rotate_cw();
            }
            // J: join — fill the gap between the cluster at/right of the
            // cursor and the next cluster on the row with right-facing belts.
            Command::JoinClusters(count) => {
                let mut placed_any = false;
                for _ in 0..count {
                    let y = self.cursor_y;
                    // Locate the end of the current (or next) cluster.
                    let end1 = if map.entity_at(self.cursor_x, y).is_some() {
                        map.find_end_of_cluster(self.cursor_x, y).0
                    } else {
                        // Find the first entity to the right on this row
                        match (self.cursor_x..map.width)
                            .find(|&x| map.entity_at(x, y).is_some())
                        {
                            Some(x) => map.find_end_of_cluster(x, y).0,
                            None => break,
                        }
                    };
                    // Find the start of the next cluster after the gap.
                    let start2 = match ((end1 + 1)..map.width)
                        .find(|&x| map.entity_at(x, y).is_some())
                    {
                        Some(x) => x,
                        None => break,
                    };
                    if start2 <= end1 + 1 {
                        // No gap — nothing to join.
                        break;
                    }
                    if !placed_any {
                        undo.push_snapshot(world, map, inventory);
                        placed_any = true;
                    }
                    for x in (end1 + 1)..start2 {
                        let _ = map.place_entity_on_map(
                            world,
                            x,
                            y,
                            EntityType::BasicBelt,
                            Facing::Right,
                            true,
                        );
                    }
                    // Move the cursor to the joint, like vim's J.
                    self.cursor_x = end1 + 1;
                }
                if placed_any {
                    self.status_message = "Joined clusters".to_string();
                }
            }
            // Ctrl-a / Ctrl-x: tier upgrade/downgrade under the cursor.
            Command::TierUp(count) | Command::TierDown(count) => {
                let up = matches!(cmd, Command::TierUp(_));
                // Only snapshot if the entity actually has a ladder.
                let can_change = map
                    .entity_type_at(world, self.cursor_x, self.cursor_y)
                    .map(|et| {
                        if up {
                            crate::vim::operators::upgrade_tier(et).is_some()
                        } else {
                            crate::vim::operators::downgrade_tier(et).is_some()
                        }
                    })
                    .unwrap_or(false);
                if can_change {
                    undo.push_snapshot(world, map, inventory);
                    for _ in 0..count {
                        if !crate::vim::operators::change_tier_at(
                            world,
                            map,
                            self.cursor_x,
                            self.cursor_y,
                            up,
                        ) {
                            break;
                        }
                    }
                }
            }
            // R mode: replace the tile under the cursor and advance.
            Command::ReplaceTile(entity_type) => {
                undo.push_snapshot(world, map, inventory);
                if map.entity_at(self.cursor_x, self.cursor_y).is_some() {
                    crate::vim::operators::collect_resources_at(
                        world, map, inventory, self.cursor_x, self.cursor_y,
                    );
                    map.remove_multitile_entity(world, self.cursor_x, self.cursor_y);
                }
                let facing = self.parser.insert_facing;
                let _ = map.place_multitile_entity(
                    world,
                    self.cursor_x,
                    self.cursor_y,
                    entity_type,
                    facing,
                    true,
                );
                self.last_insert_pos = Some((self.cursor_x, self.cursor_y));
                let (dx, dy) = facing.offset();
                let nx = self.cursor_x as isize + dx;
                let ny = self.cursor_y as isize + dy;
                if map.in_bounds_signed(nx, ny) {
                    self.cursor_x = nx as usize;
                    self.cursor_y = ny as usize;
                }
            }
            // gi: jump to where the last insert session happened.
            Command::JumpLastInsert => {
                if let Some((x, y)) = self.last_insert_pos {
                    self.cursor_x = x;
                    self.cursor_y = y;
                }
            }
            Command::RotateEntityUnderCursor => {
                if let Some(entity) = map.entity_at(self.cursor_x, self.cursor_y) {
                    // Resolve to anchor if this is a secondary tile
                    let anchor = if let Ok(pob) = world.get::<&crate::ecs::components::PartOfBuilding>(entity) {
                        pob.anchor
                    } else {
                        entity
                    };

                    let entity_type = world.get::<&crate::ecs::components::EntityKind>(anchor)
                        .map(|k| k.kind)
                        .ok();
                    let old_facing = world.get::<&crate::ecs::components::FacingComponent>(anchor)
                        .map(|f| f.facing)
                        .ok();
                    let anchor_pos = world.get::<&crate::ecs::components::Position>(anchor)
                        .map(|p| (p.x, p.y))
                        .ok();

                    if let (Some(et), Some(of), Some((ax, ay))) = (entity_type, old_facing, anchor_pos) {
                        let new_facing = of.rotate_cw();
                        let old_fp = crate::map::multitile::building_footprint(et).rotate_to(of);
                        let new_fp = crate::map::multitile::building_footprint(et).rotate_to(new_facing);

                        // Pre-check: verify rotated footprint fits
                        let mut old_tiles = std::collections::HashSet::new();
                        for fy in 0..old_fp.height {
                            for fx in 0..old_fp.width {
                                old_tiles.insert((ax + fx, ay + fy));
                            }
                        }
                        let fits = (0..new_fp.height).all(|fy| {
                            (0..new_fp.width).all(|fx| {
                                let nx = ax + fx;
                                let ny = ay + fy;
                                map.in_bounds(nx, ny)
                                    && (old_tiles.contains(&(nx, ny))
                                        || map.entity_at(nx, ny).is_none())
                            })
                        });

                        if fits {
                            undo.push_snapshot(world, map, inventory);
                            map.remove_multitile_entity(world, ax, ay);
                            let _ = map.place_multitile_entity(world, ax, ay, et, new_facing, true);
                        }
                    }
                } else {
                    // No entity under cursor: fall back to rotating insert_facing
                    self.parser.insert_facing = self.parser.insert_facing.rotate_cw();
                }
                // Set dot-repeat for ~
                self.dot.last_edit = Some(crate::vim::dot::ReplayableCommand::NormalEdit {
                    keystrokes: vec![crossterm::event::KeyEvent::new(
                        crossterm::event::KeyCode::Char('~'),
                        crossterm::event::KeyModifiers::NONE,
                    )],
                });
            }

            // Undo/Redo
            Command::Undo => {
                undo.undo(world, map, inventory);
            }
            Command::Redo => {
                undo.redo(world, map, inventory);
            }
            Command::DotRepeat => {
                if let Some(keys) = self.dot.get_repeat_keys().map(|k| k.to_vec()) {
                    for key in keys {
                        let cmds = self.parser.handle_key_event(key);
                        for c in cmds {
                            self.execute_command(c, map, world, undo, inventory);
                        }
                    }
                }
            }

            // Search
            Command::SearchNext(count) => {
                for _ in 0..count {
                    if let Some((x, y)) = self.search.next_match() {
                        self.cursor_x = x;
                        self.cursor_y = y;
                    }
                }
            }
            Command::SearchPrev(count) => {
                for _ in 0..count {
                    if let Some((x, y)) = self.search.prev_match() {
                        self.cursor_x = x;
                        self.cursor_y = y;
                    }
                }
            }
            Command::SearchWordUnderCursor(forward) => {
                // Find entity type under cursor and search for it
                if let Some(entity_type) = map.entity_type_at(world, self.cursor_x, self.cursor_y)
                {
                    self.push_jump();
                    self.search
                        .set_pattern(entity_type.name(), forward);
                    self.recompute_search_matches(world);
                    self.jump_to_nearest_search_match(forward);
                }
            }
            Command::ExecuteSearch(ref text, forward) => {
                self.search.set_pattern(text, forward);
                if self.search.pattern.is_none() {
                    self.status_message = format!("Pattern not found: {text}");
                } else {
                    self.recompute_search_matches(world);
                    if self.search.matches.is_empty() {
                        self.status_message = format!("Pattern not found: {text}");
                    } else {
                        self.push_jump();
                        self.jump_to_nearest_search_match(forward);
                    }
                }
            }

            // Splits
            Command::SplitVertical => {
                self.status_message = "Split vertical".to_string();
            }
            Command::SplitHorizontal => {
                self.status_message = "Split horizontal".to_string();
            }
            Command::FocusPane(_dir) => {
                self.status_message = "Focus pane".to_string();
            }
            Command::ClosePane => {
                self.status_message = "Close pane".to_string();
            }
            Command::CloseOtherPanes => {
                self.status_message = "Close other panes".to_string();
            }
            Command::EqualizePanes => {
                self.status_message = "Equalize panes".to_string();
            }
            Command::CyclePane => {
                self.status_message = "Cycle pane".to_string();
            }
            Command::SwapPanes => {
                self.status_message = "Swap panes".to_string();
            }
            Command::RotatePanes => {
                self.status_message = "Rotate panes".to_string();
            }

            // Viewport scrolling. The InputState only mirrors viewport_top;
            // the real Viewport is adjusted by the session (which owns it).
            Command::ScrollCenterCursor => {
                self.viewport_top = self
                    .cursor_y
                    .saturating_sub(self.viewport_height / 2)
                    .min(map.height.saturating_sub(1));
            }
            Command::ScrollCursorTop => {
                self.viewport_top = self.cursor_y.min(map.height.saturating_sub(1));
            }
            Command::ScrollCursorBottom => {
                self.viewport_top = (self.cursor_y + 1)
                    .saturating_sub(self.viewport_height)
                    .min(map.height.saturating_sub(1));
            }
            Command::ScrollHalfPageDown(count)
            | Command::ScrollHalfPageUp(count)
            | Command::ScrollFullPageDown(count)
            | Command::ScrollFullPageUp(count) => {
                let page = match cmd {
                    Command::ScrollHalfPageDown(_) | Command::ScrollHalfPageUp(_) => {
                        (self.viewport_height / 2).max(1)
                    }
                    _ => self.viewport_height.max(1),
                };
                let n = page * count;
                let down = matches!(
                    cmd,
                    Command::ScrollHalfPageDown(_) | Command::ScrollFullPageDown(_)
                );
                if down {
                    self.cursor_y = (self.cursor_y + n).min(map.height.saturating_sub(1));
                    self.viewport_top =
                        (self.viewport_top + n).min(map.height.saturating_sub(1));
                } else {
                    self.cursor_y = self.cursor_y.saturating_sub(n);
                    self.viewport_top = self.viewport_top.saturating_sub(n);
                }
            }

            // :s/old/new/[g] and friends — entity-type substitution.
            Command::CmdSubstitute {
                scope,
                ref pattern,
                ref replacement,
                global,
            } => {
                self.last_subst = Some((pattern.clone(), replacement.clone(), global));
                self.run_substitution(
                    scope,
                    pattern,
                    replacement,
                    global,
                    map,
                    world,
                    undo,
                    inventory,
                );
            }
            // & — repeat the last :s on the current row.
            Command::RepeatSubstitute => {
                if let Some((pattern, replacement, global)) = self.last_subst.clone() {
                    self.run_substitution(
                        SubstScope::CurrentRow,
                        &pattern,
                        &replacement,
                        global,
                        map,
                        world,
                        undo,
                        inventory,
                    );
                } else {
                    self.status_message = "No previous substitution".to_string();
                }
            }
            // :g/pattern/d — delete every entity matching the pattern.
            Command::CmdGlobalDelete(ref pattern) => {
                match resolve_entity_name(pattern) {
                    Some(target) => {
                        // Collect anchor positions of every matching entity.
                        let mut positions: Vec<(usize, usize)> = Vec::new();
                        for (entity, (pos, kind)) in world
                            .query::<(
                                &crate::ecs::components::Position,
                                &crate::ecs::components::EntityKind,
                            )>()
                            .iter()
                        {
                            if kind.kind == target
                                && world
                                    .get::<&crate::ecs::components::PartOfBuilding>(entity)
                                    .is_err()
                            {
                                positions.push((pos.x, pos.y));
                            }
                        }
                        if positions.is_empty() {
                            self.status_message = format!("No matches for {pattern}");
                        } else {
                            undo.push_snapshot(world, map, inventory);
                            let mut count = 0usize;
                            for (x, y) in positions {
                                crate::vim::operators::collect_resources_at(
                                    world, map, inventory, x, y,
                                );
                                if map.remove_multitile_entity(world, x, y) {
                                    count += 1;
                                }
                            }
                            self.status_message = format!("{count} deleted");
                        }
                    }
                    None => {
                        self.status_message = format!("Unknown entity: {pattern}");
                    }
                }
            }

            // Sidebar
            Command::ToggleSidebar => {
                self.sidebar_visible = !self.sidebar_visible;
            }

            // Save/Quit
            Command::SaveAndQuit => {
                self.status_message = "Save and quit (ZZ)".to_string();
            }
            Command::QuitNoSave => {
                self.status_message = "Quit without saving (ZQ)".to_string();
            }

            // Command mode commands are handled by the application layer.
            // We just store them as status messages here.
            Command::CmdSave(_) => {
                self.status_message = "Save".to_string();
            }
            Command::CmdQuit(_) => {
                self.status_message = "Quit".to_string();
            }
            Command::CmdSaveQuit => {
                self.status_message = "Save and quit".to_string();
            }
            Command::CmdLoad(ref path) => {
                self.status_message = format!("Load: {path}");
            }
            Command::CmdSetSpeed(s) => {
                self.status_message = format!("Speed: {s}");
            }
            Command::CmdPause => {
                self.status_message = "Paused".to_string();
            }
            Command::CmdResume => {
                self.status_message = "Running".to_string();
            }
            Command::CmdStep => {
                self.status_message = "Step".to_string();
            }
            Command::CmdStats => {
                self.status_message = "Stats".to_string();
            }
            Command::CmdRegisters => {
                let regs = self.registers.list();
                self.status_message = format!("Registers: {} entries", regs.len());
            }
            Command::CmdMarks => {
                let marks = self.marks.list();
                self.status_message = format!("Marks: {} entries", marks.len());
            }
            Command::CmdMapInfo => {
                self.status_message =
                    format!("Map: {}x{}", map.width, map.height);
            }
            Command::CmdHelp(ref topic) => {
                self.status_message =
                    format!("Help: {}", topic.as_deref().unwrap_or("general"));
            }
            Command::CmdLevel(lvl) => {
                self.status_message = format!("Level: {:?}", lvl);
            }
            Command::CmdRestart => {
                self.status_message = "Restart".to_string();
            }
            Command::CmdFreeplay => {
                self.status_message = "Freeplay".to_string();
            }
            Command::CmdMenu => {
                self.status_message = "Menu".to_string();
            }
            Command::CmdNoHighlight => {
                self.search.clear();
            }
            Command::CmdVersion => {
                self.status_message = format!("VimForge v{}", env!("CARGO_PKG_VERSION"));
            }

            // Text object operations
            Command::TextObjectOp(ref op, inner, obj_char, reg) => {
                let range = self.text_object_range(inner, obj_char, map, world);
                if !range.tiles.is_empty() {
                    self.apply_operator_to_range(op, &range, reg, map, world, undo, inventory);
                }
            }

            // Economy / expansion commands
            Command::CmdContracts => {
                self.status_message = "Contract board".to_string();
            }
            Command::CmdMarket => {
                self.status_message = "Market prices".to_string();
            }
            Command::CmdFinance => {
                self.status_message = "Financial overview".to_string();
            }
            Command::CmdLoan => {
                self.status_message = "Loan management".to_string();
            }
            Command::CmdRecipe(num) => {
                self.status_message = format!("Recipe: {:?}", num);
            }
            Command::CmdResearch => {
                self.status_message = "Research tree".to_string();
            }
            Command::CmdSell => {
                self.status_message = "Sell resources".to_string();
            }
            Command::CmdCampaign => {
                self.status_message = "Campaign progress".to_string();
            }
            Command::CmdPrestige => {
                self.status_message = "Prestige options".to_string();
            }
            Command::CmdSeed => {
                self.status_message = "Map seed info".to_string();
            }
        }

        // Clamp cursor to map bounds after every command
        let (cx, cy) = map.clamp(self.cursor_x, self.cursor_y);
        self.cursor_x = cx;
        self.cursor_y = cy;
    }

    /// Rebuild the search match list for the current search pattern:
    /// the anchor tile of every entity of the matching type, in row-major
    /// (reading) order.
    fn recompute_search_matches(&mut self, world: &hecs::World) {
        self.search.matches.clear();
        self.search.current_match = 0;
        let target = match self.search.pattern {
            Some(t) => t,
            None => return,
        };
        let mut found: Vec<(usize, usize)> = Vec::new();
        for (entity, (pos, kind)) in world
            .query::<(
                &crate::ecs::components::Position,
                &crate::ecs::components::EntityKind,
            )>()
            .iter()
        {
            if kind.kind == target
                && world
                    .get::<&crate::ecs::components::PartOfBuilding>(entity)
                    .is_err()
            {
                found.push((pos.x, pos.y));
            }
        }
        found.sort_by_key(|&(x, y)| (y, x));
        self.search.matches = found;
    }

    /// Jump to the first match after (forward) or before (backward) the
    /// cursor in row-major order, wrapping around — vim search semantics.
    fn jump_to_nearest_search_match(&mut self, forward: bool) {
        if self.search.matches.is_empty() {
            return;
        }
        let key = (self.cursor_y, self.cursor_x);
        let idx = if forward {
            self.search
                .matches
                .iter()
                .position(|&(x, y)| (y, x) > key)
                .unwrap_or(0)
        } else {
            self.search
                .matches
                .iter()
                .rposition(|&(x, y)| (y, x) < key)
                .unwrap_or(self.search.matches.len() - 1)
        };
        self.search.current_match = idx;
        let (x, y) = self.search.matches[idx];
        self.cursor_x = x;
        self.cursor_y = y;
    }

    // -----------------------------------------------------------------
    // Jumplist (Ctrl-o / Ctrl-i)
    // -----------------------------------------------------------------

    /// Record the current position before a "big" jump (gg/G/search/marks/
    /// paragraph/machine motions).
    fn push_jump(&mut self) {
        let pos = (self.cursor_x, self.cursor_y);
        self.jumplist.truncate(self.jump_idx);
        if self.jumplist.last() != Some(&pos) {
            self.jumplist.push(pos);
        }
        self.jump_idx = self.jumplist.len();
    }

    /// Ctrl-o: go back one entry in the jumplist.
    fn jump_back(&mut self) {
        if self.jump_idx == 0 {
            return;
        }
        if self.jump_idx == self.jumplist.len() {
            // Remember where we are so Ctrl-i can return here.
            self.jumplist.push((self.cursor_x, self.cursor_y));
        }
        self.jump_idx -= 1;
        let (x, y) = self.jumplist[self.jump_idx];
        self.cursor_x = x;
        self.cursor_y = y;
    }

    /// Ctrl-i: go forward one entry in the jumplist.
    fn jump_forward(&mut self) {
        if self.jump_idx + 1 < self.jumplist.len() {
            self.jump_idx += 1;
            let (x, y) = self.jumplist[self.jump_idx];
            self.cursor_x = x;
            self.cursor_y = y;
        }
    }

    // -----------------------------------------------------------------
    // Motion resolution (shared by plain motions and operator+motion)
    // -----------------------------------------------------------------

    /// Destination of `e`/`E` with count: end of the current cluster, or of
    /// the next cluster when already at an end.
    fn end_cluster_dest(&self, map: &Map, count: usize, _big: bool) -> (usize, usize) {
        let (mut cx, mut cy) = (self.cursor_x, self.cursor_y);
        for _ in 0..count {
            let (ex, ey) = map.find_end_of_cluster(cx, cy);
            if (ex, ey) == (cx, cy) {
                // Already at a cluster end (or on empty space): hop to the
                // next entity and take its cluster end.
                if let Some((nx, ny)) = map.find_next_entity(cx, cy) {
                    let (ex2, ey2) = map.find_end_of_cluster(nx, ny);
                    cx = ex2;
                    cy = ey2;
                } else {
                    break;
                }
            } else {
                cx = ex;
                cy = ey;
            }
        }
        (cx, cy)
    }

    /// Destination of `ge` with count: the end of the previous cluster.
    fn prev_cluster_end_dest(&self, map: &Map, count: usize) -> (usize, usize) {
        let (mut cx, mut cy) = (self.cursor_x, self.cursor_y);
        for _ in 0..count {
            // Step to the start of the current cluster first so
            // find_prev_entity lands on the previous cluster's end.
            let mut sx = cx;
            if map.entity_at(cx, cy).is_some() {
                while sx > 0 && map.entity_at(sx - 1, cy).is_some() {
                    sx -= 1;
                }
            }
            if let Some((px, py)) = map.find_prev_entity(sx, cy) {
                cx = px;
                cy = py;
            } else {
                break;
            }
        }
        (cx, cy)
    }

    /// The last entity's column in a row (g_).
    fn last_entity_in_row(&self, map: &Map, y: usize) -> Option<usize> {
        (0..map.width).rev().find(|&x| map.entity_at(x, y).is_some())
    }

    /// Destination of `(` / `)`: the previous/next machine (non-belt
    /// building) anchor in reading order. Does not wrap.
    fn machine_dest(
        &self,
        map: &Map,
        world: &hecs::World,
        count: usize,
        forward: bool,
    ) -> Option<(usize, usize)> {
        use crate::ecs::components::{EntityKind, PartOfBuilding, Position};

        let anchor_at = |x: usize, y: usize| -> Option<hecs::Entity> {
            map.entity_at(x, y).map(|e| {
                world
                    .get::<&PartOfBuilding>(e)
                    .map(|p| p.anchor)
                    .unwrap_or(e)
            })
        };

        let (mut cx, mut cy) = (self.cursor_x, self.cursor_y);
        let mut moved = false;
        for _ in 0..count {
            let cur_anchor = anchor_at(cx, cy);
            let mut found: Option<(usize, usize)> = None;

            let mut consider = |x: usize, y: usize| -> bool {
                if let Some(a) = anchor_at(x, y) {
                    if Some(a) != cur_anchor {
                        if let Ok(kind) = world.get::<&EntityKind>(a) {
                            if !crate::vim::text_objects::is_belt(kind.kind) {
                                let pos = world
                                    .get::<&Position>(a)
                                    .map(|p| (p.x, p.y))
                                    .unwrap_or((x, y));
                                found = Some(pos);
                                return true;
                            }
                        }
                    }
                }
                false
            };

            if forward {
                'fwd: for y in cy..map.height {
                    let x_start = if y == cy { cx + 1 } else { 0 };
                    for x in x_start..map.width {
                        if consider(x, y) {
                            break 'fwd;
                        }
                    }
                }
            } else {
                'bwd: for y in (0..=cy).rev() {
                    let x_end = if y == cy { cx } else { map.width };
                    for x in (0..x_end).rev() {
                        if consider(x, y) {
                            break 'bwd;
                        }
                    }
                }
            }

            match found {
                Some((x, y)) => {
                    cx = x;
                    cy = y;
                    moved = true;
                }
                None => break,
            }
        }
        if moved {
            Some((cx, cy))
        } else {
            None
        }
    }

    /// Destination of `%`: follow the belt/pipe chain from the cursor to
    /// its endpoint. Returns None when there is nothing to follow.
    fn match_connection_dest(
        &self,
        map: &Map,
        world: &hecs::World,
    ) -> Option<(usize, usize)> {
        let entity = map.entity_at(self.cursor_x, self.cursor_y)?;
        let kind = world
            .get::<&crate::ecs::components::EntityKind>(entity)
            .ok()?;
        if !kind.kind.has_facing() {
            return None;
        }
        let fc = world
            .get::<&crate::ecs::components::FacingComponent>(entity)
            .ok()?;
        let (dx, dy) = fc.facing.offset();
        let mut cx = self.cursor_x as isize + dx;
        let mut cy = self.cursor_y as isize + dy;
        // Walk forward along the chain
        while map.in_bounds_signed(cx, cy) {
            if let Some(next_e) = map.entity_at(cx as usize, cy as usize) {
                if let Ok(nf) =
                    world.get::<&crate::ecs::components::FacingComponent>(next_e)
                {
                    let (ndx, ndy) = nf.facing.offset();
                    cx += ndx;
                    cy += ndy;
                    continue;
                }
            }
            break;
        }
        // Step back to last valid position
        let fx = (cx - dx) as usize;
        let fy = (cy - dy) as usize;
        if (fx, fy) != (self.cursor_x, self.cursor_y) && map.in_bounds(fx, fy) {
            Some((fx, fy))
        } else {
            None
        }
    }

    /// Resolve a MotionKind into a destination tile plus vim range
    /// classification (linewise, inclusive). Returns None when the motion
    /// fails (e.g. `f` target not found), which aborts a pending operator —
    /// same as vim.
    ///
    /// Uses the same map query code paths as the plain motion commands, and
    /// updates the same repeat-state (last find, search index).
    fn motion_destination(
        &mut self,
        kind: MotionKind,
        count: usize,
        map: &Map,
        world: &hecs::World,
    ) -> Option<ResolvedMotion> {
        let origin = (self.cursor_x, self.cursor_y);
        let (mut cx, mut cy) = origin;

        let charwise = |dest: (usize, usize), inclusive: bool| {
            Some(ResolvedMotion {
                dest,
                linewise: false,
                inclusive,
            })
        };
        let linewise = |dest: (usize, usize)| {
            Some(ResolvedMotion {
                dest,
                linewise: true,
                inclusive: true,
            })
        };

        match kind {
            MotionKind::Left => charwise(
                crate::vim::motions::move_direction(
                    cx, cy,
                    crate::resources::Direction::Left,
                    count, map.width, map.height,
                ),
                false,
            ),
            MotionKind::Right => charwise(
                crate::vim::motions::move_direction(
                    cx, cy,
                    crate::resources::Direction::Right,
                    count, map.width, map.height,
                ),
                false,
            ),
            MotionKind::Up => linewise(crate::vim::motions::move_direction(
                cx, cy,
                crate::resources::Direction::Up,
                count, map.width, map.height,
            )),
            MotionKind::Down => linewise(crate::vim::motions::move_direction(
                cx, cy,
                crate::resources::Direction::Down,
                count, map.width, map.height,
            )),
            MotionKind::WordForward => {
                for _ in 0..count {
                    match map.find_next_entity(cx, cy) {
                        Some((x, y)) => {
                            cx = x;
                            cy = y;
                        }
                        None => break,
                    }
                }
                charwise((cx, cy), false)
            }
            MotionKind::WordBackward => {
                for _ in 0..count {
                    match map.find_prev_entity(cx, cy) {
                        Some((x, y)) => {
                            cx = x;
                            cy = y;
                        }
                        None => break,
                    }
                }
                charwise((cx, cy), false)
            }
            MotionKind::BigWordForward => {
                for _ in 0..count {
                    match map.find_next_entity_big(world, cx, cy) {
                        Some((x, y)) => {
                            cx = x;
                            cy = y;
                        }
                        None => break,
                    }
                }
                charwise((cx, cy), false)
            }
            MotionKind::BigWordBackward => {
                for _ in 0..count {
                    match map.find_prev_entity_big(world, cx, cy) {
                        Some((x, y)) => {
                            cx = x;
                            cy = y;
                        }
                        None => break,
                    }
                }
                charwise((cx, cy), false)
            }
            MotionKind::WordEnd => charwise(self.end_cluster_dest(map, count, false), true),
            MotionKind::BigWordEnd => charwise(self.end_cluster_dest(map, count, true), true),
            // ge is inclusive in vim; on the grid we approximate it as
            // exclusive-of-cursor (like b) so `dge` keeps the cursor tile.
            MotionKind::WordEndBack => charwise(self.prev_cluster_end_dest(map, count), false),
            MotionKind::LineStart => charwise((0, cy), false),
            MotionKind::LineEnd => charwise((map.width.saturating_sub(1), cy), true),
            MotionKind::FirstEntity => {
                let x = map.first_entity_in_row(cy).unwrap_or(cx);
                charwise((x, cy), false)
            }
            MotionKind::LastEntity => {
                let x = self.last_entity_in_row(map, cy)?;
                charwise((x, cy), true)
            }
            MotionKind::Column => charwise(
                (count.saturating_sub(1).min(map.width.saturating_sub(1)), cy),
                false,
            ),
            MotionKind::MapStart(row) => {
                linewise(crate::vim::motions::map_start(row, map.height))
            }
            MotionKind::MapEnd(row) => linewise(crate::vim::motions::map_end(row, map.height)),
            MotionKind::ViewportTop => linewise((cx, self.viewport_top)),
            MotionKind::ViewportMiddle => linewise((
                cx,
                (self.viewport_top + self.viewport_height / 2)
                    .min(map.height.saturating_sub(1)),
            )),
            MotionKind::ViewportBottom => linewise((
                cx,
                (self.viewport_top + self.viewport_height.saturating_sub(1))
                    .min(map.height.saturating_sub(1)),
            )),
            MotionKind::Find(entity_type, forward) | MotionKind::Til(entity_type, forward) => {
                let til = matches!(kind, MotionKind::Til(..));
                self.last_find_entity = Some(entity_type);
                self.last_find_forward = forward;
                self.last_find_til = til;
                let dest = self.resolve_find(map, world, entity_type, count, forward, til)?;
                charwise(dest, forward)
            }
            MotionKind::RepeatFind(same_dir) => {
                let entity_type = self.last_find_entity?;
                let forward = if same_dir {
                    self.last_find_forward
                } else {
                    !self.last_find_forward
                };
                let til = self.last_find_til;
                let dest = self.resolve_find(map, world, entity_type, count, forward, til)?;
                charwise(dest, forward)
            }
            MotionKind::NextParagraph => {
                for _ in 0..count {
                    cy = map.find_next_paragraph(cy);
                }
                // find_next_paragraph lands on the next paragraph's first
                // row; an operator must stop short of it (vim's d} does not
                // eat the next paragraph).
                let dest_y = cy.saturating_sub(1).max(origin.1);
                linewise((0, dest_y))
            }
            MotionKind::PrevParagraph => {
                for _ in 0..count {
                    cy = map.find_prev_paragraph(cy);
                }
                linewise((0, cy))
            }
            MotionKind::NextMachine => {
                charwise(self.machine_dest(map, world, count, true)?, false)
            }
            MotionKind::PrevMachine => {
                charwise(self.machine_dest(map, world, count, false)?, false)
            }
            MotionKind::RowDown => {
                let y = (cy + count).min(map.height.saturating_sub(1));
                linewise((map.first_entity_in_row(y).unwrap_or(0), y))
            }
            MotionKind::RowUp => {
                let y = cy.saturating_sub(count);
                linewise((map.first_entity_in_row(y).unwrap_or(0), y))
            }
            MotionKind::MatchConnection => {
                charwise(self.match_connection_dest(map, world)?, true)
            }
            MotionKind::SearchNext => {
                for _ in 0..count {
                    if let Some((x, y)) = self.search.next_match() {
                        cx = x;
                        cy = y;
                    }
                }
                if (cx, cy) == origin {
                    return None;
                }
                charwise((cx, cy), false)
            }
            MotionKind::SearchPrev => {
                for _ in 0..count {
                    if let Some((x, y)) = self.search.prev_match() {
                        cx = x;
                        cy = y;
                    }
                }
                if (cx, cy) == origin {
                    return None;
                }
                charwise((cx, cy), false)
            }
        }
    }

    /// Shared f/F/t/T destination logic (used by Find/Til/RepeatFind).
    fn resolve_find(
        &self,
        map: &Map,
        world: &hecs::World,
        entity_type: EntityType,
        count: usize,
        forward: bool,
        til: bool,
    ) -> Option<(usize, usize)> {
        let (mut cx, mut cy) = (self.cursor_x, self.cursor_y);
        let mut moved = false;
        for _ in 0..count {
            let found = if forward {
                map.find_entity_type_forward(world, cx, cy, entity_type)
            } else {
                map.find_entity_type_backward(world, cx, cy, entity_type)
            };
            if let Some((x, y)) = found {
                if til {
                    if forward {
                        if x > 0 && (x, y) != (cx, cy) {
                            cx = x.saturating_sub(1);
                            cy = y;
                            moved = true;
                        }
                    } else if x + 1 < map.width {
                        cx = x + 1;
                        cy = y;
                        moved = true;
                    }
                } else {
                    cx = x;
                    cy = y;
                    moved = true;
                }
            }
        }
        if moved {
            Some((cx, cy))
        } else {
            None
        }
    }

    /// Apply an operator to a resolved range: undo snapshot (for mutating
    /// operators), register write, and insert-mode entry for Change.
    #[allow(clippy::too_many_arguments)]
    fn apply_operator_to_range(
        &mut self,
        op: &Operator,
        range: &Range,
        reg: Option<char>,
        map: &mut Map,
        world: &mut hecs::World,
        undo: &mut UndoStack,
        inventory: &mut Inventory,
    ) {
        if range.tiles.is_empty() {
            return;
        }
        if !matches!(op, Operator::Yank) {
            undo.push_snapshot(world, map, inventory);
        }
        let bp = crate::vim::operators::apply_operator(op, world, map, inventory, range);
        if let Some(bp) = bp {
            self.registers
                .set_blueprint(reg, bp, matches!(op, Operator::Yank));
        }
        if matches!(op, Operator::Change) {
            self.parser.mode = Mode::Insert;
            self.parser.replace_mode = false;
            self.dot.start_recording();
            self.last_insert_pos = Some((self.cursor_x, self.cursor_y));
        }
    }

    /// Compute a text object's tile range at the cursor (shared by
    /// operator text objects and visual-mode text objects).
    fn text_object_range(
        &self,
        inner: bool,
        obj_char: char,
        map: &Map,
        world: &hecs::World,
    ) -> Range {
        let (x, y) = (self.cursor_x, self.cursor_y);
        match obj_char {
            'w' => {
                if inner {
                    crate::vim::text_objects::inner_word(map, x, y)
                } else {
                    crate::vim::text_objects::around_word(map, x, y)
                }
            }
            'p' => {
                if inner {
                    crate::vim::text_objects::inner_paragraph(map, x, y)
                } else {
                    crate::vim::text_objects::around_paragraph(map, x, y)
                }
            }
            // All bracket flavors plus the sentence object alias to the
            // wall-enclosed block (machine cluster).
            's' | 'b' | 'B' | '(' | ')' | '[' | ']' | '{' | '}' | '<' | '>' => {
                if inner {
                    crate::vim::text_objects::inner_block(map, world, x, y)
                } else {
                    crate::vim::text_objects::around_block(map, world, x, y)
                }
            }
            '"' | '\'' => crate::vim::text_objects::belt_run(map, world, x, y, !inner),
            't' => crate::vim::text_objects::machine_object(map, world, x, y, !inner),
            _ => Range::empty(),
        }
    }

    /// Store the current selection for gv, then clear it.
    fn end_visual_selection(&mut self) {
        if let (Some(anchor), Some(kind)) = (self.visual_anchor, self.visual_kind) {
            self.last_visual = Some((anchor, (self.cursor_x, self.cursor_y), kind));
        }
        self.visual_anchor = None;
        self.visual_kind = None;
        self.visual_override = None;
    }

    /// Run a :s substitution over the scoped rows. Replaces entity types,
    /// preserving facing, and reports "N substitutions".
    #[allow(clippy::too_many_arguments)]
    fn run_substitution(
        &mut self,
        scope: SubstScope,
        pattern: &str,
        replacement: &str,
        global: bool,
        map: &mut Map,
        world: &mut hecs::World,
        undo: &mut UndoStack,
        inventory: &mut Inventory,
    ) {
        let old_type = match resolve_entity_name(pattern) {
            Some(t) => t,
            None => {
                self.status_message = format!("Unknown entity: {pattern}");
                return;
            }
        };
        let new_type = match resolve_entity_name(replacement) {
            Some(t) => t,
            None => {
                self.status_message = format!("Unknown entity: {replacement}");
                return;
            }
        };

        let max_row = map.height.saturating_sub(1);
        let (row_start, row_end) = match scope {
            SubstScope::CurrentRow => (self.cursor_y, self.cursor_y),
            SubstScope::WholeMap => (0, max_row),
            SubstScope::Rows(a, b) => {
                let (lo, hi) = if a <= b { (a, b) } else { (b, a) };
                (lo.min(max_row), hi.min(max_row))
            }
        };

        // Collect targets first (anchor position + facing), deduplicating
        // multi-tile buildings. Without /g only the first match per row.
        let mut seen: std::collections::HashSet<hecs::Entity> = std::collections::HashSet::new();
        let mut targets: Vec<(usize, usize, crate::resources::Facing)> = Vec::new();
        for y in row_start..=row_end {
            let mut row_hits = 0usize;
            for x in 0..map.width {
                if let Some(entity) = map.entity_at(x, y) {
                    let anchor = world
                        .get::<&crate::ecs::components::PartOfBuilding>(entity)
                        .map(|p| p.anchor)
                        .unwrap_or(entity);
                    if !seen.insert(anchor) {
                        continue;
                    }
                    let kind = world
                        .get::<&crate::ecs::components::EntityKind>(anchor)
                        .ok()
                        .map(|k| k.kind);
                    if kind != Some(old_type) {
                        continue;
                    }
                    let (ax, ay) = world
                        .get::<&crate::ecs::components::Position>(anchor)
                        .map(|p| (p.x, p.y))
                        .unwrap_or((x, y));
                    let facing = world
                        .get::<&crate::ecs::components::FacingComponent>(anchor)
                        .ok()
                        .map(|f| f.facing)
                        .unwrap_or(Facing::Right);
                    targets.push((ax, ay, facing));
                    row_hits += 1;
                    if !global {
                        break;
                    }
                    let _ = row_hits;
                }
            }
        }

        if targets.is_empty() {
            self.status_message = format!("Pattern not found: {pattern}");
            return;
        }

        undo.push_snapshot(world, map, inventory);
        let mut count = 0usize;
        for (x, y, facing) in targets {
            crate::vim::operators::collect_resources_at(world, map, inventory, x, y);
            map.remove_multitile_entity(world, x, y);
            if map
                .place_multitile_entity(world, x, y, new_type, facing, true)
                .is_some()
            {
                count += 1;
            }
        }
        self.status_message = format!("{count} substitutions");
    }

    /// Compute the visual selection range from the anchor to the cursor.
    fn compute_visual_range(
        &self,
        anchor_x: usize,
        anchor_y: usize,
        map: &Map,
    ) -> crate::commands::Range {
        // A text-object selection (viw etc.) pins the exact tile set.
        if let Some(ref r) = self.visual_override {
            return r.clone();
        }
        match self.visual_kind {
            Some(VisualKind::Char) => {
                // Character-wise selection: all tiles from anchor to cursor
                let min_y = anchor_y.min(self.cursor_y);
                let max_y = anchor_y.max(self.cursor_y);
                let mut tiles = Vec::new();
                for row in min_y..=max_y {
                    let start_x = if row == min_y && min_y == max_y {
                        anchor_x.min(self.cursor_x)
                    } else if row == min_y {
                        if anchor_y < self.cursor_y {
                            anchor_x
                        } else {
                            self.cursor_x
                        }
                    } else {
                        0
                    };
                    let end_x = if row == min_y && min_y == max_y {
                        anchor_x.max(self.cursor_x)
                    } else if row == max_y {
                        if anchor_y < self.cursor_y {
                            self.cursor_x
                        } else {
                            anchor_x
                        }
                    } else {
                        map.width.saturating_sub(1)
                    };
                    for x in start_x..=end_x {
                        if x < map.width {
                            tiles.push((x, row));
                        }
                    }
                }
                crate::commands::Range {
                    tiles,
                    linewise: false,
                }
            }
            Some(VisualKind::Line) => {
                let min_y = anchor_y.min(self.cursor_y);
                let max_y = anchor_y.max(self.cursor_y);
                crate::commands::Range::linewise_rows(min_y, max_y, map.width)
            }
            Some(VisualKind::Block) => {
                let min_x = anchor_x.min(self.cursor_x);
                let max_x = anchor_x.max(self.cursor_x);
                let min_y = anchor_y.min(self.cursor_y);
                let max_y = anchor_y.max(self.cursor_y);
                crate::commands::Range::block(min_x, min_y, max_x, max_y)
            }
            None => crate::commands::Range::empty(),
        }
    }

    /// Paste a blueprint at the cursor position.
    fn paste_blueprint(
        &mut self,
        map: &mut Map,
        world: &mut hecs::World,
        bp: &crate::commands::Blueprint,
        _before: bool,
    ) {
        for entity in &bp.entities {
            let x = self.cursor_x + entity.offset_x;
            let y = self.cursor_y + entity.offset_y;
            if map.in_bounds(x, y) && map.entity_at(x, y).is_none() {
                let _ = map.place_multitile_entity(
                    world,
                    x,
                    y,
                    entity.entity_type,
                    entity.facing,
                    true,
                );
            }
        }
    }

    /// Get a display string for the current mode.
    pub fn mode_string(&self) -> String {
        match self.parser.mode {
            Mode::Normal => "NORMAL".to_string(),
            Mode::Insert if self.parser.replace_mode => "REPLACE".to_string(),
            Mode::Insert => {
                if let Some(cat_name) = self.parser.insert_category_name() {
                    let arrow = self.parser.insert_facing.arrow_glyph();
                    format!("INSERT [{cat_name}] [{arrow}]")
                } else {
                    "INSERT".to_string()
                }
            }
            Mode::Visual => "VISUAL".to_string(),
            Mode::VisualLine => "V-LINE".to_string(),
            Mode::VisualBlock => "V-BLOCK".to_string(),
            Mode::Command => "COMMAND".to_string(),
            Mode::Search => "SEARCH".to_string(),
        }
    }
}
