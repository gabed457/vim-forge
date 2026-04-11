use std::collections::HashSet;

use crate::levels::config::{get_level, total_levels, CompletionCondition, LevelConfig};

pub struct TutorialState {
    pub current_level: usize,
    pub levels_completed: Vec<usize>,
    pub current_hint_index: usize,
    pub visited_positions: HashSet<(usize, usize)>,
    pub commands_used: HashSet<String>,
    pub edit_count: usize,
}

impl TutorialState {
    pub fn new(starting_level: usize) -> Self {
        TutorialState {
            current_level: starting_level,
            levels_completed: Vec::new(),
            current_hint_index: 0,
            visited_positions: HashSet::new(),
            commands_used: HashSet::new(),
            edit_count: 0,
        }
    }

    pub fn new_with_progress(starting_level: usize, completed: Vec<usize>) -> Self {
        TutorialState {
            current_level: starting_level,
            levels_completed: completed,
            current_hint_index: 0,
            visited_positions: HashSet::new(),
            commands_used: HashSet::new(),
            edit_count: 0,
        }
    }

    /// Check if the current level's completion condition is met.
    ///
    /// Parameters represent the current game state:
    /// - `ore_delivered`: total ore delivered to output bins
    /// - `ingots_delivered`: total ingots delivered to output bins
    /// - `widgets_produced`: total widgets delivered to output bins
    /// - `cursor_pos`: current cursor position (x, y)
    pub fn check_completion(
        &self,
        ore_delivered: u64,
        ingots_delivered: u64,
        widgets_produced: u64,
        _cursor_pos: (usize, usize),
    ) -> bool {
        let level = match get_level(self.current_level) {
            Some(l) => l,
            None => return false,
        };

        match &level.completion {
            CompletionCondition::ProduceWidgets(target) => widgets_produced >= *target,
            CompletionCondition::DeliverOre(target) => ore_delivered >= *target,
            CompletionCondition::DeliverIngots(target) => ingots_delivered >= *target,
            CompletionCondition::NavigateToAll(positions) => {
                positions.iter().all(|pos| self.visited_positions.contains(pos))
            }
            CompletionCondition::UseCommands(cmds) => {
                cmds.iter().all(|cmd| self.commands_used.contains(cmd))
            }
            CompletionCondition::ScoreInMoves(target_widgets, max_moves) => {
                widgets_produced >= *target_widgets && self.edit_count <= *max_moves
            }
            CompletionCondition::Custom(_) => {
                // Custom conditions are evaluated externally; default to false here.
                false
            }
        }
    }

    /// Advance to the next hint for the current level.
    pub fn advance_hint(&mut self) {
        if let Some(level) = get_level(self.current_level) {
            if self.current_hint_index < level.hints.len().saturating_sub(1) {
                self.current_hint_index += 1;
            }
        }
    }

    /// Check if a command is allowed in the current level.
    /// Returns true if all commands are allowed (allowed_commands is None)
    /// or if the command is in the allowed list.
    pub fn is_command_allowed(&self, cmd: &str) -> bool {
        let level = match get_level(self.current_level) {
            Some(l) => l,
            None => return true,
        };

        match &level.allowed_commands {
            None => true,
            Some(allowed) => allowed.iter().any(|a| *a == cmd),
        }
    }

    /// Mark the current level as complete and return the next level number,
    /// or None if all levels are done.
    pub fn complete_level(&mut self) -> Option<usize> {
        if !self.levels_completed.contains(&self.current_level) {
            self.levels_completed.push(self.current_level);
        }

        let next = self.current_level + 1;
        if next <= total_levels() {
            self.current_level = next;
            self.current_hint_index = 0;
            self.visited_positions.clear();
            self.commands_used.clear();
            self.edit_count = 0;
            Some(next)
        } else {
            None
        }
    }

    /// A level is unlocked if it is level 1, or if the previous level has been completed.
    pub fn is_level_unlocked(&self, level: usize) -> bool {
        if level <= 1 {
            return true;
        }
        self.levels_completed.contains(&(level - 1))
    }

    /// Freeplay is unlocked after completing all tutorial levels.
    pub fn is_freeplay_unlocked(&self) -> bool {
        self.levels_completed.len() >= total_levels()
    }

    /// Record that the cursor visited a position.
    pub fn visit_position(&mut self, x: usize, y: usize) {
        self.visited_positions.insert((x, y));
    }

    /// Record that a command was used.
    pub fn use_command(&mut self, cmd: &str) {
        self.commands_used.insert(cmd.to_string());
    }

    /// Record that an edit was made (entity placed, deleted, or modified).
    pub fn record_edit(&mut self) {
        self.edit_count += 1;
    }

    /// Automatically advance hint based on player progress.
    /// Advances roughly every 5 new positions visited or every 2 edits.
    pub fn auto_advance_hint(&mut self) {
        let config = match get_level(self.current_level) {
            Some(c) => c,
            None => return,
        };
        let num_hints = config.hints.len();
        if num_hints <= 1 {
            return;
        }
        let progress = self.visited_positions.len() + self.edit_count * 3;
        let new_index = (progress / 5).min(num_hints - 1);
        if new_index > self.current_hint_index {
            self.current_hint_index = new_index;
        }
    }

    /// Get the current level config, if it exists.
    pub fn current_level_config(&self) -> Option<LevelConfig> {
        get_level(self.current_level)
    }
}
