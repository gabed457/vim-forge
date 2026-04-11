use std::collections::HashMap;

/// Stores named marks (a-z, A-Z) and the previous jump position.
pub struct MarkStore {
    marks: HashMap<char, (usize, usize)>,
    prev_jump_pos: Option<(usize, usize)>,
}

impl MarkStore {
    pub fn new() -> Self {
        MarkStore {
            marks: HashMap::new(),
            prev_jump_pos: None,
        }
    }

    /// Set a mark at the given position.
    pub fn set(&mut self, name: char, x: usize, y: usize) {
        self.marks.insert(name, (x, y));
    }

    /// Get a mark position.
    pub fn get(&self, name: char) -> Option<(usize, usize)> {
        self.marks.get(&name).copied()
    }

    /// Record the cursor position before a jump (for '' and `` commands).
    pub fn set_prev_jump(&mut self, x: usize, y: usize) {
        self.prev_jump_pos = Some((x, y));
    }

    /// Get the previous jump position.
    pub fn get_prev_jump(&self) -> Option<(usize, usize)> {
        self.prev_jump_pos
    }

    /// List all marks for `:marks` display.
    pub fn list(&self) -> Vec<(char, usize, usize)> {
        let mut result: Vec<_> = self.marks.iter().map(|(&c, &(x, y))| (c, x, y)).collect();
        result.sort_by_key(|(c, _, _)| *c);
        result
    }

    /// Remove a mark.
    pub fn remove(&mut self, name: char) {
        self.marks.remove(&name);
    }
}
