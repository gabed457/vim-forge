use crate::resources::{EntityType, Facing};

use super::config::{CompletionCondition, LevelConfig, LevelEntity};

/// Level 28: "Golf II" — fix the template FIRST, then replicate. The
/// template cell is broken in two places; copying it unfixed copies the
/// defects into every site and blows the budget.
///
/// Layout (42×28) — the level-27 widget cell at band y=3 but sabotaged,
/// plus three empty deposit/bin sites at y=9, 15, 21:
///
///   Defect 1: the SMELTER IS MISSING — bare gap at (8..10, 3..5).
///   Defect 2: a WALL is bricked into the output run at (30,5).
///
/// Edit math (documented for the budget):
///   - Intended: is<Esc> at (8,3) [1] + rc at (30,5) [1] + 4yy [0]
///     + p ×3 [3]                                        = 5 edits
///   - Budget: 5 + slack                                  = 7 edits
///   - Copy-then-fix: 3 pastes + 2 template fixes + 2×3 copy fixes
///                                                        = 11 edits — bust.
///   - Tile-by-tile: ~33 per cell                         — laughably bust.
///
/// Completion: ScoreInMoves(8 widgets, max 7 edits).
pub fn config() -> LevelConfig {
    let mut entities = broken_template_cell(3);

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
        number: 28,
        name: "Golf II",
        map_width: 42,
        map_height: 28,
        entities,
        objective: "Par 7: the template is broken twice. Fix it ONCE, then paste — never fix a copy.",
        hints: vec![
            "EDIT BUDGET: 7. The template cell has two defects: copy it now and every site inherits them.",
            "Defect 1: no smelter. Stand at (8,3) and drop one: i s Esc — one edit.",
            "Defect 2: a wall in the output run at (30,5). r c replaces it — one edit.",
            "NOW it's golf I again: 4yy on row 3 (free), then one p per site at column 4, rows 9/15/21.",
            "Fix-once-then-paste = 5 edits. Paste-then-fix-thrice = 11. The budget knows the difference.",
        ],
        allowed_commands: None,
        completion: CompletionCondition::ScoreInMoves(8, 7),
    }
}

/// The level-27 template cell at band top `y`, minus the smelter, with a
/// wall bricked into the output run at (30, y+2).
fn broken_template_cell(y: usize) -> Vec<LevelEntity> {
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

    for x in 4..=7 {
        entities.push(belt(x, y + 1, Facing::Right));
    }
    // DEFECT 1: no smelter at (8, y).
    for x in 11..=12 {
        entities.push(belt(x, y + 1, Facing::Right));
    }
    entities.push(big(13, y, EntityType::Splitter));
    entities.push(belt(16, y, Facing::Down));
    for x in 16..=19 {
        entities.push(belt(x, y + 1, Facing::Right));
    }
    for x in 16..=19 {
        entities.push(belt(x, y + 2, Facing::Right));
    }
    entities.push(big(20, y, EntityType::Assembler));
    for x in 23..=36 {
        if x == 30 {
            // DEFECT 2: a wall bricked into the output run.
            entities.push(LevelEntity {
                x,
                y: y + 2,
                entity_type: EntityType::Wall,
                facing: Facing::Right,
                player_placed: false,
            });
        } else {
            entities.push(belt(x, y + 2, Facing::Right));
        }
    }
    entities.push(belt(37, y + 2, Facing::Up));
    entities.push(belt(37, y + 1, Facing::Right));

    entities
}
