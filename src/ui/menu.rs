use ratatui::layout::Rect;
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, BorderType, Borders, Clear, Paragraph};
use ratatui::Frame;

use crate::app::AppState;

/// Render the main menu screen (centered box with title and options).
pub fn render_menu(frame: &mut Frame, frame_size: Rect, app: &AppState) {
    // Clear the entire frame with dark background
    let bg_block = Block::default().style(Style::default().bg(Color::Rgb(15, 15, 25)));
    frame.render_widget(bg_block, frame_size);

    let menu_area = menu_area(frame_size);

    frame.render_widget(Clear, menu_area);

    let block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Double)
        .border_style(Style::default().fg(Color::Cyan))
        .style(Style::default().bg(Color::Rgb(20, 20, 35)));

    let inner = block.inner(menu_area);
    frame.render_widget(block, menu_area);

    if inner.height == 0 || inner.width == 0 {
        return;
    }

    let mut lines: Vec<Line> = Vec::new();

    // Title
    lines.push(Line::from(""));
    lines.push(Line::from(Span::styled(
        center_text("VIM FORGE", inner.width as usize),
        Style::default()
            .fg(Color::Cyan)
            .add_modifier(Modifier::BOLD),
    )));
    lines.push(Line::from(Span::styled(
        center_text("A Vim-Grammar Factory Builder", inner.width as usize),
        Style::default().fg(Color::DarkGray),
    )));
    lines.push(Line::from(""));
    lines.push(Line::from(""));

    // Menu options
    lines.push(menu_option("[1]", "Tutorial", true, inner.width as usize));
    lines.push(Line::from(""));
    lines.push(menu_option(
        "[2]",
        "Freeplay",
        app.freeplay_unlocked,
        inner.width as usize,
    ));
    lines.push(Line::from(""));
    lines.push(menu_option(
        "[3]",
        "Load Save",
        app.has_save,
        inner.width as usize,
    ));
    lines.push(Line::from(""));
    lines.push(menu_option("[4]", "Quit", true, inner.width as usize));
    lines.push(Line::from(""));
    lines.push(Line::from(""));

    // Footer
    lines.push(Line::from(Span::styled(
        center_text("Press a number key to select", inner.width as usize),
        Style::default().fg(Color::DarkGray),
    )));

    let paragraph = Paragraph::new(lines);
    frame.render_widget(paragraph, inner);
}

/// Compute the centered menu box area.
fn menu_area(frame_size: Rect) -> Rect {
    let menu_w = 40u16.min(frame_size.width.saturating_sub(4));
    let menu_h = 18u16.min(frame_size.height.saturating_sub(2));
    let x = (frame_size.width.saturating_sub(menu_w)) / 2 + frame_size.x;
    let y = (frame_size.height.saturating_sub(menu_h)) / 2 + frame_size.y;
    Rect::new(x, y, menu_w, menu_h)
}

/// Create a menu option line with grayed-out support.
fn menu_option<'a>(key: &str, label: &str, enabled: bool, width: usize) -> Line<'a> {
    let text = format!("{}  {}", key, label);
    let suffix = if !enabled { " (locked)" } else { "" };
    let full = format!("{}{}", text, suffix);
    let centered = center_text(&full, width);

    if enabled {
        Line::from(vec![
            Span::styled(
                centered,
                Style::default()
                    .fg(Color::White)
                    .add_modifier(Modifier::BOLD),
            ),
        ])
    } else {
        Line::from(vec![Span::styled(
            centered,
            Style::default().fg(Color::DarkGray),
        )])
    }
}

/// Center a string within a given width.
fn center_text(text: &str, width: usize) -> String {
    if text.len() >= width {
        return text.to_string();
    }
    let pad = (width - text.len()) / 2;
    format!("{}{}", " ".repeat(pad), text)
}
