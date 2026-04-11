use crate::resources::{EntityType, Facing};

use super::config::{CompletionCondition, LevelConfig, LevelEntity};

/// Level 10: "Macro Factory" — Record and replay macros to build 5 lines.
pub fn config() -> LevelConfig {
    let mut entities = Vec::new();

    // 5 ore deposits
    for i in 0..5 {
        let y = 2 + i * 4;
        entities.push(LevelEntity {
            x: 1,
            y,
            entity_type: EntityType::OreDeposit,
            facing: Facing::Right,
            player_placed: false,
        });
    }

    // 5 output bins
    for i in 0..5 {
        let y = 2 + i * 4;
        entities.push(LevelEntity {
            x: 48,
            y,
            entity_type: EntityType::OutputBin,
            facing: Facing::Right,
            player_placed: false,
        });
    }

    LevelConfig {
        number: 10,
        name: "Macro Factory",
        map_width: 50,
        map_height: 20,
        entities,
        objective: "Record macro qa...q, replay 4@a. Build all 5 lines.",
        hints: vec![
            "Press qa to start recording a macro into register 'a'.",
            "Build one complete production line from ore to output.",
            "Press q to stop recording.",
            "Move to the next ore deposit.",
            "Press 4@a to replay the macro 4 times for the remaining lines.",
        ],
        allowed_commands: None,
        completion: CompletionCondition::ProduceWidgets(15),
    }
}
