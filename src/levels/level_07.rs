use crate::resources::{EntityType, Facing};

use super::config::{CompletionCondition, LevelConfig, LevelEntity};

/// Level 7: "Blueprints" — Use named registers to save and paste factory sections.
pub fn config() -> LevelConfig {
    let entities = vec![
        // Ore deposits (3×2)
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
        LevelEntity {
            x: 1, y: 21,
            entity_type: EntityType::OreDeposit,
            facing: Facing::Right,
            player_placed: false,
        },
        // Output bins (3×2) for merged pairs
        LevelEntity {
            x: 50, y: 6,
            entity_type: EntityType::OutputBin,
            facing: Facing::Right,
            player_placed: false,
        },
        LevelEntity {
            x: 50, y: 18,
            entity_type: EntityType::OutputBin,
            facing: Facing::Right,
            player_placed: false,
        },
    ];

    LevelConfig {
        number: 7,
        name: "Blueprints",
        map_width: 54,
        map_height: 26,
        entities,
        objective: "Use named registers (\"a) to save and replicate blueprints. 10 widgets.",
        hints: vec![
            "Build a two-row production cluster near the top. Both rows feed into one output bin.",
            "In Normal mode, press \"a then 2yy to yank 2 rows into named register 'a'.",
            "Navigate down to the next cluster area with j or G.",
            "Press \"a then p to paste your saved blueprint from register 'a'.",
            "Named registers (a-z) persist until overwritten. Produce 10 widgets to win!",
        ],
        allowed_commands: None,
        completion: CompletionCondition::ProduceWidgets(10),
    }
}
