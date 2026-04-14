use crate::resources::{EntityType, Facing};

use super::config::{CompletionCondition, LevelConfig, LevelEntity};

/// Level 3: "Smelting" — Build a smelting pipeline.
pub fn config() -> LevelConfig {
    let entities = vec![
        // OreDeposit (3×2) at (2,7): output port at (4,8)
        LevelEntity {
            x: 2, y: 7,
            entity_type: EntityType::OreDeposit,
            facing: Facing::Right,
            player_placed: false,
        },
        // OutputBin (3×2) at (38,7): input port at (38,8)
        LevelEntity {
            x: 38, y: 7,
            entity_type: EntityType::OutputBin,
            facing: Facing::Right,
            player_placed: false,
        },
    ];

    LevelConfig {
        number: 3,
        name: "Smelting",
        map_width: 44,
        map_height: 18,
        entities,
        objective: "Build: Ore -> Belts -> Smelter -> Belts -> Output. Deliver 3 ingots.",
        hints: vec![
            "Build a smelting pipeline! Move to the tile right of the deposit's output arrow (row 8).",
            "Press i, then c to lay belts rightward. Stop partway — leave room for the smelter.",
            "Move UP one tile (k), then press s. Smelter is 3x3; its input/output align with row 8.",
            "After placing, the cursor jumps right. Move DOWN (j) back to row 8, then c for more belts.",
            "Lay belts all the way to the Output Bin. Press Esc when done. Deliver 3 ingots!",
        ],
        allowed_commands: None,
        completion: CompletionCondition::DeliverIngots(3),
    }
}
