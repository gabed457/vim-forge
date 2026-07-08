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

/// The pseudo-level number used for the freeplay sandbox map.
pub const FREEPLAY_LEVEL: usize = 31;

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
        14 => Some(super::level_14::config()),
        15 => Some(super::level_15::config()),
        16 => Some(super::level_16::config()),
        17 => Some(super::level_17::config()),
        18 => Some(super::level_18::config()),
        19 => Some(super::level_19::config()),
        20 => Some(super::level_20::config()),
        21 => Some(super::level_21::config()),
        22 => Some(super::level_22::config()),
        23 => Some(super::level_23::config()),
        24 => Some(super::level_24::config()),
        25 => Some(super::level_25::config()),
        26 => Some(super::level_26::config()),
        27 => Some(super::level_27::config()),
        28 => Some(super::level_28::config()),
        29 => Some(super::level_29::config()),
        30 => Some(super::level_30::config()),
        31 => Some(super::freeplay::config()),
        _ => None,
    }
}

pub fn total_levels() -> usize {
    30
}
