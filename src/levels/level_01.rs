use crate::resources::{EntityType, Facing};

use super::config::{CompletionCondition, LevelConfig, LevelEntity};

/// Level 1: "Movement" — Pre-built working factory. Learn navigation.
pub fn config() -> LevelConfig {
    let mut entities = Vec::new();

    // === Row 1: belt_y=5, anchors at y=4 ===
    // OreDeposit (3×2) at (3,4): output port at (5,5)
    entities.push(LevelEntity {
        x: 3, y: 4,
        entity_type: EntityType::OreDeposit,
        facing: Facing::Right,
        player_placed: false,
    });
    // Belts from (6,5) to (16,5) — 11 tiles
    for x in 6..=16 {
        entities.push(LevelEntity {
            x, y: 5,
            entity_type: EntityType::BasicBelt,
            facing: Facing::Right,
            player_placed: false,
        });
    }
    // Smelter (3×3) at (17,4): input at (17,5), output at (19,5)
    entities.push(LevelEntity {
        x: 17, y: 4,
        entity_type: EntityType::Smelter,
        facing: Facing::Right,
        player_placed: false,
    });
    // Belts from (20,5) to (30,5) — 11 tiles
    for x in 20..=30 {
        entities.push(LevelEntity {
            x, y: 5,
            entity_type: EntityType::BasicBelt,
            facing: Facing::Right,
            player_placed: false,
        });
    }
    // OutputBin (3×2) at (31,4): input at (31,5)
    entities.push(LevelEntity {
        x: 31, y: 4,
        entity_type: EntityType::OutputBin,
        facing: Facing::Right,
        player_placed: false,
    });

    // === Row 2: belt_y=16, anchors at y=15 ===
    entities.push(LevelEntity {
        x: 3, y: 15,
        entity_type: EntityType::OreDeposit,
        facing: Facing::Right,
        player_placed: false,
    });
    for x in 6..=16 {
        entities.push(LevelEntity {
            x, y: 16,
            entity_type: EntityType::BasicBelt,
            facing: Facing::Right,
            player_placed: false,
        });
    }
    entities.push(LevelEntity {
        x: 17, y: 15,
        entity_type: EntityType::Smelter,
        facing: Facing::Right,
        player_placed: false,
    });
    for x in 20..=30 {
        entities.push(LevelEntity {
            x, y: 16,
            entity_type: EntityType::BasicBelt,
            facing: Facing::Right,
            player_placed: false,
        });
    }
    entities.push(LevelEntity {
        x: 31, y: 15,
        entity_type: EntityType::OutputBin,
        facing: Facing::Right,
        player_placed: false,
    });

    LevelConfig {
        number: 1,
        name: "Movement",
        map_width: 44,
        map_height: 22,
        entities,
        objective: "Move to BOTH Output Bins using Vim movement keys",
        hints: vec![
            "Welcome to VimForge! Two production lines are already running. Learn to navigate!",
            "Use h/j/k/l to move the cursor: h=left, j=down, k=up, l=right.",
            "Use number prefixes for speed: 5l moves 5 right, 10j moves 10 down.",
            "Press 0 to jump to line start, $ to line end. Press gg for map top, G for map bottom.",
            "Move your cursor onto BOTH green Output Bins (right side, rows 4 and 15) to win!",
        ],
        allowed_commands: Some(vec![
            "h", "j", "k", "l", "0", "$", "gg", "G", "H", "M", "L",
        ]),
        completion: CompletionCondition::NavigateToAll(vec![(31, 4), (31, 15)]),
    }
}
