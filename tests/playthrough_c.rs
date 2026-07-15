//! Full playthrough tests for campaign levels 14-20: drive the REAL game
//! (GameSession — the same code path main.rs uses) with literal keystrokes
//! and verify each level can be completed the way a player would complete
//! it, following the intended teaching of each level's hints.
//!
//! Levels covered (Acts III-IV of the vim curriculum):
//!   14 "Word Hops"           — w b e ge W B E ^ g_   (UseCommands)
//!   15 "Precision Strikes"   — f t F T ; ,           (DeliverIngots)
//!   16 "Pages & Paragraphs"  — { } C-d C-u H M L zz  (UseCommands)
//!   17 "Substitute Teacher"  — s S C D ce            (DeliverIngots)
//!   18 "Inner Peace"         — cit diw da( di"       (DeliverIngots)
//!   19 "The Upgrader"        — C-a C-x gU gu         (UseCommands)
//!   20 "Join & Open"         — J (+counts)           (DeliverOre)

use vimforge::app::Mode;
use vimforge::game::session::GameSession;
use vimforge::resources::{EntityType, Facing};

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

fn facing_at(s: &GameSession, x: usize, y: usize) -> Option<Facing> {
    s.app.map.entity_facing_at(&s.app.world, x, y)
}

/// Level 14 "Word Hops" (UseCommands: w b e ge E ^).
///
/// Row 6 holds five clusters — belts (5..9), walls (20..25), belts (40..46),
/// walls (60..68), belts (80..85) — separated by wide empty stretches. The
/// word motions hop them exactly as the hints describe.
#[test]
fn level_14_word_motions_hop_the_clusters() {
    let mut s = session_at_level(14);

    // w from empty ground hops to the first entity of the map (reading order).
    s.feed_keys("w");
    assert_eq!(s.cursor(), (5, 6), "w should hop to cluster A's start");

    // e -> end of the current cluster.
    s.feed_keys("e");
    assert_eq!(s.cursor(), (9, 6), "e should land on cluster A's end");

    // w from a cluster end hops the gap to the next cluster's start.
    s.feed_keys("w");
    assert_eq!(s.cursor(), (20, 6), "w should hop the gap to cluster B");

    // E (big-word end) -> end of the wall cluster.
    s.feed_keys("E");
    assert_eq!(s.cursor(), (25, 6), "E should land on cluster B's end");

    // W hops to the next DIFFERENT building type (belts of cluster C).
    s.feed_keys("W");
    assert_eq!(s.cursor(), (40, 6), "W should hop to the next belt cluster");
    assert_eq!(s.entity_type_at(40, 6), Some(EntityType::BasicBelt));

    // b jumps back to the previous entity (cluster B's end).
    s.feed_keys("b");
    assert_eq!(s.cursor(), (25, 6), "b should step back to cluster B's end");

    // ge jumps back to the END of the PREVIOUS cluster (A).
    s.feed_keys("ge");
    assert_eq!(s.cursor(), (9, 6), "ge should land on cluster A's end");

    // g_ -> last entity in the row; ^ -> first entity in the row.
    s.feed_keys("g_");
    assert_eq!(s.cursor(), (85, 6), "g_ should land on the row's last entity");
    s.feed_keys("^");
    assert_eq!(s.cursor(), (5, 6), "^ should snap to the row's first entity");

    // All of w b e ge E ^ have now been used; completion checks on tick.
    s.tick(1);
    assert_eq!(s.current_level(), Some(15), "level 14 should auto-advance");
}

/// Level 15 "Precision Strikes" (DeliverIngots(8)) — fs finds each wrong
/// smelter, ~ rotates it, ; repeats the find, , goes back, tb stops short
/// of a bin.
#[test]
fn level_15_find_repeat_rotate_all_four_smelters() {
    let mut s = session_at_level(15);

    // fs: find the first smelter (anchor (14,2), facing Up).
    s.feed_keys("fs");
    assert_eq!(s.cursor(), (14, 2), "fs should land on smelter 1's anchor");
    assert_eq!(facing_at(&s, 14, 2), Some(Facing::Up));
    s.feed_keys("~"); // Up -> Right
    assert_eq!(facing_at(&s, 14, 2), Some(Facing::Right));

    // ; repeats the find: next smelter (14,8), facing Left.
    s.feed_keys(";");
    assert_eq!(s.cursor(), (14, 8), "; should jump to smelter 2");
    s.feed_keys("~~"); // Left -> Up -> Right
    assert_eq!(facing_at(&s, 14, 8), Some(Facing::Right));

    // ; again: smelter 3 (14,14), facing Down.
    s.feed_keys(";");
    assert_eq!(s.cursor(), (14, 14), "; should jump to smelter 3");
    s.feed_keys("~~~"); // Down -> Left -> Up -> Right
    assert_eq!(facing_at(&s, 14, 14), Some(Facing::Right));

    // ; again: smelter 4 (14,20), facing Up.
    s.feed_keys(";");
    assert_eq!(s.cursor(), (14, 20), "; should jump to smelter 4");
    s.feed_keys("~");
    assert_eq!(facing_at(&s, 14, 20), Some(Facing::Right));

    // , repeats the find backward: back to smelter 3.
    s.feed_keys(",");
    assert_eq!(s.cursor(), (14, 14), ", should jump back to smelter 3");

    // tb: 'til the next output bin — one tile short of its anchor (30,14).
    s.feed_keys("tb");
    assert_eq!(s.cursor(), (29, 14), "tb should stop one tile before the bin");

    run_until_level(&mut s, 16, 3000);
}

/// Level 16 "Pages & Paragraphs" (UseCommands: { } ctrl-d ctrl-u ctrl-f
/// ctrl-b zz zt zb) on a 120-row map whose station blocks sit at
/// y = 5, 25, 45, 65, 85, 105. The viewport is 44 rows tall.
///
/// NOTE: H/M/L were meant to be part of this level but are unreachable —
/// the parser requires empty modifiers for them while shifted letters
/// always carry SHIFT (engine bug; see the level's doc comment).
#[test]
fn level_16_paragraphs_pages_and_viewport_scrolls() {
    let mut s = session_at_level(16);

    // } hops station blocks (first entity row of each paragraph).
    s.feed_keys("}");
    assert_eq!(s.cursor(), (0, 5), "}} should land on station 1");
    s.feed_keys("}");
    assert_eq!(s.cursor(), (0, 25), "}} should land on station 2");
    s.feed_keys("}");
    assert_eq!(s.cursor(), (0, 45), "}} should land on station 3");

    // { goes back up to just above the current station.
    s.feed_keys("{");
    assert_eq!(s.cursor(), (0, 44), "{{ should stop just above station 3");

    // Ctrl-d / Ctrl-u: half a page (viewport is 44 rows tall -> 22 rows).
    s.feed_keys("<C-d>");
    assert_eq!(s.cursor().1, 66, "Ctrl-d should move half a page down");
    s.feed_keys("<C-u>");
    assert_eq!(s.cursor().1, 44, "Ctrl-u should move half a page back up");

    // Ctrl-f / Ctrl-b: a full page.
    s.feed_keys("<C-f>");
    assert_eq!(s.cursor().1, 88, "Ctrl-f should move a full page down");
    s.feed_keys("<C-b>");
    assert_eq!(s.cursor().1, 44, "Ctrl-b should move a full page back up");

    // zz centers the view on the cursor (row 44, height 44 -> offset 22).
    s.feed_keys("zz");
    assert_eq!(s.viewport.offset_y, 22, "zz should center the view on row 44");

    // zt / zb push the cursor row to the view's top / bottom (with a small
    // scroll margin, like vim's scrolloff).
    s.feed_keys("zt");
    assert!(
        s.viewport.offset_y > 22 && s.viewport.offset_y <= 44,
        "zt should scroll the cursor row toward the view top (offset {})",
        s.viewport.offset_y
    );
    s.feed_keys("zb");
    assert!(
        s.viewport.offset_y < 22,
        "zb should scroll the cursor row toward the view bottom (offset {})",
        s.viewport.offset_y
    );

    s.tick(1);
    assert_eq!(s.current_level(), Some(17), "level 16 should auto-advance");
}

/// Level 17 "Substitute Teacher" (DeliverIngots(6)) — s, ce, C, S and D
/// each repair the zone they were designed for.
#[test]
fn level_17_substitute_change_and_delete_family() {
    let mut s = session_at_level(17);

    // ---- Zone A: three walls on the line; fw + s c Esc + ; per wall.
    s.feed_keys("fw");
    assert_eq!(s.cursor(), (13, 7), "fw should find the first wall");
    s.feed_keys("sc<Esc>"); // s: delete + insert; c: belt; back to normal
    assert_eq!(s.entity_type_at(13, 7), Some(EntityType::BasicBelt));
    s.feed_keys(";");
    assert_eq!(s.cursor(), (15, 7), "; should find the second wall");
    s.feed_keys("sc<Esc>");
    assert_eq!(s.entity_type_at(15, 7), Some(EntityType::BasicBelt));
    s.feed_keys(";");
    assert_eq!(s.cursor(), (17, 7), "; should find the third wall");
    s.feed_keys("sc<Esc>");
    assert_eq!(s.entity_type_at(17, 7), Some(EntityType::BasicBelt));

    // ---- Zone B: up-belt block (32..=37) ending at the gap (38,7).
    goto(&mut s, 32, 7);
    s.feed_keys("ce"); // change to cluster end: wipes 32..=37, enters insert
    assert_eq!(s.mode(), Mode::Insert);
    s.feed_keys("ccccccc<Esc>"); // 7 belts: 32..=38 (covers the gap too)
    for x in 32..=38 {
        assert_eq!(
            facing_at(&s, x, 7),
            Some(Facing::Right),
            "belt at ({x},7) should be rebuilt facing Right"
        );
    }

    // ---- Zone C: left-belt tail 47..=57; C changes to the end of the row.
    goto(&mut s, 47, 7);
    s.feed_keys("C");
    assert_eq!(s.mode(), Mode::Insert);
    assert_eq!(s.entity_type_at(57, 7), None, "C should wipe the whole tail");
    s.feed_keys("cccccccccc<Esc>"); // 10 belts: 47..=56
    assert_eq!(s.entity_type_at(56, 7), Some(EntityType::BasicBelt));
    assert_eq!(s.cursor(), (57, 7));

    // Drop into the bin below: two down-belts in column 57 (rows 7-8).
    place_run(&mut s, 57, 7, 'D', 'c', 2);
    assert_eq!(facing_at(&s, 57, 7), Some(Facing::Down));
    assert_eq!(facing_at(&s, 57, 8), Some(Facing::Down));

    // ---- Scrap rows: S changes (wipes) row 2, D deletes row 3's tail.
    goto(&mut s, 10, 2);
    s.feed_keys("S<Esc>");
    assert_eq!(s.entity_type_at(10, 2), None, "S should wipe scrap row 2");
    assert_eq!(s.entity_type_at(40, 2), None);
    goto(&mut s, 30, 3);
    s.feed_keys("D");
    assert_eq!(s.entity_type_at(30, 3), None, "D should delete to row end");
    assert_eq!(s.entity_type_at(50, 3), None);
    assert_eq!(
        s.entity_type_at(29, 3),
        Some(EntityType::BasicBelt),
        "D must not touch tiles left of the cursor"
    );

    run_until_level(&mut s, 18, 3000);
}

/// Level 18 "Inner Peace" (DeliverIngots(8)) — each obstruction falls to
/// exactly one text object: cit, diw, da(, di".
#[test]
fn level_18_text_objects_excise_every_blocker() {
    let mut s = session_at_level(18);

    // ---- 1. Wrong machine: cit + s swaps the assembler for a smelter.
    assert_eq!(s.entity_type_at(19, 9), Some(EntityType::Assembler));
    goto(&mut s, 19, 9);
    s.feed_keys("cit"); // change the machine's footprint, enter insert
    assert_eq!(s.mode(), Mode::Insert);
    assert_eq!(s.entity_type_at(20, 10), None, "cit should raze the assembler");
    s.feed_keys("s<Esc>"); // drop a smelter at the same anchor
    assert_eq!(s.entity_type_at(19, 9), Some(EntityType::Smelter));
    assert_eq!(facing_at(&s, 19, 9), Some(Facing::Right));

    // ---- 2. Junk island (30..=35,10): diw deletes exactly that cluster.
    goto(&mut s, 32, 10);
    s.feed_keys("diw");
    assert_eq!(s.entity_type_at(30, 10), None, "diw should clear the island");
    assert_eq!(s.entity_type_at(35, 10), None);
    assert_eq!(
        s.entity_type_at(28, 10),
        Some(EntityType::BasicBelt),
        "diw must not eat the good belts beyond the gap"
    );
    // Rebuild the span including both one-tile gaps: 29..=36.
    place_run(&mut s, 29, 10, 'R', 'c', 8);

    // ---- 3. Sealed compound: stand inside, da( razes walls and contents.
    goto(&mut s, 48, 10);
    s.feed_keys("da(");
    assert_eq!(s.entity_type_at(43, 10), None, "da( should raze the ring wall");
    assert_eq!(s.entity_type_at(53, 10), None);
    assert_eq!(s.entity_type_at(46, 8), None, "da( should raze interior junk");
    assert_eq!(s.entity_type_at(47, 12), None);
    assert_eq!(
        s.entity_type_at(42, 10),
        Some(EntityType::BasicBelt),
        "da( must not touch the line outside the ring"
    );
    // Re-lay the crossing: 43..=53.
    place_run(&mut s, 43, 10, 'R', 'c', 11);

    // ---- 4. Siphon column: di" deletes the whole up-facing belt run.
    goto(&mut s, 59, 7);
    s.feed_keys("di\"");
    assert_eq!(s.entity_type_at(59, 4), None, "di\" should clear the run's top");
    assert_eq!(s.entity_type_at(59, 10), None, "di\" should clear the crossing");
    assert_eq!(
        s.entity_type_at(58, 10),
        Some(EntityType::BasicBelt),
        "di\" must not touch the horizontal line"
    );
    // One right-facing belt restores the crossing.
    place_run(&mut s, 59, 10, 'R', 'c', 1);

    run_until_level(&mut s, 19, 3000);
}

/// Level 19 "The Upgrader" (UseCommands: ctrl-a ctrl-x gU gu) — tier the
/// line up with Ctrl-a/gUU, tier the scrap row down with guu.
#[test]
fn level_19_tier_ladder_up_and_down() {
    let mut s = session_at_level(19);

    // Ctrl-a / Ctrl-x on a single belt.
    assert_eq!(s.entity_type_at(5, 6), Some(EntityType::BasicBelt));
    goto(&mut s, 5, 6);
    s.feed_keys("<C-a>");
    assert_eq!(s.entity_type_at(5, 6), Some(EntityType::FastBelt));
    s.feed_keys("<C-a>");
    assert_eq!(s.entity_type_at(5, 6), Some(EntityType::ExpressBelt));
    s.feed_keys("<C-x>");
    assert_eq!(s.entity_type_at(5, 6), Some(EntityType::FastBelt));

    // gUU: upgrade everything upgradable on the row (machines are skipped).
    s.feed_keys("gUU");
    assert_eq!(s.entity_type_at(5, 6), Some(EntityType::ExpressBelt));
    assert_eq!(s.entity_type_at(11, 6), Some(EntityType::FastBelt));
    assert_eq!(s.entity_type_at(37, 6), Some(EntityType::FastBelt));
    assert_eq!(s.entity_type_at(12, 5), Some(EntityType::Smelter), "no ladder");
    s.feed_keys("gUU");
    assert_eq!(s.entity_type_at(11, 6), Some(EntityType::ExpressBelt));
    assert_eq!(s.entity_type_at(37, 6), Some(EntityType::ExpressBelt));

    // guu: downgrade the wasteful ExpressBelt scrap row.
    assert_eq!(s.entity_type_at(15, 11), Some(EntityType::ExpressBelt));
    goto(&mut s, 15, 11);
    s.feed_keys("guu");
    assert_eq!(s.entity_type_at(15, 11), Some(EntityType::FastBelt));
    assert_eq!(s.entity_type_at(30, 11), Some(EntityType::FastBelt));
    s.feed_keys("guu");
    assert_eq!(s.entity_type_at(15, 11), Some(EntityType::BasicBelt));

    // ctrl-a, ctrl-x, gU, gu all used; completion checks on tick.
    s.tick(1);
    assert_eq!(s.current_level(), Some(20), "level 19 should auto-advance");
}

/// Level 20 "Join & Open" (DeliverOre(12)) — 2J bridges both gaps of each
/// row; the ore only flows across joined rows.
#[test]
fn level_20_join_bridges_every_gap() {
    let mut s = session_at_level(20);

    for &y in &[4usize, 10, 16] {
        // The two gaps are open before the join.
        assert_eq!(s.entity_type_at(15, y), None);
        assert_eq!(s.entity_type_at(30, y), None);

        // Stand on the first cluster, 2J bridges both gaps.
        goto(&mut s, 6, y);
        s.feed_keys("2J");

        for x in 15..=19 {
            assert_eq!(
                s.entity_type_at(x, y),
                Some(EntityType::BasicBelt),
                "J should fill gap 1 at ({x},{y})"
            );
            assert_eq!(facing_at(&s, x, y), Some(Facing::Right));
        }
        for x in 30..=34 {
            assert_eq!(
                s.entity_type_at(x, y),
                Some(EntityType::BasicBelt),
                "J should fill gap 2 at ({x},{y})"
            );
            assert_eq!(facing_at(&s, x, y), Some(Facing::Right));
        }
        // Like vim's J, the cursor sits at the (second) joint.
        assert_eq!(s.cursor(), (30, y));
    }

    run_until_level(&mut s, 21, 3000);
}
