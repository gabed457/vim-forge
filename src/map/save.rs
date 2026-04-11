use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;

use crate::commands::Blueprint;
use crate::ecs::components::*;
use crate::game::inventory::Inventory;
use crate::resources::{EntityType, Facing, Resource};

#[derive(Serialize, Deserialize)]
pub struct SaveData {
    pub version: u32,
    pub map_width: usize,
    pub map_height: usize,
    pub entities: Vec<SavedEntity>,
    pub resources: Vec<SavedResource>,
    pub registers: HashMap<char, Blueprint>,
    pub marks: HashMap<char, (usize, usize)>,
    pub score: ScoreData,
    pub simulation_speed: u32,
    pub tick_count: u64,
    pub tutorial_state: Option<TutorialSaveState>,
    #[serde(default)]
    pub inventory: Inventory,
}

#[derive(Serialize, Deserialize)]
pub struct SavedEntity {
    pub x: usize,
    pub y: usize,
    pub entity_type: EntityType,
    pub facing: Facing,
    pub processing_ticks: Option<u32>,
    pub input_a: Option<Resource>,
    pub input_b: Option<Resource>,
    pub output: Option<Resource>,
    pub ore_emit_counter: Option<u32>,
    pub output_counts: Option<(u64, u64, u64)>,
    pub splitter_state: Option<SplitterOutput>,
    pub merger_state: Option<MergerPriority>,
    pub player_placed: bool,
}

#[derive(Serialize, Deserialize)]
pub struct SavedResource {
    pub x: usize,
    pub y: usize,
    pub resource: Resource,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct ScoreData {
    pub total_widgets: u64,
    pub total_ingots: u64,
    pub total_ore: u64,
}

impl ScoreData {
    pub fn new() -> Self {
        ScoreData {
            total_widgets: 0,
            total_ingots: 0,
            total_ore: 0,
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct TutorialSaveState {
    pub current_level: usize,
    pub levels_completed: Vec<usize>,
    pub commands_learned: Vec<String>,
}

pub fn save_dir() -> std::path::PathBuf {
    let home = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
    Path::new(&home).join(".vimforge")
}

pub fn default_save_path() -> std::path::PathBuf {
    save_dir().join("save.json")
}

pub fn save_game(data: &SaveData, path: &Path) -> Result<(), String> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).map_err(|e| format!("Failed to create directory: {}", e))?;
    }
    let json =
        serde_json::to_string_pretty(data).map_err(|e| format!("Serialization error: {}", e))?;
    std::fs::write(path, json).map_err(|e| format!("Write error: {}", e))?;
    Ok(())
}

pub fn load_game(path: &Path) -> Result<SaveData, String> {
    let json = std::fs::read_to_string(path).map_err(|e| format!("Read error: {}", e))?;
    serde_json::from_str(&json).map_err(|e| format!("Deserialization error: {}", e))
}
