use crate::resources::{EntityType, Facing};

use super::config::{CompletionCondition, LevelConfig, LevelEntity};

/// Level 3: "Smelting" — Build a smelting pipeline.
pub fn config() -> LevelConfig {
    let entities = vec![
        LevelEntity {
            x: 1,
            y: 4,
            entity_type: EntityType::OreDeposit,
            facing: Facing::Right,
            player_placed: false,
        },
        LevelEntity {
            x: 18,
            y: 4,
            entity_type: EntityType::OutputBin,
            facing: Facing::Right,
            player_placed: false,
        },
    ];

    LevelConfig {
        number: 3,
        name: "Smelting",
        map_width: 20,
        map_height: 8,
        entities,
        objective: "Build: Ore -> Conveyors -> Smelter -> Conveyors -> Output. Deliver 3 ingots.",
        hints: vec![
            "Place conveyors from the ore deposit toward the middle.",
            "Place a smelter (s in Insert mode) to convert ore into ingots.",
            "Smelters take input from behind and output forward.",
            "Continue with conveyors from the smelter to the output bin.",
        ],
        allowed_commands: None,
        completion: CompletionCondition::DeliverIngots(3),
    }
}
