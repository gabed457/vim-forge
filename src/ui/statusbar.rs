use ratatui::layout::Rect;
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::Paragraph;
use ratatui::Frame;

use crate::app::{AppState, Mode};

/// Status bar bed color (slightly blue-black, matches the grid chrome).
const BAR_BG: Color = Color::Rgb(24, 28, 38);

/// Render the status bar at the bottom of the screen, vim-style:
///
/// `[ MODE ]` chip with per-mode color | status message ...
/// ... cash | tick | pending keys (showcmd) | `● REC @a` | `[col,row]`
pub fn render_statusbar(frame: &mut Frame, area: Rect, app: &AppState) {
    if area.height == 0 || area.width == 0 {
        return;
    }

    // If we are in command or search mode, show the input line
    if app.mode == Mode::Command || app.mode == Mode::Search {
        render_command_line(frame, area, app);
        return;
    }

    // Status-flash overrides the bar bed color briefly (success/failure).
    let bar_bg = match &app.animations.status_flash {
        Some(sf) => Color::Rgb(sf.color.0 / 3, sf.color.1 / 3, sf.color.2 / 3),
        None => BAR_BG,
    };

    let mut spans: Vec<Span> = Vec::new();

    // -- Left: mode chip --
    let (mode_text, mode_style) = mode_chip(app);
    spans.push(Span::styled(mode_text, mode_style));
    spans.push(Span::styled(" ", Style::default().bg(bar_bg)));

    // Status message (error or info)
    if !app.status_message.is_empty() {
        let msg_style = if app.status_error {
            Style::default()
                .fg(Color::Rgb(255, 90, 90))
                .bg(bar_bg)
                .add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(Color::Rgb(150, 155, 170)).bg(bar_bg)
        };
        // Truncate message to fit
        let max_msg_len = (area.width as usize).saturating_sub(44);
        let msg: String = if app.status_message.chars().count() > max_msg_len {
            let cut: String = app
                .status_message
                .chars()
                .take(max_msg_len.saturating_sub(1))
                .collect();
            format!("{}\u{2026}", cut)
        } else {
            app.status_message.clone()
        };
        spans.push(Span::styled(msg, msg_style));
    }

    // -- Right side, built right-to-left conceptually:
    //    cash | tick | showcmd | REC | position
    let mut right_spans: Vec<Span> = Vec::new();

    // Cash readout. Tutorial levels use a near-infinite sentinel balance —
    // render those as ∞ instead of a 19-digit wall of numbers.
    let cash_text = if app.economy.cash.abs() >= 1_000_000_000_000 {
        "$\u{221E} ".to_string()
    } else {
        format!("${} ", group_thousands(app.economy.cash))
    };
    right_spans.push(Span::styled(
        cash_text,
        Style::default()
            .fg(Color::Rgb(120, 220, 130))
            .bg(bar_bg)
            .add_modifier(Modifier::BOLD),
    ));

    // Tick readout
    right_spans.push(Span::styled(
        format!("\u{2699}{} ", app.simulation.tick_count),
        Style::default().fg(Color::Rgb(110, 120, 140)).bg(bar_bg),
    ));

    // Pending keys buffer — vim's 'showcmd', right-aligned
    if !app.pending_keys.is_empty() {
        right_spans.push(Span::styled(
            format!("{} ", app.pending_keys),
            Style::default()
                .fg(Color::Rgb(240, 215, 90))
                .bg(bar_bg)
                .add_modifier(Modifier::BOLD),
        ));
    }

    // Recording indicator, vim-style: ● REC @a
    if let Some(reg) = app.recording_macro {
        right_spans.push(Span::styled(
            format!("\u{25CF} REC @{} ", reg),
            Style::default()
                .fg(Color::Rgb(255, 70, 70))
                .bg(bar_bg)
                .add_modifier(Modifier::BOLD),
        ));
    }

    // Cursor position
    right_spans.push(Span::styled(
        format!("[{},{}]", app.cursor_x, app.cursor_y),
        Style::default().fg(Color::Rgb(130, 140, 160)).bg(bar_bg),
    ));

    // Calculate right alignment padding
    let left_content_len: usize = spans.iter().map(|s| s.content.chars().count()).sum();
    let right_content_len: usize = right_spans.iter().map(|s| s.content.chars().count()).sum();
    let total_content = left_content_len + right_content_len;
    let padding = (area.width as usize).saturating_sub(total_content);
    if padding > 0 {
        spans.push(Span::styled(" ".repeat(padding), Style::default().bg(bar_bg)));
    }
    spans.extend(right_spans);

    let line = Line::from(spans);
    let paragraph = Paragraph::new(line).style(Style::default().bg(bar_bg));
    frame.render_widget(paragraph, area);
}

/// Render the command/search input line with a live block cursor.
fn render_command_line(frame: &mut Frame, area: Rect, app: &AppState) {
    let (prefix, prompt_color) = match app.mode {
        Mode::Command => (":", Color::Rgb(240, 210, 90)),
        Mode::Search => {
            if app.search.forward {
                ("/", Color::Rgb(240, 210, 90))
            } else {
                ("?", Color::Rgb(240, 210, 90))
            }
        }
        _ => ("", Color::Rgb(220, 220, 220)),
    };

    let line = Line::from(vec![
        Span::styled(
            prefix,
            Style::default()
                .fg(prompt_color)
                .bg(BAR_BG)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled(
            app.command_buffer.clone(),
            Style::default().fg(Color::Rgb(235, 235, 235)).bg(BAR_BG),
        ),
        Span::styled(
            "\u{2588}", // block cursor
            Style::default().fg(Color::Rgb(235, 235, 235)).bg(BAR_BG),
        ),
    ]);

    let paragraph = Paragraph::new(line).style(Style::default().bg(BAR_BG));
    frame.render_widget(paragraph, area);
}

/// Mode chip text + style: solid color block per mode, vim airline-style.
/// NORMAL grey-blue, INSERT green, VISUAL orange, COMMAND/SEARCH yellow.
fn mode_chip(app: &AppState) -> (String, Style) {
    let chip = |fg: (u8, u8, u8), bg: (u8, u8, u8)| {
        Style::default()
            .fg(Color::Rgb(fg.0, fg.1, fg.2))
            .bg(Color::Rgb(bg.0, bg.1, bg.2))
            .add_modifier(Modifier::BOLD)
    };
    match &app.mode {
        Mode::Normal => (" NORMAL ".to_string(), chip((225, 232, 245), (68, 82, 112))),
        Mode::Insert => {
            let arrow = app.insert_facing.arrow_glyph();
            (
                format!(" INSERT {} ", arrow),
                chip((10, 30, 12), (110, 200, 110)),
            )
        }
        Mode::Visual => (" VISUAL ".to_string(), chip((30, 18, 4), (235, 150, 50))),
        Mode::VisualLine => (" V-LINE ".to_string(), chip((30, 18, 4), (235, 150, 50))),
        Mode::VisualBlock => (" V-BLOCK ".to_string(), chip((30, 18, 4), (235, 150, 50))),
        Mode::Command | Mode::Search => (
            " CMD ".to_string(),
            chip((30, 26, 4), (225, 195, 70)),
        ),
        Mode::Menu => (" MENU ".to_string(), chip((150, 155, 170), (32, 36, 48))),
    }
}

/// Format an integer with thousands separators: 1234567 -> "1,234,567".
fn group_thousands(n: i64) -> String {
    let negative = n < 0;
    let digits = n.unsigned_abs().to_string();
    let mut out = String::with_capacity(digits.len() + digits.len() / 3 + 1);
    let len = digits.len();
    for (i, ch) in digits.chars().enumerate() {
        if i > 0 && (len - i) % 3 == 0 {
            out.push(',');
        }
        out.push(ch);
    }
    if negative {
        format!("-{}", out)
    } else {
        out
    }
}
