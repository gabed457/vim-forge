use crossterm::event::KeyEvent;

/// Represents a command sequence that can be repeated with the `.` key.
#[derive(Clone, Debug)]
pub enum ReplayableCommand {
    /// A normal-mode edit command (like `dw`, `x`, `>>`, etc.).
    NormalEdit { keystrokes: Vec<KeyEvent> },
    /// An insert session: entering insert mode, placing entities, and exiting.
    InsertSession { keystrokes: Vec<KeyEvent> },
}

/// Tracks the last edit command for dot-repeat.
pub struct DotRepeat {
    pub last_edit: Option<ReplayableCommand>,
    /// Accumulates keystrokes for the current edit in progress.
    current_keystrokes: Vec<KeyEvent>,
    /// Whether we are currently recording an edit.
    recording: bool,
}

impl DotRepeat {
    pub fn new() -> Self {
        DotRepeat {
            last_edit: None,
            current_keystrokes: Vec::new(),
            recording: false,
        }
    }

    /// Start recording keystrokes for a new edit.
    pub fn start_recording(&mut self) {
        self.recording = true;
        self.current_keystrokes.clear();
    }

    /// Record a keystroke during an edit.
    pub fn record_key(&mut self, key: KeyEvent) {
        if self.recording {
            self.current_keystrokes.push(key);
        }
    }

    /// Finish recording a normal-mode edit.
    pub fn finish_normal_edit(&mut self) {
        if self.recording && !self.current_keystrokes.is_empty() {
            self.last_edit = Some(ReplayableCommand::NormalEdit {
                keystrokes: self.current_keystrokes.clone(),
            });
        }
        self.recording = false;
        self.current_keystrokes.clear();
    }

    /// Finish recording an insert session.
    pub fn finish_insert_session(&mut self) {
        if self.recording && !self.current_keystrokes.is_empty() {
            self.last_edit = Some(ReplayableCommand::InsertSession {
                keystrokes: self.current_keystrokes.clone(),
            });
        }
        self.recording = false;
        self.current_keystrokes.clear();
    }

    /// Cancel recording without saving.
    pub fn cancel_recording(&mut self) {
        self.recording = false;
        self.current_keystrokes.clear();
    }

    /// Get the keystrokes for dot-repeat, if any.
    pub fn get_repeat_keys(&self) -> Option<&[KeyEvent]> {
        self.last_edit.as_ref().map(|cmd| match cmd {
            ReplayableCommand::NormalEdit { keystrokes } => keystrokes.as_slice(),
            ReplayableCommand::InsertSession { keystrokes } => keystrokes.as_slice(),
        })
    }

    /// Check if we are currently recording.
    pub fn is_recording(&self) -> bool {
        self.recording
    }
}
