//! The state machine input parser. Takes crossterm KeyEvents and produces Commands.

use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

use crate::commands::{Command, Operator, Range};
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
            KeyCode::Char('h') if mods.is_empty() => {
                self.try_motion_or_move(Command::Move(Direction::Left, self.effective_count()))
            }
            KeyCode::Char('j') if mods.is_empty() => {
                self.try_motion_or_move(Command::Move(Direction::Down, self.effective_count()))
            }
            KeyCode::Char('k') if mods.is_empty() => {
                self.try_motion_or_move(Command::Move(Direction::Up, self.effective_count()))
            }
            KeyCode::Char('l') if mods.is_empty() => {
                self.try_motion_or_move(Command::Move(Direction::Right, self.effective_count()))
            }

            // Word/entity navigation
            KeyCode::Char('w') if mods.is_empty() => {
                self.try_motion_or_move(Command::JumpNextEntity(self.effective_count()))
            }
            KeyCode::Char('W') => {
                self.try_motion_or_move(Command::JumpNextEntityBig(self.effective_count()))
            }
            KeyCode::Char('b') if mods.is_empty() => {
                self.try_motion_or_move(Command::JumpPrevEntity(self.effective_count()))
            }
            KeyCode::Char('B') => {
                self.try_motion_or_move(Command::JumpPrevEntityBig(self.effective_count()))
            }
            KeyCode::Char('e') if mods.is_empty() => {
                self.try_motion_or_move(Command::JumpEndCluster)
            }

            // Line start/end
            KeyCode::Char('0') if mods.is_empty() => {
                self.try_motion_or_move(Command::LineStart)
            }
            KeyCode::Char('$') | KeyCode::End => {
                self.try_motion_or_move(Command::LineEnd)
            }
            KeyCode::Char('^') => {
                self.try_motion_or_move(Command::FirstEntityInRow)
            }
            KeyCode::Home => {
                self.try_motion_or_move(Command::LineStart)
            }

            // Map start/end
            KeyCode::Char('G') => {
                let row = self.count1.or(self.count2);
                self.try_motion_or_move(Command::MapEnd(row))
            }

            // Viewport positions
            KeyCode::Char('H') if mods.is_empty() => {
                self.try_motion_or_move(Command::ViewportTop)
            }
            KeyCode::Char('M') if mods.is_empty() => {
                self.try_motion_or_move(Command::ViewportMiddle)
            }
            KeyCode::Char('L') if mods.is_empty() => {
                self.try_motion_or_move(Command::ViewportBottom)
            }

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
            KeyCode::Char(';') => {
                self.try_motion_or_move(Command::RepeatFind(true))
            }
            KeyCode::Char(',') => {
                self.try_motion_or_move(Command::RepeatFind(false))
            }

            // Paragraph navigation
            KeyCode::Char('}') => {
                self.try_motion_or_move(Command::NextParagraph(self.effective_count()))
            }
            KeyCode::Char('{') => {
                self.try_motion_or_move(Command::PrevParagraph(self.effective_count()))
            }

            // Match connection
            KeyCode::Char('%') => {
                self.try_motion_or_move(Command::MatchConnection)
            }

            // Operators
            KeyCode::Char('d') if mods.is_empty() => self.handle_operator_key(Operator::Delete, 'd'),
            KeyCode::Char('y') if mods.is_empty() => self.handle_operator_key(Operator::Yank, 'y'),
            KeyCode::Char('c') if mods.is_empty() => {
                if self.operator.is_none() {
                    self.handle_operator_key(Operator::Change, 'c')
                } else {
                    // 'c' after operator: not valid, reset
                    self.reset_pending();
                    vec![]
                }
            }
            KeyCode::Char('>') => self.handle_operator_key(Operator::RotateCW, '>'),
            KeyCode::Char('<') => self.handle_operator_key(Operator::RotateCCW, '<'),

            // Text objects (only valid after an operator)
            KeyCode::Char('i') if mods.is_empty() && self.operator.is_some() => {
                self.awaiting = Awaiting::Motion;
                self.command_buffer.push('i');
                vec![]
            }
            KeyCode::Char('a') if mods.is_empty() && self.operator.is_some() => {
                self.awaiting = Awaiting::Motion;
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

            // Undo / Redo
            KeyCode::Char('u') if mods.is_empty() => {
                self.reset_pending();
                vec![Command::Undo]
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
            KeyCode::Char('n') if mods.is_empty() => {
                let count = self.effective_count();
                self.reset_pending();
                vec![Command::SearchNext(count)]
            }
            KeyCode::Char('N') => {
                let count = self.effective_count();
                self.reset_pending();
                vec![Command::SearchPrev(count)]
            }
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

            // Toggle facing
            KeyCode::Char('~') => {
                self.reset_pending();
                vec![Command::ToggleFacing]
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

            // ZZ / ZQ
            KeyCode::Char('Z') => {
                self.command_buffer.push('Z');
                // Wait for the second Z or Q -- we handle this inline
                // by checking the command buffer on the next key
                // For simplicity, track via command_buffer
                vec![]
            }

            // Arrow keys for movement
            KeyCode::Left => {
                self.try_motion_or_move(Command::Move(Direction::Left, self.effective_count()))
            }
            KeyCode::Right => {
                self.try_motion_or_move(Command::Move(Direction::Right, self.effective_count()))
            }
            KeyCode::Up => {
                self.try_motion_or_move(Command::Move(Direction::Up, self.effective_count()))
            }
            KeyCode::Down => {
                self.try_motion_or_move(Command::Move(Direction::Down, self.effective_count()))
            }

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
                    Operator::Delete => Command::DemolishLine(count),
                    Operator::Yank => Command::YankLine(count, reg),
                    Operator::Change => Command::ChangeLine(count),
                    Operator::RotateCW => Command::RotateCWLine(count),
                    Operator::RotateCCW => Command::RotateCCWLine(count),
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
    /// - If an operator is pending, apply the operator to the motion range
    /// - Otherwise, just produce the motion command
    fn try_motion_or_move(&mut self, motion_cmd: Command) -> Vec<Command> {
        if let Some(op) = self.operator.take() {
            // We have an operator; combine with the motion.
            // The handler will resolve the actual range.
            let reg = self.register;
            let cmd = match op {
                Operator::Delete => Command::Demolish(Range::empty()),
                Operator::Yank => Command::Yank(Range::empty(), reg),
                Operator::Change => Command::Change(Range::empty()),
                Operator::RotateCW => Command::RotateCW(Range::empty()),
                Operator::RotateCCW => Command::RotateCCW(Range::empty()),
            };
            self.reset_pending();
            // Return both: the motion (for range computation) and the operator.
            // The handler uses the motion to compute the range, then applies the operator.
            vec![motion_cmd, cmd]
        } else {
            self.reset_pending();
            vec![motion_cmd]
        }
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
            let count = self.effective_count();
            let cmd = if til {
                Command::TilEntity(entity_type, count, forward)
            } else {
                Command::FindEntity(entity_type, count, forward)
            };
            self.try_motion_or_move(cmd)
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
            KeyCode::Char(c) if c.is_ascii_alphanumeric() || c == '"' => {
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

    /// Handle the second g in gg.
    fn handle_second_g(&mut self, key: KeyEvent) -> Vec<Command> {
        match key.code {
            KeyCode::Char('g') => {
                let row = self.count1.or(self.count2);
                self.try_motion_or_move(Command::MapStart(row))
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
            KeyCode::Char('o') => Some(Command::CloseOtherPanes),
            KeyCode::Char('=') => Some(Command::EqualizePanes),
            _ => None,
        };
        self.reset_pending();
        cmd.into_iter().collect()
    }

    /// Handle text objects after operator + i/a.
    /// Called when we have an operator and the user typed 'i' or 'a' followed by
    /// a text object key.
    fn _handle_text_object_key(&mut self, _inner: bool, key: KeyEvent) -> Vec<Command> {
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

        // This is used as a signal to the handler about what text object to compute.
        // The handler will compute the actual Range based on the text object type.
        // We produce the operator command with an empty range; the handler
        // interprets the presence of preceding motion commands to know the object.
        // For simplicity, we encode the text object as a special motion command
        // that the handler understands.

        // Actually, let's just reset and return nothing if invalid
        self.command_buffer.push(obj_char);

        // Valid text objects: w, p, b, (, )
        let _valid = matches!(obj_char, 'w' | 'p' | 'b' | '(' | ')');
        if !_valid {
            self.reset_pending();
            return vec![];
        }

        // The operator should still be set -- produce the operator command.
        // The handler needs to know: operator, inner/around, object type.
        // We'll return the operator with the range set to empty and rely on
        // the handler to re-interpret. Instead, let's return the info as commands.
        if let Some(op) = self.operator.take() {
            let reg = self.register;
            let cmd = match op {
                Operator::Delete => Command::Demolish(Range::empty()),
                Operator::Yank => Command::Yank(Range::empty(), reg),
                Operator::Change => Command::Change(Range::empty()),
                Operator::RotateCW => Command::RotateCW(Range::empty()),
                Operator::RotateCCW => Command::RotateCCW(Range::empty()),
            };
            self.reset_pending();
            // Encode the text object info in a way the handler can use.
            // We return the operator command; the command_buffer will have been
            // set with the full sequence for the handler to parse.
            // Actually, we need a better approach. Let's not use command_buffer.
            // Instead, we return the info directly.
            vec![cmd]
        } else {
            self.reset_pending();
            vec![]
        }
    }

    // -----------------------------------------------------------------------
    // Insert mode
    // -----------------------------------------------------------------------

    fn handle_insert(&mut self, key: KeyEvent) -> Vec<Command> {
        let mods = key.modifiers;

        match key.code {
            KeyCode::Esc => {
                self.mode = Mode::Normal;
                self.reset_pending();
                vec![Command::ExitToNormal]
            }

            // Place entities
            KeyCode::Char('s') if mods.is_empty() => {
                vec![Command::PlaceEntity(EntityType::Smelter)]
            }
            KeyCode::Char('a') if mods.is_empty() => {
                vec![Command::PlaceEntity(EntityType::Assembler)]
            }
            KeyCode::Char('c') if mods.is_empty() => {
                vec![Command::PlaceEntity(EntityType::Conveyor)]
            }
            KeyCode::Char('p') if mods.is_empty() => {
                vec![Command::PlaceEntity(EntityType::Splitter)]
            }
            KeyCode::Char('e') if mods.is_empty() => {
                vec![Command::PlaceEntity(EntityType::Merger)]
            }
            KeyCode::Char('w') if mods.is_empty() => {
                vec![Command::PlaceEntity(EntityType::Wall)]
            }

            // Move without placing (lowercase hjkl)
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

            // Change facing + move (Shift+HJKL)
            KeyCode::Char('H') => {
                vec![
                    Command::SetInsertFacing(Facing::Left),
                    Command::InsertMoveOnly(Direction::Left),
                ]
            }
            KeyCode::Char('J') => {
                vec![
                    Command::SetInsertFacing(Facing::Down),
                    Command::InsertMoveOnly(Direction::Down),
                ]
            }
            KeyCode::Char('K') => {
                vec![
                    Command::SetInsertFacing(Facing::Up),
                    Command::InsertMoveOnly(Direction::Up),
                ]
            }
            KeyCode::Char('L') => {
                vec![
                    Command::SetInsertFacing(Facing::Right),
                    Command::InsertMoveOnly(Direction::Right),
                ]
            }

            // Arrow keys: change facing + move
            KeyCode::Left => {
                vec![
                    Command::SetInsertFacing(Facing::Left),
                    Command::InsertMoveOnly(Direction::Left),
                ]
            }
            KeyCode::Right => {
                vec![
                    Command::SetInsertFacing(Facing::Right),
                    Command::InsertMoveOnly(Direction::Right),
                ]
            }
            KeyCode::Up => {
                vec![
                    Command::SetInsertFacing(Facing::Up),
                    Command::InsertMoveOnly(Direction::Up),
                ]
            }
            KeyCode::Down => {
                vec![
                    Command::SetInsertFacing(Facing::Down),
                    Command::InsertMoveOnly(Direction::Down),
                ]
            }

            // Backspace: undo last placement
            KeyCode::Backspace => {
                vec![Command::InsertBackspace]
            }

            _ => vec![],
        }
    }

    // -----------------------------------------------------------------------
    // Visual mode
    // -----------------------------------------------------------------------

    fn handle_visual(&mut self, key: KeyEvent) -> Vec<Command> {
        let mods = key.modifiers;
        match key.code {
            // Motions extend the selection
            KeyCode::Char('h') if mods.is_empty() => vec![Command::Move(Direction::Left, 1)],
            KeyCode::Char('j') if mods.is_empty() => vec![Command::Move(Direction::Down, 1)],
            KeyCode::Char('k') if mods.is_empty() => vec![Command::Move(Direction::Up, 1)],
            KeyCode::Char('l') if mods.is_empty() => vec![Command::Move(Direction::Right, 1)],

            KeyCode::Char('w') if mods.is_empty() => vec![Command::JumpNextEntity(1)],
            KeyCode::Char('W') => vec![Command::JumpNextEntityBig(1)],
            KeyCode::Char('b') if mods.is_empty() => vec![Command::JumpPrevEntity(1)],
            KeyCode::Char('B') => vec![Command::JumpPrevEntityBig(1)],
            KeyCode::Char('e') if mods.is_empty() => vec![Command::JumpEndCluster],

            KeyCode::Char('0') if mods.is_empty() => vec![Command::LineStart],
            KeyCode::Char('$') => vec![Command::LineEnd],
            KeyCode::Char('^') => vec![Command::FirstEntityInRow],

            KeyCode::Char('G') => vec![Command::MapEnd(None)],

            KeyCode::Char('}') => vec![Command::NextParagraph(1)],
            KeyCode::Char('{') => vec![Command::PrevParagraph(1)],
            KeyCode::Char('%') => vec![Command::MatchConnection],

            KeyCode::Char('H') if mods.is_empty() => vec![Command::ViewportTop],
            KeyCode::Char('M') if mods.is_empty() => vec![Command::ViewportMiddle],
            KeyCode::Char('L') if mods.is_empty() => vec![Command::ViewportBottom],

            KeyCode::Left => vec![Command::Move(Direction::Left, 1)],
            KeyCode::Right => vec![Command::Move(Direction::Right, 1)],
            KeyCode::Up => vec![Command::Move(Direction::Up, 1)],
            KeyCode::Down => vec![Command::Move(Direction::Down, 1)],

            // Operators on the selection
            KeyCode::Char('d') if mods.is_empty() => {
                self.mode = Mode::Normal;
                vec![Command::VisualOperator(Operator::Delete)]
            }
            KeyCode::Char('y') if mods.is_empty() => {
                self.mode = Mode::Normal;
                vec![Command::VisualOperator(Operator::Yank)]
            }
            KeyCode::Char('c') if mods.is_empty() => {
                self.mode = Mode::Insert;
                vec![Command::VisualOperator(Operator::Change)]
            }
            KeyCode::Char('>') => {
                self.mode = Mode::Normal;
                vec![Command::VisualOperator(Operator::RotateCW)]
            }
            KeyCode::Char('<') => {
                self.mode = Mode::Normal;
                vec![Command::VisualOperator(Operator::RotateCCW)]
            }

            // Swap anchor
            KeyCode::Char('o') if mods.is_empty() => {
                vec![Command::VisualSwapAnchor]
            }

            // Paste over selection
            KeyCode::Char('p') if mods.is_empty() => {
                self.mode = Mode::Normal;
                vec![Command::VisualPaste(self.register)]
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
            _ => {
                // Unknown command
                vec![]
            }
        }
    }

    /// Simple tab completion for command names.
    fn tab_complete_command(&mut self) {
        let candidates = [
            "write", "quit", "wq", "edit", "speed", "pause", "resume", "step", "stats",
            "registers", "marks", "mapinfo", "help", "level", "restart", "freeplay", "menu",
            "nohlsearch", "version",
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
                    self.search_history.push(pattern);
                }
                self.mode = Mode::Normal;
                self.search_history_idx = None;
                // The search was already initiated via EnterSearch; the handler uses
                // search_line to perform the search when exiting search mode.
                vec![Command::ExitToNormal]
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
            s.push(match op {
                Operator::Delete => 'd',
                Operator::Yank => 'y',
                Operator::Change => 'c',
                Operator::RotateCW => '>',
                Operator::RotateCCW => '<',
            });
        }
        if !self.count_buffer.is_empty() {
            s.push_str(&self.count_buffer);
        }
        s
    }
}
