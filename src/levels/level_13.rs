use crate::resources::{EntityType, Facing};

use super::config::{CompletionCondition, LevelConfig, LevelEntity};

/// Level 13: "Split View" — Use split views to build cross-map conveyor chains.
pub fn config() -> LevelConfig {
    let mut entities = Vec::new();

    // Top-left factory producing ingots
    entities.push(LevelEntity {
        x: 2,
        y: 2,
        entity_type: EntityType::OreDeposit,
        facing: Facing::Right,
        player_placed: false,
    });
    entities.push(LevelEntity {
        x: 2,
        y: 4,
        entity_type: EntityType::OreDeposit,
        facing: Facing::Right,
        player_placed: false,
    });
    // Conveyor chains for top ore line
    for x in 3..=7 {
        entities.push(LevelEntity {
            x,
            y: 2,
            entity_type: EntityType::BasicBelt,
            facing: Facing::Right,
            player_placed: false,
        });
    }
    entities.push(LevelEntity {
        x: 8,
        y: 2,
        entity_type: EntityType::Smelter,
        facing: Facing::Right,
        player_placed: false,
    });
    for x in 9..=14 {
        entities.push(LevelEntity {
            x,
            y: 2,
            entity_type: EntityType::BasicBelt,
            facing: Facing::Right,
            player_placed: false,
        });
    }

    // Conveyor chains for bottom ore line
    for x in 3..=7 {
        entities.push(LevelEntity {
            x,
            y: 4,
            entity_type: EntityType::BasicBelt,
            facing: Facing::Right,
            player_placed: false,
        });
    }
    entities.push(LevelEntity {
        x: 8,
        y: 4,
        entity_type: EntityType::Smelter,
        facing: Facing::Right,
        player_placed: false,
    });
    for x in 9..=14 {
        entities.push(LevelEntity {
            x,
            y: 4,
            entity_type: EntityType::BasicBelt,
            facing: Facing::Right,
            player_placed: false,
        });
    }

    // Bottom-right assembly area — output bin
    entities.push(LevelEntity {
        x: 75,
        y: 37,
        entity_type: EntityType::OutputBin,
        facing: Facing::Right,
        player_placed: false,
    });

    LevelConfig {
        number: 13,
        name: "Split View",
        map_width: 80,
        map_height: 40,
        entities,
        objective: "Use split views, build cross-map conveyor chain.",
        hints: vec![
            "Use :sp or :vs to split the view.",
            "Use Ctrl-w + h/j/k/l to navigate between splits.",
            "One split can show the ingot output area at top-left.",
            "The other split shows the assembly area at bottom-right.",
            "Build a long conveyor chain connecting the two areas with an assembler.",
        ],
        allowed_commands: None,
        completion: CompletionCondition::ProduceWidgets(5),
    }
}
