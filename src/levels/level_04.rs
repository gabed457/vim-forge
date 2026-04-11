use crate::resources::{EntityType, Facing};

use super::config::{CompletionCondition, LevelConfig, LevelEntity};

/// Level 4: "Full Production" — Two ore lines, smelters, assembler, widgets.
pub fn config() -> LevelConfig {
    let entities = vec![
        LevelEntity {
            x: 1,
            y: 3,
            entity_type: EntityType::OreDeposit,
            facing: Facing::Right,
            player_placed: false,
        },
        LevelEntity {
            x: 1,
            y: 9,
            entity_type: EntityType::OreDeposit,
            facing: Facing::Right,
            player_placed: false,
        },
        LevelEntity {
            x: 23,
            y: 6,
            entity_type: EntityType::OutputBin,
            facing: Facing::Right,
            player_placed: false,
        },
    ];

    LevelConfig {
        number: 4,
        name: "Full Production",
        map_width: 25,
        map_height: 12,
        entities,
        objective: "Two ore lines -> smelters -> assembler -> widgets. Deliver 3 widgets.",
        hints: vec![
            "Build two parallel conveyor lines from each ore deposit.",
            "Place a smelter on each line to produce ingots.",
            "Route both ingot lines to an assembler (a in Insert mode).",
            "Assemblers take inputs from the sides and output forward.",
            "Connect the assembler output to the bin with conveyors.",
        ],
        allowed_commands: None,
        completion: CompletionCondition::ProduceWidgets(3),
    }
}
