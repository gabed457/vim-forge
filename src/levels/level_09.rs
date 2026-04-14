use crate::resources::{EntityType, Facing};

use super::config::{CompletionCondition, LevelConfig, LevelEntity};

/// Level 9: "Find & Jump" — Navigate a large factory with f, /, %.
pub fn config() -> LevelConfig {
    let mut entities = Vec::new();

    // Cluster 1 (top-left, belt_y=4): missing conveyor gap
    // OreDeposit (3×2) at (2,3): output at (4,4)
    entities.push(LevelEntity {
        x: 2, y: 3,
        entity_type: EntityType::OreDeposit,
        facing: Facing::Right,
        player_placed: false,
    });
    for x in 5..=8 {
        entities.push(LevelEntity {
            x, y: 4,
            entity_type: EntityType::BasicBelt,
            facing: Facing::Right,
            player_placed: false,
        });
    }
    // Gap at x=9
    // Smelter (3×3) at (10,3): input (10,4), output (12,4)
    entities.push(LevelEntity {
        x: 10, y: 3,
        entity_type: EntityType::Smelter,
        facing: Facing::Right,
        player_placed: false,
    });
    for x in 13..=18 {
        entities.push(LevelEntity {
            x, y: 4,
            entity_type: EntityType::BasicBelt,
            facing: Facing::Right,
            player_placed: false,
        });
    }
    // OutputBin (3×2) at (19,3): input at (19,4)
    entities.push(LevelEntity {
        x: 19, y: 3,
        entity_type: EntityType::OutputBin,
        facing: Facing::Right,
        player_placed: false,
    });

    // Cluster 2 (top-right, belt_y=6): smelter facing wrong
    // OreDeposit (3×2) at (40,5): output at (42,6)
    entities.push(LevelEntity {
        x: 40, y: 5,
        entity_type: EntityType::OreDeposit,
        facing: Facing::Right,
        player_placed: false,
    });
    for x in 43..=48 {
        entities.push(LevelEntity {
            x, y: 6,
            entity_type: EntityType::BasicBelt,
            facing: Facing::Right,
            player_placed: false,
        });
    }
    // Smelter (3×3) at (49,5) facing Left (wrong!)
    entities.push(LevelEntity {
        x: 49, y: 5,
        entity_type: EntityType::Smelter,
        facing: Facing::Left,
        player_placed: false,
    });
    for x in 52..=58 {
        entities.push(LevelEntity {
            x, y: 6,
            entity_type: EntityType::BasicBelt,
            facing: Facing::Right,
            player_placed: false,
        });
    }
    // OutputBin (3×2) at (59,5): input at (59,6)
    entities.push(LevelEntity {
        x: 59, y: 5,
        entity_type: EntityType::OutputBin,
        facing: Facing::Right,
        player_placed: false,
    });

    // Cluster 3 (middle, belt_y=15): missing conveyor gap
    // OreDeposit (3×2) at (22,14): output at (24,15)
    entities.push(LevelEntity {
        x: 22, y: 14,
        entity_type: EntityType::OreDeposit,
        facing: Facing::Right,
        player_placed: false,
    });
    for x in 25..=27 {
        entities.push(LevelEntity {
            x, y: 15,
            entity_type: EntityType::BasicBelt,
            facing: Facing::Right,
            player_placed: false,
        });
    }
    // Gap at x=28,29
    // Smelter (3×3) at (30,14): input (30,15), output (32,15)
    entities.push(LevelEntity {
        x: 30, y: 14,
        entity_type: EntityType::Smelter,
        facing: Facing::Right,
        player_placed: false,
    });
    for x in 33..=38 {
        entities.push(LevelEntity {
            x, y: 15,
            entity_type: EntityType::BasicBelt,
            facing: Facing::Right,
            player_placed: false,
        });
    }
    // OutputBin (3×2) at (39,14): input at (39,15)
    entities.push(LevelEntity {
        x: 39, y: 14,
        entity_type: EntityType::OutputBin,
        facing: Facing::Right,
        player_placed: false,
    });

    // Cluster 4 (bottom-left, belt_y=23): wrong-facing belt
    // OreDeposit (3×2) at (5,22): output at (7,23)
    entities.push(LevelEntity {
        x: 5, y: 22,
        entity_type: EntityType::OreDeposit,
        facing: Facing::Right,
        player_placed: false,
    });
    for x in 8..=10 {
        entities.push(LevelEntity {
            x, y: 23,
            entity_type: EntityType::BasicBelt,
            facing: Facing::Right,
            player_placed: false,
        });
    }
    // Wrong-facing belt
    entities.push(LevelEntity {
        x: 11, y: 23,
        entity_type: EntityType::BasicBelt,
        facing: Facing::Up,
        player_placed: false,
    });
    // Smelter (3×3) at (12,22): input (12,23), output (14,23)
    entities.push(LevelEntity {
        x: 12, y: 22,
        entity_type: EntityType::Smelter,
        facing: Facing::Right,
        player_placed: false,
    });
    for x in 15..=20 {
        entities.push(LevelEntity {
            x, y: 23,
            entity_type: EntityType::BasicBelt,
            facing: Facing::Right,
            player_placed: false,
        });
    }
    // OutputBin (3×2) at (21,22): input at (21,23)
    entities.push(LevelEntity {
        x: 21, y: 22,
        entity_type: EntityType::OutputBin,
        facing: Facing::Right,
        player_placed: false,
    });

    // Cluster 5 (bottom-right, belt_y=26): missing smelter
    // OreDeposit (3×2) at (42,25): output at (44,26)
    entities.push(LevelEntity {
        x: 42, y: 25,
        entity_type: EntityType::OreDeposit,
        facing: Facing::Right,
        player_placed: false,
    });
    for x in 45..=49 {
        entities.push(LevelEntity {
            x, y: 26,
            entity_type: EntityType::BasicBelt,
            facing: Facing::Right,
            player_placed: false,
        });
    }
    // Gap at x=50..52 (missing smelter)
    for x in 53..=58 {
        entities.push(LevelEntity {
            x, y: 26,
            entity_type: EntityType::BasicBelt,
            facing: Facing::Right,
            player_placed: false,
        });
    }
    // OutputBin (3×2) at (59,25): input at (59,26)
    entities.push(LevelEntity {
        x: 59, y: 25,
        entity_type: EntityType::OutputBin,
        facing: Facing::Right,
        player_placed: false,
    });

    LevelConfig {
        number: 9,
        name: "Find & Jump",
        map_width: 65,
        map_height: 30,
        entities,
        objective: "Navigate and fix all 5 broken clusters using f, /, and %.",
        hints: vec![
            "Huge map! Use fs to jump to the next smelter, fb to jump to the next output bin.",
            "Use / to search (e.g. /smelter then Enter). Press n for next match, N for previous.",
            "Stand on a belt and press % to follow the connection chain — find where it breaks!",
            "Five clusters are broken: missing belts, wrong facings, or missing smelters.",
            "Use ~ to rotate wrong entities, i then c for missing belts, i then s for missing smelters.",
        ],
        allowed_commands: None,
        completion: CompletionCondition::Custom("all_5_clusters_producing".to_string()),
    }
}
