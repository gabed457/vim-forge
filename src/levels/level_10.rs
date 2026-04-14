use crate::resources::{EntityType, Facing};

use super::config::{CompletionCondition, LevelConfig, LevelEntity};

/// Level 10: "Macro Factory" — Record and replay macros to build 5 lines.
pub fn config() -> LevelConfig {
    let mut entities = Vec::new();

    // 5 ore deposits (3×2): anchors at belt_y-1
    for i in 0..5 {
        let y = 2 + i * 6;
        entities.push(LevelEntity {
            x: 1,
            y,
            entity_type: EntityType::OreDeposit,
            facing: Facing::Right,
            player_placed: false,
        });
    }

    // 5 output bins (3×2): anchors at belt_y-1
    for i in 0..5 {
        let y = 2 + i * 6;
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
        map_width: 52,
        map_height: 32,
        entities,
        objective: "Record a macro (qa...q), then replay it 4 times (4@a). Build all 5 lines.",
        hints: vec![
            "5 rows of ore and output bins. Build ONE line, record it as a macro, replay it 4 times!",
            "Position at the first row's belt start. Press qa to record into register 'a'.",
            "Build the line: i, c...c, k, s, j, c...c, (route to assembler), Esc. Move to next row start.",
            "Press q to stop recording. Now press 4@a to replay the macro on all remaining rows!",
            "Tip: The macro must end at the same relative position for the next row. 15 widgets to win!",
        ],
        allowed_commands: None,
        completion: CompletionCondition::ProduceWidgets(15),
    }
}
