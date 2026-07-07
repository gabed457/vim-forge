//! Full playthrough tests for campaign levels 8-13: drive the REAL game
//! (GameSession — the same code path main.rs uses) with literal keystrokes
//! and verify each level can be completed the way a player would complete it,
//! following the intended teaching of each level's hints.
//!
//! ## Game bugs found and fixed while writing these tests
//!
//! 1. **`/` search, `n`, `N` and `*` never worked** (src/vim/parser.rs,
//!    src/input/handler.rs, src/commands.rs). Typing `/smelter<CR>` only
//!    returned `ExitToNormal`; nothing ever resolved the typed pattern,
//!    populated `SearchState::matches`, or moved the cursor, so `n`/`N`
//!    (which walk the `matches` list) were permanent no-ops and `*` set a
//!    pattern with an empty match list. Level 9's whole lesson is
//!    "/search then n". Fix: a new `Command::ExecuteSearch(pattern, forward)`
//!    is emitted when the search line is submitted; the input handler
//!    resolves it via `EntityType::from_search_prefix`, collects all anchor
//!    tiles of that entity type in row-major order, and jumps to the first
//!    match after the cursor (wrapping), exactly like vim. `*` now populates
//!    the same match list and jumps too.
//!
//! 2. **Finishing level 13 restarted level 13 forever instead of unlocking
//!    freeplay** (src/tutorial/engine.rs `complete_level`). For the final
//!    level, `next = 14 > total_levels() = 13`, so `current_level` was left
//!    at 13; `GameSession::post_tick` then saw a "completed" level 13,
//!    concluded it was a normal level, and called `start_level(13)` again —
//!    the campaign could never be finished. Fix: `complete_level` now parks
//!    `current_level` on `FREEPLAY_LEVEL` (14) when the campaign is done,
//!    which is exactly the state `post_tick` interprets as "unlock freeplay".
//!
//! 3. **Level 12 was impossible as designed** (src/levels/level_12.rs). Its
//!    completion was `ProduceWidgets(4)`, but widgets (CircuitBoards) require
//!    an Assembler with TWO ingot input streams, while the level's four
//!    clusters each have a single ore line and the hints teach exactly
//!    "belts + a smelter" per cluster ("Each cluster needs more belts and a
//!    smelter"), i.e. ingot production. Fix: completion changed to
//!    `DeliverIngots(4)` (and the hint text now says ingots), matching the
//!    level's own instructions.
//!
//! These tests also depend on the belt-corner fix documented in
//! tests/playthrough_a.rs (conveyors accept input from behind and both
//! sides, so L-shaped belt paths work); level 8's pre-built cluster and the
//! routing in levels 10 and 13 are impossible without it.
//!
//! ## Design warts noticed but deliberately NOT changed
//!
//! - `%` (follow connection) steps back using the direction the walk
//!   STARTED with instead of the last direction walked, so following a chain
//!   that bends lands one tile off diagonally (see level 9, cluster 4).
//! - Operator+motion combos (`dw`, `d3l`, `>l`, ...) execute the motion but
//!   apply the operator to an empty range (a no-op); only `dd`/`yy`/`cc`,
//!   visual-mode operators and text objects actually edit. No level's
//!   intended solution needs operator+motion, so it is left as-is.
//! - Counts are ignored in visual mode (`25l` moves one tile), so block
//!   selections must be extended one keypress at a time.

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

fn facing_at(s: &GameSession, x: usize, y: usize) -> Option<Facing> {
    s.app.map.entity_facing_at(&s.app.world, x, y)
}

/// Level 8 "Block Select" (ProduceWidgets(6)) — MUST use Ctrl-v Visual Block.
///
/// The top half is a complete working cluster (two ore lines -> two smelters
/// -> one assembler -> bin); the bottom half has only ore deposits at (3,17)
/// and (3,21) and a bin at (32,19) — the SAME relative layout, 14 rows lower.
/// Select the machine/belt block (6,3)-(31,9) with Ctrl-v, yank it, and
/// paste it at (6,17). Ore deposits and bins are not player-placeable so the
/// yank skips them; the pasted machines line up with the bottom deposits/bin
/// exactly.
#[test]
fn level_8_visual_block_copy_clones_working_cluster() {
    let mut s = session_at_level(8);

    // Sanity: the working cluster's machines are where the layout says.
    assert_eq!(s.entity_type_at(13, 3), Some(EntityType::Smelter));
    assert_eq!(s.entity_type_at(13, 7), Some(EntityType::Smelter));
    assert_eq!(s.entity_type_at(22, 4), Some(EntityType::Assembler));

    // Visual-Block select (6,3)..(31,9). Counts are ignored in visual mode,
    // so extend the selection one keypress at a time like a real player.
    goto(&mut s, 6, 3);
    s.feed_keys("<C-v>");
    assert_eq!(s.mode(), Mode::VisualBlock);
    s.feed_keys(&"l".repeat(25));
    s.feed_keys(&"j".repeat(6));
    assert_eq!(s.cursor(), (31, 9));
    s.feed_keys("y");
    assert_eq!(s.mode(), Mode::Normal);

    // Paste 14 rows lower. Blueprint offsets are relative to the yanked
    // block's min anchor (6,3), so pasting at (6,17) shifts everything +14.
    goto(&mut s, 6, 17);
    s.feed_keys("p");

    // The clone lines up with the bottom deposits and bin.
    assert_eq!(s.entity_type_at(13, 17), Some(EntityType::Smelter));
    assert_eq!(s.entity_type_at(13, 21), Some(EntityType::Smelter));
    assert_eq!(s.entity_type_at(22, 18), Some(EntityType::Assembler));
    assert_eq!(s.entity_type_at(6, 18), Some(EntityType::BasicBelt));
    assert_eq!(s.entity_type_at(31, 20), Some(EntityType::BasicBelt));
    // The corner belts kept their facings.
    assert_eq!(facing_at(&s, 20, 18), Some(Facing::Down));
    assert_eq!(facing_at(&s, 20, 22), Some(Facing::Up));

    run_until_level(&mut s, 9, 3000);
}

/// Level 9 "Find & Jump" (custom: every output bin receives >= 1 item) —
/// MUST navigate with f, /, n and %.
///
/// Five broken clusters: belt gap at (9,4); smelter at (49,5) facing Left;
/// belt gap at (28..29,15); belt at (11,23) facing Up; missing smelter at
/// (50,25).
#[test]
fn level_9_find_search_percent_fix_all_five_clusters() {
    let mut s = session_at_level(9);

    // ---- Cluster 1: `fc` jumps to the first belt, `%` walks to the break.
    s.feed_keys("fc");
    assert_eq!(s.cursor(), (5, 4), "fc should land on the first belt");
    s.feed_keys("%");
    assert_eq!(s.cursor(), (8, 4), "% should stop where the chain breaks");
    s.feed_keys("lic<Esc>"); // fill the gap at (9,4)
    assert_eq!(s.entity_type_at(9, 4), Some(EntityType::BasicBelt));

    // ---- Cluster 2: search for the wrong-facing smelter and rotate it.
    s.feed_keys("/smelter<CR>");
    assert_eq!(s.cursor(), (49, 5), "/smelter should jump to next smelter");
    s.feed_keys("~~"); // Left -> Up -> Right
    assert_eq!(facing_at(&s, 49, 5), Some(Facing::Right));

    // ---- Cluster 3: search for the next ore deposit, follow the belts.
    s.feed_keys("/ore<CR>");
    assert_eq!(s.cursor(), (22, 14), "/ore should jump to cluster 3 deposit");
    s.feed_keys("j3l"); // onto the first belt at (25,15)
    assert_eq!(s.cursor(), (25, 15));
    s.feed_keys("%");
    assert_eq!(s.cursor(), (27, 15), "% should stop at the 2-tile gap");
    s.feed_keys("licc<Esc>"); // fill (28,15) and (29,15)
    assert_eq!(s.entity_type_at(28, 15), Some(EntityType::BasicBelt));
    assert_eq!(s.entity_type_at(29, 15), Some(EntityType::BasicBelt));

    // ---- Cluster 4: `n` = next /ore match; % reveals the bad belt.
    s.feed_keys("n");
    assert_eq!(s.cursor(), (5, 22), "n should jump to cluster 4 deposit");
    s.feed_keys("j3l"); // onto the first belt at (8,23)
    s.feed_keys("%");
    // The chain turns Up at the wrong-facing belt (11,23); % steps back with
    // the walk's ORIGINAL direction (a known wart), landing at (10,22).
    assert_eq!(s.cursor(), (10, 22));
    s.feed_keys("jl~"); // step onto (11,23) and rotate Up -> Right
    assert_eq!(facing_at(&s, 11, 23), Some(Facing::Right));

    // ---- Cluster 5: `n` again, then place the missing smelter.
    s.feed_keys("n");
    assert_eq!(s.cursor(), (42, 25), "n should jump to cluster 5 deposit");
    s.feed_keys("j3l"); // onto the first belt at (45,26)
    s.feed_keys("%");
    assert_eq!(s.cursor(), (49, 26), "% should stop at the smelter-sized gap");
    s.feed_keys("lk"); // smelter anchor goes at (50,25)
    s.feed_keys("is<Esc>");
    assert_eq!(s.entity_type_at(50, 25), Some(EntityType::Smelter));

    run_until_level(&mut s, 10, 3000);
}

/// Level 10 "Macro Factory" (ProduceWidgets(15)) — MUST record with qa...q
/// and replay with 4@a.
///
/// Five identical rows (ore deposit at (1, 2+6i), bin at (48, 2+6i)). The
/// macro builds one complete widget line with purely relative motions and
/// ends 6 rows below its own starting tile, so 4@a stamps out the other four
/// rows. Line layout for band y (belt row r=y+1):
///   belts (4..9,r) -> smelter (10,y) -> belts (13..38,r) -> corner belt
///   (39,r) rotated Down -> belt (39,r+1) feeds assembler input B at
///   (40,r+1) while input A is pulled from (39,r); assembler (40,y) ->
///   belts (43..46,r+1) -> corner (47,r+1) rotated Up -> belt (47,r) ->
///   bin input port (48,r).
#[test]
fn level_10_macro_recorded_once_replayed_four_times() {
    let mut s = session_at_level(10);

    // The macro body: everything a player does for ONE row, ending at the
    // next row's starting tile. No absolute jumps, no facing keys (all
    // placements happen facing Right; corners are made with `~` afterwards).
    let macro_body = format!(
        "i{belts1}ksj{belts2}kajj{belts3}<Esc>h~~~kic<Esc>9h~jic<Esc>36h5j",
        belts1 = "c".repeat(6),  // (4..9, r)
        belts2 = "c".repeat(27), // (13..39, r)
        belts3 = "c".repeat(5),  // (43..47, r+1)
    );

    goto(&mut s, 4, 3);
    s.feed_keys("qa");
    assert_eq!(s.input.parser.recording_macro, Some('a'));
    s.feed_keys(&macro_body);
    s.feed_keys("q");
    assert_eq!(s.input.parser.recording_macro, None);

    // Row 0 (band y=2, belt row 3) was fully built by the recording itself.
    assert_eq!(s.entity_type_at(4, 3), Some(EntityType::BasicBelt));
    assert_eq!(s.entity_type_at(10, 2), Some(EntityType::Smelter));
    assert_eq!(s.entity_type_at(40, 2), Some(EntityType::Assembler));
    assert_eq!(facing_at(&s, 39, 3), Some(Facing::Down), "corner into input B");
    assert_eq!(facing_at(&s, 39, 4), Some(Facing::Right));
    assert_eq!(facing_at(&s, 47, 4), Some(Facing::Up), "corner up to the bin");
    assert_eq!(facing_at(&s, 47, 3), Some(Facing::Right));
    // The macro must end at the next row's starting tile for @a to chain.
    assert_eq!(s.cursor(), (4, 9), "macro must end one band lower");

    // Replay on the remaining four rows.
    s.feed_keys("4@a");
    for i in 1..5 {
        let y = 2 + i * 6;
        assert_eq!(
            s.entity_type_at(10, y),
            Some(EntityType::Smelter),
            "row {i} smelter from macro replay"
        );
        assert_eq!(
            s.entity_type_at(40, y),
            Some(EntityType::Assembler),
            "row {i} assembler from macro replay"
        );
        assert_eq!(facing_at(&s, 39, y + 1), Some(Facing::Down));
        assert_eq!(facing_at(&s, 47, y + 2), Some(Facing::Up));
    }

    run_until_level(&mut s, 11, 3000);
}

/// Level 11 "The Dot" (custom: all 22 belts face Right) — MUST use ~ once
/// and then dot-repeat with `l.` across the row.
#[test]
fn level_11_tilde_then_dot_repeat_fixes_all_belts() {
    let mut s = session_at_level(11);

    // All 22 belts at (6..27, 10) start facing Up.
    assert_eq!(facing_at(&s, 6, 10), Some(Facing::Up));
    assert_eq!(facing_at(&s, 27, 10), Some(Facing::Up));

    // Rotate the first belt with ~ (Up -> Right)...
    goto(&mut s, 6, 10);
    s.feed_keys("~");
    assert_eq!(facing_at(&s, 6, 10), Some(Facing::Right));

    // ...then `l .` down the row: dot repeats the last edit (the rotate).
    for _ in 0..21 {
        s.feed_keys("l.");
    }
    assert_eq!(s.cursor(), (27, 10));
    for x in 6..=27 {
        assert_eq!(
            facing_at(&s, x, 10),
            Some(Facing::Right),
            "belt at ({x},10) should have been rotated by dot-repeat"
        );
    }

    // Completion is evaluated on the next simulation tick.
    s.tick(1);
    assert_eq!(s.current_level(), Some(12), "level 11 should auto-advance");
}

/// Level 12 "Marks & Navigation" (DeliverIngots(4) — see bug #3 in the file
/// header) — MUST set marks at all four corner clusters and jump with `x.
///
/// Each corner cluster has an ore line and a bin but no smelter and a belt
/// gap. Per cluster: set a mark, drop a smelter on the line, belt the output
/// to the bin. Then hop between all four marks to prove they work.
#[test]
fn level_12_marks_navigate_four_corner_clusters() {
    let mut s = session_at_level(12);

    // ---- Cluster 1 (top-left): smelter (11,3), belts (14..20, 4).
    goto(&mut s, 11, 3);
    s.feed_keys("ma");
    s.feed_keys("isj"); // smelter at (11,3), cursor drops to belt row 4
    s.feed_keys(&"c".repeat(7));
    s.feed_keys("<Esc>");
    assert_eq!(s.entity_type_at(11, 3), Some(EntityType::Smelter));
    assert_eq!(s.entity_type_at(20, 4), Some(EntityType::BasicBelt));

    // ---- Cluster 2 (top-right): smelter (70,3), belts (73..76, 4).
    goto(&mut s, 70, 3);
    s.feed_keys("mb");
    s.feed_keys("isj");
    s.feed_keys(&"c".repeat(4));
    s.feed_keys("<Esc>");
    assert_eq!(s.entity_type_at(70, 3), Some(EntityType::Smelter));
    assert_eq!(s.entity_type_at(76, 4), Some(EntityType::BasicBelt));

    // ---- Cluster 3 (bottom-left): smelter (10,35), belts (13..20, 36).
    goto(&mut s, 10, 35);
    s.feed_keys("mc");
    s.feed_keys("isj");
    s.feed_keys(&"c".repeat(8));
    s.feed_keys("<Esc>");
    assert_eq!(s.entity_type_at(10, 35), Some(EntityType::Smelter));
    assert_eq!(s.entity_type_at(20, 36), Some(EntityType::BasicBelt));

    // ---- Cluster 4 (bottom-right): smelter (69,35), belts (72..76, 36).
    goto(&mut s, 69, 35);
    s.feed_keys("md");
    s.feed_keys("isj");
    s.feed_keys(&"c".repeat(5));
    s.feed_keys("<Esc>");
    assert_eq!(s.entity_type_at(69, 35), Some(EntityType::Smelter));
    assert_eq!(s.entity_type_at(76, 36), Some(EntityType::BasicBelt));

    // ---- The lesson: hop between all four clusters via exact marks.
    s.feed_keys("`a");
    assert_eq!(s.cursor(), (11, 3), "`a should jump to cluster 1 mark");
    s.feed_keys("`b");
    assert_eq!(s.cursor(), (70, 3), "`b should jump to cluster 2 mark");
    s.feed_keys("`c");
    assert_eq!(s.cursor(), (10, 35), "`c should jump to cluster 3 mark");
    s.feed_keys("`d");
    assert_eq!(s.cursor(), (69, 35), "`d should jump to cluster 4 mark");
    // 'a (row-jump form) lands on the first entity of the marked row.
    s.feed_keys("'a");
    assert_eq!(s.cursor(), (2, 3), "'a should jump to first entity in row 3");

    run_until_level(&mut s, 13, 3000);
}

/// Level 13 "Split View" (ProduceWidgets(5)). Splits are visual-only stubs;
/// exercise every Ctrl-w key to prove they do not crash, then build the
/// cross-map route: both prebuilt ingot lines (rows 3 and 7, ending at x=20)
/// feed an assembler at (24,2); its output runs along row 4 to x=75, turns
/// down column 76, and enters the bin's input port at (77,38).
/// Completing this final level must unlock freeplay (bug #2 in the header).
#[test]
fn level_13_split_keys_and_cross_map_route_unlock_freeplay() {
    let mut s = session_at_level(13);

    // Exercise the split-window keys (visual-only stubs must not crash).
    s.feed_keys("<C-w>v<C-w>l<C-w>h<C-w>j<C-w>k<C-w>s<C-w>=<C-w>o<C-w>q");
    assert_eq!(s.mode(), Mode::Normal);
    assert_eq!(s.cursor(), (0, 0), "ctrl-w commands should not move cursor");

    // ---- Feed input A (row 3): extend the top ingot line to the assembler.
    place_run(&mut s, 21, 3, 'R', 'c', 3); // belts (21..23, 3)
    // Assembler anchor (24,2): occupies (24..26, 2..5), inputs (24,3) and
    // (24,4), output (26,4).
    place_run(&mut s, 24, 2, 'R', 'a', 1);

    // ---- Feed input B (row 7): extend, then climb to row 4.
    place_run(&mut s, 21, 7, 'R', 'c', 2); // belts (21..22, 7)
    place_run(&mut s, 23, 7, 'U', 'c', 3); // corner + climb: (23,7)(23,6)(23,5) facing Up
    place_run(&mut s, 23, 4, 'R', 'c', 1); // belt (23,4) -> input port (24,4)

    // ---- Output: row 4 across the map, down column 76, into the bin.
    place_run(&mut s, 27, 4, 'R', 'c', 49); // belts (27..75, 4)
    place_run(&mut s, 76, 4, 'D', 'c', 34); // corner + belts (76, 4..37) facing Down
    place_run(&mut s, 76, 38, 'R', 'c', 1); // belt (76,38) -> bin port (77,38)

    assert_eq!(s.entity_type_at(24, 2), Some(EntityType::Assembler));
    assert_eq!(facing_at(&s, 23, 7), Some(Facing::Up));
    assert_eq!(facing_at(&s, 23, 4), Some(Facing::Right));
    assert_eq!(facing_at(&s, 76, 4), Some(Facing::Down));
    assert_eq!(facing_at(&s, 76, 38), Some(Facing::Right));

    // Completing the final campaign level unlocks freeplay instead of
    // auto-advancing to another level.
    assert!(!s.app.freeplay_unlocked);
    for _ in 0..3000 {
        s.tick(1);
        if s.app.freeplay_unlocked {
            break;
        }
    }
    let (_, _, widgets, _) = s.output_totals();
    assert!(
        s.app.freeplay_unlocked,
        "level 13 should unlock freeplay; widgets delivered = {widgets}"
    );
}
