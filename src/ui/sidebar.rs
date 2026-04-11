use ratatui::layout::Rect;
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Paragraph};
use ratatui::Frame;

use crate::app::AppState;
use crate::ecs::components::{EntityKind, FacingComponent, OutputCounter, PartOfBuilding};
use crate::render::glyphs;
use crate::resources::{Facing, Resource};

/// Render the sidebar widget into the given area.
///
/// The sidebar is 16 columns wide with a left border (`|`).
/// Contents:
/// - Title: VimForge
/// - Resource counts (Widgets/Ingots/Ore produced)
/// - Economy summary (cash, debt, P/L)
/// - Power info
/// - Pollution
/// - Tick count and simulation speed
/// - Day/night indicator
/// - Recent registers (up to 5)
/// - Set marks
pub fn render_sidebar(frame: &mut Frame, area: Rect, app: &AppState) {
    let block = Block::default()
        .borders(Borders::LEFT)
        .border_style(Style::default().fg(Color::Rgb(60, 60, 70)))
        .title(Span::styled(
            " VimForge ",
            Style::default()
                .fg(Color::Rgb(80, 200, 220))
                .add_modifier(Modifier::BOLD),
        ));

    let inner = block.inner(area);
    frame.render_widget(block, area);

    if inner.width == 0 || inner.height == 0 {
        return;
    }

    let mut lines: Vec<Line> = Vec::new();

    // -- Resource counts --
    let (ore_total, ingot_total, widget_total) = total_output_counts(app);

    lines.push(section_header("Output"));
    lines.push(Line::from(vec![
        Span::styled("Widgets: ", Style::default().fg(Color::Rgb(140, 140, 140))),
        Span::styled(
            format!("{}", widget_total),
            Style::default()
                .fg(Color::Rgb(80, 220, 80))
                .add_modifier(Modifier::BOLD),
        ),
    ]));
    lines.push(Line::from(vec![
        Span::styled("Ingots:  ", Style::default().fg(Color::Rgb(140, 140, 140))),
        Span::styled(
            format!("{}", ingot_total),
            Style::default().fg(Color::Rgb(220, 220, 220)),
        ),
    ]));
    lines.push(Line::from(vec![
        Span::styled("Ore:     ", Style::default().fg(Color::Rgb(140, 140, 140))),
        Span::styled(
            format!("{}", ore_total),
            Style::default().fg(Color::Rgb(180, 140, 60)),
        ),
    ]));

    // -- Inventory (only if non-empty) --
    if app.inventory.total() > 0 {
        lines.push(Line::from(""));
        lines.push(section_header("Inventory"));
        let widget_inv = app.inventory.get(Resource::CircuitBoard);
        let ingot_inv = app.inventory.get(Resource::IronIngot);
        let ore_inv = app.inventory.get(Resource::IronOre);
        if widget_inv > 0 {
            lines.push(Line::from(vec![
                Span::styled("Widgets: ", Style::default().fg(Color::Rgb(140, 140, 140))),
                Span::styled(
                    format!("{}", widget_inv),
                    Style::default()
                        .fg(Color::Rgb(80, 220, 80))
                        .add_modifier(Modifier::BOLD),
                ),
            ]));
        }
        if ingot_inv > 0 {
            lines.push(Line::from(vec![
                Span::styled("Ingots:  ", Style::default().fg(Color::Rgb(140, 140, 140))),
                Span::styled(
                    format!("{}", ingot_inv),
                    Style::default().fg(Color::Rgb(220, 220, 220)),
                ),
            ]));
        }
        if ore_inv > 0 {
            lines.push(Line::from(vec![
                Span::styled("Ore:     ", Style::default().fg(Color::Rgb(140, 140, 140))),
                Span::styled(
                    format!("{}", ore_inv),
                    Style::default().fg(Color::Rgb(180, 140, 60)),
                ),
            ]));
        }
    }

    lines.push(Line::from(""));

    // -- Tick / speed --
    lines.push(section_header("Sim"));
    lines.push(Line::from(vec![
        Span::styled("Tick: ", Style::default().fg(Color::Rgb(140, 140, 140))),
        Span::styled(
            format!("{}", app.simulation.tick_count),
            Style::default().fg(Color::Rgb(220, 220, 220)),
        ),
    ]));
    let speed_text = if app.simulation.is_paused() {
        "PAUSED".to_string()
    } else {
        format!("{}x", app.simulation.speed)
    };
    let speed_color = if app.simulation.is_paused() {
        Color::Rgb(220, 60, 60)
    } else {
        Color::Rgb(60, 220, 60)
    };
    lines.push(Line::from(vec![
        Span::styled("Speed: ", Style::default().fg(Color::Rgb(140, 140, 140))),
        Span::styled(speed_text, Style::default().fg(speed_color)),
    ]));

    lines.push(Line::from(""));

    // -- Registers (up to 5 most recent) --
    let reg_list = app.registers.list();
    if !reg_list.is_empty() {
        lines.push(section_header("Regs"));
        for (name, content) in reg_list.iter().take(5) {
            let display = if content.len() > 10 {
                format!("{}...", &content[..10])
            } else {
                content.clone()
            };
            lines.push(Line::from(vec![
                Span::styled(
                    format!("{} ", name),
                    Style::default()
                        .fg(Color::Rgb(80, 200, 220))
                        .add_modifier(Modifier::BOLD),
                ),
                Span::styled(display, Style::default().fg(Color::Rgb(140, 140, 140))),
            ]));
        }
        lines.push(Line::from(""));
    }

    // -- Building info (when cursor is on a building) --
    if let Some(ent) = app.map.entity_at(app.cursor_x, app.cursor_y) {
        // Resolve anchor for multi-tile buildings
        let anchor = if let Ok(pob) = app.world.get::<&PartOfBuilding>(ent) {
            pob.anchor
        } else {
            ent
        };

        if let Ok(ek) = app.world.get::<&EntityKind>(anchor) {
            let entity_type = ek.kind;
            let facing = app
                .world
                .get::<&FacingComponent>(anchor)
                .map(|f| f.facing)
                .unwrap_or(Facing::Right);
            let (fr, fg, fb) = glyphs::building_fg(entity_type);
            let fg_style = Style::default()
                .fg(Color::Rgb(fr, fg, fb))
                .add_modifier(Modifier::BOLD);

            lines.push(Line::from(""));
            lines.push(section_header("Building"));
            lines.push(Line::from(Span::styled(
                entity_type.name(),
                fg_style,
            )));

            // Show ASCII art preview (static, all tile rows)
            let art = glyphs::building_art(entity_type);
            for idx in 0..art.rows.len() {
                let [c0, c1] = glyphs::entity_art(entity_type, facing, idx);
                lines.push(Line::from(Span::styled(
                    format!(" {}{}", c0, c1),
                    fg_style,
                )));
            }
        }
    }

    lines.push(Line::from(""));

    // -- Marks --
    let mark_list = app.marks.list();
    if !mark_list.is_empty() {
        lines.push(section_header("Marks"));
        for (ch, x, y) in mark_list.iter().take(8) {
            lines.push(Line::from(vec![
                Span::styled(
                    format!("'{}' ", ch),
                    Style::default()
                        .fg(Color::Rgb(200, 100, 200))
                        .add_modifier(Modifier::BOLD),
                ),
                Span::styled(
                    format!("[{},{}]", x, y),
                    Style::default().fg(Color::Rgb(140, 140, 140)),
                ),
            ]));
        }
    }

    // Truncate to available height
    lines.truncate(inner.height as usize);

    let paragraph = Paragraph::new(lines);
    frame.render_widget(paragraph, inner);
}

/// Section header with yellow bold text using Rgb.
fn section_header<'a>(text: &str) -> Line<'a> {
    Line::from(Span::styled(
        format!("-- {} --", text),
        Style::default()
            .fg(Color::Rgb(220, 200, 60))
            .add_modifier(Modifier::BOLD),
    ))
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
