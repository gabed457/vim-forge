use ratatui::layout::Rect;
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, BorderType, Borders, Clear, Paragraph};
use ratatui::Frame;

use crate::app::AppState;

/// ASCII art lines for "VIM".
const VIM_ART: [&str; 5] = [
    " __     __ ___  __  __ ",
    " \\ \\   / /|_ _||  \\/  |",
    "  \\ \\ / /  | | | |\\/| |",
    "   \\ V /   | | | |  | |",
    "    \\_/   |___||_|  |_|",
];

/// ASCII art lines for "FORGE".
const FORGE_ART: [&str; 5] = [
    " _____ ___  ____   ____ _____",
    "|  ___/ _ \\|  _ \\ / ___| ____|",
    "| |_ | | | | |_) | |  _|  _|  ",
    "|  _|| |_| |  _ <| |_| | |___ ",
    "|_|   \\___/|_| \\_\\\\____|_____|",
];

/// Gradient for VIM: cyan to purple.
fn vim_gradient(col: usize) -> Color {
    // Interpolate from (80, 200, 220) to (160, 80, 220) over 23 chars
    let t = (col as f64 / 22.0).min(1.0);
    let r = (80.0 + 80.0 * t) as u8;
    let g = (200.0 - 120.0 * t) as u8;
    let b = 220u8;
    Color::Rgb(r, g, b)
}

/// Gradient for FORGE: orange to gold.
fn forge_gradient(col: usize) -> Color {
    // Interpolate from (220, 120, 40) to (255, 200, 60) over 30 chars
    let t = (col as f64 / 29.0).min(1.0);
    let r = (220.0 + 35.0 * t).min(255.0) as u8;
    let g = (120.0 + 80.0 * t) as u8;
    let b = (40.0 + 20.0 * t) as u8;
    Color::Rgb(r, g, b)
}

/// Build a line of ASCII art with per-character gradient coloring.
fn gradient_line<'a>(text: &str, gradient_fn: fn(usize) -> Color) -> Line<'a> {
    let spans: Vec<Span> = text
        .chars()
        .enumerate()
        .map(|(i, ch)| {
            let color = gradient_fn(i);
            Span::styled(
                String::from(ch),
                Style::default()
                    .fg(color)
                    .add_modifier(Modifier::BOLD),
            )
        })
        .collect();
    Line::from(spans)
}

/// Render the main menu screen (centered box with ASCII art title and options).
pub fn render_menu(frame: &mut Frame, frame_size: Rect, app: &AppState) {
    // Clear the entire frame with dark background
    let bg_block = Block::default().style(Style::default().bg(Color::Rgb(10, 10, 18)));
    frame.render_widget(bg_block, frame_size);

    let menu_area = menu_area(frame_size);

    frame.render_widget(Clear, menu_area);

    let block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Double)
        .border_style(Style::default().fg(Color::Rgb(80, 200, 220)))
        .style(Style::default().bg(Color::Rgb(15, 15, 28)));

    let inner = block.inner(menu_area);
    frame.render_widget(block, menu_area);

    if inner.height == 0 || inner.width == 0 {
        return;
    }

    let mut lines: Vec<Line> = Vec::new();

    lines.push(Line::from(""));

    // ASCII art title - VIM
    for art_line in &VIM_ART {
        lines.push(center_gradient_line(art_line, vim_gradient, inner.width as usize));
    }

    // ASCII art title - FORGE
    for art_line in &FORGE_ART {
        lines.push(center_gradient_line(art_line, forge_gradient, inner.width as usize));
    }

    lines.push(Line::from(""));
    lines.push(Line::from(Span::styled(
        center_text("A Vim-Grammar Factory Builder", inner.width as usize),
        Style::default().fg(Color::Rgb(90, 90, 100)),
    )));
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
        Style::default().fg(Color::Rgb(70, 70, 80)),
    )));

    let paragraph = Paragraph::new(lines);
    frame.render_widget(paragraph, inner);
}

/// Compute the centered menu box area.
fn menu_area(frame_size: Rect) -> Rect {
    let menu_w = 50u16.min(frame_size.width.saturating_sub(4));
    let menu_h = 28u16.min(frame_size.height.saturating_sub(2));
    let x = (frame_size.width.saturating_sub(menu_w)) / 2 + frame_size.x;
    let y = (frame_size.height.saturating_sub(menu_h)) / 2 + frame_size.y;
    Rect::new(x, y, menu_w, menu_h)
}

/// Center a gradient line within a width.
fn center_gradient_line<'a>(
    text: &str,
    gradient_fn: fn(usize) -> Color,
    width: usize,
) -> Line<'a> {
    let text_len = text.len();
    if text_len >= width {
        return gradient_line(text, gradient_fn);
    }
    let pad = (width - text_len) / 2;
    let mut spans = vec![Span::raw(" ".repeat(pad))];
    for (i, ch) in text.chars().enumerate() {
        let color = gradient_fn(i);
        spans.push(Span::styled(
            String::from(ch),
            Style::default()
                .fg(color)
                .add_modifier(Modifier::BOLD),
        ));
    }
    Line::from(spans)
}

/// Create a menu option line. Gold highlight for enabled, gray for locked.
fn menu_option<'a>(key: &str, label: &str, enabled: bool, width: usize) -> Line<'a> {
    let text = format!("{}  {}", key, label);
    let suffix = if !enabled { " (locked)" } else { "" };
    let full = format!("{}{}", text, suffix);
    let centered = center_text(&full, width);

    if enabled {
        Line::from(vec![Span::styled(
            centered,
            Style::default()
                .fg(Color::Rgb(255, 200, 60))
                .add_modifier(Modifier::BOLD),
        )])
    } else {
        Line::from(vec![Span::styled(
            centered,
            Style::default().fg(Color::Rgb(70, 70, 80)),
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
