use crate::resources::{EntityType, Facing};

use super::config::{CompletionCondition, LevelConfig, LevelEntity};

/// Level 8: "Block Select" — Use Visual-Block mode to copy rectangular sections.
pub fn config() -> LevelConfig {
    let mut entities = Vec::new();

    // Pre-built 4-row cluster in rows 2-5
    // Row 2: ore -> conveyors -> smelter -> conveyors
    entities.push(LevelEntity {
        x: 1,
        y: 2,
        entity_type: EntityType::OreDeposit,
        facing: Facing::Right,
        player_placed: false,
    });
    for x in 2..=6 {
        entities.push(LevelEntity {
            x,
            y: 2,
            entity_type: EntityType::Conveyor,
            facing: Facing::Right,
            player_placed: false,
        });
    }
    entities.push(LevelEntity {
        x: 7,
        y: 2,
        entity_type: EntityType::Smelter,
        facing: Facing::Right,
        player_placed: false,
    });
    for x in 8..=12 {
        entities.push(LevelEntity {
            x,
            y: 2,
            entity_type: EntityType::Conveyor,
            facing: Facing::Down,
            player_placed: false,
        });
    }

    // Row 3: conveyors merging down
    entities.push(LevelEntity {
        x: 12,
        y: 3,
        entity_type: EntityType::Conveyor,
        facing: Facing::Down,
        player_placed: false,
    });

    // Row 4: ore -> conveyors -> smelter -> conveyors merging up
    entities.push(LevelEntity {
        x: 1,
        y: 4,
        entity_type: EntityType::OreDeposit,
        facing: Facing::Right,
        player_placed: false,
    });
    for x in 2..=6 {
        entities.push(LevelEntity {
            x,
            y: 4,
            entity_type: EntityType::Conveyor,
            facing: Facing::Right,
            player_placed: false,
        });
    }
    entities.push(LevelEntity {
        x: 7,
        y: 4,
        entity_type: EntityType::Smelter,
        facing: Facing::Right,
        player_placed: false,
    });
    for x in 8..=12 {
        entities.push(LevelEntity {
            x,
            y: 4,
            entity_type: EntityType::Conveyor,
            facing: Facing::Up,
            player_placed: false,
        });
    }

    // Row 5: assembler and output
    entities.push(LevelEntity {
        x: 13,
        y: 3,
        entity_type: EntityType::Assembler,
        facing: Facing::Right,
        player_placed: false,
    });
    for x in 14..=18 {
        entities.push(LevelEntity {
            x,
            y: 3,
            entity_type: EntityType::Conveyor,
            facing: Facing::Right,
            player_placed: false,
        });
    }
    entities.push(LevelEntity {
        x: 19,
        y: 3,
        entity_type: EntityType::OutputBin,
        facing: Facing::Right,
        player_placed: false,
    });

    // Second cluster area: ore deposits and bins for rows 12-15
    entities.push(LevelEntity {
        x: 1,
        y: 12,
        entity_type: EntityType::OreDeposit,
        facing: Facing::Right,
        player_placed: false,
    });
    entities.push(LevelEntity {
        x: 1,
        y: 14,
        entity_type: EntityType::OreDeposit,
        facing: Facing::Right,
        player_placed: false,
    });
    entities.push(LevelEntity {
        x: 19,
        y: 13,
        entity_type: EntityType::OutputBin,
        facing: Facing::Right,
        player_placed: false,
    });

    LevelConfig {
        number: 8,
        name: "Block Select",
        map_width: 40,
        map_height: 20,
        entities,
        objective: "Visual-Block to copy rectangular factory section.",
        hints: vec![
            "Use Ctrl-v to enter Visual-Block mode.",
            "Select the rectangular area of the working cluster.",
            "Use y to yank the selection.",
            "Navigate to the target area and p to paste.",
            "Visual-Block copies a rectangular region, not just lines.",
        ],
        allowed_commands: None,
        completion: CompletionCondition::ProduceWidgets(6),
    }
}
