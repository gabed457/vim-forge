use crate::resources::{EntityType, Facing};

use super::config::{CompletionCondition, LevelConfig, LevelEntity};

/// Level 13: "Split View" — Use split views to build cross-map conveyor chains.
pub fn config() -> LevelConfig {
    let mut entities = Vec::new();

    // Top-left factory producing ingots
    // Top ore line (belt_y=3)
    entities.push(LevelEntity {
        x: 2, y: 2,
        entity_type: EntityType::OreDeposit,
        facing: Facing::Right,
        player_placed: false,
    });
    for x in 5..=9 {
        entities.push(LevelEntity {
            x, y: 3,
            entity_type: EntityType::BasicBelt,
            facing: Facing::Right,
            player_placed: false,
        });
    }
    entities.push(LevelEntity {
        x: 10, y: 2,
        entity_type: EntityType::Smelter,
        facing: Facing::Right,
        player_placed: false,
    });
    for x in 13..=20 {
        entities.push(LevelEntity {
            x, y: 3,
            entity_type: EntityType::BasicBelt,
            facing: Facing::Right,
            player_placed: false,
        });
    }

    // Bottom ore line (belt_y=7)
    entities.push(LevelEntity {
        x: 2, y: 6,
        entity_type: EntityType::OreDeposit,
        facing: Facing::Right,
        player_placed: false,
    });
    for x in 5..=9 {
        entities.push(LevelEntity {
            x, y: 7,
            entity_type: EntityType::BasicBelt,
            facing: Facing::Right,
            player_placed: false,
        });
    }
    entities.push(LevelEntity {
        x: 10, y: 6,
        entity_type: EntityType::Smelter,
        facing: Facing::Right,
        player_placed: false,
    });
    for x in 13..=20 {
        entities.push(LevelEntity {
            x, y: 7,
            entity_type: EntityType::BasicBelt,
            facing: Facing::Right,
            player_placed: false,
        });
    }

    // Bottom-right assembly area — output bin
    entities.push(LevelEntity {
        x: 77, y: 37,
        entity_type: EntityType::OutputBin,
        facing: Facing::Right,
        player_placed: false,
    });

    LevelConfig {
        number: 13,
        name: "Split View",
        map_width: 82,
        map_height: 42,
        entities,
        objective: "Work both ends of a huge map: marks + zz + a cross-map route. 5 widgets.",
        hints: vec![
            "Ingot production is top-left, the Output Bin is bottom-right. Connect them across the map!",
            "Drop marks at both work sites: ma at the ingot lines, mb at the bin. Jump with `a and `b.",
            "Press zz to center the view on your cursor after each jump — never lose your place.",
            "Build the long belt chain with facing changes (Shift-J/K or arrows in insert mode).",
            "Route through an assembler to make widgets. (Ctrl-w split panes are on the roadmap!)",
        ],
        allowed_commands: None,
        completion: CompletionCondition::ProduceWidgets(5),
    }
}
