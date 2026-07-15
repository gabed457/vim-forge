use std::collections::HashMap;

use crossterm::event::KeyEvent;

use crate::commands::{Blueprint, RegisterContent};

/// Stores all named registers, the unnamed register (""), the yank register
/// ("0), the shifting delete history ("1-"9), the black hole ("_), and the
/// uppercase append registers ("A-"Z append to "a-"z).
pub struct RegisterStore {
    registers: HashMap<char, RegisterContent>,
    unnamed: Option<RegisterContent>,
    yank_register: Option<RegisterContent>,
    /// "1 through "9: most recent delete first; each new delete shifts
    /// the older ones down (vim's numbered-register behavior).
    numbered: Vec<Option<RegisterContent>>,
}

impl RegisterStore {
    pub fn new() -> Self {
        RegisterStore {
            registers: HashMap::new(),
            unnamed: None,
            yank_register: None,
            numbered: vec![None; 9],
        }
    }

    /// Get the content of a register. `None` means the unnamed register.
    pub fn get(&self, reg: Option<char>) -> Option<&RegisterContent> {
        match reg {
            None => self.unnamed.as_ref(),
            Some('"') => self.unnamed.as_ref(),
            Some('0') => self.yank_register.as_ref(),
            Some('_') => None,
            Some(c @ '1'..='9') => {
                let idx = c as usize - '1' as usize;
                self.numbered[idx].as_ref()
            }
            // Uppercase registers read the same content as their lowercase
            // counterpart (append targets the lowercase register).
            Some(c) if c.is_ascii_uppercase() => {
                self.registers.get(&c.to_ascii_lowercase())
            }
            Some(c) => self.registers.get(&c),
        }
    }

    /// Set a register's content. `None` means the unnamed register.
    pub fn set(&mut self, reg: Option<char>, content: RegisterContent) {
        match reg {
            None | Some('"') => {
                self.unnamed = Some(content);
            }
            Some('_') => {}
            Some('0') => {
                self.yank_register = Some(content);
            }
            Some(c @ '1'..='9') => {
                let idx = c as usize - '1' as usize;
                self.numbered[idx] = Some(content);
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
    ///
    /// - `"_` (black hole): nothing is stored anywhere — deletes with `"_`
    ///   never clobber the unnamed register or the delete history.
    /// - `"A`-`"Z`: the blueprint is APPENDED to the lowercase register
    ///   (offset below its existing content) instead of replacing it.
    /// - Deletes shift the numbered history `"1 → "2 → ... → "9`; yanks go
    ///   to `"0`.
    pub fn set_blueprint(&mut self, reg: Option<char>, bp: Blueprint, is_yank: bool) {
        // Black hole: swallow everything.
        if reg == Some('_') {
            return;
        }

        // Uppercase register: append to the lowercase register's content.
        if let Some(c) = reg {
            if c.is_ascii_uppercase() {
                let lower = c.to_ascii_lowercase();
                let merged = match self.registers.get(&lower) {
                    Some(RegisterContent::Blueprint(existing)) => {
                        append_blueprints(existing, &bp)
                    }
                    _ => bp,
                };
                let content = RegisterContent::Blueprint(merged);
                self.unnamed = Some(content.clone());
                if is_yank {
                    self.yank_register = Some(content.clone());
                } else {
                    self.push_numbered(content.clone());
                }
                self.registers.insert(lower, content);
                return;
            }
        }

        let content = RegisterContent::Blueprint(bp);
        // Always set the unnamed register
        self.unnamed = Some(content.clone());
        if is_yank {
            self.yank_register = Some(content.clone());
        } else {
            self.push_numbered(content.clone());
        }
        // If a named register was specified, also set it
        if let Some(c) = reg {
            if c != '"' && !c.is_ascii_digit() {
                self.registers.insert(c, content);
            }
        }
    }

    /// Shift the numbered delete history: "1 becomes "2, ..., "8 becomes
    /// "9 (the oldest falls off), and the new content becomes "1.
    fn push_numbered(&mut self, content: RegisterContent) {
        self.numbered.insert(0, Some(content));
        self.numbered.truncate(9);
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
        for (i, slot) in self.numbered.iter().enumerate() {
            if let Some(ref c) = slot {
                result.push((format!("\"{}", i + 1), format_content(c)));
            }
        }
        let mut sorted: Vec<_> = self.registers.iter().collect();
        sorted.sort_by_key(|(k, _)| **k);
        for (ch, content) in sorted {
            result.push((format!("\"{ch}"), format_content(content)));
        }
        result
    }
}

/// Merge two blueprints for the "A-"Z append registers: `b` is placed
/// below `a` (offset by a's height), widths take the max.
fn append_blueprints(a: &Blueprint, b: &Blueprint) -> Blueprint {
    if a.is_empty() {
        return b.clone();
    }
    if b.is_empty() {
        return a.clone();
    }
    let mut entities = a.entities.clone();
    for e in &b.entities {
        let mut e = e.clone();
        e.offset_y += a.height;
        entities.push(e);
    }
    Blueprint {
        entities,
        width: a.width.max(b.width),
        height: a.height + b.height,
        linewise: a.linewise || b.linewise,
    }
}

fn format_content(content: &RegisterContent) -> String {
    match content {
        RegisterContent::Blueprint(bp) => bp.summary(),
        RegisterContent::Macro(keys) => format!("macro ({} keys)", keys.len()),
    }
}
