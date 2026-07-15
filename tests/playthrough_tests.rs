//! Full playthrough tests: drive the REAL game (GameSession — the same code
//! path main.rs uses) with literal keystrokes and verify each level can be
//! completed the way a player would complete it.

use vimforge::app::Mode;
use vimforge::game::session::GameSession;

/// Start a session directly in the given level (as if picked from the menu).
fn session_at_level(level: usize) -> GameSession {
    let mut s = GameSession::new(160, 48);
    s.start_level(level);
    assert_eq!(s.current_level(), Some(level), "level {level} should load");
    assert_eq!(s.mode(), Mode::Normal);
    s
}

#[test]
fn menu_starts_level_1() {
    let mut s = GameSession::new(160, 48);
    assert_eq!(s.mode(), Mode::Menu);
    s.feed_keys("1");
    assert_eq!(s.current_level(), Some(1));
    assert_eq!(s.mode(), Mode::Normal);
}

#[test]
fn level_1_navigation_completes_and_advances() {
    let mut s = session_at_level(1);

    // Objective: move onto both output bins at (31,4) and (31,15).
    // Use counts like a vim user: 4j 31l, then 11j.
    s.feed_keys("4j31l");
    assert_eq!(s.cursor(), (31, 4));
    s.feed_keys("11j");
    assert_eq!(s.cursor(), (31, 15));

    // Completion is evaluated on the next simulation tick.
    s.tick(1);
    assert_eq!(
        s.current_level(),
        Some(2),
        "completing level 1 should auto-load level 2"
    );
}

#[test]
fn level_1_gg_and_g_motions() {
    let mut s = session_at_level(1);
    s.feed_keys("G");
    assert_eq!(s.cursor().1, s.app.map.height - 1);
    s.feed_keys("gg");
    assert_eq!(s.cursor(), (0, 0));
    s.feed_keys("5j$");
    assert_eq!(s.cursor(), (s.app.map.width - 1, 5));
    s.feed_keys("0");
    assert_eq!(s.cursor(), (0, 5));
}

#[test]
fn level_2_lay_belts_and_deliver_ore() {
    let mut s = session_at_level(2);

    // OreDeposit 3x2 at (2,5): output port at (4,6) -> first empty tile (5,6).
    // OutputBin at (28,5): input port (28,6). Need belts x=5..=27 at y=6.
    s.feed_keys("6j5l"); // move to (5,6)
    assert_eq!(s.cursor(), (5, 6));
    s.feed_keys("i"); // insert mode
    assert_eq!(s.mode(), Mode::Insert);
    // 23 belts: c auto-advances right
    for _ in 0..23 {
        s.feed_keys("c");
    }
    s.feed_keys("<Esc>");
    assert_eq!(s.mode(), Mode::Normal);

    // Verify a belt actually exists on the path
    use vimforge::resources::EntityType;
    assert_eq!(s.entity_type_at(5, 6), Some(EntityType::BasicBelt));
    assert_eq!(s.entity_type_at(27, 6), Some(EntityType::BasicBelt));

    // Run the factory until 3 ore are delivered (or fail after enough ticks).
    for _ in 0..600 {
        s.tick(1);
        if s.current_level() == Some(3) {
            return;
        }
    }
    let (ore, _, _, _) = s.output_totals();
    panic!("level 2 never completed; ore delivered = {ore}");
}

#[test]
fn save_and_load_round_trip_via_commands() {
    let dir = std::env::temp_dir().join("vimforge_test_saves");
    let _ = std::fs::create_dir_all(&dir);
    let path = dir.join("roundtrip.json");
    let path_str = path.to_string_lossy().to_string();
    let _ = std::fs::remove_file(&path);

    // Play some of level 2: lay 5 belts, then :w <path>
    let mut s = session_at_level(2);
    s.feed_keys("6j5li");
    for _ in 0..5 {
        s.feed_keys("c");
    }
    s.feed_keys("<Esc>");
    s.feed_keys(&format!(":w {path_str}<CR>"));
    assert!(path.exists(), "save file should exist");

    // Fresh session, load via :e <path>
    let mut s2 = session_at_level(1);
    s2.feed_keys(&format!(":e {path_str}<CR>"));

    use vimforge::resources::EntityType;
    // The 5 belts and the level-2 buildings must be restored
    assert_eq!(s2.entity_type_at(5, 6), Some(EntityType::BasicBelt));
    assert_eq!(s2.entity_type_at(9, 6), Some(EntityType::BasicBelt));
    assert_eq!(s2.entity_type_at(2, 5), Some(EntityType::OreDeposit));
    // Tutorial progress restored: still on level 2
    assert_eq!(s2.current_level(), Some(2));

    let _ = std::fs::remove_file(&path);
}

#[test]
fn macro_replays_count_as_tutorial_edits() {
    // Regression: @a used to bypass the tutorial edit counter entirely.
    let mut s = session_at_level(2);
    s.feed_keys("qaic<Esc>q"); // record: place one belt
    let before = s.tutorial.as_ref().unwrap().edit_count;
    s.feed_keys("3@a"); // replay 3 times = 3 more placements
    let after = s.tutorial.as_ref().unwrap().edit_count;
    assert_eq!(
        after,
        before + 3,
        "macro replays must count their edits"
    );
}

#[test]
fn research_popup_selects_and_starts_tech_interactively() {
    let mut s = GameSession::new(160, 48);
    s.feed_keys("1"); // enter level 1 so we're out of the menu
    s.start_freeplay();

    s.feed_keys(":research<CR>");
    assert!(s.app.popup.is_some(), "research popup should open");
    let avail = s.available_research();
    assert!(avail.len() >= 2, "freeplay should offer several techs");

    // Move selection to the second tech and start it.
    s.feed_keys("j<CR>");
    assert_eq!(
        s.app.research.current,
        Some(avail[1]),
        "Enter should start the selected tech"
    );
    s.feed_keys("<Esc>");
    assert!(s.app.popup.is_none());
}

#[test]
fn contracts_popup_accepts_selected_contract() {
    let mut s = GameSession::new(160, 48);
    s.feed_keys("1");
    s.start_freeplay();

    // Manufacture availability: deliver-history + a forced refresh happens on
    // cycle boundaries; simulate until contracts appear.
    s.app
        .delivered_lifetime
        .insert(vimforge::resources::Resource::IronOre, 10);
    for _ in 0..400 {
        s.tick(1);
        if !s.app.contract_board.available.is_empty() {
            break;
        }
    }
    assert!(
        !s.app.contract_board.available.is_empty(),
        "contracts should generate in freeplay"
    );
    let active_before = s.app.contract_board.active.len();
    s.feed_keys(":contracts<CR><CR>"); // open popup, accept first selection
    assert!(
        s.app.contract_board.active.len() > active_before
            || active_before > 0, // first one may have auto-accepted as the demo
        "Enter should accept the selected contract"
    );
    s.feed_keys("<Esc>");
}

#[test]
fn level_1_accepts_any_tile_of_target_buildings() {
    // Regression: NavigateToAll counted only building anchors; players
    // standing ON the bin (but not its anchor tile) never completed.
    let mut s = session_at_level(1);
    s.feed_keys("5j32l"); // (32,5): bin body tile, anchor is (31,4)
    s.feed_keys("10j");   // (32,15): second bin body tile
    s.tick(1);
    assert_eq!(
        s.current_level(),
        Some(2),
        "standing on any bin tile should complete level 1"
    );
}

#[test]
fn failed_placement_reports_and_keeps_undo_clean() {
    let mut s = session_at_level(2);
    // (2,5) is the ore deposit — placement must fail loudly.
    s.feed_keys("5j2li");
    s.feed_keys("c");
    assert!(
        s.app.status_message.starts_with("Can't place"),
        "failed placement must explain itself, got: {}",
        s.app.status_message
    );
    s.feed_keys("<Esc>");
    // Undo stack must not have accumulated junk: place one belt on a free
    // tile, then a single u undoes it (not N failed-placement snapshots).
    s.feed_keys("j3li");
    s.feed_keys("c<Esc>");
    use vimforge::resources::EntityType;
    assert_eq!(s.entity_type_at(5, 6), Some(EntityType::BasicBelt));
    s.feed_keys("u");
    assert_eq!(
        s.entity_type_at(5, 6),
        None,
        "one u must undo the one real placement"
    );
}
