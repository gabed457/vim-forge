//! Rendering tests: the visual contract for the full-bleed rebuild.
//!
//! Proven here, at real terminal sizes, through the same draw path main.rs
//! uses:
//!   (a) FULL-BLEED — zero cells keep the TestBackend default symbol+style
//!       on every screen (title / level select / gameplay / sandbox) at
//!       80x24, 140x40 and 210x52; the grid rect itself is airtight even
//!       without the root background fill.
//!   (b) adaptive zoom — level 1 at 210x52 renders at S >= 2 with multi-cell
//!       sprites (furnace mouth, crate slats, port sockets, belt chevrons).
//!   (c) the Command Dock shows the minimap on wide terminals.
//!   (d) overlay intro cards appear on level entry and FALL THROUGH: any key
//!       dismisses the card AND still executes.
//!   (e) zi/zo/zf zoom keys drive the viewport scale.
//!   (f) the cursor is visible with its high-contrast style at every scale.
//!   (g) tiny terminals never panic.
//!   (h) the day/night tint still works at every scale and never touches
//!       the cursor.
//!
//! Rendering never mutates game state: every frame is drawn from &GameSession.

use ratatui::backend::TestBackend;
use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::style::{Color, Modifier};
use ratatui::Terminal;

use vimforge::app::{MenuScreen, Mode};
use vimforge::game::session::GameSession;
use vimforge::render::colors;
use vimforge::render::highlights::{highlight_style, HighlightType};
use vimforge::resources::{EntityType, Facing, Resource};
use vimforge::ui;

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Render one full frame exactly the way main.rs does, into a TestBackend.
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

            // Root background fill (no pure black), as in main.rs.
            frame.render_widget(
                ratatui::widgets::Block::default().style(
                    ratatui::style::Style::default().bg(ratatui::style::Color::Rgb(10, 12, 16)),
                ),
                size,
            );

            let areas = ui::layout::compute_layout(size, app.show_sidebar, app.show_tutorial);
            if let Some(area) = areas.tutorial_bar {
                if let Some(ref tut) = session.tutorial {
                    ui::tutorial_bar::render_tutorial_bar(frame, area, app, tut);
                }
            }
            ui::grid_render::render_grid(frame, areas.game_grid, app, &session.viewport);
            if let Some(area) = areas.sidebar {
                ui::sidebar::render_command_dock(frame, area, app, Some(&session.viewport));
            }
            ui::statusbar::render_statusbar_ex(
                frame,
                areas.status_bar,
                app,
                Some(session.viewport.scale),
            );
            if app.popup.is_some() {
                ui::popup::render_popup(frame, size, app);
            }
            if let Some(ref card) = app.overlay {
                ui::menu::render_overlay_card(frame, size, app, card);
            }
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

/// A cell is "default" when it still looks exactly like a fresh TestBackend
/// cell: blank symbol, reset colors, no modifiers. Full-bleed screens must
/// have ZERO of these.
fn is_default_cell(cell: &ratatui::buffer::Cell) -> bool {
    cell.symbol() == " "
        && cell.fg == Color::Reset
        && cell.bg == Color::Reset
        && cell.modifier.is_empty()
}

/// Count untouched cells inside a rect of the buffer; also report the first.
fn default_cells_in(buf: &Buffer, rect: Rect) -> (usize, Option<(u16, u16)>) {
    let mut n = 0;
    let mut first = None;
    for y in rect.y..rect.y + rect.height {
        for x in rect.x..rect.x + rect.width {
            if is_default_cell(&buf[(x, y)]) {
                if first.is_none() {
                    first = Some((x, y));
                }
                n += 1;
            }
        }
    }
    (n, first)
}

fn assert_full_bleed(buf: &Buffer, what: &str) {
    let (n, first) = default_cells_in(buf, *buf.area());
    assert_eq!(
        n,
        0,
        "{}: {} unpainted cells (first at {:?}) — full-bleed broken:\n{}",
        what,
        n,
        first,
        buffer_text(buf)
    );
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

/// Count cells matching an exact (fg, bg) pair anywhere in the buffer.
fn cells_with_style(buf: &Buffer, fg: Color, bg: Color) -> usize {
    let area = buf.area();
    let mut n = 0;
    for y in 0..area.height {
        for x in 0..area.width {
            let cell = &buf[(x, y)];
            if cell.fg == fg && cell.bg == bg {
                n += 1;
            }
        }
    }
    n
}

/// A session parked inside level `n`, intro card dismissed with a throwaway
/// key so gameplay frames are unobstructed (the dismissal key still executes,
/// which is exactly the fall-through contract).
fn level_session(level: usize, w: usize, h: usize) -> GameSession {
    let mut session = GameSession::new(w, h);
    session.start_level(level);
    assert!(session.app.overlay.is_some(), "level entry must show an intro card");
    session.feed_keys("j"); // dismiss AND move down one row
    assert!(session.app.overlay.is_none());
    session
}

/// A sandbox (freeplay) session.
fn sandbox_session(w: usize, h: usize) -> GameSession {
    let mut session = GameSession::new(w, h);
    session.start_freeplay();
    session
}

/// The three contract sizes: minimum, mid, wide.
const SIZES: [(u16, u16); 3] = [(80, 24), (140, 40), (210, 52)];

// ---------------------------------------------------------------------------
// (a) Full-bleed proof: zero default cells on every screen, every size
// ---------------------------------------------------------------------------

#[test]
fn test_full_bleed_title_screen() {
    for (w, h) in SIZES {
        let session = GameSession::new(w as usize, h as usize);
        assert_eq!(session.app.mode, Mode::Menu);
        let buf = draw_frame(&session, w, h);
        assert_full_bleed(&buf, &format!("title {}x{}", w, h));
    }
}

#[test]
fn test_full_bleed_level_select() {
    for (w, h) in SIZES {
        let mut session = GameSession::new(w as usize, h as usize);
        session.app.menu_screen = MenuScreen::LevelSelect;
        let buf = draw_frame(&session, w, h);
        assert_full_bleed(&buf, &format!("level select {}x{}", w, h));
    }
}

#[test]
fn test_full_bleed_help_screen() {
    for (w, h) in SIZES {
        let mut session = GameSession::new(w as usize, h as usize);
        session.app.menu_screen = MenuScreen::Help;
        let buf = draw_frame(&session, w, h);
        assert_full_bleed(&buf, &format!("help {}x{}", w, h));
    }
}

#[test]
fn test_full_bleed_level_1_gameplay() {
    for (w, h) in SIZES {
        let session = level_session(1, w as usize, h as usize);
        let buf = draw_frame(&session, w, h);
        assert_full_bleed(&buf, &format!("level 1 {}x{}", w, h));
    }
}

#[test]
fn test_full_bleed_level_9_gameplay() {
    for (w, h) in SIZES {
        let session = level_session(9, w as usize, h as usize);
        let buf = draw_frame(&session, w, h);
        assert_full_bleed(&buf, &format!("level 9 {}x{}", w, h));
    }
}

#[test]
fn test_full_bleed_sandbox() {
    for (w, h) in SIZES {
        let session = sandbox_session(w as usize, h as usize);
        let buf = draw_frame(&session, w, h);
        assert_full_bleed(&buf, &format!("sandbox {}x{}", w, h));
    }
}

#[test]
fn test_full_bleed_intro_card_over_gameplay() {
    // The intro card itself must not punch holes in the frame.
    for (w, h) in SIZES {
        let mut session = GameSession::new(w as usize, h as usize);
        session.start_level(1);
        assert!(session.app.overlay.is_some());
        let buf = draw_frame(&session, w, h);
        assert_full_bleed(&buf, &format!("intro card {}x{}", w, h));
    }
}

/// The grid rect on its own (no root fill underneath) must be airtight at
/// every scale, and must never paint outside its rect.
#[test]
fn test_grid_rect_airtight_and_clipped_at_all_scales() {
    for scale in 1..=3u8 {
        let mut session = level_session(1, 210, 52);
        // Force the scale through the real zoom path.
        while session.viewport.scale < scale {
            session.feed_keys("zi");
        }
        while session.viewport.scale > scale {
            session.feed_keys("zo");
        }
        assert_eq!(session.viewport.scale, scale);

        let backend = TestBackend::new(120, 40);
        let mut terminal = Terminal::new(backend).expect("terminal");
        let grid_rect = Rect::new(10, 5, 90, 28);
        terminal
            .draw(|frame| {
                ui::grid_render::render_grid(frame, grid_rect, &session.app, &session.viewport);
            })
            .expect("draw");
        let buf = terminal.backend().buffer().clone();

        // Inside: every cell painted.
        let (n, first) = default_cells_in(&buf, grid_rect);
        assert_eq!(
            n, 0,
            "S={}: {} unpainted cells inside the grid rect (first {:?})",
            scale, n, first
        );
        // Outside: nothing painted.
        for y in 0..40u16 {
            for x in 0..120u16 {
                let inside = (10..100).contains(&x) && (5..33).contains(&y);
                if !inside {
                    assert!(
                        is_default_cell(&buf[(x, y)]),
                        "S={}: cell ({}, {}) outside the grid rect was painted",
                        scale,
                        x,
                        y
                    );
                }
            }
        }
    }
}

// ---------------------------------------------------------------------------
// (b) Adaptive zoom: big sprites at 210x52 on level 1
// ---------------------------------------------------------------------------

#[test]
fn test_level_1_zooms_in_with_sprites_at_210x52() {
    let session = level_session(1, 210, 52);
    assert!(
        session.viewport.scale >= 2,
        "level 1 (44x22 map) must auto-fit to S>=2 at 210x52, got S={}",
        session.viewport.scale
    );

    let buf = draw_frame(&session, 210, 52);
    let text = buffer_text(&buf);

    // Smelter sprite: the furnace mouth glyphs are painted in every state.
    assert!(
        text.contains('▐') && text.contains('▌'),
        "smelter furnace mouth (▐▌) missing from the level-1 frame:\n{}",
        text
    );
    // Output bin sprite: slatted crate.
    assert!(text.contains('▦'), "output bin crate slats (▦) missing:\n{}", text);
    // Belts: marching chevrons on the right-facing lanes.
    assert!(text.contains('»'), "belt chevrons (») missing:\n{}", text);
    // Port sockets: at least one output socket on a sprite edge.
    assert!(text.contains('●'), "output port socket (●) missing:\n{}", text);
    // Deposit sprite: rock face shading.
    assert!(
        text.contains('▒') || text.contains('▓'),
        "ore deposit rock face (▒/▓) missing:\n{}",
        text
    );
}

#[test]
fn test_smelter_flames_animate_while_processing() {
    let mut session = level_session(1, 210, 52);
    // Run the preplaced ore -> belt -> smelter line until the smelter is
    // actually smelting, then the firebox must show flame glyphs.
    let mut saw_flames = false;
    for _ in 0..40 {
        session.tick(10);
        let buf = draw_frame(&session, 210, 52);
        let text = buffer_text(&buf);
        if text.contains('▲') || text.contains('△') {
            saw_flames = true;
            break;
        }
    }
    assert!(saw_flames, "smelter never showed processing flames (▲/△) in 400 ticks");
}

#[test]
fn test_cargo_chips_ride_belts_at_scale_2() {
    let mut session = level_session(1, 210, 52);
    // Drop an item onto a preplaced belt tile and make sure a bold chip
    // appears (2-cell resource chip riding the lane).
    let mut belt_at = None;
    'outer: for y in 0..session.app.map.height {
        for x in 0..session.app.map.width {
            if session.entity_type_at(x, y) == Some(EntityType::BasicBelt) {
                belt_at = Some((x, y));
                break 'outer;
            }
        }
    }
    let (bx, by) = belt_at.expect("level 1 preplaces belts");
    session.app.map.set_resource(bx, by, Resource::IronOre);
    let buf = draw_frame(&session, 210, 52);

    // Somewhere in the frame there is a BOLD cell using the iron-ore color.
    let (r, g, b) = Resource::IronOre.color();
    let area = buf.area();
    let mut found = false;
    'scan: for y in 0..area.height {
        for x in 0..area.width {
            let cell = &buf[(x, y)];
            if cell.modifier.contains(Modifier::BOLD) && cell.fg == Color::Rgb(r, g, b) {
                found = true;
                break 'scan;
            }
        }
    }
    assert!(found, "no bold iron-ore cargo chip found on the belt");
}

// ---------------------------------------------------------------------------
// (c) Command Dock + minimap on wide terminals
// ---------------------------------------------------------------------------

#[test]
fn test_dock_shows_minimap_at_210x52() {
    let session = level_session(1, 210, 52);
    let buf = draw_frame(&session, 210, 52);
    let text = buffer_text(&buf);
    assert!(text.contains("MAP"), "dock minimap section missing:\n{}", text);
    assert!(text.contains("STATS"), "dock stats section missing:\n{}", text);
    assert!(text.contains("KEYS"), "dock keys section missing:\n{}", text);

    // The minimap paints colored half-blocks inside the dock (right 30 cols).
    let area = buf.area();
    let dock_x0 = area.width - ui::layout::DOCK_WIDTH;
    let mut half_blocks = 0;
    for y in 0..area.height {
        for x in dock_x0..area.width {
            if buf[(x, y)].symbol() == "▀" {
                half_blocks += 1;
            }
        }
    }
    assert!(
        half_blocks > 10,
        "expected a block of ▀ minimap cells in the dock, found {}",
        half_blocks
    );
}

#[test]
fn test_dock_hidden_below_140_cols() {
    let areas = ui::layout::compute_layout(Rect::new(0, 0, 139, 40), true, true);
    assert!(areas.sidebar.is_none(), "dock must hide below 140 cols");
    let areas = ui::layout::compute_layout(Rect::new(0, 0, 140, 40), true, true);
    assert!(areas.sidebar.is_some(), "dock must appear at 140 cols");
    assert_eq!(areas.sidebar.unwrap().width, ui::layout::DOCK_WIDTH);
}

// ---------------------------------------------------------------------------
// (d) Overlay cards: intro on level entry, any-key fall-through
// ---------------------------------------------------------------------------

#[test]
fn test_intro_card_appears_and_falls_through() {
    let mut session = GameSession::new(210, 52);
    session.start_level(1);

    // Card is up and rendered.
    assert!(session.app.overlay.is_some());
    let buf = draw_frame(&session, 210, 52);
    let text = buffer_text(&buf);
    assert!(
        text.contains("L E V E L  1") || text.contains("L E V E L   1"),
        "intro card headline missing:\n{}",
        text
    );
    assert!(text.contains("MOVEMENT"), "level name missing on the card:\n{}", text);
    assert!(text.contains("press any key"), "dismiss hint missing:\n{}", text);

    // Fall-through: 'j' dismisses the card AND executes (cursor moves down).
    assert_eq!(session.cursor(), (0, 0));
    session.feed_keys("j");
    assert!(session.app.overlay.is_none(), "any key must dismiss the card");
    assert_eq!(session.cursor(), (0, 1), "the dismissing key must still execute");

    // Card gone from the next frame.
    let text = buffer_text(&draw_frame(&session, 210, 52));
    assert!(!text.contains("press any key"), "card still rendered after dismissal");
}

#[test]
fn test_intro_card_auto_expires() {
    let mut session = GameSession::new(210, 52);
    session.start_level(1);
    assert!(session.app.overlay.is_some());
    session.tick(301); // OVERLAY_TICKS = 300
    assert!(session.app.overlay.is_none(), "card must auto-dismiss after 300 ticks");
}

// ---------------------------------------------------------------------------
// (e) Zoom keys
// ---------------------------------------------------------------------------

#[test]
fn test_zoom_keys_change_viewport_scale() {
    let mut session = level_session(1, 210, 52);
    let fit = session.viewport.scale;
    assert!(fit >= 2, "level 1 at 210x52 should fit at S>=2");

    session.feed_keys("zo");
    assert_eq!(session.viewport.scale, fit - 1, "zo zooms out one step");
    assert!(session.viewport.manual_zoom, "zo sets the manual-zoom flag");

    session.feed_keys("zi");
    assert_eq!(session.viewport.scale, fit, "zi zooms back in");
    session.feed_keys("zizi");
    assert_eq!(session.viewport.scale, 3, "zoom clamps at S=3");

    session.feed_keys("zf");
    assert_eq!(session.viewport.scale, fit, "zf re-fits the map");
    assert!(!session.viewport.manual_zoom, "zf clears the manual-zoom flag");

    // Manual zoom survives a terminal resize (auto-fit must not override it).
    session.feed_keys("zo");
    let manual = session.viewport.scale;
    session.set_term_size(210, 52);
    assert_eq!(session.viewport.scale, manual, "resize must respect manual zoom");
}

// ---------------------------------------------------------------------------
// (f) Cursor visibility at every scale
// ---------------------------------------------------------------------------

#[test]
fn test_cursor_visible_at_every_scale() {
    let expected = highlight_style(HighlightType::Cursor);
    let (efg, ebg) = (expected.fg.unwrap(), expected.bg.unwrap());
    assert_ne!(efg, ebg);

    for scale in 1..=3u8 {
        let mut session = level_session(1, 210, 52);
        while session.viewport.scale < scale {
            session.feed_keys("zi");
        }
        while session.viewport.scale > scale {
            session.feed_keys("zo");
        }
        assert_eq!(session.viewport.scale, scale);
        let buf = draw_frame(&session, 210, 52);
        assert!(
            cells_with_style(&buf, efg, ebg) >= 1,
            "cursor core (fg {:?} on bg {:?}) not found at S={}",
            efg,
            ebg,
            scale
        );
    }
}

#[test]
fn test_insert_cursor_is_green_at_scale_2() {
    let mut session = level_session(1, 210, 52);
    assert!(session.viewport.scale >= 2);
    session.app.mode = Mode::Insert;
    let buf = draw_frame(&session, 210, 52);
    let expected = highlight_style(HighlightType::CursorInsert);
    assert!(
        cells_with_style(&buf, expected.fg.unwrap(), expected.bg.unwrap()) >= 1,
        "insert-mode cursor core missing at S>=2"
    );
}

// ---------------------------------------------------------------------------
// (g) Tiny terminals never panic
// ---------------------------------------------------------------------------

#[test]
fn test_tiny_terminals_do_not_panic() {
    // Just below minimum: the too-small path for menu AND gameplay.
    let menu_session = GameSession::new(79, 20);
    let buf = draw_frame(&menu_session, 79, 20);
    assert!(buffer_text(&buf).contains("Terminal too small"));

    let game_session = level_session(1, 79, 20);
    let buf = draw_frame(&game_session, 79, 20);
    assert!(buffer_text(&buf).contains("Terminal too small"));

    // Truly tiny sizes must not panic in any state.
    for (w, h) in [(79u16, 20u16), (10, 3), (2, 2), (1, 1)] {
        let session = GameSession::new(w as usize, h as usize);
        let _ = draw_frame(&session, w, h);
        let mut session = GameSession::new(w as usize, h as usize);
        session.start_level(1);
        let _ = draw_frame(&session, w, h);
        let mut session = GameSession::new(w as usize, h as usize);
        session.start_freeplay();
        let _ = draw_frame(&session, w, h);
    }
}

// ---------------------------------------------------------------------------
// (h) Day/night tint
// ---------------------------------------------------------------------------

#[test]
fn test_day_night_tint_changes_the_frame_at_scale_2() {
    let mut session = level_session(1, 210, 52);
    assert!(session.viewport.scale >= 2);
    session.app.day_tick = 0;
    let day = draw_frame(&session, 210, 52);
    session.app.day_tick = 300;
    let golden = draw_frame(&session, 210, 52);
    session.app.day_tick = 480;
    let night = draw_frame(&session, 210, 52);

    let d1 = diff_cells(&day, &golden);
    let d2 = diff_cells(&day, &night);
    assert!(d1 > 50, "tick 0 vs 300 should retint many cells at S=2, got {}", d1);
    assert!(d2 > 50, "tick 0 vs 480 should retint many cells at S=2, got {}", d2);
}

#[test]
fn test_day_night_tint_changes_the_frame_at_scale_1() {
    let mut session = level_session(9, 140, 40); // 65x30 map -> S=1
    assert_eq!(session.viewport.scale, 1);
    session.app.day_tick = 0;
    let day = draw_frame(&session, 140, 40);
    session.app.day_tick = 480;
    let night = draw_frame(&session, 140, 40);
    assert!(diff_cells(&day, &night) > 50, "night must retint the compact path too");
}

#[test]
fn test_day_night_multiplier_is_smooth() {
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
    assert_eq!(colors::day_night_multiplier(100), (1.0, 1.0, 1.0));
    let night = colors::day_night_multiplier(480);
    assert!(night.0 < 0.7 && night.2 > 0.85, "night is dim and blue: {:?}", night);
}

#[test]
fn test_cursor_is_not_tinted_by_night() {
    let mut session = level_session(1, 210, 52);
    session.app.day_tick = 480;
    let buf = draw_frame(&session, 210, 52);
    let expected = highlight_style(HighlightType::Cursor);
    assert!(
        cells_with_style(&buf, expected.fg.unwrap(), expected.bg.unwrap()) >= 1,
        "cursor must keep its exact bright style at midnight"
    );
}

// ---------------------------------------------------------------------------
// Compact path (S = 1) still works for huge maps
// ---------------------------------------------------------------------------

#[test]
fn test_compact_path_belt_arrows_and_cargo() {
    // Level 9's 65x30 map forces S=1 at 140x40. Place a belt with cargo and
    // check the compact glyphs.
    let mut session = level_session(9, 140, 40);
    assert_eq!(session.viewport.scale, 1);
    let app = &mut session.app;
    let _ = app.map.place_entity_on_map(
        &mut app.world,
        30,
        20,
        EntityType::BasicBelt,
        Facing::Right,
        true,
    );
    app.map.set_resource(30, 20, Resource::IronOre);
    let buf = draw_frame(&session, 140, 40);
    let text = buffer_text(&buf);
    assert!(text.contains('▶'), "compact right-belt arrow missing:\n{}", text);
}

// ---------------------------------------------------------------------------
// Status bar
// ---------------------------------------------------------------------------

#[test]
fn test_statusbar_zoom_and_level_chips() {
    let session = level_session(1, 210, 52);
    let buf = draw_frame(&session, 210, 52);
    let text = buffer_text(&buf);
    let expected_zoom = format!("\u{2315}{}x", session.viewport.scale);
    assert!(text.contains(&expected_zoom), "zoom chip {} missing:\n{}", expected_zoom, text);
    assert!(text.contains("L1/30"), "level progress chip missing:\n{}", text);
    assert!(text.contains("NORMAL"), "mode chip missing:\n{}", text);
}

#[test]
fn test_statusbar_shows_insert_chip_in_insert_mode() {
    let mut session = level_session(3, 140, 40);
    session.app.mode = Mode::Insert;
    let buf = draw_frame(&session, 140, 40);
    let text = buffer_text(&buf);
    assert!(text.contains("INSERT"), "INSERT chip missing:\n{}", text);

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
    let mut session = level_session(3, 140, 40);
    session.app.recording_macro = Some('a');
    session.app.pending_keys = "2d".to_string();
    let buf = draw_frame(&session, 140, 40);
    let text = buffer_text(&buf);
    assert!(text.contains("REC @a"), "recording indicator missing:\n{}", text);
    assert!(text.contains("2d"), "showcmd pending keys missing:\n{}", text);
    assert!(text.contains('$'), "cash readout missing:\n{}", text);
}

#[test]
fn test_statusbar_command_line_shows_prompt_and_cursor() {
    let mut session = level_session(3, 140, 40);
    session.app.mode = Mode::Command;
    session.app.command_buffer = "w".to_string();
    let buf = draw_frame(&session, 140, 40);
    let text = buffer_text(&buf);
    assert!(text.contains(":w\u{2588}"), "command line with cursor missing:\n{}", text);
}

// ---------------------------------------------------------------------------
// Top bar (tutorial bar)
// ---------------------------------------------------------------------------

#[test]
fn test_top_bar_level_chip_objective_and_key_caps() {
    let session = level_session(3, 210, 52);
    let buf = draw_frame(&session, 210, 52);
    let text = buffer_text(&buf);
    assert!(text.contains("LEVEL 3"), "level chip missing:\n{}", text);
    assert!(text.contains("HINT"), "hint carousel missing:\n{}", text);
    // Live progress readout (level 3 counts ingots).
    assert!(text.contains("ingots"), "live progress readout missing:\n{}", text);

    // Key tokens inside hint prose get the gold key-cap fg somewhere in the
    // two top rows.
    let gold = Color::Rgb(255, 216, 100);
    let mut found = false;
    'outer: for y in 0..2u16 {
        for x in 0..buf.area().width {
            if buf[(x, y)].fg == gold {
                found = true;
                break 'outer;
            }
        }
    }
    assert!(found, "no key token highlighted in the top bar:\n{}", text);
}

// ---------------------------------------------------------------------------
// Layout degradation
// ---------------------------------------------------------------------------

#[test]
fn test_layout_degrades_gracefully() {
    // Wide + tall: everything present, top bar is 2 rows.
    let areas = ui::layout::compute_layout(Rect::new(0, 0, 210, 52), true, true);
    assert_eq!(areas.tutorial_bar.unwrap().height, 2);
    assert!(areas.sidebar.is_some());
    assert_eq!(areas.status_bar.height, 1);

    // Below 30 rows: top bar collapses to 1 row.
    let areas = ui::layout::compute_layout(Rect::new(0, 0, 210, 29), true, true);
    assert_eq!(areas.tutorial_bar.unwrap().height, 1);

    // Very short: top bar drops before the grid does.
    let areas = ui::layout::compute_layout(Rect::new(0, 0, 120, 10), true, true);
    assert!(areas.tutorial_bar.is_none());
    assert!(areas.game_grid.height >= 8);

    // Narrow: the dock collapses, grid keeps every column.
    let areas = ui::layout::compute_layout(Rect::new(0, 0, 100, 24), true, false);
    assert!(areas.sidebar.is_none());
    assert_eq!(areas.game_grid.width, 100);
}

// ---------------------------------------------------------------------------
// Menu details
// ---------------------------------------------------------------------------

#[test]
fn test_title_screen_entries_and_wordmark() {
    let session = GameSession::new(210, 52);
    let buf = draw_frame(&session, 210, 52);
    let text = buffer_text(&buf);
    assert!(text.contains("Campaign"), "Campaign entry missing:\n{}", text);
    assert!(text.contains("Sandbox"), "Sandbox entry missing:\n{}", text);
    assert!(text.contains("Quit"), "Quit entry missing:\n{}", text);
    assert!(text.contains("[1]"), "quick-key labels missing:\n{}", text);
    assert!(text.contains("f a c t o r y"), "tagline missing:\n{}", text);
    assert!(
        text.contains(concat!("v", env!("CARGO_PKG_VERSION"))),
        "version missing:\n{}",
        text
    );

    // Gradient logotype: bold cells spanning more than 8 distinct RGB fgs.
    let mut colors_seen = std::collections::HashSet::new();
    let area = buf.area();
    for y in 0..area.height {
        for x in 0..area.width {
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
        "expected a per-column logotype gradient, saw {} colors",
        colors_seen.len()
    );
}

#[test]
fn test_title_backdrop_animates() {
    let mut session = GameSession::new(140, 40);
    session.app.animations.frame_counter = 0;
    let a = draw_frame(&session, 140, 40);
    session.app.animations.frame_counter = 9;
    let b = draw_frame(&session, 140, 40);
    assert!(diff_cells(&a, &b) > 0, "title backdrop/accents should animate across frames");
}

#[test]
fn test_level_select_grid_and_acts() {
    let mut session = GameSession::new(210, 52);
    // Force a fresh profile: a save on disk (e.g. written by other test
    // binaries' autosaves) must not leak progress into this assertion.
    session.app.campaign_completed.clear();
    session.app.has_save = false;
    session.app.saved_level = None;
    session.app.menu_screen = MenuScreen::LevelSelect;
    let buf = draw_frame(&session, 210, 52);
    let text = buffer_text(&buf);
    assert!(text.contains("LEVEL SELECT"), "header missing:\n{}", text);
    assert!(text.contains("SURVIVAL"), "act I label missing:\n{}", text);
    assert!(text.contains("MASTERY"), "act VI label missing:\n{}", text);
    assert!(text.contains("01"), "level number tiles missing:\n{}", text);
    // Fresh profile: only level 1 unlocked -> the next marker ▶ and locks ×.
    assert!(text.contains('▶'), "next-level marker missing:\n{}", text);
    assert!(text.contains('×'), "locked-tile marker missing:\n{}", text);
}

// ---------------------------------------------------------------------------
// Frame dumps (run with --nocapture to eyeball)
// ---------------------------------------------------------------------------

#[test]
fn test_dump_frames_for_eyeballing() {
    let menu = GameSession::new(210, 52);
    println!("--- TITLE 210x52 ---");
    println!("{}", buffer_text(&draw_frame(&menu, 210, 52)));

    let game = level_session(1, 210, 52);
    println!("--- LEVEL 1 210x52 ---");
    println!("{}", buffer_text(&draw_frame(&game, 210, 52)));
}
