use crate::resources::{EntityType, Facing};

use super::config::{CompletionCondition, LevelConfig, LevelEntity};

/// Level 11: "The Dot" — Use ~ (rotate) and . (repeat) to fix conveyor facings.
pub fn config() -> LevelConfig {
    let mut entities = Vec::new();

    // Ore deposit at (1,7)
    entities.push(LevelEntity {
        x: 1,
        y: 7,
        entity_type: EntityType::OreDeposit,
        facing: Facing::Right,
        player_placed: false,
    });

    // 15 conveyors at y=7, x=2..16, all facing Up (should face Right)
    for x in 2..=16 {
        entities.push(LevelEntity {
            x,
            y: 7,
            entity_type: EntityType::BasicBelt,
            facing: Facing::Up,
            player_placed: false,
        });
    }

    // Output bin at (18,7)
    entities.push(LevelEntity {
        x: 18,
        y: 7,
        entity_type: EntityType::OutputBin,
        facing: Facing::Right,
        player_placed: false,
    });

    LevelConfig {
        number: 11,
        name: "The Dot",
        map_width: 30,
        map_height: 15,
        entities,
        objective: "Use ~ and . to fix all conveyors.",
        hints: vec![
            "The ~ key rotates the entity under the cursor clockwise.",
            "Press ~ on a conveyor facing Up to rotate it to Right.",
            "The . key repeats your last action.",
            "Move to the next conveyor with l, then press . to rotate it too.",
            "This is much faster than re-placing each conveyor!",
        ],
        allowed_commands: None,
        completion: CompletionCondition::Custom("all_conveyors_facing_right".to_string()),
    }
}
