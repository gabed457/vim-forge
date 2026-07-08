use crate::resources::{EntityType, Facing};

use super::config::{CompletionCondition, LevelConfig, LevelEntity};

/// Level 21: "Visual Mastery" — counted visual selects, visual r/x/~,
/// text objects in visual (vab), gv reselect, and Visual-Block I column
/// insert.
///
/// Layout (44×15) — three ore→smelter→bin lines, each sabotaged so that a
/// different visual-mode tool is the natural fix:
///
///   Line 1 (belt row 3):  belts (4..=9) Right, belts (10..=15) REVERSED
///     (facing Left — fix: `v5l` then `~`), belts (16..=19), then the wall
///     canyon at x=20, belts (21..=23), Smelter(24,2), belts (27..=39),
///     OutputBin(40,2).
///   Line 2 (belt row 7):  belts (4..=19), canyon, belt (21), then the
///     smelter SITE is fenced in by a wall ring (22,5)-(28,9) with an empty
///     5×3 interior — fix: `vab` + `x` to demolish the fence, `is<Esc>` to
///     drop the smelter at (24,6), then `J` twice to belt up to its ports.
///   Line 3 (belt row 11): belts (4..=15), WALLS (16..=19) (fix: `v3l`
///     then `r` `c`), canyon, belts (21..=23), Smelter(24,10),
///     belts (27..=39), OutputBin(40,10).
///
///   The canyon: a column of walls at x=20 spanning rows 2..=12 blocks all
///   three lines. THE fix: `Ctrl-v` select the column, `x` to demolish it,
///   `gv` to reselect the same block, then `I` `c` — Visual-Block column
///   insert drops one right-facing belt per row, feeding every line at once.
///
/// Completion: every output bin receives at least one item (the shared
/// custom "all bins producing" condition) — all three lines must flow, so
/// none of the visual repairs can be skipped.
pub fn config() -> LevelConfig {
    let mut entities = Vec::new();

    let belt = |x: usize, y: usize, facing: Facing| LevelEntity {
        x,
        y,
        entity_type: EntityType::BasicBelt,
        facing,
        player_placed: false,
    };
    let wall = |x: usize, y: usize| LevelEntity {
        x,
        y,
        entity_type: EntityType::Wall,
        facing: Facing::Right,
        player_placed: false,
    };
    let big = |x: usize, y: usize, entity_type: EntityType| LevelEntity {
        x,
        y,
        entity_type,
        facing: Facing::Right,
        player_placed: false,
    };

    // Deposits and bins for all three lines (bands y = 2, 6, 10).
    for y in [2usize, 6, 10] {
        entities.push(big(1, y, EntityType::OreDeposit));
        entities.push(big(40, y, EntityType::OutputBin));
    }
    // Lines 1 and 3 already have their smelters; line 2's site is fenced.
    entities.push(big(24, 2, EntityType::Smelter));
    entities.push(big(24, 10, EntityType::Smelter));

    // --- Line 1 (belt row 3): reversed segment at x=10..=15 ---
    for x in 4..=9 {
        entities.push(belt(x, 3, Facing::Right));
    }
    for x in 10..=15 {
        entities.push(belt(x, 3, Facing::Left)); // sabotage: reversed
    }
    for x in 16..=19 {
        entities.push(belt(x, 3, Facing::Right));
    }
    for x in 21..=23 {
        entities.push(belt(x, 3, Facing::Right));
    }
    for x in 27..=39 {
        entities.push(belt(x, 3, Facing::Right));
    }

    // --- Line 2 (belt row 7): fenced smelter site ---
    for x in 4..=19 {
        entities.push(belt(x, 7, Facing::Right));
    }
    entities.push(belt(21, 7, Facing::Right));
    for x in 29..=39 {
        entities.push(belt(x, 7, Facing::Right));
    }

    // --- Line 3 (belt row 11): walls where belts belong at x=16..=19 ---
    for x in 4..=15 {
        entities.push(belt(x, 11, Facing::Right));
    }
    for x in 16..=19 {
        entities.push(wall(x, 11)); // sabotage: walls in the line
    }
    for x in 21..=23 {
        entities.push(belt(x, 11, Facing::Right));
    }
    for x in 27..=39 {
        entities.push(belt(x, 11, Facing::Right));
    }

    // --- The wall canyon: column x=20, rows 2..=12 (blocks all lines) ---
    for y in 2..=12 {
        entities.push(wall(20, y));
    }

    // --- The construction fence around line 2's smelter site ---
    // Ring (22,5)-(28,9); interior (23..=27, 6..=8) stays empty.
    for x in 22..=28 {
        entities.push(wall(x, 5));
        entities.push(wall(x, 9));
    }
    for y in 6..=8 {
        entities.push(wall(22, y));
        entities.push(wall(28, y));
    }

    LevelConfig {
        number: 21,
        name: "Visual Mastery",
        map_width: 44,
        map_height: 15,
        entities,
        objective: "Repair all 3 lines with visual mode: v+counts, r, ~, x, vab, gv, and Ctrl-v I.",
        hints: vec![
            "Row 3: six belts face LEFT. On the first one press v5l to select all six, then ~ flips them.",
            "Row 11: walls sit in the line. v3l selects them, then r c replaces the selection with belts.",
            "Row 7's smelter site is fenced! Stand inside, vab selects fence+yard, x demolishes it. is<Esc> at (24,6) drops the smelter, then J J belts it in.",
            "The wall canyon at column 20 blocks everything. Ctrl-v, 10j selects it, x demolishes.",
            "After x press gv to RESELECT the block, then I c: block-insert lays one belt per row — all 3 lines fed at once. viw/vab select objects, o swaps ends.",
        ],
        allowed_commands: None,
        completion: CompletionCondition::Custom("all_5_clusters_producing".to_string()),
    }
}
