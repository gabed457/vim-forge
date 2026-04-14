use crate::resources::{EntityType, Facing};

use super::config::{CompletionCondition, LevelConfig, LevelEntity};

/// Freeplay mode: large open map with scattered resources.
/// Uses fixed (deterministic) positions for reproducibility.
pub fn config() -> LevelConfig {
    let mut entities = Vec::new();

    // 10 ore deposits (3x2) scattered across the map
    let ore_positions: [(usize, usize); 10] = [
        (2, 5),
        (2, 15),
        (2, 30),
        (2, 45),
        (2, 60),
        (55, 5),
        (55, 15),
        (55, 30),
        (55, 45),
        (55, 60),
    ];

    for &(x, y) in &ore_positions {
        entities.push(LevelEntity {
            x,
            y,
            entity_type: EntityType::OreDeposit,
            facing: Facing::Right,
            player_placed: false,
        });
    }

    // 4 output bins (3x2)
    let bin_positions: [(usize, usize); 4] = [
        (110, 10),
        (110, 25),
        (110, 45),
        (110, 60),
    ];

    for &(x, y) in &bin_positions {
        entities.push(LevelEntity {
            x,
            y,
            entity_type: EntityType::OutputBin,
            facing: Facing::Right,
            player_placed: false,
        });
    }

    // Some walls for structure (1x1 each)
    // Horizontal wall segments
    for x in 30..=35 {
        entities.push(LevelEntity {
            x,
            y: 0,
            entity_type: EntityType::Wall,
            facing: Facing::Right,
            player_placed: false,
        });
    }
    for x in 70..=75 {
        entities.push(LevelEntity {
            x,
            y: 79,
            entity_type: EntityType::Wall,
            facing: Facing::Right,
            player_placed: false,
        });
    }
    // Vertical wall segments
    for y in 30..=45 {
        entities.push(LevelEntity {
            x: 45,
            y,
            entity_type: EntityType::Wall,
            facing: Facing::Right,
            player_placed: false,
        });
    }
    for y in 15..=28 {
        entities.push(LevelEntity {
            x: 85,
            y,
            entity_type: EntityType::Wall,
            facing: Facing::Right,
            player_placed: false,
        });
    }

    LevelConfig {
        number: 0,
        name: "Freeplay",
        map_width: 120,
        map_height: 80,
        entities,
        objective: "Build whatever you want! No restrictions, no completion condition.",
        hints: vec![
            "This is freeplay mode. All commands are available.",
            "Experiment with different factory layouts.",
            "Try to maximize widget production!",
        ],
        allowed_commands: None,
        completion: CompletionCondition::Custom("freeplay".to_string()),
    }
}
