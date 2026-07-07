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
