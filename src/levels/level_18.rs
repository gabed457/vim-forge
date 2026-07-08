use crate::resources::{EntityType, Facing};

use super::config::{CompletionCondition, LevelConfig, LevelEntity};

/// Level 18: "Inner Peace" — text objects: cit, diw, da(, di".
///
/// One ore line, four textbook obstructions, each shaped so that exactly one
/// text object removes it surgically:
///
///   1. An ASSEMBLER squats where the smelter belongs (anchor (19,9)).
///      `cit` changes the machine's whole footprint; then `s` drops a
///      smelter in its place without moving the cursor.
///   2. A junk island of up-facing belts (30..=35,10), isolated by one-tile
///      gaps at 29 and 36: `diw` deletes exactly that cluster (an adjacent
///      diw would eat good belts — the gaps make it precise).
///   3. A SEALED wall compound (ring x=43..53, y=6..14) sits across the
///      line; inside it, junk. `da(` (stand inside, e.g. (48,10)) deletes
///      the interior AND the walls in one blow; then re-lay the crossing.
///   4. A belt-run siphon: the belt at (59,10) faces Up and feeds a dead-end
///      column of up-belts (59,4..=9). Every item diverts and is lost.
///      `di"` (the belt-run object) deletes the entire straight run —
///      all seven tiles — in one command.
///
/// Completion: DeliverIngots(8). Every obstruction physically blocks or
/// diverts the flow, so all four repairs are mandatory — the text-object
/// commands cannot be faked.
fn belt(entities: &mut Vec<LevelEntity>, x: usize, y: usize, facing: Facing) {
    entities.push(LevelEntity {
        x,
        y,
        entity_type: EntityType::BasicBelt,
        facing,
        player_placed: false,
    });
}

fn wall(entities: &mut Vec<LevelEntity>, x: usize, y: usize) {
    entities.push(LevelEntity {
        x,
        y,
        entity_type: EntityType::Wall,
        facing: Facing::Right,
        player_placed: false,
    });
}

pub fn config() -> LevelConfig {
    let mut entities = Vec::new();

    // Deposit (2,9): output port (4,10). Main belt row is 10.
    entities.push(LevelEntity {
        x: 2,
        y: 9,
        entity_type: EntityType::OreDeposit,
        facing: Facing::Right,
        player_placed: false,
    });
    for x in 5..=18 {
        belt(&mut entities, x, 10, Facing::Right);
    }

    // Obstruction 1: the WRONG machine — an assembler (3x4) where a smelter
    // (3x3) belongs. Its anchor is (19,9); a smelter placed at the same
    // anchor lines its ports up with the belt row.
    entities.push(LevelEntity {
        x: 19,
        y: 9,
        entity_type: EntityType::Assembler,
        facing: Facing::Right,
        player_placed: false,
    });

    for x in 22..=28 {
        belt(&mut entities, x, 10, Facing::Right);
    }

    // Obstruction 2: junk island (gaps at 29 and 36 keep diw surgical).
    for x in 30..=35 {
        belt(&mut entities, x, 10, Facing::Up);
    }

    for x in 37..=42 {
        belt(&mut entities, x, 10, Facing::Right);
    }

    // Obstruction 3: sealed wall compound, ring x=43..53, y=6..14.
    for x in 43..=53 {
        wall(&mut entities, x, 6);
        wall(&mut entities, x, 14);
    }
    for y in 7..=13 {
        wall(&mut entities, 43, y);
        wall(&mut entities, 53, y);
    }
    // Junk inside the compound.
    for x in 45..=51 {
        belt(&mut entities, x, 10, Facing::Up);
    }
    for x in 46..=48 {
        wall(&mut entities, x, 8);
    }
    belt(&mut entities, 47, 12, Facing::Left);

    for x in 54..=58 {
        belt(&mut entities, x, 10, Facing::Right);
    }

    // Obstruction 4: the siphon — a straight up-facing belt run crossing
    // the line at (59,10) and running up to (59,4).
    for y in 4..=10 {
        belt(&mut entities, 59, y, Facing::Up);
    }

    for x in 60..=70 {
        belt(&mut entities, x, 10, Facing::Right);
    }

    // OutputBin (71,9): input port (71,10).
    entities.push(LevelEntity {
        x: 71,
        y: 9,
        entity_type: EntityType::OutputBin,
        facing: Facing::Right,
        player_placed: false,
    });

    LevelConfig {
        number: 18,
        name: "Inner Peace",
        map_width: 80,
        map_height: 22,
        entities,
        objective: "Excise the blockers with cit, diw, da( and di\". Deliver 8 ingots.",
        hints: vec![
            "An Assembler squats where a Smelter belongs (col 19). cit changes a machine: cit, then s.",
            "diw deletes one whole cluster: use it on the up-belt island at col 30, rebuild with i c.",
            "A sealed wall ring blocks the line. Stand INSIDE (col 48) — da( razes walls and contents.",
            "The up-belt column at 59 siphons every item. di\" deletes a whole belt run in one hit.",
            "Re-lay the crossings with right belts (i, then c). Deliver 8 iron ingots.",
        ],
        allowed_commands: None,
        completion: CompletionCondition::DeliverIngots(8),
    }
}
