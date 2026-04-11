use ratatui::layout::Rect;
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, BorderType, Borders, Clear, Paragraph};
use ratatui::Frame;

use crate::app::{AppState, PopupKind};

/// Dark background color for the popup.
const POPUP_BG: Color = Color::Rgb(20, 22, 30);

/// Drop shadow color.
const SHADOW_CHAR: char = '\u{2591}'; // light shade
const SHADOW_COLOR: Color = Color::Rgb(8, 8, 12);

/// Render a popup overlay if one is active.
///
/// Centered floating popup:
/// - Width: 60% of terminal, min 40, max 80
/// - Double-line border with gold title
/// - Dark blue-gray background
/// - Drop shadow (1 tile offset right and below)
/// - Scrollable with j/k
/// - Dismissed with Esc/q
pub fn render_popup(frame: &mut Frame, frame_size: Rect, app: &AppState) {
    let popup_kind = match &app.popup {
        Some(kind) => kind,
        None => return,
    };

    let area = popup_area(frame_size);

    // Render drop shadow (1 tile right, 1 tile below)
    render_drop_shadow(frame, area, frame_size);

    // Clear the background
    frame.render_widget(Clear, area);

    let (title, lines) = popup_content(popup_kind, app);

    let block = Block::default()
        .title(Span::styled(
            format!(" {} ", title),
            Style::default()
                .fg(Color::Rgb(255, 200, 60))
                .add_modifier(Modifier::BOLD),
        ))
        .borders(Borders::ALL)
        .border_type(BorderType::Double)
        .border_style(Style::default().fg(Color::Rgb(100, 100, 120)))
        .style(Style::default().bg(POPUP_BG));

    let inner = block.inner(area);
    frame.render_widget(block, area);

    if inner.height == 0 || inner.width == 0 {
        return;
    }

    // Apply scroll offset
    let visible_height = inner.height as usize;
    let scroll_offset = app.popup_scroll.min(lines.len().saturating_sub(visible_height));
    let visible_lines: Vec<Line> = lines
        .into_iter()
        .skip(scroll_offset)
        .take(visible_height)
        .collect();

    let paragraph = Paragraph::new(visible_lines);
    frame.render_widget(paragraph, inner);
}

/// Render the drop shadow effect.
fn render_drop_shadow(frame: &mut Frame, popup_area: Rect, frame_size: Rect) {
    let buf = frame.buffer_mut();

    // Right edge shadow (1 column to the right of popup)
    let shadow_x = popup_area.x + popup_area.width;
    if shadow_x < frame_size.x + frame_size.width {
        for y in (popup_area.y + 1)..=(popup_area.y + popup_area.height) {
            if y < frame_size.y + frame_size.height {
                let cell = &mut buf[(shadow_x, y)];
                cell.set_char(SHADOW_CHAR);
                cell.set_style(Style::default().fg(SHADOW_COLOR).bg(SHADOW_COLOR));
            }
        }
    }

    // Bottom edge shadow (1 row below popup)
    let shadow_y = popup_area.y + popup_area.height;
    if shadow_y < frame_size.y + frame_size.height {
        for x in (popup_area.x + 1)..=(popup_area.x + popup_area.width) {
            if x < frame_size.x + frame_size.width {
                let cell = &mut buf[(x, shadow_y)];
                cell.set_char(SHADOW_CHAR);
                cell.set_style(Style::default().fg(SHADOW_COLOR).bg(SHADOW_COLOR));
            }
        }
    }
}

/// Compute the popup area: centered, 60% width (min 40, max 80), 70% height.
fn popup_area(frame_size: Rect) -> Rect {
    let popup_w = {
        let pct = (frame_size.width as u32 * 60 / 100) as u16;
        pct.max(40).min(80).min(frame_size.width)
    };
    let popup_h = {
        let pct = (frame_size.height as u32 * 70 / 100) as u16;
        pct.max(10).min(frame_size.height)
    };
    let x = (frame_size.width.saturating_sub(popup_w)) / 2 + frame_size.x;
    let y = (frame_size.height.saturating_sub(popup_h)) / 2 + frame_size.y;
    Rect::new(x, y, popup_w, popup_h)
}

/// Generate the title and content lines for a popup.
fn popup_content<'a>(kind: &PopupKind, app: &AppState) -> (&'static str, Vec<Line<'a>>) {
    match kind {
        PopupKind::Help(topic) => help_content(topic.as_deref()),
        PopupKind::Stats => stats_content(app),
        PopupKind::Registers => registers_content(app),
        PopupKind::Marks => marks_content(app),
        PopupKind::Contracts => contracts_content(),
        PopupKind::Market => market_content(),
        PopupKind::Finance => finance_content(),
        PopupKind::Research => research_content(),
    }
}

fn help_content<'a>(topic: Option<&str>) -> (&'static str, Vec<Line<'a>>) {
    let mut lines = Vec::new();
    let title = "Help";

    match topic {
        Some("insert") | Some("i") => {
            lines.push(styled_header("Insert Mode"));
            lines.push(line_kv("s", "Place smelter"));
            lines.push(line_kv("a", "Place assembler"));
            lines.push(line_kv("c", "Place conveyor"));
            lines.push(line_kv("p", "Place splitter"));
            lines.push(line_kv("e", "Place merger"));
            lines.push(line_kv("w", "Place wall"));
            lines.push(line_kv("h/j/k/l", "Change facing"));
            lines.push(line_kv("Esc", "Return to normal mode"));
        }
        Some("visual") | Some("v") => {
            lines.push(styled_header("Visual Mode"));
            lines.push(line_kv("v", "Character-wise visual"));
            lines.push(line_kv("V", "Line-wise visual"));
            lines.push(line_kv("Ctrl-v", "Block visual"));
            lines.push(line_kv("d", "Demolish selection"));
            lines.push(line_kv("y", "Yank selection"));
            lines.push(line_kv("r/R", "Rotate CW/CCW"));
            lines.push(line_kv("o", "Swap anchor"));
        }
        _ => {
            lines.push(styled_header("VimForge Help"));
            lines.push(Line::from(""));
            lines.push(styled_header("Movement"));
            lines.push(line_kv("h/j/k/l", "Move cursor"));
            lines.push(line_kv("w/b", "Next/prev entity"));
            lines.push(line_kv("W/B", "Next/prev entity (big)"));
            lines.push(line_kv("0/$", "Line start/end"));
            lines.push(line_kv("^", "First entity in row"));
            lines.push(line_kv("gg/G", "Map start/end"));
            lines.push(line_kv("H/M/L", "Viewport top/mid/bottom"));
            lines.push(line_kv("f/F", "Find entity forward/back"));
            lines.push(line_kv("%", "Jump to connected machine"));
            lines.push(Line::from(""));
            lines.push(styled_header("Editing"));
            lines.push(line_kv("i", "Enter insert mode"));
            lines.push(line_kv("d{motion}", "Demolish"));
            lines.push(line_kv("y{motion}", "Yank (copy)"));
            lines.push(line_kv("p/P", "Paste after/before"));
            lines.push(line_kv("x", "Delete under cursor"));
            lines.push(line_kv("r{type}", "Replace entity"));
            lines.push(line_kv("~", "Toggle facing"));
            lines.push(line_kv("u/Ctrl-r", "Undo/redo"));
            lines.push(line_kv(".", "Repeat last change"));
            lines.push(Line::from(""));
            lines.push(styled_header("Commands"));
            lines.push(line_kv(":w", "Save"));
            lines.push(line_kv(":q", "Quit"));
            lines.push(line_kv(":speed N", "Set sim speed"));
            lines.push(line_kv(":pause/:resume", "Pause/resume sim"));
            lines.push(line_kv(":stats", "Show statistics"));
            lines.push(line_kv(":contracts", "Contract board"));
            lines.push(line_kv(":market", "Resource market"));
            lines.push(line_kv(":finance", "Finance overview"));
            lines.push(line_kv(":research", "Tech tree"));
            lines.push(Line::from(""));
            lines.push(Line::from(Span::styled(
                "Press Esc or q to close",
                Style::default().fg(Color::Rgb(70, 70, 80)),
            )));
        }
    }

    (title, lines)
}

fn stats_content<'a>(app: &AppState) -> (&'static str, Vec<Line<'a>>) {
    let mut lines = Vec::new();

    lines.push(styled_header("Statistics"));
    lines.push(Line::from(""));
    lines.push(line_kv(
        "Map size",
        &format!("{}x{}", app.map.width, app.map.height),
    ));
    lines.push(line_kv("Tick", &format!("{}", app.simulation.tick_count)));
    lines.push(line_kv("Speed", &format!("{}x", app.simulation.speed)));

    // Count entities by type
    let mut entity_counts: std::collections::HashMap<crate::resources::EntityType, usize> =
        std::collections::HashMap::new();
    for (_ent, kind) in app.world.query::<&crate::ecs::components::EntityKind>().iter() {
        *entity_counts.entry(kind.kind).or_insert(0) += 1;
    }

    lines.push(Line::from(""));
    lines.push(styled_header("Entities"));
    let type_order = [
        crate::resources::EntityType::OreDeposit,
        crate::resources::EntityType::Smelter,
        crate::resources::EntityType::Assembler,
        crate::resources::EntityType::BasicBelt,
        crate::resources::EntityType::Splitter,
        crate::resources::EntityType::Merger,
        crate::resources::EntityType::OutputBin,
        crate::resources::EntityType::Wall,
    ];
    for et in &type_order {
        let count = entity_counts.get(et).copied().unwrap_or(0);
        if count > 0 {
            lines.push(line_kv(et.name(), &format!("{}", count)));
        }
    }

    // Output totals
    let mut ore_total = 0u64;
    let mut ingot_total = 0u64;
    let mut widget_total = 0u64;
    for (_ent, counter) in app.world.query::<&crate::ecs::components::OutputCounter>().iter() {
        ore_total += counter.ore_count();
        ingot_total += counter.ingot_count();
        widget_total += counter.widget_count();
    }
    lines.push(Line::from(""));
    lines.push(styled_header("Total Output"));
    lines.push(line_kv("Widgets", &format!("{}", widget_total)));
    lines.push(line_kv("Ingots", &format!("{}", ingot_total)));
    lines.push(line_kv("Ore", &format!("{}", ore_total)));

    ("Stats", lines)
}

fn registers_content<'a>(app: &AppState) -> (&'static str, Vec<Line<'a>>) {
    let mut lines = Vec::new();
    lines.push(styled_header("Registers"));
    lines.push(Line::from(""));

    let reg_list = app.registers.list();
    if reg_list.is_empty() {
        lines.push(Line::from(Span::styled(
            "(no registers set)",
            Style::default().fg(Color::Rgb(70, 70, 80)),
        )));
    } else {
        for (name, content) in &reg_list {
            lines.push(Line::from(vec![
                Span::styled(
                    format!("{:<4} ", name),
                    Style::default()
                        .fg(Color::Rgb(80, 200, 220))
                        .add_modifier(Modifier::BOLD),
                ),
                Span::styled(
                    content.clone(),
                    Style::default().fg(Color::Rgb(220, 220, 220)),
                ),
            ]));
        }
    }

    ("Registers", lines)
}

fn marks_content<'a>(app: &AppState) -> (&'static str, Vec<Line<'a>>) {
    let mut lines = Vec::new();
    lines.push(styled_header("Marks"));
    lines.push(Line::from(""));

    let mark_list = app.marks.list();
    if mark_list.is_empty() {
        lines.push(Line::from(Span::styled(
            "(no marks set)",
            Style::default().fg(Color::Rgb(70, 70, 80)),
        )));
    } else {
        lines.push(Line::from(vec![
            Span::styled(
                "Mark  ",
                Style::default()
                    .fg(Color::Rgb(140, 140, 140))
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(
                "Position",
                Style::default()
                    .fg(Color::Rgb(140, 140, 140))
                    .add_modifier(Modifier::BOLD),
            ),
        ]));
        for (ch, x, y) in &mark_list {
            lines.push(Line::from(vec![
                Span::styled(
                    format!(" '{}' ", ch),
                    Style::default()
                        .fg(Color::Rgb(200, 100, 200))
                        .add_modifier(Modifier::BOLD),
                ),
                Span::styled(
                    format!("  [{}, {}]", x, y),
                    Style::default().fg(Color::Rgb(220, 220, 220)),
                ),
            ]));
        }
    }

    ("Marks", lines)
}

/// Contract board popup.
fn contracts_content<'a>() -> (&'static str, Vec<Line<'a>>) {
    let mut lines = Vec::new();
    lines.push(styled_header("Contract Board"));
    lines.push(Line::from(""));
    lines.push(Line::from(Span::styled(
        "No contracts available yet.",
        Style::default().fg(Color::Rgb(140, 140, 140)),
    )));
    lines.push(Line::from(""));
    lines.push(Line::from(Span::styled(
        "Contracts will appear as the economy system is wired in.",
        Style::default().fg(Color::Rgb(90, 90, 100)),
    )));
    ("Contracts", lines)
}

/// Market prices popup.
fn market_content<'a>() -> (&'static str, Vec<Line<'a>>) {
    let mut lines = Vec::new();
    lines.push(styled_header("Resource Market"));
    lines.push(Line::from(""));
    lines.push(Line::from(vec![
        Span::styled(
            format!("{:<22}", "Resource"),
            Style::default()
                .fg(Color::Rgb(140, 140, 140))
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled(
            "Price",
            Style::default()
                .fg(Color::Rgb(140, 140, 140))
                .add_modifier(Modifier::BOLD),
        ),
    ]));
    // Show some base resource prices
    let resources = [
        ("Iron Ore", 1.0),
        ("Copper Ore", 1.0),
        ("Coal", 1.0),
        ("Iron Ingot", 5.0),
        ("Copper Ingot", 5.0),
        ("Steel", 5.0),
        ("Circuit Board", 25.0),
        ("Processor", 100.0),
        ("Quantum Processor", 500.0),
    ];
    for (name, price) in &resources {
        lines.push(Line::from(vec![
            Span::styled(
                format!("{:<22}", name),
                Style::default().fg(Color::Rgb(200, 200, 200)),
            ),
            Span::styled(
                format!("${:.0}", price),
                Style::default().fg(Color::Rgb(80, 220, 80)),
            ),
        ]));
    }
    ("Market", lines)
}

/// Finance overview popup.
fn finance_content<'a>() -> (&'static str, Vec<Line<'a>>) {
    let mut lines = Vec::new();
    lines.push(styled_header("Finance Overview"));
    lines.push(Line::from(""));
    lines.push(Line::from(Span::styled(
        "Finance tracking will be available once the economy is wired in.",
        Style::default().fg(Color::Rgb(140, 140, 140)),
    )));
    ("Finance", lines)
}

/// Research/tech tree popup.
fn research_content<'a>() -> (&'static str, Vec<Line<'a>>) {
    let mut lines = Vec::new();
    lines.push(styled_header("Research Tree"));
    lines.push(Line::from(""));
    lines.push(Line::from(Span::styled(
        "Research will be available once the tech tree is wired in.",
        Style::default().fg(Color::Rgb(140, 140, 140)),
    )));
    ("Research", lines)
}

/// Helper to create a styled section header line. Uses Rgb.
fn styled_header<'a>(text: &str) -> Line<'a> {
    Line::from(Span::styled(
        text.to_string(),
        Style::default()
            .fg(Color::Rgb(220, 200, 60))
            .add_modifier(Modifier::BOLD),
    ))
}

/// Helper to create a key-value line. Uses Rgb.
fn line_kv<'a>(key: &str, value: &str) -> Line<'a> {
    Line::from(vec![
        Span::styled(
            format!("  {:<16}", key),
            Style::default().fg(Color::Rgb(80, 200, 220)),
        ),
        Span::styled(
            value.to_string(),
            Style::default().fg(Color::Rgb(220, 220, 220)),
        ),
    ])
}
