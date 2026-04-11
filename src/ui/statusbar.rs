use ratatui::layout::Rect;
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::Paragraph;
use ratatui::Frame;

use crate::app::{AppState, Mode};

/// Render the status bar at the bottom of the screen.
///
/// Layout:
/// - Left: mode indicator with color coding
/// - Center: pending command buffer
/// - Right: cursor position [col,row] and optional recording indicator
pub fn render_statusbar(frame: &mut Frame, area: Rect, app: &AppState) {
    if area.height == 0 || area.width == 0 {
        return;
    }

    // If we are in command or search mode, show the input line
    if app.mode == Mode::Command || app.mode == Mode::Search {
        render_command_line(frame, area, app);
        return;
    }

    let mut spans: Vec<Span> = Vec::new();

    // -- Left: mode indicator --
    let (mode_text, mode_style) = mode_display(app);
    spans.push(Span::styled(mode_text, mode_style));

    // Status message (error or info)
    if !app.status_message.is_empty() {
        spans.push(Span::raw(" "));
        let msg_style = if app.status_error {
            Style::default().fg(Color::Red)
        } else {
            Style::default().fg(Color::Gray)
        };
        // Truncate message to fit
        let max_msg_len = (area.width as usize).saturating_sub(30);
        let msg = if app.status_message.len() > max_msg_len {
            format!("{}...", &app.status_message[..max_msg_len.saturating_sub(3)])
        } else {
            app.status_message.clone()
        };
        spans.push(Span::styled(msg, msg_style));
    }

    // -- Center: pending command keys --
    let pending = &app.pending_keys;
    if !pending.is_empty() {
        // Calculate space to push pending toward center
        let left_used: usize = spans.iter().map(|s| s.content.len()).sum();
        let right_used = 14usize; // rough estimate for position display
        let center = (area.width as usize).saturating_sub(left_used + right_used) / 2;
        let pad = center.saturating_sub(0);
        if pad > 0 {
            spans.push(Span::raw(" ".repeat(pad)));
        }
        spans.push(Span::styled(
            pending.clone(),
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        ));
    }

    // -- Right: position and recording indicator --
    let mut right_spans: Vec<Span> = Vec::new();

    // Recording indicator
    if let Some(reg) = app.recording_macro {
        right_spans.push(Span::styled(
            format!("recording @{} ", reg),
            Style::default()
                .fg(Color::Red)
                .add_modifier(Modifier::BOLD),
        ));
    }

    // Cursor position
    right_spans.push(Span::styled(
        format!("[{},{}]", app.cursor_x, app.cursor_y),
        Style::default().fg(Color::DarkGray),
    ));

    // Calculate right alignment padding
    let left_content_len: usize = spans.iter().map(|s| s.content.len()).sum();
    let right_content_len: usize = right_spans.iter().map(|s| s.content.len()).sum();
    let total_content = left_content_len + right_content_len;
    let padding = (area.width as usize).saturating_sub(total_content);
    if padding > 0 {
        spans.push(Span::raw(" ".repeat(padding)));
    }
    spans.extend(right_spans);

    let line = Line::from(spans);
    let paragraph = Paragraph::new(line);
    frame.render_widget(paragraph, area);
}

/// Render the command/search input line.
fn render_command_line(frame: &mut Frame, area: Rect, app: &AppState) {
    let prefix = match app.mode {
        Mode::Command => ":",
        Mode::Search => {
            if app.search.forward {
                "/"
            } else {
                "?"
            }
        }
        _ => "",
    };

    let line = Line::from(vec![
        Span::styled(
            prefix,
            Style::default()
                .fg(Color::White)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled(
            app.command_buffer.clone(),
            Style::default().fg(Color::White),
        ),
        Span::styled(
            "\u{2588}", // block cursor
            Style::default().fg(Color::White),
        ),
    ]);

    let paragraph = Paragraph::new(line);
    frame.render_widget(paragraph, area);
}

/// Get the display text and style for the current mode.
fn mode_display(app: &AppState) -> (String, Style) {
    match &app.mode {
        Mode::Normal => (
            " NORMAL ".to_string(),
            Style::default().fg(Color::White),
        ),
        Mode::Insert => {
            let arrow = app.insert_facing.arrow_glyph();
            (
                format!(" INSERT [{}] ", arrow),
                Style::default()
                    .fg(Color::White)
                    .bg(Color::Rgb(0, 80, 0)),
            )
        }
        Mode::Visual => (
            " VISUAL ".to_string(),
            Style::default()
                .fg(Color::White)
                .bg(Color::Rgb(120, 80, 0)),
        ),
        Mode::VisualLine => (
            " VISUAL LINE ".to_string(),
            Style::default()
                .fg(Color::White)
                .bg(Color::Rgb(120, 80, 0)),
        ),
        Mode::VisualBlock => (
            " VISUAL BLOCK ".to_string(),
            Style::default()
                .fg(Color::White)
                .bg(Color::Rgb(120, 80, 0)),
        ),
        Mode::Command | Mode::Search => (
            String::new(),
            Style::default(),
        ),
        Mode::Menu => (
            " MENU ".to_string(),
            Style::default().fg(Color::DarkGray),
        ),
    }
}
