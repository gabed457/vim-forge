use crate::resources::{EntityType, Facing};

use super::config::{CompletionCondition, LevelConfig, LevelEntity};

/// Freeplay mode: large open map with scattered resources.
/// Uses fixed (deterministic) positions for reproducibility.
pub fn config() -> LevelConfig {
    let mut entities = Vec::new();

    // 10 ore deposits scattered across the map
    let ore_positions: [(usize, usize); 10] = [
        (2, 5),
        (2, 15),
        (2, 25),
        (2, 35),
        (2, 45),
        (40, 5),
        (40, 15),
        (40, 25),
        (40, 35),
        (40, 45),
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

    // 4 output bins
    let bin_positions: [(usize, usize); 4] = [(75, 10), (75, 20), (75, 30), (75, 40)];

    for &(x, y) in &bin_positions {
        entities.push(LevelEntity {
            x,
            y,
            entity_type: EntityType::OutputBin,
            facing: Facing::Right,
            player_placed: false,
        });
    }

    // Some walls for structure
    // Horizontal wall segments
    for x in 20..=25 {
        entities.push(LevelEntity {
            x,
            y: 0,
            entity_type: EntityType::Wall,
            facing: Facing::Right,
            player_placed: false,
        });
    }
    for x in 50..=55 {
        entities.push(LevelEntity {
            x,
            y: 49,
            entity_type: EntityType::Wall,
            facing: Facing::Right,
            player_placed: false,
        });
    }
    // Vertical wall segments
    for y in 20..=30 {
        entities.push(LevelEntity {
            x: 30,
            y,
            entity_type: EntityType::Wall,
            facing: Facing::Right,
            player_placed: false,
        });
    }
    for y in 10..=18 {
        entities.push(LevelEntity {
            x: 60,
            y,
            entity_type: EntityType::Wall,
            facing: Facing::Right,
            player_placed: false,
        });
    }

    LevelConfig {
        number: 0,
        name: "Freeplay",
        map_width: 80,
        map_height: 50,
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
