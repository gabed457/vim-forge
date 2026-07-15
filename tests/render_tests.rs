//! Rendering tests: prove the UI draws correctly (and never panics) using
//! ratatui's TestBackend. Game state is built through the public GameSession /
//! AppState APIs; rendering itself never mutates state.

use ratatui::backend::TestBackend;
use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::style::{Color, Modifier};
use ratatui::Terminal;

use vimforge::app::Mode;
use vimforge::game::session::GameSession;
use vimforge::render::colors;
use vimforge::render::highlights::{highlight_style, HighlightType};
use vimforge::resources::{EntityType, Facing, Resource};
use vimforge::ui;

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Render one full frame the same way main.rs does, into a TestBackend.
fn draw_frame(session: &GameSession, width: u16, height: u16) -> Buffer {
    let backend = TestBackend::new(width, height);
    let mut terminal = Terminal::new(backend).expect("terminal");
    terminal
        .draw(|frame| {
            let size = frame.area();
            let app = &session.app;

            if ui::layout::is_terminal_too_small(size) {
                let area = ui::layout::too_small_area(size);
                let msg = ratatui::widgets::Paragraph::new("Terminal too small (need 80x24)");
                frame.render_widget(msg, area);
                return;
            }

            if app.mode == Mode::Menu {
                ui::menu::render_menu(frame, size, app);
                return;
            }

            let areas = ui::layout::compute_layout(size, app.show_sidebar, app.show_tutorial);
            if let Some(area) = areas.tutorial_bar {
                if let Some(ref tut) = session.tutorial {
                    ui::tutorial_bar::render_tutorial_bar(frame, area, app, tut);
                }
            }
            ui::grid_render::render_grid(frame, areas.game_grid, app, &session.viewport);
            ui::statusbar::render_statusbar(frame, areas.status_bar, app);
        })
        .expect("draw");
    terminal.backend().buffer().clone()
}

/// Flatten a buffer to plain text (one line per row) for eyeballing and
/// substring assertions.
fn buffer_text(buf: &Buffer) -> String {
    let area = buf.area();
    let mut out = String::new();
    for y in 0..area.height {
        for x in 0..area.width {
            out.push_str(buf[(x, y)].symbol());
        }
        out.push('\n');
    }
    out
}

/// Count cells whose (symbol, fg, bg) differ between two same-sized buffers.
fn diff_cells(a: &Buffer, b: &Buffer) -> usize {
    assert_eq!(a.area(), b.area());
    let area = a.area();
    let mut n = 0;
    for y in 0..area.height {
        for x in 0..area.width {
            let ca = &a[(x, y)];
            let cb = &b[(x, y)];
            if ca.symbol() != cb.symbol() || ca.fg != cb.fg || ca.bg != cb.bg {
                n += 1;
            }
        }
    }
    n
}

/// A mid-game session with belts, a smelter, an item on a belt, and a mark.
fn midgame_session(term_w: usize, term_h: usize) -> GameSession {
    let mut session = GameSession::new(term_w, term_h);
    session.start_level(3);
    let app = &mut session.app;

    // A row of belts heading right, plus one heading down.
    for x in 12..18 {
        let _ = app.map.place_entity_on_map(
            &mut app.world,
            x,
            10,
            EntityType::BasicBelt,
            Facing::Right,
            true,
        );
    }
    let _ = app.map.place_entity_on_map(
        &mut app.world,
        18,
        10,
        EntityType::BasicBelt,
        Facing::Down,
        true,
    );

    // An item riding one of the belts.
    app.map.set_resource(14, 10, Resource::IronOre);

    // A vim mark badge.
    app.marks.set('a', 20, 12);

    // Cursor parked at a known position.
    app.cursor_x = 5;
    app.cursor_y = 5;
    session
}

/// Locate the two screen cells of a map tile in a rendered frame.
fn tile_screen_cells(session: &GameSession, buf_area: Rect, x: usize, y: usize) -> (u16, u16, u16) {
    let areas = ui::layout::compute_layout(
        buf_area,
        session.app.show_sidebar,
        session.app.show_tutorial,
    );
    let vp = &session.viewport;
    let cell0_x =
        areas.game_grid.x + vp.pad_left + ((x - vp.offset_x) as u16) * 2;
    let cell_y = areas.game_grid.y + vp.pad_top + (y - vp.offset_y) as u16;
    (cell0_x, cell0_x + 1, cell_y)
}

// ---------------------------------------------------------------------------
// No-panic sweeps across terminal sizes
// ---------------------------------------------------------------------------

#[test]
fn test_gameplay_renders_without_panic_at_common_sizes() {
    for (w, h) in [(80u16, 24u16), (120, 40), (200, 60)] {
        let session = midgame_session(w as usize, h as usize);
        let buf = draw_frame(&session, w, h);
        // The frame must contain the status bar mode chip somewhere.
        assert!(
            buffer_text(&buf).contains("NORMAL"),
            "expected NORMAL chip at {}x{}",
            w,
            h
        );
    }
}

#[test]
fn test_menu_renders_without_panic_at_common_sizes() {
    for (w, h) in [(80u16, 24u16), (120, 40), (200, 60)] {
        let session = GameSession::new(w as usize, h as usize);
        assert_eq!(session.app.mode, Mode::Menu);
        let buf = draw_frame(&session, w, h);
        let text = buffer_text(&buf);
        assert!(text.contains("[1]"), "menu should list option [1] at {}x{}", w, h);
        assert!(text.contains("Tutorial"), "menu should list Tutorial at {}x{}", w, h);
        assert!(text.contains("Quit"), "menu should list Quit at {}x{}", w, h);
    }
}

#[test]
fn test_too_small_terminal_path_does_not_panic() {
    // Below the 80x24 minimum: both menu and gameplay states.
    let menu_session = GameSession::new(79, 20);
    let buf = draw_frame(&menu_session, 79, 20);
    assert!(buffer_text(&buf).contains("Terminal too small"));

    let game_session = midgame_session(79, 20);
    let buf = draw_frame(&game_session, 79, 20);
    assert!(buffer_text(&buf).contains("Terminal too small"));

    // Truly tiny sizes must not panic either.
    for (w, h) in [(10u16, 3u16), (2, 2), (1, 1)] {
        let session = midgame_session(w as usize, h as usize);
        let _ = draw_frame(&session, w, h);
    }
}

#[test]
fn test_grid_never_paints_outside_its_rect() {
    // Render the grid into a small rect surrounded by sentinel cells and
    // verify nothing outside the rect was touched.
    let session = midgame_session(120, 40);
    let backend = TestBackend::new(60, 20);
    let mut terminal = Terminal::new(backend).expect("terminal");
    terminal
        .draw(|frame| {
            let grid_rect = Rect::new(10, 5, 30, 10);
            ui::grid_render::render_grid(frame, grid_rect, &session.app, &session.viewport);
        })
        .expect("draw");
    let buf = terminal.backend().buffer().clone();
    for y in 0..20u16 {
        for x in 0..60u16 {
            let inside = (10..40).contains(&x) && (5..15).contains(&y);
            if !inside {
                assert_eq!(
                    buf[(x, y)].symbol(),
                    " ",
                    "cell ({}, {}) outside the grid rect was painted",
                    x,
                    y
                );
            }
        }
    }
}

// ---------------------------------------------------------------------------
// Cursor
// ---------------------------------------------------------------------------

#[test]
fn test_cursor_cell_has_high_contrast_style() {
    let session = midgame_session(120, 40);
    let buf = draw_frame(&session, 120, 40);
    let (cx, _, cy) = tile_screen_cells(&session, *buf.area(), 5, 5);

    let cell = &buf[(cx, cy)];
    let expected = highlight_style(HighlightType::Cursor);
    assert_eq!(cell.bg, expected.bg.unwrap(), "cursor bg must be the bright block");
    assert_eq!(cell.fg, expected.fg.unwrap(), "cursor fg must be the dark ink");
    // Explicit fg+bg means the cursor can never vanish on any tile color.
    assert_ne!(cell.fg, cell.bg);
}

#[test]
fn test_insert_cursor_is_green_block() {
    let mut session = midgame_session(120, 40);
    session.app.mode = Mode::Insert;
    let buf = draw_frame(&session, 120, 40);
    let (cx, _, cy) = tile_screen_cells(&session, *buf.area(), 5, 5);
    let expected = highlight_style(HighlightType::CursorInsert);
    assert_eq!(buf[(cx, cy)].bg, expected.bg.unwrap());
}

// ---------------------------------------------------------------------------
// Belts & items
// ---------------------------------------------------------------------------

#[test]
fn test_belt_tile_shows_directional_glyph() {
    let session = midgame_session(120, 40);
    let buf = draw_frame(&session, 120, 40);

    // Right-facing belt at (12, 10) -> cell 0 must be the solid right triangle.
    let (c0, _, cy) = tile_screen_cells(&session, *buf.area(), 12, 10);
    assert_eq!(buf[(c0, cy)].symbol(), "\u{25B6}", "right belt renders ▶");

    // Down-facing belt at (18, 10) -> solid down triangle.
    let (c0, _, cy) = tile_screen_cells(&session, *buf.area(), 18, 10);
    assert_eq!(buf[(c0, cy)].symbol(), "\u{25BC}", "down belt renders ▼");
}

#[test]
fn test_item_on_belt_is_visible_and_bright() {
    let session = midgame_session(120, 40);
    let buf = draw_frame(&session, 120, 40);

    // Iron ore rides the belt at (14, 10); its glyph is 'i' in cell 1,
    // while cell 0 keeps the belt's direction arrow.
    let (c0, c1, cy) = tile_screen_cells(&session, *buf.area(), 14, 10);
    assert_eq!(buf[(c0, cy)].symbol(), "\u{25B6}", "direction stays visible under cargo");
    assert_eq!(buf[(c1, cy)].symbol(), "i", "iron ore glyph rides the belt");
    assert!(
        buf[(c1, cy)].modifier.contains(Modifier::BOLD),
        "cargo glyph is bold/bright"
    );
}

// ---------------------------------------------------------------------------
// Day/night tinting
// ---------------------------------------------------------------------------

#[test]
fn test_day_night_tint_changes_the_frame() {
    let mut session = midgame_session(120, 40);
    session.app.day_tick = 0;
    let day = draw_frame(&session, 120, 40);
    session.app.day_tick = 300;
    let golden = draw_frame(&session, 120, 40);
    session.app.day_tick = 480;
    let night = draw_frame(&session, 120, 40);

    let d1 = diff_cells(&day, &golden);
    let d2 = diff_cells(&day, &night);
    assert!(d1 > 50, "tick 0 vs 300 should retint many cells, got {}", d1);
    assert!(d2 > 50, "tick 0 vs 480 should retint many cells, got {}", d2);
}

#[test]
fn test_day_night_multiplier_is_smooth() {
    // No hard switches: successive ticks never jump more than a small step.
    let mut prev = colors::day_night_multiplier(0);
    for t in 1..=600u32 {
        let cur = colors::day_night_multiplier(t);
        for (a, b) in [(prev.0, cur.0), (prev.1, cur.1), (prev.2, cur.2)] {
            assert!(
                (a - b).abs() < 0.02,
                "tint jumped at tick {}: {} -> {}",
                t,
                a,
                b
            );
        }
        prev = cur;
    }
    // Midday is neutral; deep night is dimmer and bluer.
    assert_eq!(colors::day_night_multiplier(100), (1.0, 1.0, 1.0));
    let night = colors::day_night_multiplier(480);
    assert!(night.0 < 0.7 && night.2 > 0.85, "night is dim and blue: {:?}", night);
}

#[test]
fn test_cursor_is_not_tinted_by_night() {
    // The cursor must stay the same bright block at midnight.
    let mut session = midgame_session(120, 40);
    session.app.day_tick = 480;
    let buf = draw_frame(&session, 120, 40);
    let (cx, _, cy) = tile_screen_cells(&session, *buf.area(), 5, 5);
    let expected = highlight_style(HighlightType::Cursor);
    assert_eq!(buf[(cx, cy)].bg, expected.bg.unwrap());
}

// ---------------------------------------------------------------------------
// Status bar
// ---------------------------------------------------------------------------

#[test]
fn test_statusbar_shows_insert_chip_in_insert_mode() {
    let mut session = midgame_session(120, 40);
    session.app.mode = Mode::Insert;
    let buf = draw_frame(&session, 120, 40);
    let text = buffer_text(&buf);
    assert!(text.contains("INSERT"), "INSERT chip missing:\n{}", text);

    // The chip itself is green-backed.
    let last_row = buf.area().height - 1;
    let mut found_green_chip = false;
    for x in 0..buf.area().width {
        if buf[(x, last_row)].bg == Color::Rgb(110, 200, 110) {
            found_green_chip = true;
            break;
        }
    }
    assert!(found_green_chip, "INSERT chip should have the green bed");
}

#[test]
fn test_statusbar_shows_recording_and_pending_keys() {
    let mut session = midgame_session(120, 40);
    session.app.recording_macro = Some('a');
    session.app.pending_keys = "2d".to_string();
    let buf = draw_frame(&session, 120, 40);
    let text = buffer_text(&buf);
    assert!(text.contains("REC @a"), "recording indicator missing:\n{}", text);
    assert!(text.contains("2d"), "showcmd pending keys missing:\n{}", text);
    assert!(text.contains('$'), "cash readout missing:\n{}", text);
}

#[test]
fn test_statusbar_command_line_shows_prompt_and_cursor() {
    let mut session = midgame_session(120, 40);
    session.app.mode = Mode::Command;
    session.app.command_buffer = "w".to_string();
    let buf = draw_frame(&session, 120, 40);
    let text = buffer_text(&buf);
    assert!(text.contains(":w\u{2588}"), "command line with cursor missing:\n{}", text);
}

// ---------------------------------------------------------------------------
// Tutorial bar
// ---------------------------------------------------------------------------

#[test]
fn test_tutorial_bar_highlights_key_names() {
    let mut session = midgame_session(120, 40);
    // Level 3 hint 1: "Press i, then c to lay belts rightward. ..." — has keys.
    if let Some(ref mut tut) = session.tutorial {
        tut.current_hint_index = 1;
    }
    let buf = draw_frame(&session, 120, 40);
    let text = buffer_text(&buf);
    assert!(text.contains("LEVEL 3"), "level chip missing:\n{}", text);
    assert!(text.contains("GOAL"), "goal row missing:\n{}", text);

    // Level 3's first hint mentions moving; hint row exists.
    assert!(text.contains("Hint"), "hint row missing:\n{}", text);

    // Key tokens inside prose get the gold key-cap fg. Find at least one
    // cell in the top 3 rows using that exact color.
    let gold = Color::Rgb(255, 216, 100);
    let mut found = false;
    'outer: for y in 0..3u16 {
        for x in 0..buf.area().width {
            if buf[(x, y)].fg == gold {
                found = true;
                break 'outer;
            }
        }
    }
    assert!(found, "no key token highlighted in tutorial bar:\n{}", text);
}

// ---------------------------------------------------------------------------
// Marks
// ---------------------------------------------------------------------------

#[test]
fn test_mark_renders_as_badge() {
    let session = midgame_session(120, 40);
    let buf = draw_frame(&session, 120, 40);
    let (_, c1, cy) = tile_screen_cells(&session, *buf.area(), 20, 12);
    assert_eq!(buf[(c1, cy)].symbol(), "a", "mark 'a' badge should be drawn");
    assert_eq!(
        buf[(c1, cy)].bg,
        Color::Rgb(212, 175, 55),
        "mark badge has the gold bed"
    );
}

// ---------------------------------------------------------------------------
// Menu details
// ---------------------------------------------------------------------------

#[test]
fn test_menu_has_gradient_logotype_and_version() {
    let session = GameSession::new(120, 40);
    let buf = draw_frame(&session, 120, 40);
    let text = buffer_text(&buf);
    assert!(
        text.contains(concat!("v", env!("CARGO_PKG_VERSION"))),
        "version missing:\n{}",
        text
    );
    assert!(text.contains("f a c t o r y"), "tagline missing:\n{}", text);

    // Two-tone gradient: the logotype rows must use more than 8 distinct
    // foreground colors (per-column ramp).
    let mut colors_seen = std::collections::HashSet::new();
    for y in 0..buf.area().height {
        for x in 0..buf.area().width {
            let cell = &buf[(x, y)];
            if cell.symbol() != " " && cell.modifier.contains(Modifier::BOLD) {
                if let Color::Rgb(r, g, b) = cell.fg {
                    colors_seen.insert((r, g, b));
                }
            }
        }
    }
    assert!(
        colors_seen.len() > 8,
        "expected a per-column gradient, saw {} colors",
        colors_seen.len()
    );
}

#[test]
fn test_menu_pulse_accent_animates() {
    let mut session = GameSession::new(120, 40);
    session.app.animations.frame_counter = 0;
    let a = draw_frame(&session, 120, 40);
    session.app.animations.frame_counter = 8;
    let b = draw_frame(&session, 120, 40);
    assert!(diff_cells(&a, &b) > 0, "menu accent should pulse across frames");
}

// ---------------------------------------------------------------------------
// Layout degradation
// ---------------------------------------------------------------------------

#[test]
fn test_layout_degrades_gracefully() {
    // Tall enough: tutorial bar present.
    let areas = ui::layout::compute_layout(Rect::new(0, 0, 120, 40), true, true);
    assert!(areas.tutorial_bar.is_some());
    assert!(areas.sidebar.is_some());

    // Short terminal: tutorial bar collapses before the grid does.
    let areas = ui::layout::compute_layout(Rect::new(0, 0, 120, 10), true, true);
    assert!(areas.tutorial_bar.is_none());
    assert!(areas.game_grid.height >= 8);

    // Narrow terminal: sidebar collapses.
    let areas = ui::layout::compute_layout(Rect::new(0, 0, 50, 24), true, false);
    assert!(areas.sidebar.is_none());
    assert!(areas.game_grid.width >= 40);
}

// ---------------------------------------------------------------------------
// Frame dumps (run with --nocapture to eyeball)
// ---------------------------------------------------------------------------

#[test]
fn test_dump_frames_for_eyeballing() {
    let menu = GameSession::new(100, 30);
    println!("--- MENU 100x30 ---");
    println!("{}", buffer_text(&draw_frame(&menu, 100, 30)));

    let game = midgame_session(100, 30);
    println!("--- GAMEPLAY 100x30 ---");
    println!("{}", buffer_text(&draw_frame(&game, 100, 30)));
}
