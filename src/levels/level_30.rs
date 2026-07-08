use crate::resources::{EntityType, Facing};

use super::config::{CompletionCondition, LevelConfig, LevelEntity};

/// Level 30: "Final Exam" — the victory lap. One last factory to finish,
/// and a checklist spanning the entire curriculum: every command on the
/// list must be used at least once. The map is built so each one has a
/// natural moment. Completing this level unlocks Freeplay.
///
/// Layout (50×18):
///   Main line (band y=2, belt row 3): OreDeposit(1,2), belts (4..=13),
///     a 5-tile GAP (14..=18) for J, belts (19..=27), one PIPE at (28,3)
///     for :s/pipe/belt/, belts (29..=43), OutputBin(44,2).
///   Second band (y=8): deposit and bin only — yy the finished row 3 and
///     p it here.
///   Junk row 14: walls at x=10, 12, 14, 30 — search fodder for / * and
///     targets for x, ., dd, marks and Ctrl-o.
///
/// Completion: UseCommands over 22 names — motions (w f ; %), operators
/// (d c y p), marks (m `), macros (q @), the dot, visual (v ctrl-v),
/// upgrades (gU ctrl-a), J, search (/ *), the jumplist (ctrl-o) and :s.
pub fn config() -> LevelConfig {
    let mut entities = Vec::new();

    let belt = |x: usize, y: usize| LevelEntity {
        x,
        y,
        entity_type: EntityType::BasicBelt,
        facing: Facing::Right,
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

    // Main line, gap and pipe included.
    entities.push(big(1, 2, EntityType::OreDeposit));
    entities.push(big(44, 2, EntityType::OutputBin));
    for x in 4..=13 {
        entities.push(belt(x, 3));
    }
    // gap 14..=18 (J's moment)
    for x in 19..=27 {
        entities.push(belt(x, 3));
    }
    entities.push(LevelEntity {
        x: 28,
        y: 3,
        entity_type: EntityType::Pipe,
        facing: Facing::Right,
        player_placed: false,
    });
    for x in 29..=43 {
        entities.push(belt(x, 3));
    }

    // Second band: earn it with yy + p.
    entities.push(big(1, 8, EntityType::OreDeposit));
    entities.push(big(44, 8, EntityType::OutputBin));

    // Junk row: search fodder.
    for &x in &[10usize, 12, 14, 30] {
        entities.push(wall(x, 14));
    }

    LevelConfig {
        number: 30,
        name: "Final Exam",
        map_width: 50,
        map_height: 18,
        entities,
        objective: "The exam: use every command on the checklist while finishing the last factory.",
        hints: vec![
            "Checklist: w f ; % d c y p m ` q @ . v Ctrl-v gU Ctrl-a J / * Ctrl-o :s — all of them, once.",
            "The line: w and fc/; to the belts, % rides to the break, J bridges it, :s/pipe/belt/ on row 3 melts the pipe. Ctrl-a then gUU upgrade the tier.",
            "The junk: /wall, * to the next, Ctrl-o back, mb to mark, x one wall, . the next, dd the row, `b to return.",
            "The clone: cl changes a junk tile, record qa icc<Esc> q, replay @a, then yy row 3 and p it at column 4 row 9 for the second bin.",
            "Wrap up with v and Ctrl-v yanks (visual never hurts). When the list is done — FREEPLAY UNLOCKS.",
        ],
        allowed_commands: None,
        completion: CompletionCondition::UseCommands(vec![
            "w".to_string(),
            "f".to_string(),
            ";".to_string(),
            "%".to_string(),
            "J".to_string(),
            ":s".to_string(),
            "ctrl-a".to_string(),
            "gU".to_string(),
            "/".to_string(),
            "*".to_string(),
            "ctrl-o".to_string(),
            "m".to_string(),
            ".".to_string(),
            "d".to_string(),
            "`".to_string(),
            "c".to_string(),
            "q".to_string(),
            "@".to_string(),
            "y".to_string(),
            "p".to_string(),
            "v".to_string(),
            "ctrl-v".to_string(),
        ]),
    }
}
