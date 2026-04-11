use std::collections::HashMap;

use crossterm::event::KeyEvent;

use crate::commands::{Blueprint, RegisterContent};

/// Stores all named registers, the unnamed register (""), the yank register ("0),
/// and the last delete register ("1).
pub struct RegisterStore {
    registers: HashMap<char, RegisterContent>,
    unnamed: Option<RegisterContent>,
    yank_register: Option<RegisterContent>,
    last_delete: Option<RegisterContent>,
}

impl RegisterStore {
    pub fn new() -> Self {
        RegisterStore {
            registers: HashMap::new(),
            unnamed: None,
            yank_register: None,
            last_delete: None,
        }
    }

    /// Get the content of a register. `None` means the unnamed register.
    pub fn get(&self, reg: Option<char>) -> Option<&RegisterContent> {
        match reg {
            None => self.unnamed.as_ref(),
            Some('"') => self.unnamed.as_ref(),
            Some('0') => self.yank_register.as_ref(),
            Some('1') => self.last_delete.as_ref(),
            Some(c) => self.registers.get(&c),
        }
    }

    /// Set a register's content. `None` means the unnamed register.
    pub fn set(&mut self, reg: Option<char>, content: RegisterContent) {
        match reg {
            None | Some('"') => {
                self.unnamed = Some(content);
            }
            Some('0') => {
                self.yank_register = Some(content);
            }
            Some('1') => {
                self.last_delete = Some(content);
            }
            Some(c) => {
                self.registers.insert(c, content);
            }
        }
    }

    /// Get a blueprint from a register (for paste operations).
    pub fn get_blueprint(&self, reg: Option<char>) -> Option<&Blueprint> {
        self.get(reg).and_then(|rc| match rc {
            RegisterContent::Blueprint(bp) => Some(bp),
            RegisterContent::Macro(_) => None,
        })
    }

    /// Get macro keystrokes from a register (for macro playback).
    pub fn get_macro(&self, reg: Option<char>) -> Option<&Vec<KeyEvent>> {
        // Named registers can hold macros
        let content = match reg {
            None => return None,
            Some(c) => self.registers.get(&c),
        };
        content.and_then(|rc| match rc {
            RegisterContent::Macro(keys) => Some(keys),
            RegisterContent::Blueprint(_) => None,
        })
    }

    /// Store a blueprint in a register, also updating unnamed and yank/delete.
    pub fn set_blueprint(&mut self, reg: Option<char>, bp: Blueprint, is_yank: bool) {
        let content = RegisterContent::Blueprint(bp);
        // Always set the unnamed register
        self.unnamed = Some(content.clone());
        if is_yank {
            self.yank_register = Some(content.clone());
        } else {
            self.last_delete = Some(content.clone());
        }
        // If a named register was specified, also set it
        if let Some(c) = reg {
            if c != '"' && c != '0' && c != '1' {
                self.registers.insert(c, content);
            }
        }
    }

    /// Store macro keystrokes in a named register.
    pub fn set_macro(&mut self, reg: char, keys: Vec<KeyEvent>) {
        self.registers.insert(reg, RegisterContent::Macro(keys));
    }

    /// List all non-empty registers for `:registers` display.
    pub fn list(&self) -> Vec<(String, String)> {
        let mut result = Vec::new();
        if let Some(ref c) = self.unnamed {
            result.push(("\"\"".to_string(), format_content(c)));
        }
        if let Some(ref c) = self.yank_register {
            result.push(("\"0".to_string(), format_content(c)));
        }
        if let Some(ref c) = self.last_delete {
            result.push(("\"1".to_string(), format_content(c)));
        }
        let mut sorted: Vec<_> = self.registers.iter().collect();
        sorted.sort_by_key(|(k, _)| **k);
        for (ch, content) in sorted {
            result.push((format!("\"{ch}"), format_content(content)));
        }
        result
    }
}

fn format_content(content: &RegisterContent) -> String {
    match content {
        RegisterContent::Blueprint(bp) => bp.summary(),
        RegisterContent::Macro(keys) => format!("macro ({} keys)", keys.len()),
    }
}
