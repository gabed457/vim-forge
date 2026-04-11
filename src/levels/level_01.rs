use crate::resources::{EntityType, Facing};

use super::config::{CompletionCondition, LevelConfig, LevelEntity};

/// Level 1: "Movement" — Pre-built working factory. Learn navigation.
pub fn config() -> LevelConfig {
    let mut entities = Vec::new();

    // Ore deposits
    entities.push(LevelEntity {
        x: 2,
        y: 2,
        entity_type: EntityType::OreDeposit,
        facing: Facing::Right,
        player_placed: false,
    });
    entities.push(LevelEntity {
        x: 2,
        y: 7,
        entity_type: EntityType::OreDeposit,
        facing: Facing::Right,
        player_placed: false,
    });

    // Smelters
    entities.push(LevelEntity {
        x: 8,
        y: 2,
        entity_type: EntityType::Smelter,
        facing: Facing::Right,
        player_placed: false,
    });
    entities.push(LevelEntity {
        x: 8,
        y: 7,
        entity_type: EntityType::Smelter,
        facing: Facing::Right,
        player_placed: false,
    });

    // Conveyor chains: (3,2)->(7,2) facing Right (before smelter)
    for x in 3..=7 {
        entities.push(LevelEntity {
            x,
            y: 2,
            entity_type: EntityType::BasicBelt,
            facing: Facing::Right,
            player_placed: false,
        });
    }

    // Conveyor chains: (9,2)->(14,2) facing Right (after smelter)
    for x in 9..=14 {
        entities.push(LevelEntity {
            x,
            y: 2,
            entity_type: EntityType::BasicBelt,
            facing: Facing::Right,
            player_placed: false,
        });
    }

    // Same pattern for row 7: (3,7)->(7,7) facing Right
    for x in 3..=7 {
        entities.push(LevelEntity {
            x,
            y: 7,
            entity_type: EntityType::BasicBelt,
            facing: Facing::Right,
            player_placed: false,
        });
    }

    // (9,7)->(14,7) facing Right
    for x in 9..=14 {
        entities.push(LevelEntity {
            x,
            y: 7,
            entity_type: EntityType::BasicBelt,
            facing: Facing::Right,
            player_placed: false,
        });
    }

    // Output bins
    entities.push(LevelEntity {
        x: 15,
        y: 2,
        entity_type: EntityType::OutputBin,
        facing: Facing::Right,
        player_placed: false,
    });
    entities.push(LevelEntity {
        x: 15,
        y: 7,
        entity_type: EntityType::OutputBin,
        facing: Facing::Right,
        player_placed: false,
    });

    LevelConfig {
        number: 1,
        name: "Movement",
        map_width: 20,
        map_height: 10,
        entities,
        objective: "Move to BOTH Output Bins (green B) using Vim movement keys",
        hints: vec![
            "The factory is already running! Watch ore flow through conveyors into smelters. You just need to move around.",
            "Move your cursor with h (left) j (down) k (up) l (right). Try it now!",
            "Press 0 to jump to the start of a row, $ to jump to the end.",
            "Press gg to jump to the top of the map, G to jump to the bottom.",
            "Move your cursor onto BOTH green B tiles to complete this level! One is at row 2, the other at row 7.",
        ],
        allowed_commands: Some(vec![
            "h", "j", "k", "l", "0", "$", "gg", "G", "H", "M", "L",
        ]),
        completion: CompletionCondition::NavigateToAll(vec![(15, 2), (15, 7)]),
    }
}
