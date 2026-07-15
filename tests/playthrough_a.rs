//! Full playthrough tests for campaign levels 3-7: drive the REAL game
//! (GameSession — the same code path main.rs uses) with literal keystrokes
//! and verify each level can be completed the way a player would complete it,
//! following the intended teaching of each level's hints.
//!
//! ## Game bug found and fixed while writing these tests
//!
//! **Belts could not turn corners** (src/map/multitile.rs,
//! `BuildingFootprint::new_1x1_conveyor`). The simulation routes items using
//! the port definitions of each building footprint, and the conveyor
//! footprint declared only ONE input port (from behind). That contradicted
//! `vimforge::resources::get_input_sides()`, which has always declared THREE
//! input sides for belts (behind + both sides), and it contradicted the
//! level 4 hint ("Use arrow keys in insert mode to change belt direction.
//! Turn belts to route both lines in."): an item travelling right could never
//! enter a down-facing belt, so no L-shaped belt path could ever carry an
//! item, making levels 4, 6 and 7 (which require routing two ingot lines
//! into the two left-side input ports of an assembler on adjacent rows)
//! impossible to complete. Fix: the conveyor footprint now has three input
//! ports (behind, left side, right side — rotating with facing) and still
//! refuses input from the front. `tests/multitile_tests.rs::
//! test_conveyor_has_ports` hard-coded the buggy 2-port count and was
//! updated to assert the corrected port set.

use vimforge::app::Mode;
use vimforge::game::session::GameSession;
use vimforge::resources::EntityType;

/// Start a session directly in the given level (as if picked from the menu).
fn session_at_level(level: usize) -> GameSession {
    let mut s = GameSession::new(160, 48);
    s.start_level(level);
    assert_eq!(s.current_level(), Some(level), "level {level} should load");
    assert_eq!(s.mode(), Mode::Normal);
    s
}

/// Move the cursor to an absolute map position using normal-mode motions,
/// exactly like a player: Esc, gg, then counted j/l.
fn goto(s: &mut GameSession, x: usize, y: usize) {
    s.feed_keys("<Esc>gg");
    if y > 0 {
        s.feed_keys(&format!("{y}j"));
    }
    if x > 0 {
        s.feed_keys(&format!("{x}l"));
    }
    assert_eq!(s.cursor(), (x, y), "goto({x},{y}) should land on target");
    assert_eq!(s.mode(), Mode::Normal);
}

/// Place a run of `n` entities starting at (x, y), building in direction
/// `dir` ('R', 'D', 'U' or 'L'), using insert-mode quick-place key `key`.
///
/// Exactly like a player: park the cursor one tile before the start position,
/// press `i`, press the Shift-movement key (which sets the insert facing AND
/// steps onto the start tile), then hit the quick-place key `n` times (each
/// placement auto-advances the cursor in the facing direction), then Esc.
fn place_run(s: &mut GameSession, x: usize, y: usize, dir: char, key: char, n: usize) {
    let (ax, ay, facing_key) = match dir {
        'R' => (x - 1, y, 'L'),
        'D' => (x, y - 1, 'J'),
        'U' => (x, y + 1, 'K'),
        'L' => (x + 1, y, 'H'),
        _ => panic!("bad dir {dir}"),
    };
    goto(s, ax, ay);
    s.feed_keys("i");
    assert_eq!(s.mode(), Mode::Insert);
    s.feed_keys(&facing_key.to_string());
    assert_eq!(s.cursor(), (x, y), "facing key should step onto start tile");
    for _ in 0..n {
        s.feed_keys(&key.to_string());
    }
    s.feed_keys("<Esc>");
    assert_eq!(s.mode(), Mode::Normal);
}

/// Tick the simulation until the campaign auto-advances to `next` level.
fn run_until_level(s: &mut GameSession, next: usize, max_ticks: usize) {
    for _ in 0..max_ticks {
        s.tick(1);
        if s.current_level() == Some(next) {
            return;
        }
    }
    let (ore, ingots, widgets, total) = s.output_totals();
    panic!(
        "never advanced to level {next} after {max_ticks} ticks; \
         delivered ore={ore} ingots={ingots} widgets={widgets} total={total}"
    );
}

/// Level 3 "Smelting" (DeliverIngots(3)).
///
/// Map: OreDeposit 3x2 at (2,7) with output port at (4,8); OutputBin 3x2 at
/// (38,7) with input port at (38,8). Following the hints: belts rightward on
/// row 8, move up one tile, drop a smelter (3x3, input/output aligned with
/// row 8), back down, more belts to the bin.
#[test]
fn level_3_smelting_pipeline_delivers_ingots() {
    let mut s = session_at_level(3);

    // Belts (5..=11, 8): from the deposit's output toward the smelter.
    place_run(&mut s, 5, 8, 'R', 'c', 7);
    // Smelter anchor (12,7): occupies (12..=14, 7..=9), input port (12,8)
    // fed by the belt at (11,8), output port (14,8) pushing to (15,8).
    place_run(&mut s, 12, 7, 'R', 's', 1);
    // Belts (15..=37, 8): from the smelter output to the bin's input port.
    place_run(&mut s, 15, 8, 'R', 'c', 23);

    assert_eq!(s.entity_type_at(5, 8), Some(EntityType::BasicBelt));
    assert_eq!(s.entity_type_at(11, 8), Some(EntityType::BasicBelt));
    assert_eq!(s.entity_type_at(12, 7), Some(EntityType::Smelter));
    assert_eq!(s.entity_type_at(15, 8), Some(EntityType::BasicBelt));
    assert_eq!(s.entity_type_at(37, 8), Some(EntityType::BasicBelt));

    run_until_level(&mut s, 4, 3000);
}

/// Level 4 "Full Production" (ProduceWidgets(3)).
///
/// Two ore lines (rows 5 and 14) each get a smelter, then both ingot lines
/// turn (belt corners — the intended teaching of this level's hints) into
/// the two left-side input ports of one assembler at (30,7) (ports (30,8)
/// and (30,9)), whose output at (32,9) runs by belt to the bin's input port
/// at (48,9).
#[test]
fn level_4_two_lines_one_assembler_produces_widgets() {
    let mut s = session_at_level(4);

    // ---- Top line (deposit output port (4,5) -> row 5) ----
    place_run(&mut s, 5, 5, 'R', 'c', 3); // belts (5..=7, 5)
    place_run(&mut s, 8, 4, 'R', 's', 1); // smelter (8..=10, 4..=6), in (8,5) out (10,5)
    place_run(&mut s, 11, 5, 'R', 'c', 18); // belts (11..=28, 5)
    place_run(&mut s, 29, 5, 'D', 'c', 3); // corner: belts (29,5)(29,6)(29,7) facing Down
    place_run(&mut s, 29, 8, 'R', 'c', 1); // belt (29,8) facing Right -> port (30,8)

    // ---- Bottom line (deposit output port (4,14) -> row 14) ----
    place_run(&mut s, 5, 14, 'R', 'c', 3); // belts (5..=7, 14)
    place_run(&mut s, 8, 13, 'R', 's', 1); // smelter (8..=10, 13..=15), in (8,14) out (10,14)
    place_run(&mut s, 11, 14, 'R', 'c', 18); // belts (11..=28, 14)
    place_run(&mut s, 29, 14, 'U', 'c', 5); // corner: belts (29,14) up to (29,10) facing Up
    place_run(&mut s, 29, 9, 'R', 'c', 1); // belt (29,9) facing Right -> port (30,9)

    // ---- Assembler + output run ----
    // Assembler anchor (30,7): occupies (30..=32, 7..=10), inputs (30,8) and
    // (30,9), output (32,9).
    place_run(&mut s, 30, 7, 'R', 'a', 1);
    // Belts (33..=47, 9): output to the bin's input port (48,9).
    place_run(&mut s, 33, 9, 'R', 'c', 15);

    assert_eq!(s.entity_type_at(8, 4), Some(EntityType::Smelter));
    assert_eq!(s.entity_type_at(8, 13), Some(EntityType::Smelter));
    assert_eq!(s.entity_type_at(30, 7), Some(EntityType::Assembler));
    assert_eq!(s.entity_type_at(29, 5), Some(EntityType::BasicBelt));
    assert_eq!(s.entity_type_at(29, 8), Some(EntityType::BasicBelt));
    assert_eq!(s.entity_type_at(29, 14), Some(EntityType::BasicBelt));
    assert_eq!(s.entity_type_at(29, 9), Some(EntityType::BasicBelt));
    assert_eq!(s.entity_type_at(47, 9), Some(EntityType::BasicBelt));

    run_until_level(&mut s, 5, 3000);
}

/// Level 5 "Demolish & Rebuild" (DeliverIngots(5)).
///
/// The pre-built factory is broken: belts at (10,8) face Left and (11,8)
/// faces Up, there is a gap at (12,8), the smelter at (13,7) faces Up, the
/// belt at (16,8) faces Left, and there is a gap at (28..=30, 8). Following
/// the hints: rotate the wrong pieces with `~` and fill the gaps with belts.
#[test]
fn level_5_repair_broken_factory_delivers_ingots() {
    let mut s = session_at_level(5);

    // Rotate belt (10,8): Left -> Up -> Right (two ~).
    goto(&mut s, 10, 8);
    s.feed_keys("~~");
    // Rotate belt (11,8): Up -> Right (one ~).
    s.feed_keys("l~");

    // Fill the gap at (12,8) with a belt facing Right.
    place_run(&mut s, 12, 8, 'R', 'c', 1);

    // Rotate the smelter (anchor (13,7), cursor on any of its tiles):
    // Up -> Right (one ~). After place_run the cursor sits at (13,8),
    // which is a smelter tile.
    assert_eq!(s.cursor(), (13, 8));
    s.feed_keys("~");

    // Rotate belt (16,8): Left -> Up -> Right (two ~).
    goto(&mut s, 16, 8);
    s.feed_keys("~~");

    // Fill the gap at (28..=30, 8) with belts facing Right.
    place_run(&mut s, 28, 8, 'R', 'c', 3);

    use vimforge::resources::Facing;
    assert_eq!(s.app.map.entity_facing_at(&s.app.world, 10, 8), Some(Facing::Right));
    assert_eq!(s.app.map.entity_facing_at(&s.app.world, 11, 8), Some(Facing::Right));
    assert_eq!(s.entity_type_at(12, 8), Some(EntityType::BasicBelt));
    assert_eq!(s.app.map.entity_facing_at(&s.app.world, 13, 7), Some(Facing::Right));
    assert_eq!(s.app.map.entity_facing_at(&s.app.world, 16, 8), Some(Facing::Right));
    assert_eq!(s.entity_type_at(28, 8), Some(EntityType::BasicBelt));
    assert_eq!(s.entity_type_at(30, 8), Some(EntityType::BasicBelt));

    run_until_level(&mut s, 6, 3000);
}

/// Level 6 "Copy That" (ProduceWidgets(9)) — MUST use yy and p.
///
/// Three identical rows (deposits at (1,3)/(1,9)/(1,15), bins at
/// (38,3)/(38,9)/(38,15)). Build one full widget line on the top cluster
/// (rows 3..=6): belts -> smelter -> splitter (duplicates the ingot stream)
/// -> assembler (both inputs) -> belts -> bin. Then yank the 4 built rows
/// with `4yy` and paste the whole line onto the other two clusters with `p`.
#[test]
fn level_6_yank_paste_three_lines_produce_widgets() {
    let mut s = session_at_level(6);

    // ---- Build the top cluster by hand (belt row is 4) ----
    place_run(&mut s, 4, 4, 'R', 'c', 4); // belts (4..=7, 4)
    place_run(&mut s, 8, 3, 'R', 's', 1); // smelter (8..=10, 3..=5), in (8,4) out (10,4)
    place_run(&mut s, 11, 4, 'R', 'c', 2); // belts (11..=12, 4)
    // Splitter anchor (13,3): occupies (13..=15, 3..=5), input (13,4),
    // outputs (15,3) and (15,5) — duplicating the ingot stream.
    place_run(&mut s, 13, 3, 'R', 'b', 1);
    // Upper branch: corner belt down at (16,3), then right along row 4.
    place_run(&mut s, 16, 3, 'D', 'c', 1);
    place_run(&mut s, 16, 4, 'R', 'c', 4); // belts (16..=19, 4)
    // Lower branch: straight along row 5.
    place_run(&mut s, 16, 5, 'R', 'c', 4); // belts (16..=19, 5)
    // Assembler anchor (20,3): occupies (20..=22, 3..=6), inputs (20,4) and
    // (20,5), output (22,5).
    place_run(&mut s, 20, 3, 'R', 'a', 1);
    // Output: belts along row 5, then a corner up into the bin port (38,4).
    place_run(&mut s, 23, 5, 'R', 'c', 14); // belts (23..=36, 5)
    place_run(&mut s, 37, 5, 'U', 'c', 1); // corner belt (37,5) facing Up
    place_run(&mut s, 37, 4, 'R', 'c', 1); // belt (37,4) -> bin port (38,4)

    assert_eq!(s.entity_type_at(8, 3), Some(EntityType::Smelter));
    assert_eq!(s.entity_type_at(13, 3), Some(EntityType::Splitter));
    assert_eq!(s.entity_type_at(20, 3), Some(EntityType::Assembler));
    assert_eq!(s.entity_type_at(37, 4), Some(EntityType::BasicBelt));

    // ---- Copy the built line with yy, paste with p (the level's lesson) ----
    // The cluster spans rows 3..=6; yank those 4 rows from row 3.
    goto(&mut s, 0, 3);
    s.feed_keys("4yy");
    // Blueprint offsets are relative to the yanked entities' min position
    // (4,3), so pasting with the cursor at (4, 3+6) rebuilds the line one
    // cluster (6 rows) lower, and (4, 3+12) two clusters lower.
    goto(&mut s, 4, 9);
    s.feed_keys("p");
    goto(&mut s, 4, 15);
    s.feed_keys("p");

    // Pasted clusters exist (spot-check anchors shifted by +6/+12 rows).
    assert_eq!(s.entity_type_at(8, 9), Some(EntityType::Smelter));
    assert_eq!(s.entity_type_at(20, 9), Some(EntityType::Assembler));
    assert_eq!(s.entity_type_at(37, 10), Some(EntityType::BasicBelt));
    assert_eq!(s.entity_type_at(8, 15), Some(EntityType::Smelter));
    assert_eq!(s.entity_type_at(20, 15), Some(EntityType::Assembler));
    assert_eq!(s.entity_type_at(37, 16), Some(EntityType::BasicBelt));

    run_until_level(&mut s, 7, 3000);
}

/// Level 7 "Blueprints" (ProduceWidgets(10)) — MUST use "a named registers.
///
/// Four ore rows (belt rows 4, 10, 16, 22) and two bins at (50,6) and
/// (50,18): each PAIR of ore rows feeds one assembler whose output goes to
/// one bin. Build the top two-row cluster (rows 3..=11), yank it into named
/// register 'a' with `"a9yy`, then paste it 12 rows lower with `"ap`.
#[test]
fn level_7_named_register_blueprint_produces_widgets() {
    let mut s = session_at_level(7);

    // ---- Top cluster: ore rows 4 and 10 -> assembler (20,5) -> bin (50,6) ----
    // Upper line (row 4).
    place_run(&mut s, 4, 4, 'R', 'c', 4); // belts (4..=7, 4)
    place_run(&mut s, 8, 3, 'R', 's', 1); // smelter (8..=10, 3..=5), in (8,4) out (10,4)
    place_run(&mut s, 11, 4, 'R', 'c', 8); // belts (11..=18, 4)
    place_run(&mut s, 19, 4, 'D', 'c', 2); // corner: belts (19,4)(19,5) facing Down
    place_run(&mut s, 19, 6, 'R', 'c', 1); // belt (19,6) -> assembler port (20,6)

    // Lower line (row 10).
    place_run(&mut s, 4, 10, 'R', 'c', 4); // belts (4..=7, 10)
    place_run(&mut s, 8, 9, 'R', 's', 1); // smelter (8..=10, 9..=11), in (8,10) out (10,10)
    place_run(&mut s, 11, 10, 'R', 'c', 8); // belts (11..=18, 10)
    place_run(&mut s, 19, 10, 'U', 'c', 3); // corner: belts (19,10)(19,9)(19,8) facing Up
    place_run(&mut s, 19, 7, 'R', 'c', 1); // belt (19,7) -> assembler port (20,7)

    // Assembler anchor (20,5): occupies (20..=22, 5..=8), inputs (20,6) and
    // (20,7), output (22,7).
    place_run(&mut s, 20, 5, 'R', 'a', 1);
    // Output belts (23..=49, 7) to the bin's input port (50,7).
    place_run(&mut s, 23, 7, 'R', 'c', 27);

    assert_eq!(s.entity_type_at(8, 3), Some(EntityType::Smelter));
    assert_eq!(s.entity_type_at(8, 9), Some(EntityType::Smelter));
    assert_eq!(s.entity_type_at(20, 5), Some(EntityType::Assembler));
    assert_eq!(s.entity_type_at(49, 7), Some(EntityType::BasicBelt));

    // ---- Save the cluster into named register 'a' and replicate it ----
    // The cluster spans rows 3..=11 (9 rows); yank them from row 3.
    goto(&mut s, 0, 3);
    s.feed_keys("\"a9yy");
    // Blueprint offsets are relative to the min position (4,3); the second
    // cluster's rows sit exactly 12 lower, so paste with cursor at (4,15).
    goto(&mut s, 4, 15);
    s.feed_keys("\"ap");

    // Pasted cluster exists (spot-check anchors shifted by +12 rows).
    assert_eq!(s.entity_type_at(8, 15), Some(EntityType::Smelter));
    assert_eq!(s.entity_type_at(8, 21), Some(EntityType::Smelter));
    assert_eq!(s.entity_type_at(20, 17), Some(EntityType::Assembler));
    assert_eq!(s.entity_type_at(49, 19), Some(EntityType::BasicBelt));

    run_until_level(&mut s, 8, 3000);
}
