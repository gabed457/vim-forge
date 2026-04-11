use crate::resources::{EntityType, Facing};

use super::config::{CompletionCondition, LevelConfig, LevelEntity};

/// Level 2: "First Placement" — Place conveyors to connect ore to output.
pub fn config() -> LevelConfig {
    let entities = vec![
        LevelEntity {
            x: 1,
            y: 4,
            entity_type: EntityType::OreDeposit,
            facing: Facing::Right,
            player_placed: false,
        },
        LevelEntity {
            x: 13,
            y: 4,
            entity_type: EntityType::OutputBin,
            facing: Facing::Right,
            player_placed: false,
        },
    ];

    LevelConfig {
        number: 2,
        name: "First Placement",
        map_width: 15,
        map_height: 8,
        entities,
        objective: "Place conveyors between the Ore Deposit (O) and Output Bin (B)",
        hints: vec![
            "You see O (Ore Deposit) on the left and B (Output Bin) on the right. Connect them!",
            "Move next to the O with l, then press i to enter INSERT mode (status bar turns green).",
            "In Insert mode, press c to place a conveyor. It auto-advances your cursor right.",
            "Keep pressing c to lay conveyors all the way to the B. Press Esc when done.",
            "Once the path is complete, ore flows automatically. Deliver 3 ore to win!",
        ],
        allowed_commands: None,
        completion: CompletionCondition::DeliverOre(3),
    }
}
