use std::collections::HashSet;

/// All recognized command categories.
const ALL_CATEGORIES: &[&str] = &[
    "hjkl",
    "w_b",
    "0_dollar",
    "gg_G",
    "f_char",
    "paragraph",
    "percent",
    "insert",
    "delete",
    "yank",
    "change",
    "paste",
    "visual",
    "visual_line",
    "visual_block",
    "registers",
    "marks",
    "macros",
    "dot_repeat",
    "search",
    "text_objects",
    "splits",
    "rotate",
    "undo_redo",
    "command_mode",
    "counts",
];

const TOTAL_CATEGORIES: usize = 25;

/// Tracks which vim command categories the player has learned.
pub struct ProgressionTracker {
    pub commands_learned: HashSet<String>,
}

impl ProgressionTracker {
    pub fn new() -> Self {
        ProgressionTracker {
            commands_learned: HashSet::new(),
        }
    }

    /// Mark a command category as learned.
    pub fn learn(&mut self, category: &str) {
        // Only track recognized categories
        if ALL_CATEGORIES.contains(&category) {
            self.commands_learned.insert(category.to_string());
        }
    }

    /// Calculate the player's overall proficiency as a percentage (0-100).
    pub fn proficiency_percent(&self) -> u32 {
        let learned = self
            .commands_learned
            .iter()
            .filter(|c| ALL_CATEGORIES.contains(&c.as_str()))
            .count();
        ((learned as u64 * 100) / TOTAL_CATEGORIES as u64) as u32
    }

    /// Check if a specific category has been learned.
    pub fn is_learned(&self, category: &str) -> bool {
        self.commands_learned.contains(category)
    }

    /// Get the list of all recognized categories.
    pub fn all_categories() -> &'static [&'static str] {
        ALL_CATEGORIES
    }

    /// Get the total number of categories.
    pub fn total_categories() -> usize {
        TOTAL_CATEGORIES
    }

    /// Get a list of categories not yet learned.
    pub fn remaining_categories(&self) -> Vec<&'static str> {
        ALL_CATEGORIES
            .iter()
            .filter(|c| !self.commands_learned.contains(**c))
            .copied()
            .collect()
    }
}

impl Default for ProgressionTracker {
    fn default() -> Self {
        Self::new()
    }
}
