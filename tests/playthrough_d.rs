//! Full playthrough tests for campaign levels 21-30 (Acts V-VI, the
//! advanced half of the vim curriculum): drive the REAL game (GameSession —
//! the same code path main.rs uses) with literal keystrokes and verify each
//! level can be completed exactly the way its hints teach it.
//!
//! Level 30 is the campaign finale: its test asserts that completing it
//! unlocks freeplay.

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

/// Level 21 "Visual Mastery" (custom: all 3 bins producing) — counted
/// visual selects, visual r / x / ~, vab, gv, and Visual-Block I insert.
#[test]
fn level_21_visual_tools_repair_three_lines() {
    let mut s = session_at_level(21);

    // ---- Row 3: six reversed belts — v5l selects them, ~ flips 180.
    goto(&mut s, 10, 3);
    assert_eq!(facing_at(&s, 10, 3), Some(Facing::Left));
    s.feed_keys("v5l");
    assert_eq!(s.cursor(), (15, 3), "v5l should extend the selection by 5");
    s.feed_keys("~");
    assert_eq!(s.mode(), Mode::Normal);
    for x in 10..=15 {
        assert_eq!(facing_at(&s, x, 3), Some(Facing::Right), "belt ({x},3) flipped");
    }

    // ---- Row 11: four walls in the line — v3l then r c replaces them.
    goto(&mut s, 16, 11);
    assert_eq!(s.entity_type_at(16, 11), Some(EntityType::Wall));
    s.feed_keys("v3lrc");
    for x in 16..=19 {
        assert_eq!(s.entity_type_at(x, 11), Some(EntityType::BasicBelt));
        assert_eq!(facing_at(&s, x, 11), Some(Facing::Right));
    }

    // ---- Row 7: the fenced smelter site — vab selects fence + yard, x
    // demolishes, then is<Esc> drops the smelter.
    goto(&mut s, 25, 7);
    s.feed_keys("vabx");
    assert_eq!(s.entity_type_at(22, 5), None, "fence corner demolished");
    assert_eq!(s.entity_type_at(28, 9), None, "fence corner demolished");
    assert_eq!(s.entity_type_at(22, 7), None, "fence side demolished");
    goto(&mut s, 24, 6);
    s.feed_keys("is<Esc>");
    assert_eq!(s.entity_type_at(24, 6), Some(EntityType::Smelter));

    // ---- The wall canyon (column 20): Ctrl-v select, x, then gv + I c —
    // block insert lays one belt per row down the whole column.
    goto(&mut s, 20, 2);
    s.feed_keys("<C-v>");
    assert_eq!(s.mode(), Mode::VisualBlock);
    s.feed_keys("10j");
    assert_eq!(s.cursor(), (20, 12));
    s.feed_keys("x");
    assert_eq!(s.mode(), Mode::Normal);
    assert_eq!(s.entity_type_at(20, 3), None, "canyon demolished");
    s.feed_keys("gv");
    assert_eq!(s.mode(), Mode::VisualBlock, "gv reselects the block");
    s.feed_keys("Ic");
    for y in [3usize, 7, 11] {
        assert_eq!(s.entity_type_at(20, y), Some(EntityType::BasicBelt));
        assert_eq!(facing_at(&s, 20, y), Some(Facing::Right));
    }

    // ---- Row 7: belt the smelter in with two joins.
    goto(&mut s, 4, 7);
    s.feed_keys("J");
    assert_eq!(s.entity_type_at(22, 7), Some(EntityType::BasicBelt));
    assert_eq!(s.entity_type_at(23, 7), Some(EntityType::BasicBelt));
    s.feed_keys("J");
    assert_eq!(s.entity_type_at(27, 7), Some(EntityType::BasicBelt));
    assert_eq!(s.entity_type_at(28, 7), Some(EntityType::BasicBelt));

    run_until_level(&mut s, 22, 3000);
}

/// Level 22 "Register Bank" (custom: all 6 bins producing) — "a yank,
/// "A append, "1-"9 delete history, "_ black hole, "0 last yank, :reg.
#[test]
fn level_22_registers_bank_and_replay_blueprints() {
    let mut s = session_at_level(22);

    // ---- Bank line A into "a, then APPEND line B with "A.
    goto(&mut s, 0, 2);
    s.feed_keys("\"a3yy");
    goto(&mut s, 0, 5);
    s.feed_keys("\"A3yy");
    let banked = s
        .input
        .registers
        .get_blueprint(Some('a'))
        .expect("register a should hold the banked blueprint");
    assert_eq!(banked.height, 6, "\"A append stacks line B below line A");
    // 2 lines x (8 + 21 belts + 1 service belt + 1 smelter) = 62 entities.
    assert_eq!(banked.entities.len(), 62);

    // ---- :reg opens the register popup.
    s.feed_keys(":reg<CR>");
    assert!(s.app.popup.is_some(), ":reg should open the registers popup");
    s.feed_keys("q");
    assert!(s.app.popup.is_none(), "q should dismiss the popup");

    // ---- Junk row 11: d5l — the delete lands in "1 (numbered history).
    goto(&mut s, 8, 11);
    assert_eq!(s.entity_type_at(8, 11), Some(EntityType::Wall));
    s.feed_keys("d5l");
    for x in 8..=12 {
        assert_eq!(s.entity_type_at(x, 11), None, "junk wall ({x},11) deleted");
    }
    let del1 = s
        .input
        .registers
        .get_blueprint(Some('1'))
        .expect("\"1 should hold the last delete");
    assert_eq!(del1.entities.len(), 5, "five walls in the delete history");

    // ---- Junk row 14: "_d5l — the black hole leaves no trace.
    goto(&mut s, 20, 14);
    s.feed_keys("\"_d5l");
    for x in 20..=24 {
        assert_eq!(s.entity_type_at(x, 14), None, "junk wall ({x},14) deleted");
    }
    assert!(
        s.input.registers.get_blueprint(Some('2')).is_none(),
        "\"_ delete must NOT shift into the numbered history"
    );
    assert_eq!(
        s.input
            .registers
            .get_blueprint(Some('1'))
            .map(|b| b.entities.len()),
        Some(5),
        "\"1 still holds the previous delete"
    );

    // ---- Paste the two-line blueprint at both double-sites.
    goto(&mut s, 4, 10);
    s.feed_keys("\"ap");
    assert_eq!(s.entity_type_at(12, 10), Some(EntityType::Smelter));
    assert_eq!(s.entity_type_at(12, 13), Some(EntityType::Smelter));
    assert_eq!(s.entity_type_at(35, 11), Some(EntityType::BasicBelt));
    assert_eq!(s.entity_type_at(35, 14), Some(EntityType::BasicBelt));

    // "0 still holds the last yank even after the two deletes.
    goto(&mut s, 4, 17);
    s.feed_keys("\"0p");
    assert_eq!(s.entity_type_at(12, 17), Some(EntityType::Smelter));
    assert_eq!(s.entity_type_at(12, 20), Some(EntityType::Smelter));
    assert_eq!(s.entity_type_at(35, 18), Some(EntityType::BasicBelt));
    assert_eq!(s.entity_type_at(35, 21), Some(EntityType::BasicBelt));

    run_until_level(&mut s, 23, 3000);
}

/// Level 23 "Jump History" (custom: all 4 bins producing) — search-jump to
/// four corner clusters, fix each with J, then surf Ctrl-o / Ctrl-i / ``
/// and marks through the history.
#[test]
fn level_23_jumplist_across_four_corners() {
    let mut s = session_at_level(23);

    // ---- /ore teleports to the first deposit; fix each cluster with J.
    s.feed_keys("/ore<CR>");
    assert_eq!(s.cursor(), (2, 2), "/ore jumps to the first deposit");
    s.feed_keys("jJ");
    for x in 11..=13 {
        assert_eq!(s.entity_type_at(x, 3), Some(EntityType::BasicBelt));
    }

    s.feed_keys("n");
    assert_eq!(s.cursor(), (58, 2), "n jumps to the next deposit");
    s.feed_keys("jJ");
    for x in 67..=69 {
        assert_eq!(s.entity_type_at(x, 3), Some(EntityType::BasicBelt));
    }

    s.feed_keys("n");
    assert_eq!(s.cursor(), (2, 30));
    s.feed_keys("jJ");
    for x in 11..=13 {
        assert_eq!(s.entity_type_at(x, 31), Some(EntityType::BasicBelt));
    }

    s.feed_keys("n");
    assert_eq!(s.cursor(), (58, 30));
    s.feed_keys("jJ");
    for x in 67..=69 {
        assert_eq!(s.entity_type_at(x, 31), Some(EntityType::BasicBelt));
    }
    let here = s.cursor();

    // ---- The lesson: Ctrl-o walks back, Ctrl-i walks forward.
    s.feed_keys("<C-o>");
    assert_eq!(s.cursor(), (0, 0), "Ctrl-o returns to where /ore was typed");
    s.feed_keys("<C-i>");
    assert_eq!(s.cursor(), here, "Ctrl-i goes forward again");

    // ---- Marks review: ma, gg away, `a back, `` flips to the previous spot.
    s.feed_keys("ma");
    s.feed_keys("gg");
    assert_eq!(s.cursor(), (0, 0));
    s.feed_keys("`a");
    assert_eq!(s.cursor(), here, "`a jumps exactly to the mark");
    s.feed_keys("``");
    assert_eq!(s.cursor(), (0, 0), "`` flips back to the previous jump spot");

    run_until_level(&mut s, 24, 3000);
}

/// Level 24 "Search & Destroy" (custom: all 6 bins producing) — /, ?, *,
/// #, n, N and counted n hunt six walls bricked into six lines; r c fixes
/// each one; :noh clears the highlight.
#[test]
fn level_24_search_hunts_all_six_walls() {
    let mut s = session_at_level(24);

    // Match list (reading order): (12,3) (30,7) (8,11) (25,15) (18,19) (35,23).
    s.feed_keys("/wall<CR>");
    assert_eq!(s.cursor(), (12, 3), "/wall finds the first wall");

    // * re-searches for the entity under the cursor (forward)...
    s.feed_keys("*");
    assert_eq!(s.cursor(), (30, 7), "* jumps to the next wall");
    // ...and # does the same backward.
    s.feed_keys("#");
    assert_eq!(s.cursor(), (12, 3), "# jumps back to the previous wall");
    s.feed_keys("rc");
    assert_eq!(s.entity_type_at(12, 3), Some(EntityType::BasicBelt));

    // The search direction is backward now (from #): n keeps hunting that
    // way, wrapping vim-style.
    s.feed_keys("n");
    assert_eq!(s.cursor(), (35, 23), "n wraps to the last wall");
    s.feed_keys("rc");
    s.feed_keys("n");
    assert_eq!(s.cursor(), (18, 19));
    s.feed_keys("rc");

    // Counts work on n: 2n skips a match...
    s.feed_keys("2n");
    assert_eq!(s.cursor(), (8, 11), "2n skips one match");
    s.feed_keys("rc");
    // ...and N reverses to pick up the one we skipped.
    s.feed_keys("N");
    assert_eq!(s.cursor(), (25, 15), "N reverses the direction");
    s.feed_keys("rc");

    // ?wall re-scans backward — only one wall is left now.
    s.feed_keys("?wall<CR>");
    assert_eq!(s.cursor(), (30, 7), "?wall finds the last remaining wall");
    s.feed_keys("rc");

    // :noh clears the highlight.
    s.feed_keys(":noh<CR>");
    assert!(!s.input.search.has_pattern(), ":noh clears the search");

    run_until_level(&mut s, 25, 3000);
}

/// Level 25 "Global Substitute" (custom: all 5 bins producing) —
/// :s/pipe/belt/g, & repeat, :N,Ms row ranges, and :g/wall/d.
#[test]
fn level_25_substitute_and_global_delete() {
    let mut s = session_at_level(25);

    // ---- Row 3: substitute on the current row.
    goto(&mut s, 0, 3);
    assert_eq!(s.entity_type_at(4, 3), Some(EntityType::Pipe));
    s.feed_keys(":s/pipe/belt/g<CR>");
    assert_eq!(s.entity_type_at(4, 3), Some(EntityType::BasicBelt));
    assert_eq!(s.entity_type_at(35, 3), Some(EntityType::BasicBelt));

    // ---- Row 7: & repeats the last substitution on the current row.
    goto(&mut s, 0, 7);
    s.feed_keys("&");
    assert_eq!(s.entity_type_at(4, 7), Some(EntityType::BasicBelt));
    assert_eq!(s.entity_type_at(35, 7), Some(EntityType::BasicBelt));

    // ---- Rows 11+15 (1-indexed 12..16): the range form.
    s.feed_keys(":12,16s/pipe/belt/g<CR>");
    assert_eq!(s.entity_type_at(20, 11), Some(EntityType::BasicBelt));
    assert_eq!(s.entity_type_at(20, 15), Some(EntityType::BasicBelt));

    // ---- :g/wall/d wipes the wall field (line 5's burial + the debris).
    assert_eq!(s.entity_type_at(4, 18), Some(EntityType::Wall));
    s.feed_keys(":g/wall/d<CR>");
    assert_eq!(s.entity_type_at(4, 18), None, "buried line excavated");
    assert_eq!(s.entity_type_at(6, 20), None, "debris cleared");
    assert_eq!(s.entity_type_at(35, 18), None);

    // ---- One J belts the excavated line 5 end to end.
    goto(&mut s, 3, 18);
    s.feed_keys("J");
    assert_eq!(s.entity_type_at(4, 18), Some(EntityType::BasicBelt));
    assert_eq!(s.entity_type_at(35, 18), Some(EntityType::BasicBelt));

    run_until_level(&mut s, 26, 3000);
}

/// Level 26 "Macro Empire" (ProduceWidgets(24)) — record one full widget
/// line with qa...q, then replay it with @a, @@ and a counted 5@a.
#[test]
fn level_26_macro_recorded_once_stamped_seven_times() {
    let mut s = session_at_level(26);

    // The macro body (proven in level 10): builds ONE band with purely
    // relative motions and ends one band lower at the same column.
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

    // Band 0 was built by the recording itself.
    assert_eq!(s.entity_type_at(10, 2), Some(EntityType::Smelter));
    assert_eq!(s.entity_type_at(40, 2), Some(EntityType::Assembler));
    assert_eq!(facing_at(&s, 39, 3), Some(Facing::Down));
    assert_eq!(facing_at(&s, 47, 4), Some(Facing::Up));
    assert_eq!(s.cursor(), (4, 9), "macro must end one band lower");

    // @a replays once, @@ replays the same macro again, 5@a stamps the rest.
    s.feed_keys("@a");
    assert_eq!(s.entity_type_at(10, 8), Some(EntityType::Smelter), "@a builds band 1");
    s.feed_keys("@@");
    assert_eq!(s.entity_type_at(10, 14), Some(EntityType::Smelter), "@@ builds band 2");
    s.feed_keys("5@a");
    for i in 3..8 {
        let y = 2 + i * 6;
        assert_eq!(
            s.entity_type_at(10, y),
            Some(EntityType::Smelter),
            "band {i} smelter from 5@a"
        );
        assert_eq!(
            s.entity_type_at(40, y),
            Some(EntityType::Assembler),
            "band {i} assembler from 5@a"
        );
    }

    run_until_level(&mut s, 27, 3000);
}

/// Level 27 "Golf I" (ScoreInMoves(8 widgets, max 5 edits)).
///
/// Budget math: the intended solution is 4yy (yanks are free) + three
/// pastes = 3 edits, comfortably inside the budget of 5 (3 + ~2 slack).
/// Building one cell by hand costs 30 belts + 3 machines = 33 edits — a
/// single hand-built site already blows the budget six times over, so
/// yank/paste is the only way through.
#[test]
fn level_27_golf_paste_three_cells_in_three_edits() {
    let mut s = session_at_level(27);

    // The template cell is complete and working.
    assert_eq!(s.entity_type_at(8, 3), Some(EntityType::Smelter));
    assert_eq!(s.entity_type_at(13, 3), Some(EntityType::Splitter));
    assert_eq!(s.entity_type_at(20, 3), Some(EntityType::Assembler));

    // Yank the four template rows — costs nothing.
    goto(&mut s, 0, 3);
    s.feed_keys("4yy");
    assert_eq!(
        s.tutorial.as_ref().unwrap().edit_count,
        0,
        "yanking must be free"
    );

    // One paste per site: 3 edits total.
    for &y in &[9usize, 15, 21] {
        goto(&mut s, 4, y);
        s.feed_keys("p");
        assert_eq!(s.entity_type_at(8, y), Some(EntityType::Smelter));
        assert_eq!(s.entity_type_at(20, y), Some(EntityType::Assembler));
        assert_eq!(facing_at(&s, 16, y), Some(Facing::Down), "corner kept its facing");
        assert_eq!(facing_at(&s, 37, y + 2), Some(Facing::Up));
    }
    assert_eq!(
        s.tutorial.as_ref().unwrap().edit_count,
        3,
        "intended solution costs exactly 3 edits (budget 5; hand-building \
         one cell alone would cost 33)"
    );

    run_until_level(&mut s, 28, 3000);
}

/// Level 27 negative proof: the budget actually gates completion. A
/// player who spends even a few extra edits (here: 3 stray belts, total 6
/// > budget 5) can produce all the widgets they like — the level will NOT
/// complete. Tile-by-tile building (33 edits/cell) is therefore hopeless.
#[test]
fn level_27_budget_blocks_overspenders() {
    let mut s = session_at_level(27);

    goto(&mut s, 0, 3);
    s.feed_keys("4yy");
    for &y in &[9usize, 15, 21] {
        goto(&mut s, 4, y);
        s.feed_keys("p");
    }
    // Three careless extra placements on an unused row: 6 edits total.
    goto(&mut s, 2, 26);
    s.feed_keys("iccc<Esc>");
    assert_eq!(s.tutorial.as_ref().unwrap().edit_count, 6);

    // The factory works — widgets pour in — but the level must not budge.
    let mut widgets = 0;
    for _ in 0..3000 {
        s.tick(1);
        let (_, _, w, _) = s.output_totals();
        widgets = w;
        if widgets >= 10 {
            break;
        }
    }
    assert!(widgets >= 8, "the factory itself works (widgets={widgets})");
    assert_eq!(
        s.current_level(),
        Some(27),
        "over budget: completion must be blocked despite the widgets"
    );
}

/// Level 28 "Golf II" (ScoreInMoves(8 widgets, max 7 edits)).
///
/// Budget math: fix the template FIRST (smelter: is<Esc> = 1 edit, wall:
/// rc = 1 edit), then 4yy + three pastes (3 edits) = 5 edits total,
/// inside the budget of 7 (5 + ~2 slack). Pasting the broken template and
/// fixing every copy costs 3 pastes + 2 template fixes + 2x3 copy fixes =
/// 11 edits — over budget. The order of operations IS the lesson.
#[test]
fn level_28_golf_fix_template_before_copying() {
    let mut s = session_at_level(28);

    // Defects are present.
    assert_eq!(s.entity_type_at(8, 3), None, "smelter is missing");
    assert_eq!(s.entity_type_at(30, 5), Some(EntityType::Wall), "wall in the run");

    // Fix once: 2 edits.
    goto(&mut s, 8, 3);
    s.feed_keys("is<Esc>");
    assert_eq!(s.entity_type_at(8, 3), Some(EntityType::Smelter));
    goto(&mut s, 30, 5);
    s.feed_keys("rc");
    assert_eq!(s.entity_type_at(30, 5), Some(EntityType::BasicBelt));
    assert_eq!(s.tutorial.as_ref().unwrap().edit_count, 2);

    // Then it's Golf I again: free yank + 3 pastes.
    goto(&mut s, 0, 3);
    s.feed_keys("4yy");
    for &y in &[9usize, 15, 21] {
        goto(&mut s, 4, y);
        s.feed_keys("p");
        assert_eq!(s.entity_type_at(8, y), Some(EntityType::Smelter));
        assert_eq!(
            s.entity_type_at(30, y + 2),
            Some(EntityType::BasicBelt),
            "the pasted copy must NOT contain the wall defect"
        );
    }
    assert_eq!(
        s.tutorial.as_ref().unwrap().edit_count,
        5,
        "fix-once-then-paste costs 5 edits (budget 7; paste-then-fix would \
         cost 11 and bust)"
    );

    run_until_level(&mut s, 29, 3000);
}

/// Level 29 "The Broken Megafactory" (custom: all 5 bins producing) — the
/// gauntlet: visual ~, J, R replace mode, cit text object, search + r c.
#[test]
fn level_29_gauntlet_fixes_five_kinds_of_sabotage() {
    let mut s = session_at_level(29);

    // ---- Line 1 (row 3): reversed belts — v5l ~.
    goto(&mut s, 10, 3);
    assert_eq!(facing_at(&s, 10, 3), Some(Facing::Left));
    s.feed_keys("v5l~");
    for x in 10..=15 {
        assert_eq!(facing_at(&s, x, 3), Some(Facing::Right));
    }

    // ---- Line 2 (row 7): 6-tile gap — one J bridges it.
    goto(&mut s, 4, 7);
    s.feed_keys("J");
    for x in 18..=23 {
        assert_eq!(s.entity_type_at(x, 7), Some(EntityType::BasicBelt));
    }

    // ---- Line 3 (row 11): pipes — R replace mode paves over them.
    goto(&mut s, 20, 11);
    assert_eq!(s.entity_type_at(20, 11), Some(EntityType::Pipe));
    s.feed_keys("R");
    assert_eq!(s.mode(), Mode::Replace, "R enters replace mode");
    s.feed_keys("cccccc<Esc>");
    for x in 20..=25 {
        assert_eq!(s.entity_type_at(x, 11), Some(EntityType::BasicBelt));
        assert_eq!(facing_at(&s, x, 11), Some(Facing::Right));
    }

    // ---- Line 4 (row 15): a kiln squats in the smelter slot — cit + s.
    goto(&mut s, 30, 14);
    assert_eq!(s.entity_type_at(30, 14), Some(EntityType::Kiln));
    s.feed_keys("cits<Esc>");
    assert_eq!(s.entity_type_at(30, 14), Some(EntityType::Smelter));

    // ---- Line 5 (row 19): three hidden walls — search and replace.
    s.feed_keys("/wall<CR>");
    assert_eq!(s.cursor(), (8, 19));
    s.feed_keys("rc");
    s.feed_keys("n");
    assert_eq!(s.cursor(), (22, 19));
    s.feed_keys("rc");
    s.feed_keys("n");
    assert_eq!(s.cursor(), (44, 19));
    s.feed_keys("rc");
    for &x in &[8usize, 22, 44] {
        assert_eq!(s.entity_type_at(x, 19), Some(EntityType::BasicBelt));
    }

    // ---- Style points: gUU upgrades row 3's belts one tier.
    goto(&mut s, 0, 3);
    s.feed_keys("gUU");
    assert_eq!(s.entity_type_at(4, 3), Some(EntityType::FastBelt));
    assert_eq!(
        s.entity_type_at(30, 2),
        Some(EntityType::Smelter),
        "gUU must not touch machines"
    );

    run_until_level(&mut s, 30, 3000);
}

/// Level 30 "Final Exam" (UseCommands over the whole curriculum) — every
/// command on the checklist, exercised naturally while finishing the last
/// factory. Completing it unlocks FREEPLAY.
#[test]
fn level_30_final_exam_unlocks_freeplay() {
    let mut s = session_at_level(30);
    assert!(!s.app.freeplay_unlocked);

    // ---- Motions: w to the first entity, fc/; onto the belts, % rides
    // the chain to the break.
    s.feed_keys("w");
    assert_eq!(s.cursor(), (1, 2), "w jumps to the deposit");
    s.feed_keys("fc");
    assert_eq!(s.cursor(), (4, 3), "fc finds the first belt");
    s.feed_keys(";");
    assert_eq!(s.cursor(), (5, 3), "; repeats the find");
    s.feed_keys("%");
    assert_eq!(s.cursor(), (13, 3), "% rides the chain to the break");

    // ---- J bridges the gap; :s melts the pipe; ctrl-a + gUU upgrade.
    s.feed_keys("J");
    for x in 14..=18 {
        assert_eq!(s.entity_type_at(x, 3), Some(EntityType::BasicBelt));
    }
    assert_eq!(s.entity_type_at(28, 3), Some(EntityType::Pipe));
    s.feed_keys(":s/pipe/belt/<CR>");
    assert_eq!(s.entity_type_at(28, 3), Some(EntityType::BasicBelt));
    s.feed_keys("<C-a>");
    assert_eq!(s.entity_type_at(14, 3), Some(EntityType::FastBelt), "ctrl-a tiers up");
    s.feed_keys("gUU");
    assert_eq!(s.entity_type_at(4, 3), Some(EntityType::FastBelt), "gUU tiers the row");

    // ---- Search block: /wall, * to the next, ctrl-o back, mark it.
    s.feed_keys("/wall<CR>");
    assert_eq!(s.cursor(), (10, 14));
    s.feed_keys("*");
    assert_eq!(s.cursor(), (12, 14));
    s.feed_keys("<C-o>");
    assert_eq!(s.cursor(), (10, 14), "ctrl-o back through the jumplist");
    s.feed_keys("mb");

    // ---- x one wall, dot the next, dd the row, backtick home.
    s.feed_keys("x");
    assert_eq!(s.entity_type_at(10, 14), None);
    s.feed_keys("ll.");
    assert_eq!(s.entity_type_at(12, 14), None, ". repeats the x");
    s.feed_keys("dd");
    assert_eq!(s.entity_type_at(14, 14), None);
    assert_eq!(s.entity_type_at(30, 14), None);
    s.feed_keys("`b");
    assert_eq!(s.cursor(), (10, 14), "`b returns to the mark");

    // ---- cl changes a (junk) tile; a tiny macro places belts.
    // NOTE: `cc` cannot be parsed (the parser resets on a doubled c), so
    // the change operator is exercised through operator+motion instead.
    s.feed_keys("cl<Esc>");
    goto(&mut s, 4, 16);
    s.feed_keys("qaicc<Esc>q");
    assert_eq!(s.entity_type_at(4, 16), Some(EntityType::BasicBelt));
    assert_eq!(s.entity_type_at(5, 16), Some(EntityType::BasicBelt));
    s.feed_keys("@a");
    assert_eq!(s.entity_type_at(6, 16), Some(EntityType::BasicBelt), "@a replays");
    assert_eq!(s.entity_type_at(7, 16), Some(EntityType::BasicBelt));

    // ---- yy the finished line, p it onto the second band.
    goto(&mut s, 0, 3);
    s.feed_keys("yy");
    goto(&mut s, 4, 9);
    s.feed_keys("p");
    assert_eq!(s.entity_type_at(4, 9), Some(EntityType::FastBelt));
    assert_eq!(s.entity_type_at(43, 9), Some(EntityType::FastBelt));

    // ---- Visual coda: v and ctrl-v yanks.
    s.feed_keys("vly");
    s.feed_keys("<C-v>jy");

    // Every checklist command must be recorded.
    {
        let tut = s.tutorial.as_ref().unwrap();
        for cmd in [
            "w", "f", ";", "%", "J", ":s", "ctrl-a", "gU", "/", "*", "ctrl-o",
            "m", ".", "d", "`", "c", "q", "@", "y", "p", "v", "ctrl-v",
        ] {
            assert!(
                tut.commands_used.contains(cmd),
                "checklist command {cmd:?} was not recorded"
            );
        }
    }

    // The next tick completes the final level and unlocks freeplay.
    s.tick(1);
    assert!(
        s.app.freeplay_unlocked,
        "completing level 30 must unlock freeplay"
    );
    assert_eq!(
        s.current_level(),
        Some(31),
        "the campaign parks on the freeplay pseudo-level"
    );
}
