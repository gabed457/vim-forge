use crate::resources::{EntityType, Facing};

use super::config::{CompletionCondition, LevelConfig, LevelEntity};

/// Level 5: "Demolish & Rebuild" — Fix a broken factory using delete commands.
pub fn config() -> LevelConfig {
    let mut entities = Vec::new();

    // OreDeposit (3×2) at (2,7): output port at (4,8)
    entities.push(LevelEntity {
        x: 2, y: 7,
        entity_type: EntityType::OreDeposit,
        facing: Facing::Right,
        player_placed: false,
    });

    // OutputBin (3×2) at (44,7): input port at (44,8)
    entities.push(LevelEntity {
        x: 44, y: 7,
        entity_type: EntityType::OutputBin,
        facing: Facing::Right,
        player_placed: false,
    });

    // Correct conveyors at y=8
    for x in 5..=9 {
        entities.push(LevelEntity {
            x, y: 8,
            entity_type: EntityType::BasicBelt,
            facing: Facing::Right,
            player_placed: false,
        });
    }

    // Wrong-facing conveyors
    entities.push(LevelEntity {
        x: 10, y: 8,
        entity_type: EntityType::BasicBelt,
        facing: Facing::Left,
        player_placed: false,
    });
    entities.push(LevelEntity {
        x: 11, y: 8,
        entity_type: EntityType::BasicBelt,
        facing: Facing::Up,
        player_placed: false,
    });

    // Gap at x=12

    // Smelter (3×3) at (13,7) facing Up (wrong! should be Right)
    entities.push(LevelEntity {
        x: 13, y: 7,
        entity_type: EntityType::Smelter,
        facing: Facing::Up,
        player_placed: false,
    });

    // Wrong-facing conveyor after smelter
    entities.push(LevelEntity {
        x: 16, y: 8,
        entity_type: EntityType::BasicBelt,
        facing: Facing::Left,
        player_placed: false,
    });

    // Correct conveyors
    for x in 17..=27 {
        entities.push(LevelEntity {
            x, y: 8,
            entity_type: EntityType::BasicBelt,
            facing: Facing::Right,
            player_placed: false,
        });
    }

    // Gap at x=28..30

    // More correct conveyors
    for x in 31..=43 {
        entities.push(LevelEntity {
            x, y: 8,
            entity_type: EntityType::BasicBelt,
            facing: Facing::Right,
            player_placed: false,
        });
    }

    LevelConfig {
        number: 5,
        name: "Demolish & Rebuild",
        map_width: 48,
        map_height: 18,
        entities,
        objective: "Fix the broken factory: rotate, delete, and fill gaps. Deliver 5 ingots.",
        hints: vec![
            "This factory is broken! Wrong-facing belts, a rotated smelter, and gaps in the line.",
            "In Normal mode, move onto an entity and press ~ to rotate it clockwise.",
            "Use ~ on the smelter to fix its facing. Use ~ on wrong-facing belts too!",
            "Press x to delete the entity under the cursor. Use d+motion for ranges (d3l = 3 right).",
            "Fill gaps: press i to insert, c for belts. Fix everything and deliver 5 ingots!",
        ],
        allowed_commands: None,
        completion: CompletionCondition::DeliverIngots(5),
    }
}
