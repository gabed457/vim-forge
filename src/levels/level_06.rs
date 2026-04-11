use crate::resources::{EntityType, Facing};

use super::config::{CompletionCondition, LevelConfig, LevelEntity};

/// Level 6: "Copy That" — Use yy/p to duplicate production lines.
pub fn config() -> LevelConfig {
    let entities = vec![
        // Ore deposits
        LevelEntity {
            x: 1,
            y: 3,
            entity_type: EntityType::OreDeposit,
            facing: Facing::Right,
            player_placed: false,
        },
        LevelEntity {
            x: 1,
            y: 7,
            entity_type: EntityType::OreDeposit,
            facing: Facing::Right,
            player_placed: false,
        },
        LevelEntity {
            x: 1,
            y: 11,
            entity_type: EntityType::OreDeposit,
            facing: Facing::Right,
            player_placed: false,
        },
        // Output bins
        LevelEntity {
            x: 28,
            y: 3,
            entity_type: EntityType::OutputBin,
            facing: Facing::Right,
            player_placed: false,
        },
        LevelEntity {
            x: 28,
            y: 7,
            entity_type: EntityType::OutputBin,
            facing: Facing::Right,
            player_placed: false,
        },
        LevelEntity {
            x: 28,
            y: 11,
            entity_type: EntityType::OutputBin,
            facing: Facing::Right,
            player_placed: false,
        },
    ];

    LevelConfig {
        number: 6,
        name: "Copy That",
        map_width: 30,
        map_height: 15,
        entities,
        objective: "Build one line, yy to copy, p to paste. 9 widgets total.",
        hints: vec![
            "Build a full production line on row 3 first.",
            "Use yy to yank (copy) the current row of entities.",
            "Move to the target row and use p to paste.",
            "Each line needs: ore -> conveyors -> smelter -> conveyors -> assembler -> conveyors -> bin.",
            "All three lines must produce widgets for a total of 9.",
        ],
        allowed_commands: None,
        completion: CompletionCondition::ProduceWidgets(9),
    }
}
