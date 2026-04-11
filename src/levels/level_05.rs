use crate::resources::{EntityType, Facing};

use super::config::{CompletionCondition, LevelConfig, LevelEntity};

/// Level 5: "Demolish & Rebuild" — Fix a broken factory using delete commands.
pub fn config() -> LevelConfig {
    let mut entities = Vec::new();

    // Ore deposit
    entities.push(LevelEntity {
        x: 1,
        y: 5,
        entity_type: EntityType::OreDeposit,
        facing: Facing::Right,
        player_placed: false,
    });

    // Output bin
    entities.push(LevelEntity {
        x: 23,
        y: 5,
        entity_type: EntityType::OutputBin,
        facing: Facing::Right,
        player_placed: false,
    });

    // Some correct conveyors
    for x in 2..=4 {
        entities.push(LevelEntity {
            x,
            y: 5,
            entity_type: EntityType::Conveyor,
            facing: Facing::Right,
            player_placed: false,
        });
    }

    // Wrong-facing conveyors (Left instead of Right)
    entities.push(LevelEntity {
        x: 5,
        y: 5,
        entity_type: EntityType::Conveyor,
        facing: Facing::Left,
        player_placed: false,
    });
    entities.push(LevelEntity {
        x: 6,
        y: 5,
        entity_type: EntityType::Conveyor,
        facing: Facing::Up,
        player_placed: false,
    });

    // Gap at x=7 (missing conveyor)

    // Smelter facing wrong direction
    entities.push(LevelEntity {
        x: 8,
        y: 5,
        entity_type: EntityType::Smelter,
        facing: Facing::Up,
        player_placed: false,
    });

    // More wrong-facing conveyors after smelter area
    entities.push(LevelEntity {
        x: 9,
        y: 5,
        entity_type: EntityType::Conveyor,
        facing: Facing::Left,
        player_placed: false,
    });

    // Some correct conveyors further down
    for x in 10..=14 {
        entities.push(LevelEntity {
            x,
            y: 5,
            entity_type: EntityType::Conveyor,
            facing: Facing::Right,
            player_placed: false,
        });
    }

    // Gap at x=15..16

    // More correct conveyors at the end
    for x in 17..=22 {
        entities.push(LevelEntity {
            x,
            y: 5,
            entity_type: EntityType::Conveyor,
            facing: Facing::Right,
            player_placed: false,
        });
    }

    LevelConfig {
        number: 5,
        name: "Demolish & Rebuild",
        map_width: 25,
        map_height: 10,
        entities,
        objective: "Fix the broken factory using d, x. Deliver 5 ingots.",
        hints: vec![
            "Use x to delete the entity under the cursor.",
            "Use d followed by a motion to delete a range.",
            "Delete wrong-facing conveyors and replace them.",
            "Fix the smelter by deleting it and placing a new one facing Right.",
            "Fill in any gaps with new conveyors.",
        ],
        allowed_commands: None,
        completion: CompletionCondition::DeliverIngots(5),
    }
}
