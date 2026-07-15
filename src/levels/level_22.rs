use crate::resources::{EntityType, Facing};

use super::config::{CompletionCondition, LevelConfig, LevelEntity};

/// Level 22: "Register Bank" — named registers, "A append, "0 last yank,
/// "1-"9 delete history, "_ black hole, and :reg to inspect the vault.
///
/// Layout (40×24) — two WORKING template lines stacked 3 rows apart, four
/// empty double-sites below, and junk walls littering the paste area:
///
///   Template A (rows 2..4): OreDeposit(1,2), belts (4..=11,3),
///     Smelter(12,2), belts (15..=35,3), OutputBin(36,2), plus a service
///     belt at (4,4) that pads the blueprint's bounding box to 3 rows so
///     the "A append stacks line B at exactly the right spacing.
///   Template B (rows 5..7): the same line at y=5 (service belt (4,7)).
///
///   Target sites: deposit/bin pairs at y=10 & 13 (double-site 1) and
///   y=17 & 20 (double-site 2), machines missing.
///
///   Junk: walls on row 11 (x=8..=12) and row 14 (x=20..=24) squat exactly
///   where the pasted belts must go — delete them first. d5l sends walls
///   into the numbered history ("1); "_d5l disposes without a trace.
///   (dd would flatten the site's deposit and bin too — d{motion} is the
///   precise tool here.)
///
/// Intended flow: `"a3yy` on row 2 (line A into register a), `"A3yy` on
/// row 5 (APPEND line B below it — register a is now a two-line
/// blueprint), `:reg` to admire, d5l/"_d5l the junk, then `"ap` at (4,10)
/// and `"0p` at (4,17) ("0 still holds the last yank even after the
/// deletes) — each paste builds TWO complete lines.
///
/// Completion: all six bins receive an item, so both template lines and
/// all four pasted lines must flow.
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

    // --- The two working template lines (y = 2 and y = 5) ---
    for y in [2usize, 5] {
        entities.push(big(1, y, EntityType::OreDeposit));
        entities.push(big(36, y, EntityType::OutputBin));
        entities.push(big(12, y, EntityType::Smelter));
        for x in 4..=11 {
            entities.push(belt(x, y + 1));
        }
        for x in 15..=35 {
            entities.push(belt(x, y + 1));
        }
        // Service belt: pads the yank's bounding box to exactly 3 rows so
        // "A-appended lines stack at the same 3-row spacing as the sites.
        entities.push(belt(4, y + 2));
    }

    // --- Four empty target sites (two double-sites) ---
    for y in [10usize, 13, 17, 20] {
        entities.push(big(1, y, EntityType::OreDeposit));
        entities.push(big(36, y, EntityType::OutputBin));
    }

    // --- Junk walls exactly on the paste path ---
    for x in 8..=12 {
        entities.push(wall(x, 11));
    }
    for x in 20..=24 {
        entities.push(wall(x, 14));
    }

    LevelConfig {
        number: 22,
        name: "Register Bank",
        map_width: 40,
        map_height: 24,
        entities,
        objective: "Bank a 2-line blueprint: \"a yank, \"A append, clear junk, paste it twice.",
        hints: vec![
            "Two working lines up top. On row 2 press \"a3yy — three rows into register a.",
            "On row 5 press \"A3yy — capital A APPENDS below register a: one blueprint, two lines. :reg shows the bank.",
            "Walls squat on the build sites. On row 11's first wall, d5l — deletes shift into \"1..\"9.",
            "On row 14's junk use \"_d5l — the black hole deletes without touching any register.",
            "Paste at col 4, row 10: \"ap builds BOTH lines. At row 17 try \"0p — register 0 keeps the last yank even after all those deletes.",
        ],
        allowed_commands: None,
        completion: CompletionCondition::Custom("all_5_clusters_producing".to_string()),
    }
}
