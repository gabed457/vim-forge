use ratatui::layout::Rect;
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, BorderType, Borders, Clear, Paragraph};
use ratatui::Frame;

use crate::app::AppState;

/// Hand-crafted figlet-style logotype: VIMFORGE on one line.
const LOGO_ART: [&str; 5] = [
    r"__     _____ __  __ _____ ___  ____   ____ _____",
    r"\ \   / /_ _|  \/  |  ___/ _ \|  _ \ / ___| ____|",
    r" \ \ / / | || |\/| | |_ | | | | |_) | |  _|  _|",
    r"  \ V /  | || |  | |  _|| |_| |  _ <| |_| | |___",
    r"   \_/  |___|_|  |_|_|   \___/|_| \_\\____|_____|",
];

/// Compact fallback logotype for narrow terminals (< 54 usable columns).
const LOGO_SMALL: [&str; 2] = [
    "V I M F O R G E",
    "===============",
];

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
        .border_style(Style::default().fg(Color::Rgb(86, 205, 227)))
        .style(Style::default().bg(Color::Rgb(14, 14, 26)));

    let inner = block.inner(menu_area);
    frame.render_widget(block, menu_area);

    if inner.height == 0 || inner.width == 0 {
        return;
    }

    let width = inner.width as usize;
    let mut lines: Vec<Line> = Vec::new();

    lines.push(Line::from(""));

    // Logotype: big single-line VIMFORGE, or compact fallback when narrow.
    if width >= LOGO_WIDTH + 1 {
        for art_line in &LOGO_ART {
            lines.push(center_gradient_line(art_line, LOGO_WIDTH, width));
        }
    } else {
        for art_line in &LOGO_SMALL {
            lines.push(center_gradient_line(art_line, LOGO_SMALL[0].len().max(2), width));
        }
    }

    lines.push(Line::from(""));

    // Tagline + version
    lines.push(Line::from(Span::styled(
        center_text("a  v i m - g r a m m a r  f a c t o r y", width),
        Style::default()
            .fg(Color::Rgb(140, 150, 170))
            .add_modifier(Modifier::ITALIC),
    )));
    lines.push(Line::from(Span::styled(
        center_text(concat!("v", env!("CARGO_PKG_VERSION")), width),
        Style::default().fg(Color::Rgb(80, 85, 100)),
    )));
    lines.push(Line::from(""));

    // Pulsing '>' accent cursor, vim-command style, driven by the frame counter.
    let pulse = pulse_level(app.animations.frame_counter);

    lines.push(menu_option(pulse, "[1]", "Tutorial", true, width));
    lines.push(Line::from(""));
    lines.push(menu_option(pulse, "[2]", "Freeplay", app.freeplay_unlocked, width));
    lines.push(Line::from(""));
    lines.push(menu_option(pulse, "[3]", "Load Save", app.has_save, width));
    lines.push(Line::from(""));
    lines.push(menu_option(pulse, "[4]", "Quit", true, width));
    lines.push(Line::from(""));
    lines.push(Line::from(""));

    // Footer: vim-style hint
    lines.push(Line::from(Span::styled(
        center_text(":press a number key to select", width),
        Style::default().fg(Color::Rgb(90, 95, 110)),
    )));

    let paragraph = Paragraph::new(lines);
    frame.render_widget(paragraph, inner);
}

/// Pulse brightness 0.0..1.0 from the animation frame counter (~1s period).
fn pulse_level(frame_counter: u32) -> f64 {
    let phase = (frame_counter % 30) as f64 / 30.0 * std::f64::consts::TAU;
    phase.sin() * 0.5 + 0.5
}

/// Compute the centered menu box area.
fn menu_area(frame_size: Rect) -> Rect {
    let menu_w = 60u16.min(frame_size.width.saturating_sub(2));
    let menu_h = 26u16.min(frame_size.height);
    let x = (frame_size.width.saturating_sub(menu_w)) / 2 + frame_size.x;
    let y = (frame_size.height.saturating_sub(menu_h)) / 2 + frame_size.y;
    Rect::new(x, y, menu_w, menu_h)
}

/// Center a gradient line within a width. The gradient spans `span` columns
/// of art so every logotype row sweeps the same two-tone ramp.
fn center_gradient_line<'a>(text: &str, span: usize, width: usize) -> Line<'a> {
    let text_len = text.chars().count();
    let pad = width.saturating_sub(text_len) / 2;
    let mut spans: Vec<Span> = Vec::with_capacity(text_len + 1);
    if pad > 0 {
        spans.push(Span::raw(" ".repeat(pad)));
    }
    for (i, ch) in text.chars().enumerate() {
        let t = i.min(span.saturating_sub(1));
        let color = logo_gradient(t * (LOGO_WIDTH - 1) / span.saturating_sub(1).max(1));
        spans.push(Span::styled(
            String::from(ch),
            Style::default().fg(color).add_modifier(Modifier::BOLD),
        ));
    }
    Line::from(spans)
}

/// Create a menu option line: pulsing '>' accent, vim-key chip, label.
/// Gold for enabled entries, dim slate for locked ones.
fn menu_option<'a>(pulse: f64, key: &str, label: &str, enabled: bool, width: usize) -> Line<'a> {
    let suffix = if !enabled { "  (locked)" } else { "" };
    // "> [1]  Tutorial"
    let content_len = 2 + key.len() + 2 + label.len() + suffix.len();
    let pad = width.saturating_sub(content_len) / 2;

    let mut spans: Vec<Span> = Vec::new();
    if pad > 0 {
        spans.push(Span::raw(" ".repeat(pad)));
    }

    if enabled {
        // Pulsing accent cursor
        let a = (120.0 + 135.0 * pulse) as u8;
        spans.push(Span::styled(
            "> ".to_string(),
            Style::default()
                .fg(Color::Rgb(a, (a as f64 * 0.82) as u8, 40))
                .add_modifier(Modifier::BOLD),
        ));
        spans.push(Span::styled(
            key.to_string(),
            Style::default()
                .fg(Color::Rgb(86, 205, 227))
                .add_modifier(Modifier::BOLD),
        ));
        spans.push(Span::styled(
            format!("  {}", label),
            Style::default()
                .fg(Color::Rgb(255, 210, 90))
                .add_modifier(Modifier::BOLD),
        ));
    } else {
        spans.push(Span::raw("  ".to_string()));
        spans.push(Span::styled(
            key.to_string(),
            Style::default().fg(Color::Rgb(70, 74, 88)),
        ));
        spans.push(Span::styled(
            format!("  {}{}", label, suffix),
            Style::default().fg(Color::Rgb(70, 74, 88)),
        ));
    }
    Line::from(spans)
}

/// Center a string within a given width.
fn center_text(text: &str, width: usize) -> String {
    let len = text.chars().count();
    if len >= width {
        return text.to_string();
    }
    let pad = (width - len) / 2;
    format!("{}{}", " ".repeat(pad), text)
}
