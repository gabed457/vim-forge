use crate::resources::{EntityType, Facing};

use super::config::{CompletionCondition, LevelConfig, LevelEntity};

/// Level 2: "First Placement" — Place conveyors to connect ore to output.
pub fn config() -> LevelConfig {
    let entities = vec![
        // OreDeposit (3×2) at (2,5): output port at (4,6)
        LevelEntity {
            x: 2, y: 5,
            entity_type: EntityType::OreDeposit,
            facing: Facing::Right,
            player_placed: false,
        },
        // OutputBin (3×2) at (28,5): input port at (28,6)
        LevelEntity {
            x: 28, y: 5,
            entity_type: EntityType::OutputBin,
            facing: Facing::Right,
            player_placed: false,
        },
    ];

    LevelConfig {
        number: 2,
        name: "First Placement",
        map_width: 34,
        map_height: 14,
        entities,
        objective: "Connect the Ore Deposit to the Output Bin with belts. Deliver 3 ore.",
        hints: vec![
            "Connect the Ore Deposit (left) to the Output Bin (right) with conveyor belts!",
            "Move to the empty tile just right of the deposit's output arrow (>). Row 6, column 5.",
            "Press i to enter INSERT mode. Press c to place a belt — it auto-advances right!",
            "Keep pressing c to lay belts across the row until you reach the Output Bin.",
            "Press Esc to exit insert mode. Ore flows once connected. Deliver 3 ore to win!",
        ],
        allowed_commands: None,
        completion: CompletionCondition::DeliverOre(3),
    }
}
