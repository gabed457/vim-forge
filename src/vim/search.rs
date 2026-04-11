use crate::resources::EntityType;

/// Tracks the current search pattern, direction, and cached match positions.
#[derive(Clone)]
pub struct SearchState {
    pub pattern: Option<EntityType>,
    pub pattern_text: String,
    pub forward: bool,
    pub matches: Vec<(usize, usize)>,
    pub current_match: usize,
}

impl SearchState {
    pub fn new() -> Self {
        SearchState {
            pattern: None,
            pattern_text: String::new(),
            forward: true,
            matches: Vec::new(),
            current_match: 0,
        }
    }

    /// Set a new search pattern from user input text.
    pub fn set_pattern(&mut self, text: &str, forward: bool) {
        self.pattern_text = text.to_string();
        self.forward = forward;
        self.pattern = EntityType::from_search_prefix(text);
        self.matches.clear();
        self.current_match = 0;
    }

    /// Clear the search highlighting.
    pub fn clear(&mut self) {
        self.pattern = None;
        self.pattern_text.clear();
        self.matches.clear();
        self.current_match = 0;
    }

    /// Check if we have an active search.
    pub fn has_pattern(&self) -> bool {
        self.pattern.is_some()
    }

    /// Advance to the next match, wrapping around.
    pub fn next_match(&mut self) -> Option<(usize, usize)> {
        if self.matches.is_empty() {
            return None;
        }
        if self.forward {
            self.current_match = (self.current_match + 1) % self.matches.len();
        } else {
            if self.current_match == 0 {
                self.current_match = self.matches.len() - 1;
            } else {
                self.current_match -= 1;
            }
        }
        Some(self.matches[self.current_match])
    }

    /// Go to the previous match, wrapping around.
    pub fn prev_match(&mut self) -> Option<(usize, usize)> {
        if self.matches.is_empty() {
            return None;
        }
        if self.forward {
            if self.current_match == 0 {
                self.current_match = self.matches.len() - 1;
            } else {
                self.current_match -= 1;
            }
        } else {
            self.current_match = (self.current_match + 1) % self.matches.len();
        }
        Some(self.matches[self.current_match])
    }

    /// Find the closest match to a given position and set current_match to it.
    pub fn find_nearest(&mut self, x: usize, y: usize) {
        if self.matches.is_empty() {
            return;
        }
        let mut best = 0;
        let mut best_dist = usize::MAX;
        for (i, &(mx, my)) in self.matches.iter().enumerate() {
            let dx = if mx > x { mx - x } else { x - mx };
            let dy = if my > y { my - y } else { y - my };
            let dist = dy * 1000 + dx; // Row-major distance
            if dist < best_dist {
                best_dist = dist;
                best = i;
            }
        }
        self.current_match = best;
    }
}
