//! THE definitive end-to-end proof: one single GameSession plays the ENTIRE
//! 30-level campaign exactly as a player would — start at the main menu,
//! press '1', complete every level through its natural auto-advance, finish
//! level 30, watch freeplay unlock, enter freeplay with `:freeplay`, build a
//! smelting line at a seeded iron deposit, and earn money from deliveries.
//!
//! Unlike tests/playthrough_*.rs (which call `s.start_level(N)` on a fresh
//! session per level), this chain arrives at each level via the campaign's
//! own auto-advance with the SAME session. Vim state persists across levels
//! exactly like registers/marks persist across buffers in vim:
//!
//!   - `start_level` resets the parser, cursor, world, map, undo stack and
//!     inventory, but NOT `InputState`'s registers, numbered delete history,
//!     marks, jumplist, search pattern, find-repeat state or macro registers.
//!
//! ## Chain adaptations vs. the fresh-start solutions
//!
//! Almost every fresh solution replays verbatim, because each level's
//! keystrokes establish their own state (searches, finds, marks, dot) before
//! relying on it. The one real casualty is level 22 ("Register Bank"):
//!
//!   - The fresh test asserts `"2` is EMPTY after the level's first delete
//!     and after a black-hole delete. Mid-campaign that is false: deletes
//!     from levels 17, 18, 21 (D/S, diw/da(/di", visual x) have already
//!     filled the numbered history, so level 22's `d5l` shifts a REAL old
//!     delete into `"2`. The chained solve asserts the shift semantics
//!     relatively (snapshot "1 before, verify it moved to "2, verify "_
//!     shifts nothing) instead of asserting emptiness. This is a genuine
//!     player-facing difference between "practice level 22 in isolation"
//!     and "reach level 22 mid-campaign".
//!
//! Everything else — register 'a' holding level 10's MACRO when level 22
//! overwrites it with `"a3yy`, level 26 re-recording `qa` over level 22's
//! blueprint, mark letters a-d reused by levels 12/23/30, the jumplist
//! carrying dozens of old entries into level 23's Ctrl-o/Ctrl-i drill —
//! turned out to be chain-safe because overwrite semantics are total and
//! jumplist walks are relative.

use std::sync::atomic::{AtomicUsize, Ordering};

use vimforge::app::{GameMode, Mode};
use vimforge::game::session::{parse_key_notation, GameSession};
use vimforge::resources::{EntityType, Facing, Resource};

/// Total keystrokes fed by the player over the whole campaign (macro
/// replays and auto-advances are free, exactly like real play).
static KEYSTROKES: AtomicUsize = AtomicUsize::new(0);

/// Feed keys through the real session entry point, counting them.
fn fk(s: &mut GameSession, keys: &str) {
    KEYSTROKES.fetch_add(parse_key_notation(keys).len(), Ordering::Relaxed);
    s.feed_keys(keys);
}

/// Move the cursor to an absolute map position using normal-mode motions,
/// exactly like a player: Esc, gg, then counted j/l.
fn goto(s: &mut GameSession, x: usize, y: usize) {
    fk(s, "<Esc>gg");
    if y > 0 {
        fk(s, &format!("{y}j"));
    }
    if x > 0 {
        fk(s, &format!("{x}l"));
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
    fk(s, "i");
    assert_eq!(s.mode(), Mode::Insert);
    fk(s, &facing_key.to_string());
    assert_eq!(s.cursor(), (x, y), "facing key should step onto start tile");
    for _ in 0..n {
        fk(s, &key.to_string());
    }
    fk(s, "<Esc>");
    assert_eq!(s.mode(), Mode::Normal);
}

/// Tick the simulation until the campaign auto-advances to `next`; a chain
/// break panics with the level that failed to complete.
fn run_until_level(s: &mut GameSession, next: usize, max_ticks: usize) {
    for _ in 0..max_ticks {
        s.tick(1);
        if s.current_level() == Some(next) {
            return;
        }
    }
    let (ore, ingots, widgets, total) = s.output_totals();
    panic!(
        "CHAIN BREAK at level {}: never auto-advanced to level {next} after \
         {max_ticks} ticks; delivered ore={ore} ingots={ingots} \
         widgets={widgets} total={total}",
        next - 1
    );
}

/// Every solve asserts its entry state so a chain break is attributable.
fn enter(s: &GameSession, level: usize) {
    assert_eq!(
        s.current_level(),
        Some(level),
        "chained session should have auto-advanced to level {level}"
    );
    assert_eq!(s.mode(), Mode::Normal, "level {level} should start in Normal");
    assert_eq!(s.cursor(), (0, 0), "level {level} should start at origin");
}

fn facing_at(s: &GameSession, x: usize, y: usize) -> Option<Facing> {
    s.app.map.entity_facing_at(&s.app.world, x, y)
}

// ===========================================================================
// Act I-II: levels 1-13
// ===========================================================================

/// Level 1 "Navigation": touch both output bins with counted motions.
fn solve_level_1(s: &mut GameSession) {
    enter(s, 1);
    fk(s, "4j31l");
    assert_eq!(s.cursor(), (31, 4));
    fk(s, "11j");
    assert_eq!(s.cursor(), (31, 15));
    run_until_level(s, 2, 100);
}

/// Level 2 "Belts": 23 belts from the deposit's port to the bin.
fn solve_level_2(s: &mut GameSession) {
    enter(s, 2);
    fk(s, "6j5l");
    assert_eq!(s.cursor(), (5, 6));
    fk(s, "i");
    assert_eq!(s.mode(), Mode::Insert);
    fk(s, &"c".repeat(23));
    fk(s, "<Esc>");
    assert_eq!(s.entity_type_at(5, 6), Some(EntityType::BasicBelt));
    assert_eq!(s.entity_type_at(27, 6), Some(EntityType::BasicBelt));
    run_until_level(s, 3, 600);
}

/// Level 3 "Smelting": belts -> smelter -> belts -> bin.
fn solve_level_3(s: &mut GameSession) {
    enter(s, 3);
    place_run(s, 5, 8, 'R', 'c', 7);
    place_run(s, 12, 7, 'R', 's', 1);
    place_run(s, 15, 8, 'R', 'c', 23);
    assert_eq!(s.entity_type_at(12, 7), Some(EntityType::Smelter));
    run_until_level(s, 4, 3000);
}

/// Level 4 "Full Production": two ore lines corner into one assembler.
fn solve_level_4(s: &mut GameSession) {
    enter(s, 4);
    // Top line.
    place_run(s, 5, 5, 'R', 'c', 3);
    place_run(s, 8, 4, 'R', 's', 1);
    place_run(s, 11, 5, 'R', 'c', 18);
    place_run(s, 29, 5, 'D', 'c', 3);
    place_run(s, 29, 8, 'R', 'c', 1);
    // Bottom line.
    place_run(s, 5, 14, 'R', 'c', 3);
    place_run(s, 8, 13, 'R', 's', 1);
    place_run(s, 11, 14, 'R', 'c', 18);
    place_run(s, 29, 14, 'U', 'c', 5);
    place_run(s, 29, 9, 'R', 'c', 1);
    // Assembler + output run.
    place_run(s, 30, 7, 'R', 'a', 1);
    place_run(s, 33, 9, 'R', 'c', 15);
    assert_eq!(s.entity_type_at(30, 7), Some(EntityType::Assembler));
    run_until_level(s, 5, 3000);
}

/// Level 5 "Demolish & Rebuild": rotate broken pieces with ~, fill gaps.
fn solve_level_5(s: &mut GameSession) {
    enter(s, 5);
    goto(s, 10, 8);
    fk(s, "~~");
    fk(s, "l~");
    place_run(s, 12, 8, 'R', 'c', 1);
    assert_eq!(s.cursor(), (13, 8));
    fk(s, "~"); // rotate the smelter under the cursor
    goto(s, 16, 8);
    fk(s, "~~");
    place_run(s, 28, 8, 'R', 'c', 3);
    assert_eq!(facing_at(s, 13, 7), Some(Facing::Right));
    run_until_level(s, 6, 3000);
}

/// Level 6 "Copy That": build one widget line, 4yy + p onto two clusters.
fn solve_level_6(s: &mut GameSession) {
    enter(s, 6);
    place_run(s, 4, 4, 'R', 'c', 4);
    place_run(s, 8, 3, 'R', 's', 1);
    place_run(s, 11, 4, 'R', 'c', 2);
    place_run(s, 13, 3, 'R', 'b', 1);
    place_run(s, 16, 3, 'D', 'c', 1);
    place_run(s, 16, 4, 'R', 'c', 4);
    place_run(s, 16, 5, 'R', 'c', 4);
    place_run(s, 20, 3, 'R', 'a', 1);
    place_run(s, 23, 5, 'R', 'c', 14);
    place_run(s, 37, 5, 'U', 'c', 1);
    place_run(s, 37, 4, 'R', 'c', 1);

    goto(s, 0, 3);
    fk(s, "4yy");
    goto(s, 4, 9);
    fk(s, "p");
    goto(s, 4, 15);
    fk(s, "p");
    assert_eq!(s.entity_type_at(8, 9), Some(EntityType::Smelter));
    assert_eq!(s.entity_type_at(20, 15), Some(EntityType::Assembler));
    run_until_level(s, 7, 3000);
}

/// Level 7 "Blueprints": build the top 2-row cluster, "a9yy, "ap.
fn solve_level_7(s: &mut GameSession) {
    enter(s, 7);
    place_run(s, 4, 4, 'R', 'c', 4);
    place_run(s, 8, 3, 'R', 's', 1);
    place_run(s, 11, 4, 'R', 'c', 8);
    place_run(s, 19, 4, 'D', 'c', 2);
    place_run(s, 19, 6, 'R', 'c', 1);
    place_run(s, 4, 10, 'R', 'c', 4);
    place_run(s, 8, 9, 'R', 's', 1);
    place_run(s, 11, 10, 'R', 'c', 8);
    place_run(s, 19, 10, 'U', 'c', 3);
    place_run(s, 19, 7, 'R', 'c', 1);
    place_run(s, 20, 5, 'R', 'a', 1);
    place_run(s, 23, 7, 'R', 'c', 27);

    goto(s, 0, 3);
    fk(s, "\"a9yy");
    goto(s, 4, 15);
    fk(s, "\"ap");
    assert_eq!(s.entity_type_at(8, 15), Some(EntityType::Smelter));
    assert_eq!(s.entity_type_at(20, 17), Some(EntityType::Assembler));
    run_until_level(s, 8, 3000);
}

/// Level 8 "Block Select": Ctrl-v yank the working cluster, paste below.
fn solve_level_8(s: &mut GameSession) {
    enter(s, 8);
    assert_eq!(s.entity_type_at(13, 3), Some(EntityType::Smelter));
    goto(s, 6, 3);
    fk(s, "<C-v>");
    assert_eq!(s.mode(), Mode::VisualBlock);
    fk(s, &"l".repeat(25));
    fk(s, &"j".repeat(6));
    assert_eq!(s.cursor(), (31, 9));
    fk(s, "y");
    goto(s, 6, 17);
    fk(s, "p");
    assert_eq!(s.entity_type_at(13, 17), Some(EntityType::Smelter));
    assert_eq!(s.entity_type_at(22, 18), Some(EntityType::Assembler));
    assert_eq!(facing_at(s, 20, 18), Some(Facing::Down));
    run_until_level(s, 9, 3000);
}

/// Level 9 "Find & Jump": f, /, n and % locate five broken clusters.
fn solve_level_9(s: &mut GameSession) {
    enter(s, 9);
    // Cluster 1: fc, %, fill the gap.
    fk(s, "fc");
    assert_eq!(s.cursor(), (5, 4));
    fk(s, "%");
    assert_eq!(s.cursor(), (8, 4));
    fk(s, "lic<Esc>");
    // Cluster 2: /smelter, rotate.
    fk(s, "/smelter<CR>");
    assert_eq!(s.cursor(), (49, 5));
    fk(s, "~~");
    // Cluster 3: /ore, follow the belts, fill the 2-tile gap.
    fk(s, "/ore<CR>");
    assert_eq!(s.cursor(), (22, 14));
    fk(s, "j3l%");
    assert_eq!(s.cursor(), (27, 15));
    fk(s, "licc<Esc>");
    // Cluster 4: n, %, rotate the bad belt.
    fk(s, "n");
    assert_eq!(s.cursor(), (5, 22));
    fk(s, "j3l%");
    assert_eq!(s.cursor(), (10, 22)); // % steps back with the walk's original direction
    fk(s, "jl~");
    // Cluster 5: n, place the missing smelter.
    fk(s, "n");
    assert_eq!(s.cursor(), (42, 25));
    fk(s, "j3l%");
    assert_eq!(s.cursor(), (49, 26));
    fk(s, "lkis<Esc>");
    assert_eq!(s.entity_type_at(50, 25), Some(EntityType::Smelter));
    run_until_level(s, 10, 3000);
}

/// The relative-motion macro body used by levels 10 and 26: builds one
/// complete widget band and ends one band (6 rows) lower.
fn band_macro_body() -> String {
    format!(
        "i{belts1}ksj{belts2}kajj{belts3}<Esc>h~~~kic<Esc>9h~jic<Esc>36h5j",
        belts1 = "c".repeat(6),
        belts2 = "c".repeat(27),
        belts3 = "c".repeat(5),
    )
}

/// Level 10 "Macro Factory": qa..q one band, 4@a the other four.
fn solve_level_10(s: &mut GameSession) {
    enter(s, 10);
    goto(s, 4, 3);
    fk(s, "qa");
    assert_eq!(s.input.parser.recording_macro, Some('a'));
    fk(s, &band_macro_body());
    fk(s, "q");
    assert_eq!(s.input.parser.recording_macro, None);
    assert_eq!(s.entity_type_at(10, 2), Some(EntityType::Smelter));
    assert_eq!(s.cursor(), (4, 9), "macro must end one band lower");
    fk(s, "4@a");
    for i in 1..5 {
        let y = 2 + i * 6;
        assert_eq!(s.entity_type_at(10, y), Some(EntityType::Smelter), "band {i}");
        assert_eq!(s.entity_type_at(40, y), Some(EntityType::Assembler), "band {i}");
    }
    run_until_level(s, 11, 3000);
}

/// Level 11 "The Dot": ~ once, then l. across the whole row.
fn solve_level_11(s: &mut GameSession) {
    enter(s, 11);
    assert_eq!(facing_at(s, 6, 10), Some(Facing::Up));
    goto(s, 6, 10);
    fk(s, "~");
    for _ in 0..21 {
        fk(s, "l.");
    }
    assert_eq!(s.cursor(), (27, 10));
    for x in 6..=27 {
        assert_eq!(facing_at(s, x, 10), Some(Facing::Right), "belt ({x},10)");
    }
    run_until_level(s, 12, 100);
}

/// Level 12 "Marks & Navigation": fix four corner clusters, hop the marks.
fn solve_level_12(s: &mut GameSession) {
    enter(s, 12);
    goto(s, 11, 3);
    fk(s, "maisj");
    fk(s, &"c".repeat(7));
    fk(s, "<Esc>");
    goto(s, 70, 3);
    fk(s, "mbisj");
    fk(s, &"c".repeat(4));
    fk(s, "<Esc>");
    goto(s, 10, 35);
    fk(s, "mcisj");
    fk(s, &"c".repeat(8));
    fk(s, "<Esc>");
    goto(s, 69, 35);
    fk(s, "mdisj");
    fk(s, &"c".repeat(5));
    fk(s, "<Esc>");
    // Hop between all four clusters via exact marks (overwrites whatever
    // the same letters held in earlier levels — vim semantics).
    fk(s, "`a");
    assert_eq!(s.cursor(), (11, 3));
    fk(s, "`b");
    assert_eq!(s.cursor(), (70, 3));
    fk(s, "`c");
    assert_eq!(s.cursor(), (10, 35));
    fk(s, "`d");
    assert_eq!(s.cursor(), (69, 35));
    fk(s, "'a");
    assert_eq!(s.cursor(), (2, 3));
    run_until_level(s, 13, 3000);
}

/// Level 13 "Split View": Ctrl-w stubs, then the cross-map widget route.
fn solve_level_13(s: &mut GameSession) {
    enter(s, 13);
    fk(s, "<C-w>v<C-w>l<C-w>h<C-w>j<C-w>k<C-w>s<C-w>=<C-w>o<C-w>q");
    assert_eq!(s.cursor(), (0, 0), "ctrl-w commands should not move cursor");
    place_run(s, 21, 3, 'R', 'c', 3);
    place_run(s, 24, 2, 'R', 'a', 1);
    place_run(s, 21, 7, 'R', 'c', 2);
    place_run(s, 23, 7, 'U', 'c', 3);
    place_run(s, 23, 4, 'R', 'c', 1);
    place_run(s, 27, 4, 'R', 'c', 49);
    place_run(s, 76, 4, 'D', 'c', 34);
    place_run(s, 76, 38, 'R', 'c', 1);
    assert_eq!(s.entity_type_at(24, 2), Some(EntityType::Assembler));
    assert!(!s.app.freeplay_unlocked, "freeplay must still be locked at 13");
    run_until_level(s, 14, 3000);
}

// ===========================================================================
// Act III-IV: levels 14-20
// ===========================================================================

/// Level 14 "Word Hops": w b e ge W E ^ g_ across the clusters.
fn solve_level_14(s: &mut GameSession) {
    enter(s, 14);
    fk(s, "w");
    assert_eq!(s.cursor(), (5, 6));
    fk(s, "e");
    assert_eq!(s.cursor(), (9, 6));
    fk(s, "w");
    assert_eq!(s.cursor(), (20, 6));
    fk(s, "E");
    assert_eq!(s.cursor(), (25, 6));
    fk(s, "W");
    assert_eq!(s.cursor(), (40, 6));
    fk(s, "b");
    assert_eq!(s.cursor(), (25, 6));
    fk(s, "ge");
    assert_eq!(s.cursor(), (9, 6));
    fk(s, "g_");
    assert_eq!(s.cursor(), (85, 6));
    fk(s, "^");
    assert_eq!(s.cursor(), (5, 6));
    run_until_level(s, 15, 100);
}

/// Level 15 "Precision Strikes": fs ; , tb + rotations.
fn solve_level_15(s: &mut GameSession) {
    enter(s, 15);
    fk(s, "fs");
    assert_eq!(s.cursor(), (14, 2));
    fk(s, "~");
    fk(s, ";");
    assert_eq!(s.cursor(), (14, 8));
    fk(s, "~~");
    fk(s, ";");
    assert_eq!(s.cursor(), (14, 14));
    fk(s, "~~~");
    fk(s, ";");
    assert_eq!(s.cursor(), (14, 20));
    fk(s, "~");
    fk(s, ",");
    assert_eq!(s.cursor(), (14, 14));
    fk(s, "tb");
    assert_eq!(s.cursor(), (29, 14));
    for &y in &[2usize, 8, 14, 20] {
        assert_eq!(facing_at(s, 14, y), Some(Facing::Right), "smelter row {y}");
    }
    run_until_level(s, 16, 3000);
}

/// Level 16 "Pages & Paragraphs": { } Ctrl-d/u/f/b zz zt zb.
fn solve_level_16(s: &mut GameSession) {
    enter(s, 16);
    fk(s, "}");
    assert_eq!(s.cursor(), (0, 5));
    fk(s, "}");
    assert_eq!(s.cursor(), (0, 25));
    fk(s, "}");
    assert_eq!(s.cursor(), (0, 45));
    fk(s, "{");
    assert_eq!(s.cursor(), (0, 44));
    fk(s, "<C-d>");
    assert_eq!(s.cursor().1, 66);
    fk(s, "<C-u>");
    assert_eq!(s.cursor().1, 44);
    fk(s, "<C-f>");
    assert_eq!(s.cursor().1, 88);
    fk(s, "<C-b>");
    assert_eq!(s.cursor().1, 44);
    fk(s, "zz");
    assert_eq!(s.viewport.offset_y, 22, "zz should center the view on row 44");
    fk(s, "zt");
    assert!(s.viewport.offset_y > 22 && s.viewport.offset_y <= 44);
    fk(s, "zb");
    assert!(s.viewport.offset_y < 22);
    run_until_level(s, 17, 100);
}

/// Level 17 "Substitute Teacher": s ce C S D repairs.
fn solve_level_17(s: &mut GameSession) {
    enter(s, 17);
    // Zone A: three walls, fixed with fw + s c Esc + ;.
    fk(s, "fw");
    assert_eq!(s.cursor(), (13, 7));
    fk(s, "sc<Esc>");
    fk(s, ";");
    assert_eq!(s.cursor(), (15, 7));
    fk(s, "sc<Esc>");
    fk(s, ";");
    assert_eq!(s.cursor(), (17, 7));
    fk(s, "sc<Esc>");
    // Zone B: ce wipes the up-belt block; rebuild through the gap.
    goto(s, 32, 7);
    fk(s, "ce");
    assert_eq!(s.mode(), Mode::Insert);
    fk(s, "ccccccc<Esc>");
    for x in 32..=38 {
        assert_eq!(facing_at(s, x, 7), Some(Facing::Right), "belt ({x},7)");
    }
    // Zone C: C changes to end of row; rebuild and drop into the bin.
    goto(s, 47, 7);
    fk(s, "C");
    assert_eq!(s.entity_type_at(57, 7), None);
    fk(s, "cccccccccc<Esc>");
    assert_eq!(s.cursor(), (57, 7));
    place_run(s, 57, 7, 'D', 'c', 2);
    // Scrap rows: S wipes row 2, D deletes row 3's tail.
    goto(s, 10, 2);
    fk(s, "S<Esc>");
    assert_eq!(s.entity_type_at(10, 2), None);
    goto(s, 30, 3);
    fk(s, "D");
    assert_eq!(s.entity_type_at(30, 3), None);
    assert_eq!(s.entity_type_at(29, 3), Some(EntityType::BasicBelt));
    run_until_level(s, 18, 3000);
}

/// Level 18 "Inner Peace": cit diw da( di" excise every blocker.
fn solve_level_18(s: &mut GameSession) {
    enter(s, 18);
    // 1. Wrong machine: cit + s.
    goto(s, 19, 9);
    fk(s, "cit");
    assert_eq!(s.mode(), Mode::Insert);
    fk(s, "s<Esc>");
    assert_eq!(s.entity_type_at(19, 9), Some(EntityType::Smelter));
    // 2. Junk island: diw, rebuild the span.
    goto(s, 32, 10);
    fk(s, "diw");
    assert_eq!(s.entity_type_at(30, 10), None);
    assert_eq!(s.entity_type_at(28, 10), Some(EntityType::BasicBelt));
    place_run(s, 29, 10, 'R', 'c', 8);
    // 3. Sealed compound: da( razes ring + contents, re-lay the crossing.
    goto(s, 48, 10);
    fk(s, "da(");
    assert_eq!(s.entity_type_at(43, 10), None);
    assert_eq!(s.entity_type_at(42, 10), Some(EntityType::BasicBelt));
    place_run(s, 43, 10, 'R', 'c', 11);
    // 4. Siphon column: di", restore the crossing tile.
    goto(s, 59, 7);
    fk(s, "di\"");
    assert_eq!(s.entity_type_at(59, 10), None);
    place_run(s, 59, 10, 'R', 'c', 1);
    run_until_level(s, 19, 3000);
}

/// Level 19 "The Upgrader": Ctrl-a Ctrl-x gUU guu tier the belts.
fn solve_level_19(s: &mut GameSession) {
    enter(s, 19);
    goto(s, 5, 6);
    fk(s, "<C-a>");
    assert_eq!(s.entity_type_at(5, 6), Some(EntityType::FastBelt));
    fk(s, "<C-a>");
    assert_eq!(s.entity_type_at(5, 6), Some(EntityType::ExpressBelt));
    fk(s, "<C-x>");
    assert_eq!(s.entity_type_at(5, 6), Some(EntityType::FastBelt));
    fk(s, "gUU");
    assert_eq!(s.entity_type_at(11, 6), Some(EntityType::FastBelt));
    fk(s, "gUU");
    assert_eq!(s.entity_type_at(11, 6), Some(EntityType::ExpressBelt));
    goto(s, 15, 11);
    fk(s, "guu");
    assert_eq!(s.entity_type_at(15, 11), Some(EntityType::FastBelt));
    fk(s, "guu");
    assert_eq!(s.entity_type_at(15, 11), Some(EntityType::BasicBelt));
    run_until_level(s, 20, 100);
}

/// Level 20 "Join & Open": 2J bridges both gaps of each of three rows.
fn solve_level_20(s: &mut GameSession) {
    enter(s, 20);
    for &y in &[4usize, 10, 16] {
        assert_eq!(s.entity_type_at(15, y), None);
        goto(s, 6, y);
        fk(s, "2J");
        for x in 15..=19 {
            assert_eq!(s.entity_type_at(x, y), Some(EntityType::BasicBelt));
        }
        for x in 30..=34 {
            assert_eq!(s.entity_type_at(x, y), Some(EntityType::BasicBelt));
        }
        assert_eq!(s.cursor(), (30, y));
    }
    run_until_level(s, 21, 3000);
}

// ===========================================================================
// Act V-VI: levels 21-30
// ===========================================================================

/// Level 21 "Visual Mastery": v~ v r, vab x, Ctrl-v x, gv, block-I.
fn solve_level_21(s: &mut GameSession) {
    enter(s, 21);
    goto(s, 10, 3);
    fk(s, "v5l~");
    for x in 10..=15 {
        assert_eq!(facing_at(s, x, 3), Some(Facing::Right), "belt ({x},3)");
    }
    goto(s, 16, 11);
    fk(s, "v3lrc");
    for x in 16..=19 {
        assert_eq!(s.entity_type_at(x, 11), Some(EntityType::BasicBelt));
    }
    goto(s, 25, 7);
    fk(s, "vabx");
    assert_eq!(s.entity_type_at(22, 5), None);
    goto(s, 24, 6);
    fk(s, "is<Esc>");
    assert_eq!(s.entity_type_at(24, 6), Some(EntityType::Smelter));
    goto(s, 20, 2);
    fk(s, "<C-v>10jx");
    assert_eq!(s.entity_type_at(20, 3), None);
    fk(s, "gv");
    assert_eq!(s.mode(), Mode::VisualBlock, "gv reselects the block");
    fk(s, "Ic");
    for y in [3usize, 7, 11] {
        assert_eq!(s.entity_type_at(20, y), Some(EntityType::BasicBelt));
    }
    goto(s, 4, 7);
    fk(s, "J");
    fk(s, "J");
    assert_eq!(s.entity_type_at(28, 7), Some(EntityType::BasicBelt));
    run_until_level(s, 22, 3000);
}

/// Level 22 "Register Bank" — CHAIN-ADAPTED (see file header).
///
/// The fresh-start test asserts `"2` is empty after the level's first
/// delete; mid-campaign the numbered history already holds deletes from
/// levels 17/18/21, so those assertions are replaced with relative shift
/// checks that prove the same semantics.
fn solve_level_22(s: &mut GameSession) {
    enter(s, 22);
    // "a3yy overwrites whatever register 'a' held (level 10 left a MACRO
    // in it!) and "A3yy appends line B below line A.
    goto(s, 0, 2);
    fk(s, "\"a3yy");
    goto(s, 0, 5);
    fk(s, "\"A3yy");
    let banked = s
        .input
        .registers
        .get_blueprint(Some('a'))
        .expect("register a should hold the banked blueprint");
    assert_eq!(banked.height, 6, "\"A append stacks line B below line A");
    assert_eq!(banked.entities.len(), 62);

    // :reg popup opens and closes.
    fk(s, ":reg<CR>");
    assert!(s.app.popup.is_some());
    fk(s, "q");
    assert!(s.app.popup.is_none());

    // Numbered history: mid-campaign "1 already holds level 21's last
    // delete. d5l must SHIFT it into "2 and put the five walls in "1.
    let prev1_len = s
        .input
        .registers
        .get_blueprint(Some('1'))
        .map(|b| b.entities.len());
    assert!(
        prev1_len.is_some(),
        "chained run: \"1 should already hold a delete from earlier levels"
    );
    goto(s, 8, 11);
    fk(s, "d5l");
    for x in 8..=12 {
        assert_eq!(s.entity_type_at(x, 11), None, "junk wall ({x},11) deleted");
    }
    assert_eq!(
        s.input
            .registers
            .get_blueprint(Some('1'))
            .map(|b| b.entities.len()),
        Some(5),
        "\"1 holds the five just-deleted walls"
    );
    assert_eq!(
        s.input
            .registers
            .get_blueprint(Some('2'))
            .map(|b| b.entities.len()),
        prev1_len,
        "d5l shifted the previous \"1 (an earlier level's delete) into \"2"
    );

    // Black hole: "_d5l must not shift the history at all.
    let two_before = s
        .input
        .registers
        .get_blueprint(Some('2'))
        .map(|b| b.entities.len());
    goto(s, 20, 14);
    fk(s, "\"_d5l");
    for x in 20..=24 {
        assert_eq!(s.entity_type_at(x, 14), None, "junk wall ({x},14) deleted");
    }
    assert_eq!(
        s.input
            .registers
            .get_blueprint(Some('1'))
            .map(|b| b.entities.len()),
        Some(5),
        "\"_ delete must NOT clobber \"1"
    );
    assert_eq!(
        s.input
            .registers
            .get_blueprint(Some('2'))
            .map(|b| b.entities.len()),
        two_before,
        "\"_ delete must NOT shift the numbered history"
    );

    // Paste the banked blueprint at both double-sites; "0 still holds it.
    goto(s, 4, 10);
    fk(s, "\"ap");
    assert_eq!(s.entity_type_at(12, 10), Some(EntityType::Smelter));
    assert_eq!(s.entity_type_at(12, 13), Some(EntityType::Smelter));
    goto(s, 4, 17);
    fk(s, "\"0p");
    assert_eq!(s.entity_type_at(12, 17), Some(EntityType::Smelter));
    assert_eq!(s.entity_type_at(12, 20), Some(EntityType::Smelter));
    run_until_level(s, 23, 3000);
}

/// Level 23 "Jump History": search-jump the corners, Ctrl-o/Ctrl-i, ``.
/// The jumplist already holds dozens of entries from earlier levels; the
/// walks below are relative, so they behave exactly like the fresh run.
fn solve_level_23(s: &mut GameSession) {
    enter(s, 23);
    fk(s, "/ore<CR>");
    assert_eq!(s.cursor(), (2, 2));
    fk(s, "jJ");
    fk(s, "n");
    assert_eq!(s.cursor(), (58, 2));
    fk(s, "jJ");
    fk(s, "n");
    assert_eq!(s.cursor(), (2, 30));
    fk(s, "jJ");
    fk(s, "n");
    assert_eq!(s.cursor(), (58, 30));
    fk(s, "jJ");
    let here = s.cursor();

    fk(s, "<C-o>");
    assert_eq!(s.cursor(), (0, 0), "Ctrl-o returns to where /ore was typed");
    fk(s, "<C-i>");
    assert_eq!(s.cursor(), here, "Ctrl-i goes forward again");

    fk(s, "ma");
    fk(s, "gg");
    fk(s, "`a");
    assert_eq!(s.cursor(), here);
    fk(s, "``");
    assert_eq!(s.cursor(), (0, 0));
    run_until_level(s, 24, 3000);
}

/// Level 24 "Search & Destroy": / ? * # n N hunt six bricked-in walls.
fn solve_level_24(s: &mut GameSession) {
    enter(s, 24);
    fk(s, "/wall<CR>");
    assert_eq!(s.cursor(), (12, 3));
    fk(s, "*");
    assert_eq!(s.cursor(), (30, 7));
    fk(s, "#");
    assert_eq!(s.cursor(), (12, 3));
    fk(s, "rc");
    fk(s, "n");
    assert_eq!(s.cursor(), (35, 23), "n wraps backward to the last wall");
    fk(s, "rc");
    fk(s, "n");
    assert_eq!(s.cursor(), (18, 19));
    fk(s, "rc");
    fk(s, "2n");
    assert_eq!(s.cursor(), (8, 11), "2n skips one match");
    fk(s, "rc");
    fk(s, "N");
    assert_eq!(s.cursor(), (25, 15));
    fk(s, "rc");
    fk(s, "?wall<CR>");
    assert_eq!(s.cursor(), (30, 7), "?wall finds the last remaining wall");
    fk(s, "rc");
    fk(s, ":noh<CR>");
    assert!(!s.input.search.has_pattern());
    run_until_level(s, 25, 3000);
}

/// Level 25 "Global Substitute": :s//g, &, :N,Ms, :g/wall/d, J.
fn solve_level_25(s: &mut GameSession) {
    enter(s, 25);
    goto(s, 0, 3);
    fk(s, ":s/pipe/belt/g<CR>");
    assert_eq!(s.entity_type_at(4, 3), Some(EntityType::BasicBelt));
    goto(s, 0, 7);
    fk(s, "&");
    assert_eq!(s.entity_type_at(4, 7), Some(EntityType::BasicBelt));
    fk(s, ":12,16s/pipe/belt/g<CR>");
    assert_eq!(s.entity_type_at(20, 11), Some(EntityType::BasicBelt));
    assert_eq!(s.entity_type_at(20, 15), Some(EntityType::BasicBelt));
    fk(s, ":g/wall/d<CR>");
    assert_eq!(s.entity_type_at(4, 18), None);
    goto(s, 3, 18);
    fk(s, "J");
    assert_eq!(s.entity_type_at(4, 18), Some(EntityType::BasicBelt));
    assert_eq!(s.entity_type_at(35, 18), Some(EntityType::BasicBelt));
    run_until_level(s, 26, 3000);
}

/// Level 26 "Macro Empire": qa..q, @a, @@, 5@a stamp eight bands.
/// (Register 'a' held level 22's blueprint until qa overwrites it.)
fn solve_level_26(s: &mut GameSession) {
    enter(s, 26);
    goto(s, 4, 3);
    fk(s, "qa");
    fk(s, &band_macro_body());
    fk(s, "q");
    assert_eq!(s.entity_type_at(10, 2), Some(EntityType::Smelter));
    assert_eq!(s.cursor(), (4, 9));
    fk(s, "@a");
    assert_eq!(s.entity_type_at(10, 8), Some(EntityType::Smelter), "@a band 1");
    fk(s, "@@");
    assert_eq!(s.entity_type_at(10, 14), Some(EntityType::Smelter), "@@ band 2");
    fk(s, "5@a");
    for i in 3..8 {
        let y = 2 + i * 6;
        assert_eq!(s.entity_type_at(10, y), Some(EntityType::Smelter), "band {i}");
        assert_eq!(s.entity_type_at(40, y), Some(EntityType::Assembler), "band {i}");
    }
    run_until_level(s, 27, 3000);
}

/// Level 27 "Golf I": free 4yy + three pastes = 3 edits (budget 5).
fn solve_level_27(s: &mut GameSession) {
    enter(s, 27);
    assert_eq!(
        s.tutorial.as_ref().unwrap().edit_count,
        0,
        "edit budget must start fresh after auto-advance"
    );
    goto(s, 0, 3);
    fk(s, "4yy");
    assert_eq!(s.tutorial.as_ref().unwrap().edit_count, 0, "yanks are free");
    for &y in &[9usize, 15, 21] {
        goto(s, 4, y);
        fk(s, "p");
        assert_eq!(s.entity_type_at(8, y), Some(EntityType::Smelter));
    }
    assert_eq!(s.tutorial.as_ref().unwrap().edit_count, 3);
    run_until_level(s, 28, 3000);
}

/// Level 28 "Golf II": fix the template first (2 edits), then paste (3).
fn solve_level_28(s: &mut GameSession) {
    enter(s, 28);
    assert_eq!(s.entity_type_at(8, 3), None, "smelter is missing");
    goto(s, 8, 3);
    fk(s, "is<Esc>");
    goto(s, 30, 5);
    fk(s, "rc");
    assert_eq!(s.tutorial.as_ref().unwrap().edit_count, 2);
    goto(s, 0, 3);
    fk(s, "4yy");
    for &y in &[9usize, 15, 21] {
        goto(s, 4, y);
        fk(s, "p");
        assert_eq!(
            s.entity_type_at(30, y + 2),
            Some(EntityType::BasicBelt),
            "the pasted copy must not contain the wall defect"
        );
    }
    assert_eq!(s.tutorial.as_ref().unwrap().edit_count, 5, "budget is 7");
    run_until_level(s, 29, 3000);
}

/// Level 29 "The Broken Megafactory": the five-sabotage gauntlet.
fn solve_level_29(s: &mut GameSession) {
    enter(s, 29);
    goto(s, 10, 3);
    fk(s, "v5l~");
    for x in 10..=15 {
        assert_eq!(facing_at(s, x, 3), Some(Facing::Right));
    }
    goto(s, 4, 7);
    fk(s, "J");
    for x in 18..=23 {
        assert_eq!(s.entity_type_at(x, 7), Some(EntityType::BasicBelt));
    }
    goto(s, 20, 11);
    fk(s, "R");
    assert_eq!(s.mode(), Mode::Replace);
    fk(s, "cccccc<Esc>");
    for x in 20..=25 {
        assert_eq!(s.entity_type_at(x, 11), Some(EntityType::BasicBelt));
    }
    goto(s, 30, 14);
    fk(s, "cits<Esc>");
    assert_eq!(s.entity_type_at(30, 14), Some(EntityType::Smelter));
    fk(s, "/wall<CR>");
    assert_eq!(s.cursor(), (8, 19));
    fk(s, "rc");
    fk(s, "n");
    assert_eq!(s.cursor(), (22, 19));
    fk(s, "rc");
    fk(s, "n");
    assert_eq!(s.cursor(), (44, 19));
    fk(s, "rc");
    goto(s, 0, 3);
    fk(s, "gUU");
    assert_eq!(s.entity_type_at(4, 3), Some(EntityType::FastBelt));
    run_until_level(s, 30, 3000);
}

/// Level 30 "Final Exam": the whole curriculum, then freeplay unlocks.
fn solve_level_30(s: &mut GameSession) {
    enter(s, 30);
    assert!(!s.app.freeplay_unlocked, "freeplay locked until 30 completes");

    fk(s, "w");
    assert_eq!(s.cursor(), (1, 2));
    fk(s, "fc");
    assert_eq!(s.cursor(), (4, 3));
    fk(s, ";");
    assert_eq!(s.cursor(), (5, 3));
    fk(s, "%");
    assert_eq!(s.cursor(), (13, 3));
    fk(s, "J");
    for x in 14..=18 {
        assert_eq!(s.entity_type_at(x, 3), Some(EntityType::BasicBelt));
    }
    fk(s, ":s/pipe/belt/<CR>");
    assert_eq!(s.entity_type_at(28, 3), Some(EntityType::BasicBelt));
    fk(s, "<C-a>");
    assert_eq!(s.entity_type_at(14, 3), Some(EntityType::FastBelt));
    fk(s, "gUU");
    assert_eq!(s.entity_type_at(4, 3), Some(EntityType::FastBelt));

    fk(s, "/wall<CR>");
    assert_eq!(s.cursor(), (10, 14));
    fk(s, "*");
    assert_eq!(s.cursor(), (12, 14));
    fk(s, "<C-o>");
    assert_eq!(s.cursor(), (10, 14));
    fk(s, "mb");

    fk(s, "x");
    assert_eq!(s.entity_type_at(10, 14), None);
    fk(s, "ll.");
    assert_eq!(s.entity_type_at(12, 14), None);
    fk(s, "dd");
    assert_eq!(s.entity_type_at(30, 14), None);
    fk(s, "`b");
    assert_eq!(s.cursor(), (10, 14));

    fk(s, "cl<Esc>");
    goto(s, 4, 16);
    fk(s, "qaicc<Esc>q");
    fk(s, "@a");
    assert_eq!(s.entity_type_at(7, 16), Some(EntityType::BasicBelt));

    goto(s, 0, 3);
    fk(s, "yy");
    goto(s, 4, 9);
    fk(s, "p");
    assert_eq!(s.entity_type_at(4, 9), Some(EntityType::FastBelt));

    fk(s, "vly");
    fk(s, "<C-v>jy");

    // The finale: the next ticks complete level 30 and unlock freeplay.
    for _ in 0..100 {
        s.tick(1);
        if s.app.freeplay_unlocked {
            break;
        }
    }
    assert!(
        s.app.freeplay_unlocked,
        "CHAIN BREAK at level 30: completing it must unlock freeplay"
    );
    assert_eq!(
        s.current_level(),
        Some(31),
        "the campaign parks on the freeplay pseudo-level"
    );
}

// ===========================================================================
// Freeplay: prove the live economy works after the campaign.
// ===========================================================================

/// Enter freeplay via :freeplay and run an iron->ingot line for profit.
///
/// Uses the seeded iron deposit at (30,10) (src/levels/freeplay.rs) and the
/// central output bin at (60,40). Raw ore (tier 0, $1) barely covers the
/// per-tile land lease, so the line smelts to iron ingots (tier 1, ~$4):
///   deposit port (32,11) -> belts -> smelter (36,10) -> belts along row 11
///   -> down column 59 -> into the bin's input port (60,41).
fn play_freeplay(s: &mut GameSession) {
    fk(s, ":freeplay<CR>");
    assert_eq!(s.app.game_mode, GameMode::Freeplay, ":freeplay should load");
    assert_eq!(s.current_level(), None, "freeplay is not a campaign level");
    assert_eq!(s.app.map.width, 120, "freeplay map is 120x80");
    assert_eq!(s.app.map.height, 80);
    assert_eq!(s.mode(), Mode::Normal);

    let start_cash = s.app.economy.cash;
    assert_eq!(start_cash, 25_000, "freeplay starts at Normal difficulty");

    // Build the line (all through the vim grammar, like everything above).
    place_run(s, 33, 11, 'R', 'c', 3); // deposit output -> smelter input
    place_run(s, 36, 10, 'R', 's', 1); // smelter (36..38, 10..12)
    place_run(s, 39, 11, 'R', 'c', 20); // smelter output -> column 59
    place_run(s, 59, 11, 'D', 'c', 30); // down to the bin's row
    place_run(s, 59, 41, 'R', 'c', 1); // into the bin port (60,41)
    assert_eq!(s.entity_type_at(36, 10), Some(EntityType::Smelter));

    // Run the economy: deliveries auto-sell at market price, upkeep is
    // charged every 60-tick cycle.
    s.tick(1800);

    let ingots = s
        .app
        .delivered_lifetime
        .get(&Resource::IronIngot)
        .copied()
        .unwrap_or(0);
    assert!(
        ingots > 0,
        "freeplay line must deliver iron ingots (delivered_lifetime empty)"
    );
    assert!(
        s.app.economy.total_earned > 0,
        "deliveries must be credited as sales (total_earned=0)"
    );
    assert!(
        s.app
            .market
            .supply_pressure
            .get(&Resource::IronIngot)
            .copied()
            .unwrap_or(0.0)
            > 0.0,
        "the market must record the ingot sales"
    );
    // The economy WORKS: sales outpace land-lease/wage upkeep. (Kept
    // deliberately robust: strict cash growth plus the sale evidence above.)
    assert!(
        s.app.economy.cash > start_cash,
        "freeplay cash should grow: start={start_cash}, now={}, earned={}, \
         ingots={ingots}",
        s.app.economy.cash,
        s.app.economy.total_earned
    );
}

// ===========================================================================
// The test
// ===========================================================================

#[test]
fn full_campaign_menu_to_freeplay() {
    let t0 = std::time::Instant::now();
    KEYSTROKES.store(0, Ordering::Relaxed);

    // The true entry point: main menu, press '1'.
    let mut s = GameSession::new(160, 48);
    assert_eq!(s.mode(), Mode::Menu);
    fk(&mut s, "1");
    assert_eq!(s.current_level(), Some(1), "menu '1' must start level 1");

    solve_level_1(&mut s);
    solve_level_2(&mut s);
    solve_level_3(&mut s);
    solve_level_4(&mut s);
    solve_level_5(&mut s);
    solve_level_6(&mut s);
    solve_level_7(&mut s);
    solve_level_8(&mut s);
    solve_level_9(&mut s);
    solve_level_10(&mut s);
    solve_level_11(&mut s);
    solve_level_12(&mut s);
    solve_level_13(&mut s);
    solve_level_14(&mut s);
    solve_level_15(&mut s);
    solve_level_16(&mut s);
    solve_level_17(&mut s);
    solve_level_18(&mut s);
    solve_level_19(&mut s);
    solve_level_20(&mut s);
    solve_level_21(&mut s);
    solve_level_22(&mut s);
    solve_level_23(&mut s);
    solve_level_24(&mut s);
    solve_level_25(&mut s);
    solve_level_26(&mut s);
    solve_level_27(&mut s);
    solve_level_28(&mut s);
    solve_level_29(&mut s);
    solve_level_30(&mut s);

    // All 30 levels completed in one unbroken session.
    let tut = s.tutorial.as_ref().expect("tutorial state survives level 30");
    assert_eq!(
        tut.levels_completed.len(),
        30,
        "every campaign level must be recorded as completed"
    );

    play_freeplay(&mut s);

    println!(
        "full campaign: {} keystrokes, {:.2?} wall time, final cash ${}",
        KEYSTROKES.load(Ordering::Relaxed),
        t0.elapsed(),
        s.app.economy.cash
    );
}

/// Pin the interesting carryover the chained run exposed: `start_level`
/// (and therefore campaign auto-advance) resets the world, map, parser and
/// cursor, but vim state — registers, numbered delete history, marks —
/// survives level transitions exactly like it survives buffer switches in
/// vim. This is what forced level 22's chain adaptation above.
#[test]
fn campaign_state_carryover_is_sane() {
    let mut s = GameSession::new(160, 48);
    s.start_level(2);

    // Make some state in level 2: a mark, a yank, a delete.
    s.feed_keys("6j5li");
    s.feed_keys(&"c".repeat(3));
    s.feed_keys("<Esc>");
    s.feed_keys("<Esc>gg6j5l"); // back to the first belt
    s.feed_keys("mz"); // mark z at (5,6)
    s.feed_keys("\"zyy"); // yank the row into "z
    s.feed_keys("d3l"); // operator-delete the three belts -> "1
    let yank_len = s
        .input
        .registers
        .get_blueprint(Some('z'))
        .map(|b| b.entities.len());
    assert!(yank_len.is_some(), "\"z holds the yanked row");
    let del_len = s
        .input
        .registers
        .get_blueprint(Some('1'))
        .map(|b| b.entities.len());
    assert_eq!(del_len, Some(3), "\"1 holds the deleted belts");

    // Cross a level boundary the same way auto-advance does.
    s.start_level(3);
    assert_eq!(s.cursor(), (0, 0), "cursor resets on level load");
    assert_eq!(
        s.entity_type_at(5, 6),
        None,
        "the old level's belts are gone — worlds do not leak"
    );

    // ...but vim state persists, like registers across vim buffers.
    assert_eq!(
        s.input
            .registers
            .get_blueprint(Some('z'))
            .map(|b| b.entities.len()),
        yank_len,
        "named register survives the level transition"
    );
    assert_eq!(
        s.input
            .registers
            .get_blueprint(Some('1'))
            .map(|b| b.entities.len()),
        del_len,
        "numbered delete history survives the level transition"
    );
    assert_eq!(
        s.input.marks.get('z'),
        Some((5, 6)),
        "marks survive the level transition (stale coordinates and all)"
    );
}
