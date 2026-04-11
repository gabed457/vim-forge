use ratatui::layout::Rect;
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Paragraph};
use ratatui::Frame;

use crate::app::AppState;
use crate::ecs::components::OutputCounter;

/// Render the sidebar widget into the given area.
///
/// The sidebar is 16 columns wide with a left border (`|`).
/// Contents:
/// - Title: VimForge
/// - Resource counts (Widgets/Ingots/Ore produced)
/// - Tick count and simulation speed
/// - Recent registers (up to 5)
/// - Set marks
pub fn render_sidebar(frame: &mut Frame, area: Rect, app: &AppState) {
    let block = Block::default()
        .borders(Borders::LEFT)
        .border_style(Style::default().fg(Color::DarkGray))
        .title(Span::styled(
            " VimForge ",
            Style::default()
                .fg(Color::Cyan)
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

    lines.push(Line::from(Span::styled(
        "-- Output --",
        Style::default()
            .fg(Color::Yellow)
            .add_modifier(Modifier::BOLD),
    )));
    lines.push(Line::from(vec![
        Span::styled("Widgets: ", Style::default().fg(Color::Gray)),
        Span::styled(
            format!("{}", widget_total),
            Style::default()
                .fg(Color::Green)
                .add_modifier(Modifier::BOLD),
        ),
    ]));
    lines.push(Line::from(vec![
        Span::styled("Ingots:  ", Style::default().fg(Color::Gray)),
        Span::styled(
            format!("{}", ingot_total),
            Style::default().fg(Color::White),
        ),
    ]));
    lines.push(Line::from(vec![
        Span::styled("Ore:     ", Style::default().fg(Color::Gray)),
        Span::styled(
            format!("{}", ore_total),
            Style::default().fg(Color::Rgb(180, 140, 60)),
        ),
    ]));

    // -- Inventory (only if non-empty) --
    if app.inventory.total() > 0 {
        lines.push(Line::from(""));
        lines.push(Line::from(Span::styled(
            "-- Inventory --",
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        )));
        if app.inventory.widget > 0 {
            lines.push(Line::from(vec![
                Span::styled("Widgets: ", Style::default().fg(Color::Gray)),
                Span::styled(
                    format!("{}", app.inventory.widget),
                    Style::default()
                        .fg(Color::Green)
                        .add_modifier(Modifier::BOLD),
                ),
            ]));
        }
        if app.inventory.ingot > 0 {
            lines.push(Line::from(vec![
                Span::styled("Ingots:  ", Style::default().fg(Color::Gray)),
                Span::styled(
                    format!("{}", app.inventory.ingot),
                    Style::default().fg(Color::White),
                ),
            ]));
        }
        if app.inventory.ore > 0 {
            lines.push(Line::from(vec![
                Span::styled("Ore:     ", Style::default().fg(Color::Gray)),
                Span::styled(
                    format!("{}", app.inventory.ore),
                    Style::default().fg(Color::Rgb(180, 140, 60)),
                ),
            ]));
        }
    }

    lines.push(Line::from(""));

    // -- Tick / speed --
    lines.push(Line::from(Span::styled(
        "-- Sim --",
        Style::default()
            .fg(Color::Yellow)
            .add_modifier(Modifier::BOLD),
    )));
    lines.push(Line::from(vec![
        Span::styled("Tick: ", Style::default().fg(Color::Gray)),
        Span::styled(
            format!("{}", app.simulation.tick_count),
            Style::default().fg(Color::White),
        ),
    ]));
    let speed_text = if app.simulation.is_paused() {
        "PAUSED".to_string()
    } else {
        format!("{}x", app.simulation.speed)
    };
    let speed_color = if app.simulation.is_paused() {
        Color::Red
    } else {
        Color::Green
    };
    lines.push(Line::from(vec![
        Span::styled("Speed: ", Style::default().fg(Color::Gray)),
        Span::styled(speed_text, Style::default().fg(speed_color)),
    ]));

    lines.push(Line::from(""));

    // -- Registers (up to 5 most recent) --
    let reg_list = app.registers.list();
    if !reg_list.is_empty() {
        lines.push(Line::from(Span::styled(
            "-- Regs --",
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        )));
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
                        .fg(Color::Cyan)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::styled(display, Style::default().fg(Color::Gray)),
            ]));
        }
        lines.push(Line::from(""));
    }

    // -- Marks --
    let mark_list = app.marks.list();
    if !mark_list.is_empty() {
        lines.push(Line::from(Span::styled(
            "-- Marks --",
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        )));
        for (ch, x, y) in mark_list.iter().take(8) {
            lines.push(Line::from(vec![
                Span::styled(
                    format!("'{}' ", ch),
                    Style::default()
                        .fg(Color::Magenta)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::styled(format!("[{},{}]", x, y), Style::default().fg(Color::Gray)),
            ]));
        }
    }

    // Truncate to available height
    lines.truncate(inner.height as usize);

    let paragraph = Paragraph::new(lines);
    frame.render_widget(paragraph, inner);
}

/// Sum up all OutputCounter components in the world.
fn total_output_counts(app: &AppState) -> (u64, u64, u64) {
    let mut ore = 0u64;
    let mut ingot = 0u64;
    let mut widget = 0u64;
    for (_entity, counter) in app.world.query::<&OutputCounter>().iter() {
        ore += counter.ore_count;
        ingot += counter.ingot_count;
        widget += counter.widget_count;
    }
    (ore, ingot, widget)
}
