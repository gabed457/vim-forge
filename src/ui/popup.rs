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
        PopupKind::Contracts => contracts_content(app),
        PopupKind::Market => market_content(app),
        PopupKind::Finance => finance_content(app),
        PopupKind::Research => research_content(app),
        PopupKind::Prestige => prestige_content(app),
        PopupKind::Recipes => recipes_content(app),
    }
}

fn help_content<'a>(topic: Option<&str>) -> (&'static str, Vec<Line<'a>>) {
    let mut lines = Vec::new();
    let title = "Help";

    match topic {
        Some("motions") | Some("motion") | Some("move") => {
            lines.push(styled_header("Motions"));
            lines.push(line_kv("h/j/k/l", "Move (counts work: 5l)"));
            lines.push(line_kv("w/b", "Next/prev cluster"));
            lines.push(line_kv("e/ge", "End of cluster fwd/back"));
            lines.push(line_kv("W/B/E", "Big cluster hops"));
            lines.push(line_kv("0/^/$/g_", "Row start/first/last/end"));
            lines.push(line_kv("|", "Jump to column N (5|)"));
            lines.push(line_kv("+/-", "First entity row below/above"));
            lines.push(line_kv("gg/G", "Map top/bottom (5G = row 5)"));
            lines.push(line_kv("{/}", "Prev/next paragraph band"));
            lines.push(line_kv("(/)", "Prev/next machine"));
            lines.push(line_kv("H/M/L", "Viewport top/mid/bottom"));
            lines.push(line_kv("Ctrl-d/u", "Half page down/up"));
            lines.push(line_kv("Ctrl-f/b", "Full page down/up"));
            lines.push(line_kv("zz/zt/zb", "Scroll cursor center/top/bottom"));
            lines.push(line_kv("f/F t/T", "Find/til entity (fs=smelter)"));
            lines.push(line_kv(";/,", "Repeat find same/reverse"));
            lines.push(line_kv("%", "Follow belt chain to its end"));
            lines.push(line_kv("Ctrl-o/Ctrl-i", "Jumplist back/forward"));
        }
        Some("operators") | Some("ops") | Some("d") => {
            lines.push(styled_header("Operators (compose with motions/objects)"));
            lines.push(line_kv("d{m}", "Demolish (dw, d3l, dfs, dG)"));
            lines.push(line_kv("y{m}", "Yank blueprint (yy = line)"));
            lines.push(line_kv("c{m}", "Change: demolish + insert"));
            lines.push(line_kv(">/<", "Rotate CW/CCW over range"));
            lines.push(line_kv("gU/gu", "Upgrade/downgrade tier over range"));
            lines.push(line_kv("g~", "Rotate 180 over range"));
            lines.push(line_kv("dd/yy/cc/gUU", "Doubled = whole line"));
            lines.push(line_kv("x", "Delete tile (3x = 3 tiles)"));
            lines.push(line_kv("s/S", "Substitute tile / change line"));
            lines.push(line_kv("C/D/Y", "Change/delete/yank to line end"));
            lines.push(line_kv("r{key}", "Replace tile with building"));
            lines.push(line_kv("R", "Replace mode (typing replaces)"));
            lines.push(line_kv("~", "Rotate entity under cursor"));
            lines.push(line_kv("J", "Join: bridge gap to next cluster"));
            lines.push(line_kv("Ctrl-a/Ctrl-x", "Upgrade/downgrade belt tier"));
            lines.push(line_kv("u/Ctrl-r", "Undo / redo"));
            lines.push(line_kv(".", "Repeat last edit"));
        }
        Some("objects") | Some("textobjects") | Some("o") => {
            lines.push(styled_header("Text Objects (after d/c/y/gU or v)"));
            lines.push(line_kv("iw/aw", "Inner/around cluster"));
            lines.push(line_kv("ip/ap", "Inner/around paragraph band"));
            lines.push(line_kv("ib/ab i(/a(", "Inner/around walled block"));
            lines.push(line_kv("i[/i{/i<", "Same block (bracket aliases)"));
            lines.push(line_kv("i\"/a\"", "Belt run (straight same-dir line)"));
            lines.push(line_kv("it/at", "Machine footprint / + ports"));
            lines.push(line_kv("is/as", "Machine block (sentence)"));
            lines.push(Line::from(""));
            lines.push(Line::from("Examples: diw ciw yap dab di\" gUit"));
        }
        Some("insert") | Some("i") => {
            lines.push(styled_header("Insert Mode (i a A I o O)"));
            lines.push(line_kv("i/a", "Insert here / one right"));
            lines.push(line_kv("I/A", "Insert at row start / cluster end"));
            lines.push(line_kv("o/O", "Insert one row below / above"));
            lines.push(line_kv("c", "Conveyor belt (1/2/3 = tier)"));
            lines.push(line_kv("s/a", "Smelter / assembler"));
            lines.push(line_kv("b/m", "Splitter / merger"));
            lines.push(line_kv("u/U", "Underground entrance/exit"));
            lines.push(line_kv("e/r", "Coal generator / research lab"));
            lines.push(line_kv("w/p", "Wall / pipe"));
            lines.push(line_kv("C S A Q E...", "UPPERCASE opens category menus"));
            lines.push(line_kv("Shift-HJKL", "Set facing + move"));
            lines.push(line_kv("hjkl", "Move without placing"));
            lines.push(line_kv("Backspace", "Undo last placement"));
            lines.push(line_kv("Esc", "Back to normal mode"));
        }
        Some("visual") | Some("v") => {
            lines.push(styled_header("Visual Mode"));
            lines.push(line_kv("v/V/Ctrl-v", "Char / line / block select"));
            lines.push(line_kv("motions", "Extend (counts work: v5l)"));
            lines.push(line_kv("iw/ib/i\"...", "Select text object"));
            lines.push(line_kv("d/x/y/c", "Demolish / yank / change"));
            lines.push(line_kv("r{key}", "Replace all selected"));
            lines.push(line_kv("~", "Rotate 180 selection"));
            lines.push(line_kv("u/U", "Downgrade/upgrade tiers"));
            lines.push(line_kv(">/<", "Rotate CW/CCW"));
            lines.push(line_kv("I/A (block)", "Column insert left/right edge"));
            lines.push(line_kv("o", "Swap anchor corner"));
            lines.push(line_kv("gv", "Reselect last selection"));
            lines.push(line_kv("p", "Paste over selection"));
        }
        Some("registers") | Some("reg") => {
            lines.push(styled_header("Registers & Macros"));
            lines.push(line_kv("\"a-\"z", "Named registers (\"ayy \"ap)"));
            lines.push(line_kv("\"A-\"Z", "APPEND to register"));
            lines.push(line_kv("\"0", "Last yank"));
            lines.push(line_kv("\"1-\"9", "Delete history"));
            lines.push(line_kv("\"_", "Black hole (delete, keep unnamed)"));
            lines.push(line_kv("qa...q", "Record macro into a"));
            lines.push(line_kv("@a / @@", "Play macro / replay last"));
            lines.push(line_kv("5@a", "Play macro 5 times"));
            lines.push(line_kv(":reg", "Inspect registers"));
        }
        Some("marks") | Some("m") => {
            lines.push(styled_header("Marks & Jumps"));
            lines.push(line_kv("ma", "Set mark a here"));
            lines.push(line_kv("`a / 'a", "Jump to mark exact / row"));
            lines.push(line_kv("`` / ''", "Back to previous jump"));
            lines.push(line_kv("Ctrl-o/Ctrl-i", "Jumplist older/newer"));
            lines.push(line_kv(":marks", "List marks"));
        }
        Some("search") | Some("/") => {
            lines.push(styled_header("Search"));
            lines.push(line_kv("/name", "Search forward (e.g. /smelter)"));
            lines.push(line_kv("?name", "Search backward"));
            lines.push(line_kv("n/N", "Next/previous match"));
            lines.push(line_kv("*/#", "Search entity under cursor fwd/back"));
            lines.push(line_kv(":noh", "Clear highlight"));
        }
        Some("commands") | Some("cmd") | Some(":") => {
            lines.push(styled_header("Command Line"));
            lines.push(line_kv(":s/old/new/", "Substitute in row"));
            lines.push(line_kv(":%s/belt/fastbelt/g", "Substitute whole map"));
            lines.push(line_kv(":5,10s/../../g", "Substitute row range"));
            lines.push(line_kv(":g/wall/d", "Delete all matching"));
            lines.push(line_kv("& / @:", "Repeat :s / last command"));
            lines.push(line_kv(":w :q :wq ZZ", "Save / quit"));
            lines.push(line_kv(":speed N :pause", "Sim control (:step too)"));
            lines.push(line_kv(":level N :restart", "Level select / restart"));
            lines.push(line_kv(":freeplay :menu", "Modes"));
            lines.push(line_kv(":sp :vsp Ctrl-w", "Splits (v s hjkl w c x r q o =)"));
        }
        Some("economy") | Some("eco") => {
            lines.push(styled_header("Economy & Progression (Freeplay)"));
            lines.push(line_kv(":research", "Tech tree (labs eat science packs)"));
            lines.push(line_kv(":contracts", "Delivery contracts"));
            lines.push(line_kv(":market", "Prices & trends"));
            lines.push(line_kv(":finance", "Cash / income / expenses"));
            lines.push(line_kv(":recipe", "Recipe book"));
            lines.push(line_kv(":loan", "Take a $5k loan"));
            lines.push(line_kv(":prestige", "Prestige points"));
            lines.push(Line::from(""));
            lines.push(Line::from("Deliveries auto-sell. Generators need fuel."));
        }
        _ => {
            lines.push(styled_header("VimForge Help"));
            lines.push(Line::from(""));
            lines.push(Line::from("Topic pages — :h <topic>"));
            lines.push(Line::from(""));
            lines.push(line_kv(":h motions", "hjkl w e f % {} Ctrl-d zz jumplist"));
            lines.push(line_kv(":h operators", "d c y gU x s R J Ctrl-a . u"));
            lines.push(line_kv(":h objects", "iw ip ib i\" it + friends"));
            lines.push(line_kv(":h insert", "i a A I o O + building keys"));
            lines.push(line_kv(":h visual", "v V Ctrl-v, block I/A, gv"));
            lines.push(line_kv(":h registers", "\"a \"A \"0-\"9 macros q @"));
            lines.push(line_kv(":h marks", "ma `a '' Ctrl-o/Ctrl-i"));
            lines.push(line_kv(":h search", "/ ? n N * #"));
            lines.push(line_kv(":h commands", ":s :%s :g & @: splits"));
            lines.push(line_kv(":h economy", "research contracts market"));
            lines.push(Line::from(""));
            lines.push(Line::from(Span::styled(
                "j/k scroll - Esc or q to close",
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

fn dim<'a>(text: String) -> Line<'a> {
    Line::from(Span::styled(text, Style::default().fg(Color::Rgb(140, 140, 140))))
}

fn plain<'a>(text: String) -> Line<'a> {
    Line::from(Span::styled(text, Style::default().fg(Color::Rgb(210, 210, 210))))
}

fn good<'a>(text: String) -> Line<'a> {
    Line::from(Span::styled(text, Style::default().fg(Color::Rgb(80, 220, 80))))
}

/// Contract board popup.
fn contracts_content<'a>(app: &AppState) -> (&'static str, Vec<Line<'a>>) {
    let mut lines = Vec::new();
    let board = &app.contract_board;
    let tick = app.simulation.tick_count;

    lines.push(styled_header("Contract Board"));
    lines.push(line_kv("Reputation", &format!("{}", board.reputation)));
    lines.push(line_kv(
        "Completed / Failed",
        &format!("{} / {}", board.completed_count, board.failed_count),
    ));
    lines.push(Line::from(""));

    lines.push(styled_header("Active Contracts"));
    if board.active.is_empty() {
        lines.push(dim("  (none — contracts auto-accept once you have".into()));
        lines.push(dim("   delivered the requested resource before)".into()));
    }
    for c in &board.active {
        lines.push(plain(format!("  {} [{}]", c.name, c.tier.name())));
        for req in &c.requirements {
            lines.push(dim(format!(
                "    {:<20} {}/{}",
                req.resource.name(),
                req.delivered,
                req.quantity
            )));
        }
        let left = c.deadline.saturating_sub(tick);
        lines.push(dim(format!(
            "    reward ${}   deadline in {} ticks",
            c.reward, left
        )));
    }
    lines.push(Line::from(""));

    lines.push(styled_header("Available Contracts — j/k select, Enter to accept"));
    if board.available.is_empty() {
        lines.push(dim("  (a new batch is generated every 300 ticks)".into()));
    }
    for (i, c) in board.available.iter().enumerate() {
        let selected = i == app.popup_cursor;
        if selected {
            lines.push(Line::from(vec![Span::styled(
                format!("> {} [{}]", c.name, c.tier.name()),
                Style::default()
                    .fg(Color::Rgb(20, 24, 34))
                    .bg(Color::Rgb(240, 200, 100))
                    .add_modifier(Modifier::BOLD),
            )]));
        } else {
            lines.push(plain(format!("  {} [{}]", c.name, c.tier.name())));
        }
        for req in &c.requirements {
            lines.push(dim(format!(
                "    {:<20} x{}",
                req.resource.name(),
                req.quantity
            )));
        }
        lines.push(dim(format!("    reward ${}", c.reward)));
    }
    ("Contracts", lines)
}

/// Market prices popup: live prices with trend arrows.
fn market_content<'a>(app: &AppState) -> (&'static str, Vec<Line<'a>>) {
    use crate::resources::Resource;
    let mut lines = Vec::new();
    lines.push(styled_header("Resource Market"));
    lines.push(dim("Output-bin deliveries sell at 80% of market price".into()));
    lines.push(Line::from(""));
    lines.push(Line::from(vec![
        Span::styled(
            format!("  {:<22}{:>8}{:>8}  ", "Resource", "Market", "Sell"),
            Style::default()
                .fg(Color::Rgb(140, 140, 140))
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled(
            "Trend",
            Style::default()
                .fg(Color::Rgb(140, 140, 140))
                .add_modifier(Modifier::BOLD),
        ),
    ]));

    // Fixed ladder of key resources, plus anything the player has produced.
    let mut shown: Vec<Resource> = vec![
        Resource::IronOre,
        Resource::CopperOre,
        Resource::Coal,
        Resource::IronIngot,
        Resource::CopperIngot,
        Resource::Steel,
        Resource::CopperWire,
        Resource::IronPlate,
        Resource::CircuitBoard,
        Resource::Gear,
        Resource::SciencePack1,
        Resource::Processor,
        Resource::QuantumProcessor,
    ];
    let mut extra: Vec<Resource> = app
        .delivered_lifetime
        .keys()
        .copied()
        .filter(|r| !shown.contains(r) && !r.is_waste())
        .collect();
    extra.sort_by_key(|r| r.name());
    shown.extend(extra);

    for r in shown {
        let price = app.market.current_price(r);
        if price <= 0.0 {
            continue;
        }
        let sell = app.market.sell_price(r);
        let dm = app.market.demand_modifier.get(&r).copied().unwrap_or(0.0);
        let sp = app.market.supply_pressure.get(&r).copied().unwrap_or(0.0);
        let (arrow, color) = if dm > 0.05 {
            ('\u{2191}', Color::Rgb(80, 220, 80))
        } else if dm < -0.05 || sp > 0.2 {
            ('\u{2193}', Color::Rgb(220, 80, 80))
        } else {
            ('\u{2192}', Color::Rgb(160, 160, 160))
        };
        lines.push(Line::from(vec![
            Span::styled(
                format!("  {:<22}{:>8}{:>8}  ", r.name(), format!("${:.1}", price), format!("${:.1}", sell)),
                Style::default().fg(Color::Rgb(210, 210, 210)),
            ),
            Span::styled(arrow.to_string(), Style::default().fg(color)),
        ]));
    }
    ("Market", lines)
}

/// Finance overview popup.
fn finance_content<'a>(app: &AppState) -> (&'static str, Vec<Line<'a>>) {
    let mut lines = Vec::new();
    let eco = &app.economy;
    lines.push(styled_header("Finance Overview"));
    lines.push(line_kv("Cash", &format!("${}", eco.cash)));
    lines.push(line_kv("Net worth", &format!("${}", eco.net_worth())));
    lines.push(line_kv("Difficulty", eco.difficulty.name()));
    lines.push(line_kv("Cycle", &format!("{}", eco.cycle)));
    lines.push(Line::from(""));

    lines.push(styled_header("Last Cycle"));
    lines.push(good(format!("  Income (sales)       ${:.0}", app.income_last_cycle)));
    for l in app.last_expense_report.summary_lines() {
        lines.push(dim(l));
    }
    let net = app.income_last_cycle - app.last_expense_report.total;
    lines.push(plain(format!("  {:<20} ${:.0}", "NET", net)));
    lines.push(Line::from(""));

    lines.push(styled_header("Debt"));
    lines.push(line_kv("Outstanding", &format!("${}", app.loans.total_debt())));
    lines.push(line_kv(
        "Available credit",
        &format!("${}", app.loans.available_credit),
    ));
    lines.push(line_kv(
        "Credit rating",
        &format!("{:.2}", eco.credit_rating),
    ));
    lines.push(Line::from(""));
    lines.push(dim("  :loan takes a $5000 loan over 20 cycles".into()));
    lines.push(Line::from(""));

    lines.push(styled_header("Totals"));
    lines.push(line_kv("Earned", &format!("${}", eco.total_earned)));
    lines.push(line_kv("Spent", &format!("${}", eco.total_spent)));
    ("Finance", lines)
}

/// Research/tech tree popup.
fn research_content<'a>(app: &AppState) -> (&'static str, Vec<Line<'a>>) {
    use crate::research::tree::{get_all_techs, is_available};
    let mut lines = Vec::new();
    let rs = &app.research;
    let techs = get_all_techs();
    let total = techs.iter().filter(|t| !t.is_infinite).count();

    lines.push(styled_header("Research"));
    lines.push(line_kv(
        "Completed",
        &format!("{} / {}", rs.completed.len(), total),
    ));
    lines.push(Line::from(""));

    lines.push(styled_header("Current Research"));
    match rs.current {
        Some(id) => {
            let tech = crate::research::tree::get_tech(id);
            let frac = rs.progress_fraction();
            let filled = (frac * 20.0) as usize;
            let bar: String = "\u{2588}".repeat(filled) + &"\u{2591}".repeat(20 - filled.min(20));
            lines.push(plain(format!("  {} (tier {})", tech.name, tech.tier)));
            lines.push(Line::from(vec![
                Span::styled("  ".to_string(), Style::default()),
                Span::styled(bar, Style::default().fg(Color::Rgb(80, 200, 220))),
                Span::styled(
                    format!(" {:.0}%", frac * 100.0),
                    Style::default().fg(Color::Rgb(210, 210, 210)),
                ),
            ]));
            let cost: Vec<String> = tech
                .science_cost
                .iter()
                .map(|(r, n)| format!("{} x{}", r.name(), n))
                .collect();
            lines.push(dim(format!("  needs: {}", cost.join(", "))));
        }
        None => lines.push(dim("  (idle — deliver science packs to a lab)".into())),
    }
    lines.push(Line::from(""));

    // Interactive selection: j/k moves, Enter starts researching.
    lines.push(styled_header("Available Now — j/k select, Enter to research"));
    let available: Vec<_> = techs
        .iter()
        .filter(|t| {
            !t.is_infinite
                && !rs.completed.contains(&t.id)
                && is_available(t.id, &rs.completed)
        })
        .collect();
    if available.is_empty() {
        lines.push(dim("  (nothing available — finish current research)".into()));
    }
    for (i, tech) in available.iter().enumerate() {
        let selected = i == app.popup_cursor;
        let cost: Vec<String> = tech
            .science_cost
            .iter()
            .map(|(r, n)| format!("{} x{}", r.name(), n))
            .collect();
        let prefix = if selected { "> " } else { "  " };
        let style = if selected {
            Style::default()
                .fg(Color::Rgb(20, 24, 34))
                .bg(Color::Rgb(240, 200, 100))
                .add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(Color::Rgb(200, 205, 215))
        };
        lines.push(Line::from(vec![Span::styled(
            format!(
                "{}{} (tier {}) — {}",
                prefix,
                tech.name,
                tech.tier,
                cost.join(", ")
            ),
            style,
        )]));
    }
    lines.push(Line::from(""));
    lines.push(dim("Labs consume science packs delivered by belt.".into()));
    lines.push(dim("If nothing is selected the cheapest tech auto-starts;".into()));
    lines.push(dim("finished techs unlock recipes and grant cash.".into()));
    lines.push(Line::from(""));

    for tier in 1..=5u8 {
        lines.push(styled_header(&format!("Tier {}", tier)));
        for tech in techs.iter().filter(|t| t.tier == tier && !t.is_infinite) {
            let marker = if rs.completed.contains(&tech.id) {
                ("[x]", Color::Rgb(80, 220, 80))
            } else if rs.current == Some(tech.id) {
                ("[>]", Color::Rgb(80, 200, 220))
            } else if is_available(tech.id, &rs.completed) {
                ("[ ]", Color::Rgb(210, 210, 210))
            } else {
                ("[-]", Color::Rgb(90, 90, 100))
            };
            lines.push(Line::from(vec![
                Span::styled(
                    format!("  {} ", marker.0),
                    Style::default().fg(marker.1),
                ),
                Span::styled(
                    tech.name.to_string(),
                    Style::default().fg(marker.1),
                ),
            ]));
            // What it unlocks
            let mut unlocks: Vec<String> = tech
                .unlocks_buildings
                .iter()
                .map(|b| b.name().to_string())
                .collect();
            if !tech.unlocks_recipes.is_empty() {
                unlocks.push(format!("{} recipe(s)", tech.unlocks_recipes.len()));
            }
            if tech.cash_grant > 0 {
                unlocks.push(format!("${}", tech.cash_grant));
            }
            if !unlocks.is_empty() {
                lines.push(dim(format!("        -> {}", unlocks.join(", "))));
            }
        }
        lines.push(Line::from(""));
    }
    ("Research", lines)
}

/// Prestige popup: what a prestige would currently grant.
fn prestige_content<'a>(app: &AppState) -> (&'static str, Vec<Line<'a>>) {
    use crate::scaling::prestige::{prestige_cost, PrestigeBonus};
    let mut lines = Vec::new();
    let p = &app.prestige;
    lines.push(styled_header("Prestige"));
    lines.push(line_kv("Prestige level", &format!("{}", p.level)));
    lines.push(line_kv("Points banked", &format!("{}", p.points)));
    lines.push(Line::from(""));

    let would_earn =
        (app.economy.net_worth().max(0) as u64 / 1000) + app.scaling.level as u64 * 10;
    lines.push(styled_header("If You Prestiged Now"));
    lines.push(plain(format!("  +{} prestige points", would_earn)));
    lines.push(dim(format!(
        "  (net worth ${} / 1000 + scaling {} x 10)",
        app.economy.net_worth().max(0),
        app.scaling.level
    )));
    lines.push(Line::from(""));

    lines.push(styled_header("Bonuses"));
    for b in PrestigeBonus::all() {
        let lvl = p.bonus_level(*b);
        lines.push(plain(format!("  {:<24} lv{}", b.name(), lvl)));
        lines.push(dim(format!(
            "    {} (next: {} pts)",
            b.description(),
            prestige_cost(*b, lvl)
        )));
    }
    lines.push(Line::from(""));
    lines.push(dim("The actual factory reset is not wired yet:".into()));
    lines.push(dim("prestige points accrue for a future update.".into()));
    ("Prestige", lines)
}

/// Recipe book popup: every recipe grouped by building, with lock state.
fn recipes_content<'a>(app: &AppState) -> (&'static str, Vec<Line<'a>>) {
    let mut lines = Vec::new();
    lines.push(styled_header("Recipe Book"));
    lines.push(dim("Locked recipes need research (see :research)".into()));
    lines.push(Line::from(""));

    let recipes = crate::ecs::recipes::all_recipes();
    let mut current_building: Option<crate::resources::EntityType> = None;
    for r in &recipes {
        if current_building != Some(r.building) {
            current_building = Some(r.building);
            lines.push(styled_header(r.building.name()));
        }
        let unlocked = app.simulation.config.recipe_unlocked(r);
        let ins: Vec<String> = r
            .inputs
            .iter()
            .map(|i| format!("{} x{}", i.resource.name(), i.amount))
            .collect();
        let outs: Vec<String> = r
            .outputs
            .iter()
            .map(|o| format!("{} x{}", o.resource.name(), o.amount))
            .collect();
        let text = format!(
            "  {} {} -> {} ({}t)",
            if unlocked { " " } else { "L" },
            ins.join(" + "),
            outs.join(" + "),
            r.ticks
        );
        if unlocked {
            lines.push(plain(text));
        } else {
            lines.push(dim(text));
        }
    }
    ("Recipes", lines)
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
