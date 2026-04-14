use crate::resources::{EntityType, Facing};

use super::config::{CompletionCondition, LevelConfig, LevelEntity};

/// Level 11: "The Dot" — Use ~ (rotate) and . (repeat) to fix conveyor facings.
pub fn config() -> LevelConfig {
    let mut entities = Vec::new();

    // OreDeposit (3×2) at (3,9): output at (5,10)
    entities.push(LevelEntity {
        x: 3, y: 9,
        entity_type: EntityType::OreDeposit,
        facing: Facing::Right,
        player_placed: false,
    });

    // 22 conveyors at y=10, x=6..27, all facing Up (should face Right)
    for x in 6..=27 {
        entities.push(LevelEntity {
            x, y: 10,
            entity_type: EntityType::BasicBelt,
            facing: Facing::Up,
            player_placed: false,
        });
    }

    // OutputBin (3×2) at (28,9): input at (28,10)
    entities.push(LevelEntity {
        x: 28, y: 9,
        entity_type: EntityType::OutputBin,
        facing: Facing::Right,
        player_placed: false,
    });

    LevelConfig {
        number: 11,
        name: "The Dot",
        map_width: 36,
        map_height: 22,
        entities,
        objective: "Rotate all 22 belts from Up to Right using ~ and . (dot repeat).",
        hints: vec![
            "All 22 belts face Up but should face Right. The ~ key rotates clockwise: Up -> Right.",
            "Move onto the first belt (row 10, col 6) and press ~ to rotate it.",
            "Press . (dot) to repeat your last edit. The dot command replays the previous change!",
            "Move right with l, then press . to rotate the next belt. Repeat: l then . across the row.",
            "The pattern: ~ on the first belt, then l . l . l . ... Fix all 22 in seconds!",
        ],
        allowed_commands: None,
        completion: CompletionCondition::Custom("all_conveyors_facing_right".to_string()),
    }
}
