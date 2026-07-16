//! Menu screens (Title / LevelSelect / Help) and the full-screen overlay
//! cards (level intro / level complete / campaign finale).
//!
//! Every screen is FULL-BLEED: a procedural animated factory backdrop covers
//! the entire terminal (driven by `app.animations.frame_counter`; rendering
//! never mutates game state) and the content panels sit directly on it.

use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::style::{Color, Modifier, Style};
use ratatui::Frame;

use crate::app::{AppState, MenuScreen, OverlayCard, OverlayKind};
use crate::levels::config;
use crate::render::sprites::hash2;

// ===========================================================================
// Logotype
// ===========================================================================

/// Hand-crafted figlet-style logotype: VIMFORGE on one line.
const LOGO_ART: [&str; 5] = [
    r"__     _____ __  __ _____ ___  ____   ____ _____",
    r"\ \   / /_ _|  \/  |  ___/ _ \|  _ \ / ___| ____|",
    r" \ \ / / | || |\/| | |_ | | | | |_) | |  _|  _|",
    r"  \ V /  | || |  | |  _|| |_| |  _ <| |_| | |___",
    r"   \_/  |___|_|  |_|_|   \___/|_| \_\\____|_____|",
];

/// Compact fallback logotype for narrow terminals.
const LOGO_SMALL: [&str; 2] = ["V I M F O R G E", "==============="];

/// Total columns of the big logotype (for gradient span + centering).
const LOGO_WIDTH: usize = 49;

/// Two-tone logotype gradient: forge-cyan steel on the VIM half melting into
/// ember gold across the FORGE half.
fn logo_gradient(col: usize) -> Color {
    let t = (col as f64 / (LOGO_WIDTH - 1) as f64).clamp(0.0, 1.0);
    // Steel cyan (86, 205, 227) -> ember gold (255, 196, 64)
    let r = (86.0 + (255.0 - 86.0) * t) as u8;
    let g = (205.0 + (196.0 - 205.0) * t) as u8;
    let b = (227.0 + (64.0 - 227.0) * t) as u8;
    Color::Rgb(r, g, b)
}

// ===========================================================================
// Campaign acts
// ===========================================================================

/// The six campaign acts (5 levels each): numeral, name.
pub const ACTS: [(&str, &str); 6] = [
    ("I", "SURVIVAL"),
    ("II", "EDITING"),
    ("III", "REPETITION"),
    ("IV", "PRECISION"),
    ("V", "POWER TOOLS"),
    ("VI", "MASTERY"),
];

/// Which act (0..=5) a campaign level belongs to.
pub fn act_of(level: usize) -> usize {
    (level.saturating_sub(1) / 5).min(5)
}

/// Accent color per act.
pub fn act_color(act: usize) -> Color {
    match act {
        0 => Color::Rgb(120, 205, 120), // Survival — green
        1 => Color::Rgb(86, 205, 227),  // Editing — steel cyan
        2 => Color::Rgb(255, 196, 64),  // Repetition — ember gold
        3 => Color::Rgb(240, 145, 80),  // Precision — orange
        4 => Color::Rgb(190, 145, 240), // Power tools — violet
        _ => Color::Rgb(235, 95, 95),   // Mastery — forge red
    }
}

// ===========================================================================
// Shared painting helpers (write straight into the buffer so the animated
// backdrop underneath is preserved wherever we don't paint)
// ===========================================================================

/// Write a string at (x, y), clipped to the area. Multi-width graphemes are
/// handled by ratatui's set_stringn.
fn put(buf: &mut Buffer, area: Rect, x: u16, y: u16, s: &str, style: Style) {
    if y < area.y || y >= area.y + area.height || x >= area.x + area.width {
        return;
    }
    let max = (area.x + area.width - x) as usize;
    buf.set_stringn(x, y, s, max, style);
}

/// Write a string centered on row y.
fn put_centered(buf: &mut Buffer, area: Rect, y: u16, s: &str, style: Style) {
    let len = s.chars().count() as u16;
    let x = area.x + area.width.saturating_sub(len) / 2;
    put(buf, area, x, y, s, style);
}

/// Fill a horizontal band with a background color (keeps glyphs already
/// painted there readable by overwriting them with spaces).
fn fill_band(buf: &mut Buffer, area: Rect, y: u16, x0: u16, w: u16, bg: Color) {
    if y < area.y || y >= area.y + area.height {
        return;
    }
    let end = (x0 + w).min(area.x + area.width);
    for x in x0.max(area.x)..end {
        let cell = &mut buf[(x, y)];
        cell.set_symbol(" ");
        cell.set_bg(bg);
    }
}

/// Fill a whole rect with spaces on a background color.
fn fill_rect(buf: &mut Buffer, area: Rect, rect: Rect, bg: Color) {
    for y in rect.y..rect.y + rect.height {
        fill_band(buf, area, y, rect.x, rect.width, bg);
    }
}

/// Simple word wrap (char-count based; menu text is plain ASCII).
fn wrap_text(s: &str, width: usize) -> Vec<String> {
    let mut lines = Vec::new();
    let mut cur = String::new();
    for word in s.split_whitespace() {
        if !cur.is_empty() && cur.chars().count() + 1 + word.chars().count() > width {
            lines.push(std::mem::take(&mut cur));
        }
        if !cur.is_empty() {
            cur.push(' ');
        }
        cur.push_str(word);
    }
    if !cur.is_empty() {
        lines.push(cur);
    }
    lines
}

// ===========================================================================
// Animated factory backdrop (procedural, full-bleed)
// ===========================================================================

/// Dim palette for the backdrop so foreground panels always win.
const BACKDROP_BG: Color = Color::Rgb(11, 13, 18);
const BACKDROP_BG_ALT: Color = Color::Rgb(13, 15, 21);

/// Paint the animated factory backdrop across the whole area: dithered
/// ground, belt lanes with marching chevrons, machine silhouettes with
/// drifting smoke. Deterministic in (x, y, frame) — no state mutated.
fn render_backdrop(buf: &mut Buffer, area: Rect, frame: u32) {
    if area.width == 0 || area.height == 0 {
        return;
    }
    let f = frame as i32;
    for y in area.y..area.y + area.height {
        let gy = (y - area.y) as i32;
        let lane = gy % 9 == 5; // belt lanes every 9 rows
        for x in area.x..area.x + area.width {
            let gx = (x - area.x) as i32;
            let h = hash2(gx, gy);
            let cell = &mut buf[(x, y)];
            let bg = if (gx / 11 + gy / 5) & 1 == 0 {
                BACKDROP_BG
            } else {
                BACKDROP_BG_ALT
            };
            cell.set_bg(bg);
            if lane {
                // Marching chevrons; every third lane runs the other way.
                let rev = (gy / 9) % 3 == 2;
                let phase = if rev { gx + f / 3 } else { gx - f / 3 };
                if phase.rem_euclid(7) == 0 {
                    cell.set_symbol(if rev { "«" } else { "»" });
                    cell.set_fg(Color::Rgb(52, 62, 82));
                } else {
                    cell.set_symbol("─");
                    cell.set_fg(Color::Rgb(26, 31, 41));
                }
            } else {
                // Two-tone ground dither.
                let g = match h % 23 {
                    0 => "·",
                    1 => ".",
                    2 => "'",
                    3 => "`",
                    _ => " ",
                };
                cell.set_symbol(g);
                cell.set_fg(if h & 8 == 0 {
                    Color::Rgb(30, 35, 45)
                } else {
                    Color::Rgb(23, 27, 36)
                });
            }
        }
    }

    // Machine silhouettes parked above the belt lanes + drifting smoke.
    let sil_fg = Style::default().fg(Color::Rgb(33, 40, 53));
    let smoke_fg = Style::default().fg(Color::Rgb(28, 34, 46));
    let mut gy = 5i32;
    while (gy as u16) < area.height {
        let mut gx = 3i32;
        while (gx as u16) + 9 < area.width {
            if hash2(gx / 13, gy) % 3 == 0 && gy >= 3 {
                let bx = area.x + gx as u16;
                let by = area.y + gy as u16;
                put(buf, area, bx, by - 2, "▄▄▄▄▄", sil_fg);
                put(buf, area, bx, by - 1, "█▓▓▓█", sil_fg);
                // Chimney smoke drifts upward with the frame counter.
                if gy >= 5 {
                    let puff = ((f / 6 + gx) % 3) as u16;
                    let sy = by.saturating_sub(3 + puff % 2);
                    if sy > area.y {
                        put(buf, area, bx + 4, sy, if puff == 0 { "░" } else { "▒" }, smoke_fg);
                    }
                }
            }
            gx += 13;
        }
        gy += 9;
    }
}

/// Full-width footer hint band on the bottom row.
fn render_footer(buf: &mut Buffer, area: Rect, hint: &str) {
    let y = area.y + area.height - 1;
    fill_band(buf, area, y, area.x, area.width, Color::Rgb(19, 23, 32));
    put_centered(
        buf,
        area,
        y,
        hint,
        Style::default().fg(Color::Rgb(130, 140, 160)).bg(Color::Rgb(19, 23, 32)),
    );
}

// ===========================================================================
// Entry point
// ===========================================================================

/// Render the menu (dispatches on the current menu screen).
pub fn render_menu(frame: &mut Frame, frame_size: Rect, app: &AppState) {
    if frame_size.width == 0 || frame_size.height == 0 {
        return;
    }
    match app.menu_screen {
        MenuScreen::Title => render_title(frame, frame_size, app),
        MenuScreen::LevelSelect => render_level_select(frame, frame_size, app),
        MenuScreen::Help => render_help(frame, frame_size, app),
    }
}

// ===========================================================================
// Title screen
// ===========================================================================

fn render_title(frame: &mut Frame, area: Rect, app: &AppState) {
    let buf = frame.buffer_mut();
    render_backdrop(buf, area, app.animations.frame_counter);

    let big = area.height >= 34;
    let mut y = area.y + if big { 3 } else { 1 };

    // Quiet hero band behind the logotype + tagline so backdrop lanes and
    // silhouettes never garble the wordmark.
    let hero_w = (LOGO_WIDTH as u16 + 14).min(area.width);
    let hero_x = area.x + area.width.saturating_sub(hero_w) / 2;
    let hero_h = 5 + 3 + if big { 1 } else { 0 };
    for hy in y.saturating_sub(1)..(y + hero_h).min(area.y + area.height) {
        fill_band(buf, area, hy, hero_x, hero_w, BACKDROP_BG);
    }

    // Logotype with the two-tone gradient (per-column ramp).
    if area.width as usize >= LOGO_WIDTH + 1 {
        for line in &LOGO_ART {
            let len = line.chars().count() as u16;
            let x0 = area.x + area.width.saturating_sub(len) / 2;
            for (i, ch) in line.chars().enumerate() {
                let x = x0 + i as u16;
                if x >= area.x + area.width || y >= area.y + area.height {
                    continue;
                }
                if ch != ' ' {
                    let cell = &mut buf[(x, y)];
                    cell.set_char(ch);
                    cell.set_style(
                        Style::default()
                            .fg(logo_gradient(i))
                            .add_modifier(Modifier::BOLD),
                    );
                }
            }
            y += 1;
        }
    } else {
        for line in &LOGO_SMALL {
            put_centered(
                buf,
                area,
                y,
                line,
                Style::default()
                    .fg(logo_gradient(20))
                    .add_modifier(Modifier::BOLD),
            );
            y += 1;
        }
    }

    y += 1;
    put_centered(
        buf,
        area,
        y,
        "a  v i m - g r a m m a r  f a c t o r y",
        Style::default()
            .fg(Color::Rgb(140, 150, 170))
            .add_modifier(Modifier::ITALIC),
    );
    y += 1;
    put_centered(
        buf,
        area,
        y,
        concat!("v", env!("CARGO_PKG_VERSION")),
        Style::default().fg(Color::Rgb(80, 85, 100)),
    );
    y += if big { 3 } else { 2 };

    // Menu entries on the backdrop (no floating box).
    let entries = app.title_entries();
    let pulse = pulse_level(app.animations.frame_counter);
    let band_w = 46u16.min(area.width.saturating_sub(2));
    let band_x = area.x + area.width.saturating_sub(band_w) / 2;
    let step: u16 = if big { 3 } else { 2 };

    for (i, entry) in entries.iter().enumerate() {
        if y + 1 >= area.y + area.height {
            break;
        }
        let selected = i == app.menu_cursor.min(entries.len() - 1);
        let band_bg = if selected {
            Color::Rgb(26, 32, 44)
        } else {
            Color::Rgb(15, 18, 25)
        };
        fill_band(buf, area, y, band_x, band_w, band_bg);
        if big && !entry.sub.is_empty() {
            fill_band(buf, area, y + 1, band_x, band_w, band_bg);
        }

        // Pulsing '>' accent on the selected row, vim-command style.
        let accent = if selected {
            let a = (140.0 + 115.0 * pulse) as u8;
            Style::default()
                .fg(Color::Rgb(a, (a as f64 * 0.82) as u8, 40))
                .add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(band_bg)
        };
        put(buf, area, band_x + 1, y, if selected { "▶" } else { " " }, accent.bg(band_bg));

        let key_style = Style::default()
            .fg(Color::Rgb(86, 205, 227))
            .bg(band_bg)
            .add_modifier(Modifier::BOLD);
        put(buf, area, band_x + 3, y, &format!("[{}]", entry.key), key_style);

        let name_style = if selected {
            Style::default()
                .fg(Color::Rgb(255, 216, 100))
                .bg(band_bg)
                .add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(Color::Rgb(190, 196, 210)).bg(band_bg)
        };
        put(buf, area, band_x + 8, y, entry.name, name_style);

        let sub_style = Style::default().fg(Color::Rgb(110, 118, 136)).bg(band_bg);
        if !entry.sub.is_empty() {
            if big {
                let max = band_w.saturating_sub(9) as usize;
                let sub: String = entry.sub.chars().take(max).collect();
                put(buf, area, band_x + 8, y + 1, &sub, sub_style);
            } else {
                // Inline sub, clipped to the band (with an ellipsis when it
                // doesn't fit) so it never spills onto the backdrop.
                let x = band_x + 8 + entry.name.chars().count() as u16 + 2;
                let max = (band_x + band_w).saturating_sub(x + 1) as usize;
                let full = format!("· {}", entry.sub);
                let sub: String = if full.chars().count() > max {
                    let mut s: String = full.chars().take(max.saturating_sub(1)).collect();
                    s.push('\u{2026}');
                    s
                } else {
                    full
                };
                put(buf, area, x, y, &sub, sub_style);
            }
        }
        y += step;
    }

    render_footer(buf, area, " j/k move · Enter select · 1/2/3 quick keys · h help · q quit ");
}

/// Pulse brightness 0.0..1.0 from the animation frame counter (~1s period).
fn pulse_level(frame_counter: u32) -> f64 {
    let phase = (frame_counter % 30) as f64 / 30.0 * std::f64::consts::TAU;
    phase.sin() * 0.5 + 0.5
}

// ===========================================================================
// Level select
// ===========================================================================

fn render_level_select(frame: &mut Frame, area: Rect, app: &AppState) {
    let buf = frame.buffer_mut();
    render_backdrop(buf, area, app.animations.frame_counter);

    // Header
    let done = app.campaign_progress();
    fill_band(buf, area, area.y, area.x, area.width, Color::Rgb(17, 21, 29));
    put(
        buf,
        area,
        area.x + 2,
        area.y,
        "CAMPAIGN — LEVEL SELECT",
        Style::default()
            .fg(Color::Rgb(255, 216, 100))
            .bg(Color::Rgb(17, 21, 29))
            .add_modifier(Modifier::BOLD),
    );
    let progress = format!("{done}/30 complete ");
    put(
        buf,
        area,
        area.x + area.width.saturating_sub(progress.chars().count() as u16 + 1),
        area.y,
        &progress,
        Style::default().fg(Color::Rgb(150, 158, 175)).bg(Color::Rgb(17, 21, 29)),
    );

    // Grid geometry: 6 act rows x 5 level tiles, act labels on the left.
    // The whole block is centered in the frame.
    let label_w: u16 = if area.width >= 120 { 18 } else { 5 };
    let avail_w = area.width.saturating_sub(label_w + 2);
    let avail_h = area.height.saturating_sub(4); // header + footer + breathing
    let tile_w = (avail_w / 5).clamp(8, 26);
    let tile_h = (avail_h / 6).clamp(3, 6);
    let grid_x = area.x + label_w + area.width.saturating_sub(label_w + 5 * tile_w) / 2;
    let grid_y = area.y + 2 + avail_h.saturating_sub(6 * tile_h) / 2;
    let furthest = app.furthest_campaign_level();
    let cursor = app.menu_cursor.min(29);

    for row in 0..6usize {
        let ty = grid_y + row as u16 * tile_h;
        if ty + tile_h > area.y + area.height {
            break;
        }
        // Act label as the row header, on a quiet band so backdrop belt
        // lanes never garble it.
        let (numeral, act_name) = ACTS[row];
        let label = if label_w > 6 {
            format!("{numeral:>3}  {act_name}")
        } else {
            format!("{numeral:>3}")
        };
        let label_x = grid_x.saturating_sub(label_w);
        let label_y = ty + tile_h / 2;
        fill_band(
            buf,
            area,
            label_y,
            label_x.saturating_sub(1),
            label.chars().count() as u16 + 2,
            BACKDROP_BG,
        );
        put(
            buf,
            area,
            label_x,
            label_y,
            &label,
            Style::default()
                .fg(act_color(row))
                .bg(BACKDROP_BG)
                .add_modifier(Modifier::BOLD),
        );

        for col in 0..5usize {
            let level = row * 5 + col + 1;
            let tx = grid_x + col as u16 * tile_w;
            render_level_tile(
                buf,
                area,
                Rect::new(tx, ty, tile_w.saturating_sub(1), tile_h.saturating_sub(0)),
                level,
                app.campaign_completed.contains(&level),
                level == furthest + 1,
                level > furthest + 1,
                level == cursor + 1,
            );
        }
    }

    // Footer: info for the selected tile.
    let sel_level = cursor + 1;
    let name = config::get_level(sel_level).map(|c| c.name).unwrap_or("");
    let hint = if sel_level > furthest + 1 {
        format!(" LEVEL {sel_level} — locked · complete level {} first · hjkl move · Esc back ", sel_level.saturating_sub(1))
    } else {
        format!(" LEVEL {sel_level} — {name} · Enter start · hjkl move · Esc back ")
    };
    render_footer(buf, area, &hint);
}

#[allow(clippy::too_many_arguments)]
fn render_level_tile(
    buf: &mut Buffer,
    area: Rect,
    rect: Rect,
    level: usize,
    complete: bool,
    next: bool,
    locked: bool,
    selected: bool,
) {
    if rect.width < 6 || rect.height < 3 {
        return;
    }
    let act = act_of(level);
    let accent = act_color(act);
    let dim = Color::Rgb(55, 61, 75);
    let border_fg = if selected {
        Color::Rgb(245, 245, 250)
    } else if locked {
        Color::Rgb(40, 45, 57)
    } else {
        accent
    };
    let bg = if selected {
        Color::Rgb(26, 32, 44)
    } else {
        Color::Rgb(15, 18, 25)
    };
    fill_rect(buf, area, rect, bg);

    let bstyle = Style::default().fg(border_fg).bg(bg);
    let w = rect.width as usize;
    // Border (rounded corners).
    put(buf, area, rect.x, rect.y, &format!("╭{}╮", "─".repeat(w.saturating_sub(2))), bstyle);
    for yy in rect.y + 1..rect.y + rect.height - 1 {
        put(buf, area, rect.x, yy, "│", bstyle);
        put(buf, area, rect.x + rect.width - 1, yy, "│", bstyle);
    }
    put(
        buf,
        area,
        rect.x,
        rect.y + rect.height - 1,
        &format!("╰{}╯", "─".repeat(w.saturating_sub(2))),
        bstyle,
    );

    // Row 1: number + status glyph.
    // NOTE: single-width glyphs only. The spec sketched 🔒 for locked tiles,
    // but wide emoji leave a skipped buffer cell behind (breaking the
    // zero-default-cells full-bleed guarantee) and render inconsistently
    // across terminals; '×' keeps the grid airtight.
    let (glyph, gcolor) = if complete {
        ("✓", Color::Rgb(120, 205, 120))
    } else if locked {
        ("×", dim)
    } else if next {
        ("▶", accent)
    } else {
        ("·", Color::Rgb(150, 158, 175))
    };
    let num_style = if locked {
        Style::default().fg(dim).bg(bg)
    } else {
        Style::default()
            .fg(if selected { Color::Rgb(255, 216, 100) } else { accent })
            .bg(bg)
            .add_modifier(Modifier::BOLD)
    };
    put(buf, area, rect.x + 2, rect.y + 1, &format!("{level:02}"), num_style);
    put(
        buf,
        area,
        rect.x + 5,
        rect.y + 1,
        glyph,
        Style::default().fg(gcolor).bg(bg).add_modifier(Modifier::BOLD),
    );

    // Name: own row when the tile is tall enough, else inline after number.
    let name = if locked {
        "· · ·".to_string()
    } else {
        config::get_level(level).map(|c| c.name.to_string()).unwrap_or_default()
    };
    let name_style = Style::default()
        .fg(if locked {
            dim
        } else if selected {
            Color::Rgb(235, 238, 245)
        } else {
            Color::Rgb(165, 172, 188)
        })
        .bg(bg);
    let name_max = rect.width.saturating_sub(3) as usize;
    let name_trunc: String = if name.chars().count() > name_max {
        let mut s: String = name.chars().take(name_max.saturating_sub(1)).collect();
        s.push('…');
        s
    } else {
        name
    };
    if rect.height >= 4 {
        put(buf, area, rect.x + 2, rect.y + 2, &name_trunc, name_style);
    } else {
        let x = rect.x + 8;
        let max = rect.width.saturating_sub(9) as usize;
        let inline: String = name_trunc.chars().take(max).collect();
        put(buf, area, x, rect.y + 1, &inline, name_style);
    }
}

// ===========================================================================
// Help screen
// ===========================================================================

fn render_help(frame: &mut Frame, area: Rect, app: &AppState) {
    let buf = frame.buffer_mut();
    render_backdrop(buf, area, app.animations.frame_counter);

    let panel_w = 62u16.min(area.width.saturating_sub(2));
    let lines: [(&str, &str); 14] = [
        ("VIMFORGE", "a factory game you play with vim grammar"),
        ("", ""),
        ("h j k l", "move the cursor (this is the whole game)"),
        ("i", "insert mode: place buildings (c belt, s smelter …)"),
        ("x d c y p", "delete / operators / yank / paste — with motions"),
        ("w b f /", "jump words, find entities, search"),
        ("v V Ctrl-v", "visual selections; . repeats; q records macros"),
        ("zi zo zf", "zoom the factory view in / out / fit"),
        ("u Ctrl-r", "undo / redo any edit"),
        (":w :q", "save / quit · :menu returns to this menu"),
        ("", ""),
        ("Campaign", "30 levels that teach vim one motion at a time"),
        ("Sandbox", "open factory: economy, research, contracts"),
        ("", ""),
    ];
    let panel_h = (lines.len() as u16 + 4).min(area.height.saturating_sub(2));
    let px = area.x + area.width.saturating_sub(panel_w) / 2;
    let py = area.y + area.height.saturating_sub(panel_h) / 2;
    let panel = Rect::new(px, py, panel_w, panel_h);
    fill_rect(buf, area, panel, Color::Rgb(15, 18, 25));

    put_centered(
        buf,
        area,
        py + 1,
        "H E L P",
        Style::default()
            .fg(Color::Rgb(255, 216, 100))
            .bg(Color::Rgb(15, 18, 25))
            .add_modifier(Modifier::BOLD),
    );
    let key_style = Style::default()
        .fg(Color::Rgb(86, 205, 227))
        .bg(Color::Rgb(15, 18, 25))
        .add_modifier(Modifier::BOLD);
    let text_style = Style::default().fg(Color::Rgb(165, 172, 188)).bg(Color::Rgb(15, 18, 25));
    for (i, (keys, desc)) in lines.iter().enumerate() {
        let y = py + 3 + i as u16;
        if y >= py + panel_h.saturating_sub(1) {
            break;
        }
        put(buf, area, px + 3, y, keys, key_style);
        put(buf, area, px + 15, y, desc, text_style);
    }

    render_footer(buf, area, " Esc back ");
}

// ===========================================================================
// Overlay cards (level intro / complete / finale) — rendered LAST in main.rs
// ===========================================================================

/// Render the active overlay card centered over the whole frame.
/// FALL-THROUGH: the card is display-only; keys dismiss it in the session
/// and still execute, so this never traps input.
pub fn render_overlay_card(
    frame: &mut Frame,
    frame_size: Rect,
    _app: &AppState,
    card: &OverlayCard,
) {
    if frame_size.width < 30 || frame_size.height < 10 {
        return;
    }
    let buf = frame.buffer_mut();
    let act = act_of(card.level);
    let accent = match card.kind {
        OverlayKind::LevelIntro => act_color(act),
        OverlayKind::LevelComplete => Color::Rgb(120, 205, 120),
        OverlayKind::CampaignFinale => Color::Rgb(255, 196, 64),
    };
    let bg = Color::Rgb(16, 18, 26);

    // Build the content lines first so the card sizes to fit.
    let mut rows: Vec<(String, Style)> = Vec::new();
    let title_style = Style::default().fg(accent).bg(bg).add_modifier(Modifier::BOLD);
    let text = Style::default().fg(Color::Rgb(180, 186, 200)).bg(bg);
    let dim = Style::default().fg(Color::Rgb(110, 118, 136)).bg(bg);
    let gold = Style::default()
        .fg(Color::Rgb(255, 216, 100))
        .bg(bg)
        .add_modifier(Modifier::BOLD);

    let cfg = config::get_level(card.level);
    let name = cfg.as_ref().map(|c| c.name).unwrap_or("");
    let inner_w = 56usize;

    match card.kind {
        OverlayKind::LevelIntro => {
            let (numeral, act_name) = ACTS[act];
            rows.push((format!("ACT {numeral} — {act_name}"), dim));
            rows.push((String::new(), text));
            rows.push((
                spaced(&format!("LEVEL {}", card.level)),
                title_style,
            ));
            rows.push((name.to_uppercase(), gold));
            rows.push((String::new(), text));
            let learn = cfg
                .as_ref()
                .and_then(|c| c.allowed_commands.as_ref())
                .map(|cmds| {
                    let mut keys: Vec<&str> = cmds.iter().copied().take(12).collect();
                    if cmds.len() > 12 {
                        keys.push("…");
                    }
                    keys.join("  ")
                });
            match learn {
                Some(keys) => {
                    rows.push(("You will learn:".to_string(), text));
                    rows.push((keys, gold));
                }
                None => rows.push(("Everything you know is allowed.".to_string(), text)),
            }
            rows.push((String::new(), text));
            if let Some(ref c) = cfg {
                for l in wrap_text(c.objective, inner_w).into_iter().take(2) {
                    rows.push((l, dim));
                }
            }
        }
        OverlayKind::LevelComplete => {
            rows.push((spaced(&format!("LEVEL {} COMPLETE", card.level)), title_style));
            rows.push((name.to_string(), gold));
            rows.push((String::new(), text));
            rows.push((
                format!("edits: {} · output delivered: {}", card.edits, card.output),
                text,
            ));
            let next = card.level + 1;
            if let Some(nc) = config::get_level(next) {
                if next <= 30 {
                    rows.push((String::new(), text));
                    rows.push((format!("next: LEVEL {next} — {}", nc.name), text));
                }
            }
        }
        OverlayKind::CampaignFinale => {
            rows.push(("★  ★  ★".to_string(), gold));
            rows.push((spaced("CAMPAIGN COMPLETE"), title_style));
            rows.push((String::new(), text));
            rows.push(("All 30 levels mastered — you know vim.".to_string(), text));
            rows.push((
                format!("final level: {} edits · {} delivered", card.edits, card.output),
                dim,
            ));
            rows.push((String::new(), text));
            rows.push(("The Sandbox is open: :freeplay or the menu.".to_string(), text));
        }
    }

    let card_w = ((inner_w + 6) as u16).min(frame_size.width.saturating_sub(4));
    let card_h = (rows.len() as u16 + 6).min(frame_size.height.saturating_sub(2));
    let cx = frame_size.x + frame_size.width.saturating_sub(card_w) / 2;
    let cy = frame_size.y + frame_size.height.saturating_sub(card_h) / 2;
    let rect = Rect::new(cx, cy, card_w, card_h);
    fill_rect(buf, frame_size, rect, bg);

    // Double border in the accent color.
    let bstyle = Style::default().fg(accent).bg(bg);
    let w = card_w as usize;
    put(buf, frame_size, cx, cy, &format!("╔{}╗", "═".repeat(w.saturating_sub(2))), bstyle);
    for yy in cy + 1..cy + card_h - 1 {
        put(buf, frame_size, cx, yy, "║", bstyle);
        put(buf, frame_size, cx + card_w - 1, yy, "║", bstyle);
    }
    put(
        buf,
        frame_size,
        cx,
        cy + card_h - 1,
        &format!("╚{}╝", "═".repeat(w.saturating_sub(2))),
        bstyle,
    );

    // Content, centered inside the card.
    let inner = Rect::new(cx + 1, cy, card_w.saturating_sub(2), card_h);
    for (i, (line, style)) in rows.iter().enumerate() {
        let y = cy + 2 + i as u16;
        if y >= cy + card_h.saturating_sub(2) {
            break;
        }
        put_centered(buf, inner, y, line, *style);
    }
    put_centered(
        buf,
        inner,
        cy + card_h - 2,
        "— press any key —",
        dim.add_modifier(Modifier::ITALIC),
    );
}

/// Space out letters for "big" headline text: "LEVEL 3" -> "L E V E L  3".
fn spaced(s: &str) -> String {
    let mut out = String::new();
    for (i, ch) in s.chars().enumerate() {
        if i > 0 {
            out.push(' ');
        }
        out.push(ch);
    }
    out
}
