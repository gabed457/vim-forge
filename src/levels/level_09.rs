use crate::resources::{EntityType, Facing};

use super::config::{CompletionCondition, LevelConfig, LevelEntity};

/// Level 9: "Find & Jump" — Navigate a large factory with f, /, %.
pub fn config() -> LevelConfig {
    let mut entities = Vec::new();

    // Cluster 1 (top-left area, around row 3): missing some conveyors
    entities.push(LevelEntity {
        x: 2,
        y: 3,
        entity_type: EntityType::OreDeposit,
        facing: Facing::Right,
        player_placed: false,
    });
    for x in 3..=6 {
        entities.push(LevelEntity {
            x,
            y: 3,
            entity_type: EntityType::Conveyor,
            facing: Facing::Right,
            player_placed: false,
        });
    }
    // Gap at x=7
    entities.push(LevelEntity {
        x: 8,
        y: 3,
        entity_type: EntityType::Smelter,
        facing: Facing::Right,
        player_placed: false,
    });
    for x in 9..=12 {
        entities.push(LevelEntity {
            x,
            y: 3,
            entity_type: EntityType::Conveyor,
            facing: Facing::Right,
            player_placed: false,
        });
    }
    entities.push(LevelEntity {
        x: 13,
        y: 3,
        entity_type: EntityType::OutputBin,
        facing: Facing::Right,
        player_placed: false,
    });

    // Cluster 2 (top-right area, around row 5): wrong facing
    entities.push(LevelEntity {
        x: 35,
        y: 5,
        entity_type: EntityType::OreDeposit,
        facing: Facing::Right,
        player_placed: false,
    });
    for x in 36..=40 {
        entities.push(LevelEntity {
            x,
            y: 5,
            entity_type: EntityType::Conveyor,
            facing: Facing::Right,
            player_placed: false,
        });
    }
    entities.push(LevelEntity {
        x: 41,
        y: 5,
        entity_type: EntityType::Smelter,
        facing: Facing::Left, // Wrong! Should be Right
        player_placed: false,
    });
    for x in 42..=46 {
        entities.push(LevelEntity {
            x,
            y: 5,
            entity_type: EntityType::Conveyor,
            facing: Facing::Right,
            player_placed: false,
        });
    }
    entities.push(LevelEntity {
        x: 47,
        y: 5,
        entity_type: EntityType::OutputBin,
        facing: Facing::Right,
        player_placed: false,
    });

    // Cluster 3 (middle area, around row 14): missing conveyors
    entities.push(LevelEntity {
        x: 20,
        y: 14,
        entity_type: EntityType::OreDeposit,
        facing: Facing::Right,
        player_placed: false,
    });
    for x in 21..=23 {
        entities.push(LevelEntity {
            x,
            y: 14,
            entity_type: EntityType::Conveyor,
            facing: Facing::Right,
            player_placed: false,
        });
    }
    // Gap at x=24,25
    entities.push(LevelEntity {
        x: 26,
        y: 14,
        entity_type: EntityType::Smelter,
        facing: Facing::Right,
        player_placed: false,
    });
    for x in 27..=31 {
        entities.push(LevelEntity {
            x,
            y: 14,
            entity_type: EntityType::Conveyor,
            facing: Facing::Right,
            player_placed: false,
        });
    }
    entities.push(LevelEntity {
        x: 32,
        y: 14,
        entity_type: EntityType::OutputBin,
        facing: Facing::Right,
        player_placed: false,
    });

    // Cluster 4 (bottom-left, around row 22): wrong facing conveyors
    entities.push(LevelEntity {
        x: 5,
        y: 22,
        entity_type: EntityType::OreDeposit,
        facing: Facing::Right,
        player_placed: false,
    });
    for x in 6..=8 {
        entities.push(LevelEntity {
            x,
            y: 22,
            entity_type: EntityType::Conveyor,
            facing: Facing::Right,
            player_placed: false,
        });
    }
    entities.push(LevelEntity {
        x: 9,
        y: 22,
        entity_type: EntityType::Conveyor,
        facing: Facing::Up, // Wrong!
        player_placed: false,
    });
    entities.push(LevelEntity {
        x: 10,
        y: 22,
        entity_type: EntityType::Smelter,
        facing: Facing::Right,
        player_placed: false,
    });
    for x in 11..=15 {
        entities.push(LevelEntity {
            x,
            y: 22,
            entity_type: EntityType::Conveyor,
            facing: Facing::Right,
            player_placed: false,
        });
    }
    entities.push(LevelEntity {
        x: 16,
        y: 22,
        entity_type: EntityType::OutputBin,
        facing: Facing::Right,
        player_placed: false,
    });

    // Cluster 5 (bottom-right, around row 25): missing smelter
    entities.push(LevelEntity {
        x: 40,
        y: 25,
        entity_type: EntityType::OreDeposit,
        facing: Facing::Right,
        player_placed: false,
    });
    for x in 41..=45 {
        entities.push(LevelEntity {
            x,
            y: 25,
            entity_type: EntityType::Conveyor,
            facing: Facing::Right,
            player_placed: false,
        });
    }
    // Gap at x=46 (missing smelter)
    for x in 47..=52 {
        entities.push(LevelEntity {
            x,
            y: 25,
            entity_type: EntityType::Conveyor,
            facing: Facing::Right,
            player_placed: false,
        });
    }
    entities.push(LevelEntity {
        x: 53,
        y: 25,
        entity_type: EntityType::OutputBin,
        facing: Facing::Right,
        player_placed: false,
    });

    LevelConfig {
        number: 9,
        name: "Find & Jump",
        map_width: 60,
        map_height: 30,
        entities,
        objective: "Navigate large factory with f, /, %. Fix all 5 clusters.",
        hints: vec![
            "Use f followed by a character to jump to the next entity of that type.",
            "Use / to search for entity types by name.",
            "Use % to jump between matching input/output pairs.",
            "Each cluster has a different problem: missing conveyors, wrong facings, or missing smelters.",
            "Fix all 5 clusters so they produce output.",
        ],
        allowed_commands: None,
        completion: CompletionCondition::Custom("all_5_clusters_producing".to_string()),
    }
}
