use crate::resources::{EntityType, Facing};

#[derive(Clone, Debug)]
pub struct LevelConfig {
    pub number: usize,
    pub name: &'static str,
    pub map_width: usize,
    pub map_height: usize,
    pub entities: Vec<LevelEntity>,
    pub objective: &'static str,
    pub hints: Vec<&'static str>,
    pub allowed_commands: Option<Vec<&'static str>>, // None = all allowed
    pub completion: CompletionCondition,
}

#[derive(Clone, Debug)]
pub struct LevelEntity {
    pub x: usize,
    pub y: usize,
    pub entity_type: EntityType,
    pub facing: Facing,
    pub player_placed: bool,
}

#[derive(Clone, Debug)]
pub enum CompletionCondition {
    ProduceWidgets(u64),
    DeliverOre(u64),
    DeliverIngots(u64),
    NavigateToAll(Vec<(usize, usize)>),
    UseCommands(Vec<String>),
    ScoreInMoves(u64, usize),
    Custom(String),
}

pub fn get_level(number: usize) -> Option<LevelConfig> {
    match number {
        1 => Some(super::level_01::config()),
        2 => Some(super::level_02::config()),
        3 => Some(super::level_03::config()),
        4 => Some(super::level_04::config()),
        5 => Some(super::level_05::config()),
        6 => Some(super::level_06::config()),
        7 => Some(super::level_07::config()),
        8 => Some(super::level_08::config()),
        9 => Some(super::level_09::config()),
        10 => Some(super::level_10::config()),
        11 => Some(super::level_11::config()),
        12 => Some(super::level_12::config()),
        13 => Some(super::level_13::config()),
        14 => Some(super::freeplay::config()),
        _ => None,
    }
}

pub fn total_levels() -> usize {
    13
}
