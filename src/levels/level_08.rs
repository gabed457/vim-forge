use crate::resources::{EntityType, Facing};

use super::config::{CompletionCondition, LevelConfig, LevelEntity};

/// Level 8: "Block Select" — Use Visual-Block mode to copy rectangular sections.
///
/// Layout (40×28):
///
/// Working Cluster 1 (pre-built, rows 3-10):
///   Top ore line (belt_y=4):
///     OreDeposit(3,3) 3×2 → belts (6..12,4) Right → Smelter(13,3) 3×3
///     → belts (16..19,4) Right → belt(20,4) Down
///     → belt(20,5) Right → belt(21,5) Right → Assembler input at (22,5)
///
///   Bottom ore line (belt_y=8):
///     OreDeposit(3,7) 3×2 → belts (6..12,8) Right → Smelter(13,7) 3×3
///     → belts (16..19,8) Right → belt(20,8) Up → belt(20,7) Up
///     → belt(20,6) Right → belt(21,6) Right → Assembler input at (22,6)
///
///   Assembler(22,4) 3×4: output at (24,6)
///     → belts (25..31,6) Right → OutputBin(32,5) 3×2 input at (32,6)
///
/// Cluster 2 area (player copies here, rows 17-24):
///   OreDeposit(3,17), OreDeposit(3,21), OutputBin(32,19)
pub fn config() -> LevelConfig {
    let mut entities = Vec::new();

    // ================================================================
    // Working Cluster 1: Two ore lines merging into assembler
    // ================================================================

    // --- Top ore line (belt_y=4) ---

    // OreDeposit (3×2) at (3,3): output at (5,4) facing Right
    entities.push(LevelEntity {
        x: 3, y: 3,
        entity_type: EntityType::OreDeposit,
        facing: Facing::Right,
        player_placed: false,
    });

    // Horizontal belts from ore output to smelter input: (6,4)→(12,4) Right
    for x in 6..=12 {
        entities.push(LevelEntity {
            x, y: 4,
            entity_type: EntityType::BasicBelt,
            facing: Facing::Right,
            player_placed: false,
        });
    }

    // Smelter (3×3) at (13,3): input at (13,4) Left, output at (15,4) Right
    entities.push(LevelEntity {
        x: 13, y: 3,
        entity_type: EntityType::Smelter,
        facing: Facing::Right,
        player_placed: false,
    });

    // Belts from smelter output: (16,4)→(19,4) Right
    for x in 16..=19 {
        entities.push(LevelEntity {
            x, y: 4,
            entity_type: EntityType::BasicBelt,
            facing: Facing::Right,
            player_placed: false,
        });
    }

    // Turn down toward assembler top input: belt(20,4) Down
    entities.push(LevelEntity {
        x: 20, y: 4,
        entity_type: EntityType::BasicBelt,
        facing: Facing::Down,
        player_placed: false,
    });

    // Route right into assembler top input at (22,5):
    // belt(20,5) Right, belt(21,5) Right → adjacent to assembler input
    entities.push(LevelEntity {
        x: 20, y: 5,
        entity_type: EntityType::BasicBelt,
        facing: Facing::Right,
        player_placed: false,
    });
    entities.push(LevelEntity {
        x: 21, y: 5,
        entity_type: EntityType::BasicBelt,
        facing: Facing::Right,
        player_placed: false,
    });

    // --- Bottom ore line (belt_y=8) ---

    // OreDeposit (3×2) at (3,7): output at (5,8) facing Right
    entities.push(LevelEntity {
        x: 3, y: 7,
        entity_type: EntityType::OreDeposit,
        facing: Facing::Right,
        player_placed: false,
    });

    // Horizontal belts from ore output to smelter input: (6,8)→(12,8) Right
    for x in 6..=12 {
        entities.push(LevelEntity {
            x, y: 8,
            entity_type: EntityType::BasicBelt,
            facing: Facing::Right,
            player_placed: false,
        });
    }

    // Smelter (3×3) at (13,7): input at (13,8) Left, output at (15,8) Right
    entities.push(LevelEntity {
        x: 13, y: 7,
        entity_type: EntityType::Smelter,
        facing: Facing::Right,
        player_placed: false,
    });

    // Belts from smelter output: (16,8)→(19,8) Right
    for x in 16..=19 {
        entities.push(LevelEntity {
            x, y: 8,
            entity_type: EntityType::BasicBelt,
            facing: Facing::Right,
            player_placed: false,
        });
    }

    // Turn up toward assembler bottom input:
    // belt(20,8) Up, belt(20,7) Up
    entities.push(LevelEntity {
        x: 20, y: 8,
        entity_type: EntityType::BasicBelt,
        facing: Facing::Up,
        player_placed: false,
    });
    entities.push(LevelEntity {
        x: 20, y: 7,
        entity_type: EntityType::BasicBelt,
        facing: Facing::Up,
        player_placed: false,
    });

    // Route right into assembler bottom input at (22,6):
    // belt(20,6) Right, belt(21,6) Right → adjacent to assembler input
    entities.push(LevelEntity {
        x: 20, y: 6,
        entity_type: EntityType::BasicBelt,
        facing: Facing::Right,
        player_placed: false,
    });
    entities.push(LevelEntity {
        x: 21, y: 6,
        entity_type: EntityType::BasicBelt,
        facing: Facing::Right,
        player_placed: false,
    });

    // --- Assembler and output ---

    // Assembler (3×4) at (22,4): occupies (22,4)-(24,7)
    //   inputs at (22,5) and (22,6) facing Left
    //   output at (24,6) facing Right
    entities.push(LevelEntity {
        x: 22, y: 4,
        entity_type: EntityType::Assembler,
        facing: Facing::Right,
        player_placed: false,
    });

    // Output belts from assembler output at (24,6): (25,6)→(31,6) Right
    for x in 25..=31 {
        entities.push(LevelEntity {
            x, y: 6,
            entity_type: EntityType::BasicBelt,
            facing: Facing::Right,
            player_placed: false,
        });
    }

    // OutputBin (3×2) at (32,5): occupies (32,5)-(34,6), input at (32,6) Left
    entities.push(LevelEntity {
        x: 32, y: 5,
        entity_type: EntityType::OutputBin,
        facing: Facing::Right,
        player_placed: false,
    });

    // ================================================================
    // Cluster 2 area: player copies cluster 1 here using Visual-Block
    // ================================================================

    // OreDeposit (3×2) at (3,17): top ore source for second cluster
    entities.push(LevelEntity {
        x: 3, y: 17,
        entity_type: EntityType::OreDeposit,
        facing: Facing::Right,
        player_placed: false,
    });

    // OreDeposit (3×2) at (3,21): bottom ore source for second cluster
    entities.push(LevelEntity {
        x: 3, y: 21,
        entity_type: EntityType::OreDeposit,
        facing: Facing::Right,
        player_placed: false,
    });

    // OutputBin (3×2) at (32,19): output for second cluster
    entities.push(LevelEntity {
        x: 32, y: 19,
        entity_type: EntityType::OutputBin,
        facing: Facing::Right,
        player_placed: false,
    });

    LevelConfig {
        number: 8,
        name: "Block Select",
        map_width: 40,
        map_height: 28,
        entities,
        objective: "Use Ctrl-v (Visual Block) to copy a factory section to the empty area below.",
        hints: vec![
            "The top cluster is a working factory. The bottom has ore deposits and a bin but no machines.",
            "Move cursor to the top-left corner of the working cluster's belt area (around row 4, col 6).",
            "Press Ctrl-v to enter Visual Block mode. Move to the bottom-right to select a rectangle.",
            "Press y to yank (copy) the rectangular block. Navigate down to the empty cluster area.",
            "Press p to paste. Visual Block copies a 2D rectangle — perfect for cloning factories!",
        ],
        allowed_commands: None,
        completion: CompletionCondition::ProduceWidgets(6),
    }
}
