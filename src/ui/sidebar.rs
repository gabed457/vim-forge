use ratatui::layout::Rect;
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, BorderType, Borders, Paragraph};
use ratatui::Frame;

use crate::app::{AppState, Mode};
use crate::ecs::components::{EntityKind, MultiTile, OutputCounter, Position};
use crate::render::viewport::Viewport;
use crate::resources::EntityType;

/// Dock panel background (its own bed, distinct from the grid).
const DOCK_BG: Color = Color::Rgb(16, 20, 28);
/// Dock border color.
const DOCK_BORDER: Color = Color::Rgb(58, 72, 98);
/// Label color for section rules.
const SECTION_FG: Color = Color::Rgb(220, 200, 60);
/// Dim rule color for section separators.
const RULE_FG: Color = Color::Rgb(52, 62, 84);
/// Muted label text.
const LABEL_FG: Color = Color::Rgb(140, 145, 160);
/// Empty (but in-bounds) minimap ground color.
const MINI_GROUND: (u8, u8, u8) = (27, 34, 45);
/// Viewport rectangle outline color on the minimap.
const MINI_VIEWPORT: (u8, u8, u8) = (96, 156, 216);
/// Cursor blink color on the minimap.
const MINI_CURSOR: (u8, u8, u8) = (255, 235, 130);

/// Back-compatible entry point: renders the Command Dock without the
/// viewport rectangle on the minimap (callers that only have `&AppState`).
pub fn render_sidebar(frame: &mut Frame, area: Rect, app: &AppState) {
    render_command_dock(frame, area, app, None);
}

/// Render the Command Dock: a bordered panel with its own background.
///
/// Top-to-bottom: MINIMAP (dominant-entity colored half-blocks, viewport
/// rectangle, blinking cursor dot), compact stats (output / cash / R&D bar /
/// pollution / power), then a mode-sensitive context-keys panel.
pub fn render_command_dock(
    frame: &mut Frame,
    area: Rect,
    app: &AppState,
    viewport: Option<&Viewport>,
) {
    let block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(DOCK_BORDER).bg(DOCK_BG))
        .style(Style::default().bg(DOCK_BG))
        .title(Span::styled(
            " VimForge ",
            Style::default()
                .fg(Color::Rgb(255, 200, 60))
                .add_modifier(Modifier::BOLD),
        ));

    let inner = block.inner(area);
    frame.render_widget(block, area);

    if inner.width == 0 || inner.height == 0 {
        return;
    }

    let mut lines: Vec<Line> = Vec::new();

    // ---- MINIMAP ----
    lines.push(section_rule(inner.width, "MAP"));
    lines.extend(minimap_lines(app, viewport, inner));

    // ---- Compact stats ----
    lines.push(Line::from(""));
    lines.push(section_rule(inner.width, "STATS"));
    stats_lines(app, inner.width, &mut lines);

    // ---- Context keys (mode-sensitive) ----
    lines.push(Line::from(""));
    let (mode_name, keys) = context_keys(app);
    lines.push(section_rule(inner.width, &format!("KEYS \u{00B7} {}", mode_name)));
    let key_style = Style::default()
        .fg(Color::Rgb(255, 216, 100))
        .bg(DOCK_BG)
        .add_modifier(Modifier::BOLD);
    let desc_style = Style::default().fg(Color::Rgb(170, 175, 190)).bg(DOCK_BG);
    for (key, desc) in keys {
        lines.push(Line::from(vec![
            Span::styled(format!(" {:<8}", key), key_style),
            Span::styled(desc.to_string(), desc_style),
        ]));
    }

    lines.truncate(inner.height as usize);
    let paragraph = Paragraph::new(lines).style(Style::default().bg(DOCK_BG));
    frame.render_widget(paragraph, inner);
}

// ---------------------------------------------------------------------------
// Minimap
// ---------------------------------------------------------------------------

/// How many distinct entity types a single minimap cell tracks for dominance.
const MINI_SLOTS: usize = 4;

fn div_ceil(a: usize, b: usize) -> usize {
    a.div_ceil(b.max(1))
}

/// Build the minimap lines. Each minimap cell covers NxN map tiles where
/// N = ceil(map_extent / available); each terminal cell shows two minimap
/// rows via half-blocks (`▀` fg = upper row, bg = lower row).
fn minimap_lines<'a>(app: &AppState, viewport: Option<&Viewport>, inner: Rect) -> Vec<Line<'a>> {
    let map_w = app.map.width.max(1);
    let map_h = app.map.height.max(1);

    // Budget: full inner width minus 1 col padding either side; roughly the
    // top third of the dock in rows (at least 3 rows, at most 12).
    let avail_w = (inner.width.saturating_sub(2)).max(1) as usize;
    let max_char_rows = ((inner.height as usize / 3).max(3)).min(12);
    let avail_h = max_char_rows * 2;

    let n = div_ceil(map_w, avail_w)
        .max(div_ceil(map_h, avail_h))
        .max(1);
    let mw = div_ceil(map_w, n);
    let mh = div_ceil(map_h, n);

    // Dominant-entity color per minimap cell. One pass over the ECS world
    // (entities only, not tiles): count per-type occupancy in up to
    // MINI_SLOTS slots per cell, multi-tile footprints included.
    let mut slots: Vec<[(Option<EntityType>, u16); MINI_SLOTS]> =
        vec![[(None, 0); MINI_SLOTS]; mw * mh];
    for (_e, (pos, kind, multi)) in app
        .world
        .query::<(&Position, &EntityKind, Option<&MultiTile>)>()
        .iter()
    {
        let (fw, fh) = multi.map(|m| (m.width, m.height)).unwrap_or((1, 1));
        for ty in pos.y..(pos.y + fh).min(map_h) {
            for tx in pos.x..(pos.x + fw).min(map_w) {
                let idx = (ty / n) * mw + (tx / n);
                let cell = &mut slots[idx];
                let mut placed = false;
                for slot in cell.iter_mut() {
                    match slot.0 {
                        Some(t) if t == kind.kind => {
                            slot.1 += 1;
                            placed = true;
                            break;
                        }
                        None => {
                            *slot = (Some(kind.kind), 1);
                            placed = true;
                            break;
                        }
                        _ => {}
                    }
                }
                if !placed {
                    // Slots full: fold into the first (rare, keeps it O(1)).
                    cell[0].1 += 1;
                }
            }
        }
    }

    // Resolve each cell to a color: dominant entity type's palette color.
    let mut colors: Vec<(u8, u8, u8)> = Vec::with_capacity(mw * mh);
    for cell in &slots {
        let mut best: Option<(EntityType, u16)> = None;
        for (t, count) in cell.iter() {
            if let Some(t) = t {
                if best.map(|(_, bc)| *count > bc).unwrap_or(true) {
                    best = Some((*t, *count));
                }
            }
        }
        colors.push(best.map(|(t, _)| t.color()).unwrap_or(MINI_GROUND));
    }

    // Viewport rectangle outline (only when the viewport shows a sub-region).
    if let Some(vp) = viewport {
        if vp.width < map_w || vp.height < map_h {
            let vx0 = (vp.offset_x / n).min(mw - 1);
            let vy0 = (vp.offset_y / n).min(mh - 1);
            let vx1 = ((vp.offset_x + vp.width.max(1) - 1) / n).min(mw - 1);
            let vy1 = ((vp.offset_y + vp.height.max(1) - 1) / n).min(mh - 1);
            for my in vy0..=vy1 {
                for mx in vx0..=vx1 {
                    let on_border = my == vy0 || my == vy1 || mx == vx0 || mx == vx1;
                    if on_border {
                        colors[my * mw + mx] = MINI_VIEWPORT;
                    }
                }
            }
        }
    }

    // Blinking cursor dot (frame_counter driven; rendering stays read-only).
    if (app.animations.frame_counter / 6) % 2 == 0 {
        let cx = (app.cursor_x / n).min(mw - 1);
        let cy = (app.cursor_y / n).min(mh - 1);
        colors[cy * mw + cx] = MINI_CURSOR;
    }

    // Emit half-block lines, centered horizontally.
    let pad = (inner.width as usize).saturating_sub(mw) / 2;
    let mut lines: Vec<Line> = Vec::new();
    for row_pair in 0..div_ceil(mh, 2) {
        let upper_y = row_pair * 2;
        let lower_y = upper_y + 1;
        let mut spans: Vec<Span> = Vec::with_capacity(mw + 1);
        if pad > 0 {
            spans.push(Span::styled(
                " ".repeat(pad),
                Style::default().bg(DOCK_BG),
            ));
        }
        for mx in 0..mw {
            let up = colors[upper_y * mw + mx];
            let lo = if lower_y < mh {
                colors[lower_y * mw + mx]
            } else {
                // Below the map: blend into the dock background.
                (16, 20, 28)
            };
            spans.push(Span::styled(
                "\u{2580}", // ▀ upper half block: fg = upper cell, bg = lower cell
                Style::default()
                    .fg(Color::Rgb(up.0, up.1, up.2))
                    .bg(Color::Rgb(lo.0, lo.1, lo.2)),
            ));
        }
        lines.push(Line::from(spans));
    }
    lines
}

// ---------------------------------------------------------------------------
// Compact stats
// ---------------------------------------------------------------------------

fn stats_lines(app: &AppState, width: u16, lines: &mut Vec<Line<'static>>) {
    let label = Style::default().fg(LABEL_FG).bg(DOCK_BG);

    // Output counts on one compact line.
    let (ore, ingots, widgets) = total_output_counts(app);
    lines.push(Line::from(vec![
        Span::styled(" Out  ", label),
        Span::styled(
            format!("W:{} ", widgets),
            Style::default()
                .fg(Color::Rgb(80, 220, 80))
                .bg(DOCK_BG)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled(
            format!("I:{} ", ingots),
            Style::default().fg(Color::Rgb(220, 220, 220)).bg(DOCK_BG),
        ),
        Span::styled(
            format!("O:{}", ore),
            Style::default().fg(Color::Rgb(180, 140, 60)).bg(DOCK_BG),
        ),
    ]));

    // Cash (with the tutorial-sentinel ∞).
    let cash_text = if app.economy.cash.abs() >= 1_000_000_000_000 {
        "$\u{221E}".to_string()
    } else {
        format!("${}", app.economy.cash)
    };
    lines.push(Line::from(vec![
        Span::styled(" Cash ", label),
        Span::styled(
            cash_text,
            Style::default()
                .fg(Color::Rgb(120, 220, 130))
                .bg(DOCK_BG)
                .add_modifier(Modifier::BOLD),
        ),
    ]));

    // R&D progress bar.
    let rd_style = Style::default().fg(Color::Rgb(80, 200, 220)).bg(DOCK_BG);
    match app.research.current {
        Some(id) => {
            let tech = crate::research::tree::get_tech(id);
            let frac = app.research.progress_fraction().clamp(0.0, 1.0);
            let bar_w = (width as usize).saturating_sub(14).clamp(4, 12);
            let filled = ((frac * bar_w as f64).round() as usize).min(bar_w);
            // Truncate long tech names with an ellipsis instead of a hard
            // mid-word cut.
            let name_max = (width as usize).saturating_sub(8).max(4);
            let name: String = if tech.name.chars().count() > name_max {
                let mut s: String =
                    tech.name.chars().take(name_max.saturating_sub(1)).collect();
                s.push('\u{2026}');
                s
            } else {
                tech.name.to_string()
            };
            lines.push(Line::from(vec![
                Span::styled(" R&D  ", label),
                Span::styled(name, rd_style),
            ]));
            lines.push(Line::from(vec![
                Span::styled("      ", label),
                Span::styled("\u{25B0}".repeat(filled), rd_style),
                Span::styled(
                    "\u{25B1}".repeat(bar_w - filled),
                    Style::default().fg(RULE_FG).bg(DOCK_BG),
                ),
                Span::styled(format!(" {:.0}%", frac * 100.0), rd_style),
            ]));
        }
        None => {
            lines.push(Line::from(vec![
                Span::styled(" R&D  ", label),
                Span::styled("idle", Style::default().fg(Color::Rgb(90, 90, 100)).bg(DOCK_BG)),
            ]));
        }
    }

    // Pollution.
    let pollution = app.pollution.level;
    let pollution_color = if pollution >= 500.0 {
        Color::Rgb(220, 60, 60)
    } else if pollution >= 200.0 {
        Color::Rgb(220, 180, 60)
    } else {
        Color::Rgb(120, 180, 120)
    };
    lines.push(Line::from(vec![
        Span::styled(" Poll ", label),
        Span::styled(
            format!("{:.0}", pollution),
            Style::default().fg(pollution_color).bg(DOCK_BG),
        ),
    ]));

    // Power (MW), only when generators exist.
    let power = &app.simulation.last_report;
    if power.generators_present > 0 {
        let power_color = if power.powered {
            Color::Rgb(60, 220, 60)
        } else {
            Color::Rgb(220, 60, 60)
        };
        lines.push(Line::from(vec![
            Span::styled(" Pwr  ", label),
            Span::styled(
                format!("{:.0}/{:.0}MW", power.power_demand, power.power_supply),
                Style::default().fg(power_color).bg(DOCK_BG),
            ),
        ]));
    }
}

// ---------------------------------------------------------------------------
// Context keys panel
// ---------------------------------------------------------------------------

/// Mode-sensitive key hints: quick-place keys in INSERT, movement/editing
/// basics in NORMAL, selection keys in the visual modes.
fn context_keys(app: &AppState) -> (&'static str, Vec<(&'static str, &'static str)>) {
    match app.mode {
        Mode::Insert => (
            "INSERT",
            vec![
                ("c", "belt"),
                ("s", "smelter"),
                ("a", "assembler"),
                ("p", "splitter"),
                ("e", "merger"),
                ("w", "wall"),
                ("Esc", "back to NORMAL"),
            ],
        ),
        Mode::Replace => (
            "REPLACE",
            vec![("c s a", "swap building"), ("Esc", "back to NORMAL")],
        ),
        Mode::Visual | Mode::VisualLine | Mode::VisualBlock => (
            "VISUAL",
            vec![
                ("hjkl", "grow selection"),
                ("y", "yank"),
                ("d", "delete"),
                ("Esc", "cancel"),
            ],
        ),
        Mode::Command | Mode::Search => (
            "CMD",
            vec![("Enter", "run"), ("Esc", "cancel")],
        ),
        _ => (
            "NORMAL",
            vec![
                ("h j k l", "move"),
                ("w b", "jump machine"),
                ("f", "find entity"),
                ("i", "insert (build)"),
                ("u", "undo"),
            ],
        ),
    }
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// A section separator rule: `─ LABEL ────────` in dim box-drawing with a
/// bright label. No bare `│` separators anywhere.
fn section_rule<'a>(width: u16, label: &str) -> Line<'a> {
    let rule_style = Style::default().fg(RULE_FG).bg(DOCK_BG);
    let label_style = Style::default()
        .fg(SECTION_FG)
        .bg(DOCK_BG)
        .add_modifier(Modifier::BOLD);
    let used = label.chars().count() + 3; // "─ " + label + " "
    let tail = (width as usize).saturating_sub(used);
    Line::from(vec![
        Span::styled("\u{2500} ", rule_style),
        Span::styled(label.to_string(), label_style),
        Span::styled(format!(" {}", "\u{2500}".repeat(tail)), rule_style),
    ])
}

/// Sum up all OutputCounter components in the world.
fn total_output_counts(app: &AppState) -> (u64, u64, u64) {
    let mut ore = 0u64;
    let mut ingot = 0u64;
    let mut widget = 0u64;
    for (_entity, counter) in app.world.query::<&OutputCounter>().iter() {
        ore += counter.ore_count();
        ingot += counter.ingot_count();
        widget += counter.widget_count();
    }
    (ore, ingot, widget)
}
