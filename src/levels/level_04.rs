use crate::resources::{EntityType, Facing};

use super::config::{CompletionCondition, LevelConfig, LevelEntity};

/// Level 4: "Full Production" — Two ore lines, smelters, assembler, widgets.
pub fn config() -> LevelConfig {
    let entities = vec![
        // OreDeposit (3×2) at (2,4): output at (4,5), belt_y=5
        LevelEntity {
            x: 2, y: 4,
            entity_type: EntityType::OreDeposit,
            facing: Facing::Right,
            player_placed: false,
        },
        // OreDeposit (3×2) at (2,13): output at (4,14), belt_y=14
        LevelEntity {
            x: 2, y: 13,
            entity_type: EntityType::OreDeposit,
            facing: Facing::Right,
            player_placed: false,
        },
        // OutputBin (3×2) at (48,8): input at (48,9)
        LevelEntity {
            x: 48, y: 8,
            entity_type: EntityType::OutputBin,
            facing: Facing::Right,
            player_placed: false,
        },
    ];

    LevelConfig {
        number: 4,
        name: "Full Production",
        map_width: 52,
        map_height: 20,
        entities,
        objective: "Two ore lines -> smelters -> assembler -> output. Deliver 3 widgets.",
        hints: vec![
            "You need TWO ingot lines feeding one assembler. Start with the top deposit (row 5).",
            "Build each line like Level 3: belts (c), then k + s for smelter, then j + c for more belts.",
            "Place an assembler (a, 3x4). It takes TWO inputs on its left side, one row apart.",
            "Use arrow keys in insert mode to change belt direction. Turn belts to route both lines in.",
            "Connect the assembler's right-side output to the bin with belts. 3 widgets to win!",
        ],
        allowed_commands: None,
        completion: CompletionCondition::ProduceWidgets(3),
    }
}
