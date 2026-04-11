//! Top-level input handler that:
//! - Takes a KeyEvent and the full app state
//! - Delegates to the VimParser
//! - Executes the resulting Commands against the game state
//! - Handles macro playback loop

use crossterm::event::KeyEvent;

use crate::commands::{Command, Operator};
use crate::game::inventory::Inventory;
use crate::game::undo::UndoStack;
use crate::map::grid::Map;
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
        let commands = self.parser.handle_key_event(key);
        let mut executed = Vec::new();

        for cmd in commands {
            self.execute_command(cmd.clone(), map, world, undo, inventory);
            executed.push(cmd);
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
            Command::JumpEndCluster => {
                let (x, y) = map.find_end_of_cluster(self.cursor_x, self.cursor_y);
                self.cursor_x = x;
                self.cursor_y = y;
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
                self.cursor_x = x;
                self.cursor_y = y;
            }
            Command::MapEnd(row) => {
                let (x, y) = crate::vim::motions::map_end(row, map.height);
                self.marks
                    .set_prev_jump(self.cursor_x, self.cursor_y);
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
                for _ in 0..count {
                    self.cursor_y = map.find_next_paragraph(self.cursor_y);
                }
                self.cursor_x = 0;
            }
            Command::PrevParagraph(count) => {
                for _ in 0..count {
                    self.cursor_y = map.find_prev_paragraph(self.cursor_y);
                }
                self.cursor_x = 0;
            }
            Command::MatchConnection => {
                // Match connection: find the connected entity and jump to it.
                // For now, this is a stub that could be extended with game-specific logic.
                self.status_message = "Match connection (%)".to_string();
            }
            Command::RepeatFind(same_dir) => {
                // Repeat last find/til -- handled by search state
                let _ = same_dir;
                self.status_message = "Repeat find".to_string();
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
                    self.cursor_x = x;
                    self.cursor_y = y;
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
                self.visual_anchor = None;
                self.visual_kind = None;
                if self.dot.is_recording() {
                    self.dot.finish_insert_session();
                }
            }

            // Insert mode actions
            Command::PlaceEntity(entity_type) => {
                let facing = self.parser.insert_facing;
                undo.push_snapshot(world, map, inventory);
                let _ = map.place_entity_on_map(
                    world,
                    self.cursor_x,
                    self.cursor_y,
                    entity_type,
                    facing,
                    true,
                );
                // Advance cursor in facing direction
                let (dx, dy) = facing.offset();
                let nx = self.cursor_x as isize + dx;
                let ny = self.cursor_y as isize + dy;
                if map.in_bounds_signed(nx, ny) {
                    self.cursor_x = nx as usize;
                    self.cursor_y = ny as usize;
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
                    }
                    self.visual_anchor = None;
                    self.visual_kind = None;
                }
            }
            Command::VisualSwapAnchor => {
                if let Some((ax, ay)) = self.visual_anchor {
                    self.visual_anchor = Some((self.cursor_x, self.cursor_y));
                    self.cursor_x = ax;
                    self.cursor_y = ay;
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
                    self.visual_anchor = None;
                    self.visual_kind = None;
                }
            }

            // Single-key edits
            Command::ReplaceEntity(entity_type) => {
                if map.entity_at(self.cursor_x, self.cursor_y).is_some() {
                    undo.push_snapshot(world, map, inventory);
                    crate::vim::operators::collect_resources_at(world, map, inventory, self.cursor_x, self.cursor_y);
                    map.remove_entity_from_map(world, self.cursor_x, self.cursor_y);
                    let facing = self.parser.insert_facing;
                    let _ = map.place_entity_on_map(
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
                        map.remove_entity_from_map(world, x, self.cursor_y);
                    }
                }
            }
            Command::ToggleFacing => {
                self.parser.insert_facing = self.parser.insert_facing.rotate_cw();
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
                    self.search
                        .set_pattern(entity_type.name(), forward);
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
        }

        // Clamp cursor to map bounds after every command
        let (cx, cy) = map.clamp(self.cursor_x, self.cursor_y);
        self.cursor_x = cx;
        self.cursor_y = cy;
    }

    /// Compute the visual selection range from the anchor to the cursor.
    fn compute_visual_range(
        &self,
        anchor_x: usize,
        anchor_y: usize,
        map: &Map,
    ) -> crate::commands::Range {
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
                let _ = map.place_entity_on_map(
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
    pub fn mode_string(&self) -> &'static str {
        match self.parser.mode {
            Mode::Normal => "NORMAL",
            Mode::Insert => "INSERT",
            Mode::Visual => "VISUAL",
            Mode::VisualLine => "V-LINE",
            Mode::VisualBlock => "V-BLOCK",
            Mode::Command => "COMMAND",
            Mode::Search => "SEARCH",
        }
    }
}
