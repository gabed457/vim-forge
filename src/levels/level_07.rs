use crate::resources::{EntityType, Facing};

use super::config::{CompletionCondition, LevelConfig, LevelEntity};

/// Level 7: "Blueprints" — Use named registers to save and paste factory sections.
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
        LevelEntity {
            x: 1,
            y: 15,
            entity_type: EntityType::OreDeposit,
            facing: Facing::Right,
            player_placed: false,
        },
        // Output bins
        LevelEntity {
            x: 38,
            y: 5,
            entity_type: EntityType::OutputBin,
            facing: Facing::Right,
            player_placed: false,
        },
        LevelEntity {
            x: 38,
            y: 13,
            entity_type: EntityType::OutputBin,
            facing: Facing::Right,
            player_placed: false,
        },
    ];

    LevelConfig {
        number: 7,
        name: "Blueprints",
        map_width: 40,
        map_height: 20,
        entities,
        objective: "Use named registers. \"a2yy to save, \"ap to paste. 10 widgets.",
        hints: vec![
            "Build a two-row production cluster that merges into one output.",
            "Use \"a2yy to yank 2 rows into register 'a'.",
            "Navigate to where you want the next cluster.",
            "Use \"ap to paste from register 'a'.",
            "Named registers (a-z) persist until overwritten.",
        ],
        allowed_commands: None,
        completion: CompletionCondition::ProduceWidgets(10),
    }
}
