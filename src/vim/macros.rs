use crossterm::event::KeyEvent;

use crate::vim::registers::RegisterStore;

const MAX_RECURSION_DEPTH: usize = 100;

/// Handles macro recording state and playback with recursion limiting.
pub struct MacroSystem {
    last_played: Option<char>,
    recursion_depth: usize,
}

impl MacroSystem {
    pub fn new() -> Self {
        MacroSystem {
            last_played: None,
            recursion_depth: 0,
        }
    }

    /// Get the last-played macro register (for @@ command).
    pub fn last_played(&self) -> Option<char> {
        self.last_played
    }

    /// Retrieve keystrokes for a macro from registers, respecting recursion limit.
    /// Returns None if the register has no macro or recursion limit is hit.
    pub fn get_playback_keys(
        &mut self,
        reg: char,
        registers: &RegisterStore,
    ) -> Option<Vec<KeyEvent>> {
        if self.recursion_depth >= MAX_RECURSION_DEPTH {
            return None;
        }
        self.last_played = Some(reg);
        registers.get_macro(Some(reg)).cloned()
    }

    /// Increment recursion depth before playing back a macro.
    pub fn enter_playback(&mut self) {
        self.recursion_depth += 1;
    }

    /// Decrement recursion depth after playback finishes.
    pub fn exit_playback(&mut self) {
        self.recursion_depth = self.recursion_depth.saturating_sub(1);
    }

    /// Reset recursion depth (called at the top level after all playback is done).
    pub fn reset_recursion(&mut self) {
        self.recursion_depth = 0;
    }

    /// Check if we have exceeded the recursion limit.
    pub fn recursion_exceeded(&self) -> bool {
        self.recursion_depth >= MAX_RECURSION_DEPTH
    }
}
