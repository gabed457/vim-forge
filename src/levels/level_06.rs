use crate::resources::{EntityType, Facing};

use super::config::{CompletionCondition, LevelConfig, LevelEntity};

/// Level 6: "Copy That" — Use yy/p to duplicate production lines.
pub fn config() -> LevelConfig {
    let entities = vec![
        // Ore deposits (3×2): anchors at belt_y-1
        LevelEntity {
            x: 1, y: 3,
            entity_type: EntityType::OreDeposit,
            facing: Facing::Right,
            player_placed: false,
        },
        LevelEntity {
            x: 1, y: 9,
            entity_type: EntityType::OreDeposit,
            facing: Facing::Right,
            player_placed: false,
        },
        LevelEntity {
            x: 1, y: 15,
            entity_type: EntityType::OreDeposit,
            facing: Facing::Right,
            player_placed: false,
        },
        // Output bins (3×2): anchors at belt_y-1
        LevelEntity {
            x: 38, y: 3,
            entity_type: EntityType::OutputBin,
            facing: Facing::Right,
            player_placed: false,
        },
        LevelEntity {
            x: 38, y: 9,
            entity_type: EntityType::OutputBin,
            facing: Facing::Right,
            player_placed: false,
        },
        LevelEntity {
            x: 38, y: 15,
            entity_type: EntityType::OutputBin,
            facing: Facing::Right,
            player_placed: false,
        },
    ];

    LevelConfig {
        number: 6,
        name: "Copy That",
        map_width: 42,
        map_height: 20,
        entities,
        objective: "Build one line, yy to copy, p to paste across 3 rows. 9 widgets.",
        hints: vec![
            "Three rows of ore deposits and output bins. Build a full line on the top row first.",
            "Top row belt line is at row 4. Build: belts -> smelter -> belts -> assembler -> belts.",
            "Done building? Press Esc to Normal mode. Move cursor to your built row, press yy to copy it.",
            "Move down to the next row (j), press p to paste your line. Repeat for the third row.",
            "All three lines need to produce widgets. 9 widgets total to win!",
        ],
        allowed_commands: None,
        completion: CompletionCondition::ProduceWidgets(9),
    }
}
