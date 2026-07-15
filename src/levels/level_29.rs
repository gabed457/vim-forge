use crate::resources::{EntityType, Facing};

use super::config::{CompletionCondition, LevelConfig, LevelEntity};

/// Level 29: "The Broken Megafactory" — the repair gauntlet. Five ingot
/// lines, five different kinds of sabotage, one fault per line; every tool
/// from the curriculum has its moment.
///
/// Layout (56×22) — bands y = 2, 6, 10, 14, 18 (belt row y+1):
///   OreDeposit(1,y) → belts (4..=29,y+1) → Smelter(30,y)
///   → belts (33..=49,y+1) → OutputBin(50,y)
///
/// Faults:
///   Line 1 (row 3):  belts (10..=15) REVERSED       — fix: v5l then ~
///   Line 2 (row 7):  belts (18..=23) MISSING        — fix: J joins the gap
///   Line 3 (row 11): PIPES at (20..=25)             — fix: R cccccc Esc
///   Line 4 (row 15): a KILN where the smelter goes  — fix: cit then s Esc
///   Line 5 (row 19): three WALLS at x=8, 22, 44     — fix: /wall + n + r c
///
/// Navigation is the meta-fault: the map is wide, the faults scattered —
/// search, marks and Ctrl-o are how a vim player commutes. gUU on a fixed
/// row upgrades its belts a tier for style.
///
/// Completion: every bin receives an ingot — all five faults must fall.
pub fn config() -> LevelConfig {
    let mut entities = Vec::new();

    let belt = |x: usize, y: usize, facing: Facing| LevelEntity {
        x,
        y,
        entity_type: EntityType::BasicBelt,
        facing,
        player_placed: false,
    };
    let big = |x: usize, y: usize, entity_type: EntityType| LevelEntity {
        x,
        y,
        entity_type,
        facing: Facing::Right,
        player_placed: false,
    };

    for (i, &y) in [2usize, 6, 10, 14, 18].iter().enumerate() {
        let r = y + 1;
        entities.push(big(1, y, EntityType::OreDeposit));
        entities.push(big(50, y, EntityType::OutputBin));
        // Line 4's processor slot holds a KILN (wrong machine) instead.
        if i == 3 {
            entities.push(big(30, y, EntityType::Kiln));
        } else {
            entities.push(big(30, y, EntityType::Smelter));
        }

        // Feed run (4..=29) with per-line sabotage.
        for x in 4..=29 {
            match i {
                // Line 1: (10..=15) reversed.
                0 if (10..=15).contains(&x) => {
                    entities.push(belt(x, r, Facing::Left));
                }
                // Line 2: (18..=23) missing.
                1 if (18..=23).contains(&x) => {}
                // Line 3: (20..=25) are pipes.
                2 if (20..=25).contains(&x) => {
                    entities.push(LevelEntity {
                        x,
                        y: r,
                        entity_type: EntityType::Pipe,
                        facing: Facing::Right,
                        player_placed: false,
                    });
                }
                // Line 5: walls at 8 and 22.
                4 if x == 8 || x == 22 => {
                    entities.push(LevelEntity {
                        x,
                        y: r,
                        entity_type: EntityType::Wall,
                        facing: Facing::Right,
                        player_placed: false,
                    });
                }
                _ => entities.push(belt(x, r, Facing::Right)),
            }
        }

        // Output run (33..=49); line 5 has its third wall at 44.
        for x in 33..=49 {
            if i == 4 && x == 44 {
                entities.push(LevelEntity {
                    x,
                    y: r,
                    entity_type: EntityType::Wall,
                    facing: Facing::Right,
                    player_placed: false,
                });
            } else {
                entities.push(belt(x, r, Facing::Right));
            }
        }
    }

    LevelConfig {
        number: 29,
        name: "The Broken Megafactory",
        map_width: 56,
        map_height: 22,
        entities,
        objective: "Five lines, five saboteurs. Visual ~, J, R, cit, and search — fix them all.",
        hints: vec![
            "Row 3: six belts run backwards at x=10. v5l selects them, ~ flips the lot — one edit.",
            "Row 7 has a 6-tile gap. Stand on the left belt run and J bridges straight to the smelter feed.",
            "Row 11: someone laid PIPES at x=20. R enters replace mode — cccccc Esc paves over them.",
            "Row 15: that 3x3 machine is a KILN, not a smelter. On its corner: cit deletes it and opens insert — press s, then Esc.",
            "Row 19 hides three walls. /wall<CR>, r c, then n to the next. Bonus style: gUU upgrades a row's belts a tier.",
        ],
        allowed_commands: None,
        completion: CompletionCondition::Custom("all_5_clusters_producing".to_string()),
    }
}
