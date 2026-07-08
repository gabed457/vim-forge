use crate::resources::{EntityType, Facing};

use super::config::{CompletionCondition, LevelConfig, LevelEntity};

/// Level 17: "Substitute Teacher" — the change/delete family: s S C D and
/// c+motion (ce).
///
/// One long smelting line with three distinct damage zones, plus two scrap
/// rows, each zone shaped for exactly one taught key:
///
///   Zone A — walls dropped ON the belt line at (13,7) (15,7) (17,7):
///            s deletes the tile under the cursor AND enters insert, so the
///            fix is `s c <Esc>` (find them fast with fw and ;).
///   Zone B — a block of up-facing junk belts (32..=37,7) ending at a gap
///            (38,7): `ce` changes to the cluster's end; retype 7 belts.
///   Zone C — a tail of left-facing junk (47..=57,7) running to the row's
///            end: `C` changes from the cursor to the end of the row;
///            rebuild 10 belts, then drop two down-belts at column 57 into
///            the bin below (the bin hangs under the line so C/D can never
///            hit it).
///   Scrap rows 2 and 3 — pure junk: `S` changes (wipes) a whole row, `D`
///            deletes from the cursor to the end of the row.
///
/// Completion: DeliverIngots(6) — only a fully repaired line delivers.
fn belt(entities: &mut Vec<LevelEntity>, x: usize, y: usize, facing: Facing) {
    entities.push(LevelEntity {
        x,
        y,
        entity_type: EntityType::BasicBelt,
        facing,
        player_placed: false,
    });
}

pub fn config() -> LevelConfig {
    let mut entities = Vec::new();

    // Scrap row 2: left-facing junk belts (S wipes the row).
    for x in 10..=40 {
        belt(&mut entities, x, 2, Facing::Left);
    }
    // Scrap row 3: up-facing junk belts (D from x=30 clears the tail).
    for x in 20..=50 {
        belt(&mut entities, x, 3, Facing::Up);
    }

    // Deposit (2,6): output port (4,7). Main belt row is 7.
    entities.push(LevelEntity {
        x: 2,
        y: 6,
        entity_type: EntityType::OreDeposit,
        facing: Facing::Right,
        player_placed: false,
    });
    for x in 5..=12 {
        belt(&mut entities, x, 7, Facing::Right);
    }

    // Zone A: walls interleaved with good belts.
    for &x in &[13usize, 15, 17] {
        entities.push(LevelEntity {
            x,
            y: 7,
            entity_type: EntityType::Wall,
            facing: Facing::Right,
            player_placed: false,
        });
    }
    belt(&mut entities, 14, 7, Facing::Right);
    belt(&mut entities, 16, 7, Facing::Right);

    for x in 18..=24 {
        belt(&mut entities, x, 7, Facing::Right);
    }

    // Smelter (25,6): input (25,7), output (27,7).
    entities.push(LevelEntity {
        x: 25,
        y: 6,
        entity_type: EntityType::Smelter,
        facing: Facing::Right,
        player_placed: false,
    });
    for x in 28..=31 {
        belt(&mut entities, x, 7, Facing::Right);
    }

    // Zone B: up-facing junk (32..=37), then a missing tile at (38,7).
    for x in 32..=37 {
        belt(&mut entities, x, 7, Facing::Up);
    }

    for x in 39..=46 {
        belt(&mut entities, x, 7, Facing::Right);
    }

    // Zone C: left-facing junk tail from 47 to 57 (nothing else on row 7
    // to the right of it, so C / D are safe and exact).
    for x in 47..=57 {
        belt(&mut entities, x, 7, Facing::Left);
    }

    // OutputBin (56,9): hangs BELOW the line; its top input port is (57,9),
    // fed by a player-built down-belt column at (57,7)-(57,8).
    entities.push(LevelEntity {
        x: 56,
        y: 9,
        entity_type: EntityType::OutputBin,
        facing: Facing::Right,
        player_placed: false,
    });

    LevelConfig {
        number: 17,
        name: "Substitute Teacher",
        map_width: 62,
        map_height: 14,
        entities,
        objective: "Repair with s, ce and C; scrap rows 2-3 with S and D. Deliver 6 ingots.",
        hints: vec![
            "Walls litter the belt row! fw finds one; s deletes it AND enters insert: s c Esc, then ;.",
            "ce CHANGES to the cluster's end: on the up-belt block at column 32, ce then 7 x c.",
            "C changes to the END OF THE ROW: on the left-belt tail at column 47, C then 10 x c.",
            "S changes a whole row, D deletes to row end — wipe the scrap on rows 2 and 3.",
            "Last: two DOWN belts at column 57 (i, Shift-J, c c) drop into the bin. 6 ingots!",
        ],
        allowed_commands: None,
        completion: CompletionCondition::DeliverIngots(6),
    }
}
