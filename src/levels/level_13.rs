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
        objective: "Use split views to see both ends of the map while building. 5 widgets.",
        hints: vec![
            "Ingot production is top-left, the Output Bin is bottom-right. You need to connect them!",
            "Press Ctrl-w v to split vertically. Use Ctrl-w h/l to switch between left and right panes.",
            "Keep one pane on the ingot lines (top-left) and the other on the bin area (bottom-right).",
            "Build a long belt chain with direction changes (arrow keys in insert mode) to route between.",
            "Place an assembler along the route to convert ingots to widgets. 5 widgets to win!",
        ],
        allowed_commands: None,
        completion: CompletionCondition::ProduceWidgets(5),
    }
}
