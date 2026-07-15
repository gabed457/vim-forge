use crate::resources::{EntityType, Facing};

use super::config::{CompletionCondition, LevelConfig, LevelEntity};

/// Level 27: "Golf I" — the edit budget. `p` costs ONE edit no matter how
/// big the blueprint; placing tiles by hand costs one edit per tile.
///
/// Layout (42×28) — one COMPLETE widget cell (the level-6 pattern) at band
/// y=3, and three empty deposit/bin sites at bands y=9, 15, 21 (exactly 6
/// rows apart, the cell's stride):
///
///   Template cell (band top y=3, belt row 4, branch row 5):
///     OreDeposit(1,3) → belts (4..=7,4) → Smelter(8,3) → belts (11..=12,4)
///     → Splitter(13,3) → corner belt (16,3) Down + belts (16..=19,4)
///     and belts (16..=19,5) → Assembler(20,3) → belts (23..=36,5)
///     → corner (37,5) Up → belt (37,4) → OutputBin(38,3).
///
/// Edit math (documented for the budget):
///   - Intended: 4yy (yank = 0 edits) + p + p + p           =  3 edits
///   - Budget: 3 + slack                                     =  5 edits
///   - Tile-by-tile: 30 belts + 3 machines = 33 per cell     = 99 edits — bust.
///   - Macro angle: @-replays bypass the edit counter (engine quirk), but
///     RECORDING counts, and one single-placement macro per needed piece
///     (belt-R, belt-D, belt-U, smelter, splitter, assembler) is 6
///     recorded edits — still over budget. Paste is the only way in.
///
/// Completion: ScoreInMoves(8 widgets, max 5 edits).
pub fn config() -> LevelConfig {
    let mut entities = template_cell(3);

    // Three empty sites, 6 rows apart — the same stride as the template.
    for &y in &[9usize, 15, 21] {
        entities.push(LevelEntity {
            x: 1,
            y,
            entity_type: EntityType::OreDeposit,
            facing: Facing::Right,
            player_placed: false,
        });
        entities.push(LevelEntity {
            x: 38,
            y,
            entity_type: EntityType::OutputBin,
            facing: Facing::Right,
            player_placed: false,
        });
    }

    LevelConfig {
        number: 27,
        name: "Golf I",
        map_width: 42,
        map_height: 28,
        entities,
        objective: "Par 5: one working cell, three empty sites. Paste is 1 edit; a belt is 1 edit too.",
        hints: vec![
            "EDIT BUDGET: 5. Every placement, delete or paste costs one edit — the counter is merciless.",
            "The top cell works. Its 33 tiles by hand = 33 edits PER SITE. Do not even try.",
            "Yanking is free! On the cell's top row (row 3) press 4yy — four rows into the register.",
            "One p = ONE edit, no matter how many tiles the blueprint holds.",
            "Paste with the cursor at column 4 on each site's top row (rows 9, 15, 21). Par is 3.",
        ],
        allowed_commands: None,
        completion: CompletionCondition::ScoreInMoves(8, 5),
    }
}

/// The proven level-6 widget cell, prebuilt at band top `y`.
fn template_cell(y: usize) -> Vec<LevelEntity> {
    let mut entities = Vec::new();

    let belt = |x: usize, yy: usize, facing: Facing| LevelEntity {
        x,
        y: yy,
        entity_type: EntityType::BasicBelt,
        facing,
        player_placed: false,
    };
    let big = |x: usize, yy: usize, entity_type: EntityType| LevelEntity {
        x,
        y: yy,
        entity_type,
        facing: Facing::Right,
        player_placed: false,
    };

    entities.push(big(1, y, EntityType::OreDeposit));
    entities.push(big(38, y, EntityType::OutputBin));

    // Ore feed: deposit port (3,y+1) → smelter input (8,y+1).
    for x in 4..=7 {
        entities.push(belt(x, y + 1, Facing::Right));
    }
    entities.push(big(8, y, EntityType::Smelter));
    // Smelter output (10,y+1) → splitter input (13,y+1).
    for x in 11..=12 {
        entities.push(belt(x, y + 1, Facing::Right));
    }
    entities.push(big(13, y, EntityType::Splitter));
    // Upper splitter output (15,y): corner down, then along the belt row
    // into assembler input A (20,y+1).
    entities.push(belt(16, y, Facing::Down));
    for x in 16..=19 {
        entities.push(belt(x, y + 1, Facing::Right));
    }
    // Lower splitter output (15,y+2): straight into input B (20,y+2).
    for x in 16..=19 {
        entities.push(belt(x, y + 2, Facing::Right));
    }
    entities.push(big(20, y, EntityType::Assembler));
    // Assembler output (22,y+2) → east, corner up into the bin port (38,y+1).
    for x in 23..=36 {
        entities.push(belt(x, y + 2, Facing::Right));
    }
    entities.push(belt(37, y + 2, Facing::Up));
    entities.push(belt(37, y + 1, Facing::Right));

    entities
}
