use ratatui::layout::Rect;
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::Paragraph;
use ratatui::Frame;

use crate::app::AppState;
use crate::ecs::components::OutputCounter;
use crate::levels::config::{get_level, CompletionCondition};
use crate::tutorial::engine::TutorialState;
use crate::tutorial::hints;

/// Background color for the tutorial hint bar.
const TUTORIAL_BG: Color = Color::Rgb(30, 30, 60);

/// Render the tutorial hint bar (3 rows at top of screen).
///
/// Row 1: Level name + entity legend (colored to match in-game glyphs)
/// Row 2: Goal with progress indicator
/// Row 3: Current hint with counter
pub fn render_tutorial_bar(frame: &mut Frame, area: Rect, app: &AppState, tut: &TutorialState) {
    if area.height == 0 || area.width == 0 {
        return;
    }

    let style = Style::default().fg(Color::Rgb(200, 200, 210)).bg(TUTORIAL_BG);
    let bold = style.add_modifier(Modifier::BOLD);
    let title_style = Style::default()
        .fg(Color::Rgb(80, 200, 255))
        .bg(TUTORIAL_BG)
        .add_modifier(Modifier::BOLD);
    let goal_style = Style::default()
        .fg(Color::Rgb(255, 220, 60))
        .bg(TUTORIAL_BG)
        .add_modifier(Modifier::BOLD);
    let progress_style = Style::default()
        .fg(Color::Rgb(80, 255, 120))
        .bg(TUTORIAL_BG)
        .add_modifier(Modifier::BOLD);

    let level = tut.current_level;
    let name = hints::get_level_name(level);
    let objective = hints::get_objective(level);

    let mut lines: Vec<Line> = Vec::new();

    // Row 1: Level title + colored entity legend
    let mut row1 = vec![
        Span::styled(format!(" Level {}: {} ", level, name), title_style),
        Span::styled("| ", style),
    ];
    row1.extend(entity_legend_spans(level));
    lines.push(Line::from(row1));

    // Row 2: Goal + progress indicator
    if area.height >= 2 {
        let progress = progress_text(app, tut);
        let row2 = vec![
            Span::styled(" GOAL: ", goal_style),
            Span::styled(format!("{} ", objective), bold),
            Span::styled(progress, progress_style),
        ];
        lines.push(Line::from(row2));
    }

    // Row 3: Current hint with counter
    if area.height >= 3 {
        let config = get_level(level);
        let num_hints = config.as_ref().map(|c| c.hints.len()).unwrap_or(0);
        let hint_text = hints::get_hint(level, tut.current_hint_index)
            .unwrap_or("Explore the level!");
        let hint_style = Style::default()
            .fg(Color::Rgb(200, 160, 80))
            .bg(TUTORIAL_BG)
            .add_modifier(Modifier::BOLD);
        let counter_style = Style::default()
            .fg(Color::Rgb(80, 80, 100))
            .bg(TUTORIAL_BG);
        let counter = if num_hints > 1 {
            format!("({}/{}) ", tut.current_hint_index + 1, num_hints)
        } else {
            String::new()
        };
        let row3 = vec![
            Span::styled(" Hint: ", hint_style),
            Span::styled(counter, counter_style),
            Span::styled(format!("{} ", hint_text), style),
        ];
        lines.push(Line::from(row3));
    }

    let paragraph = Paragraph::new(lines).style(style);
    frame.render_widget(paragraph, area);
}

/// Build colored legend spans showing key-action mappings for the current level.
/// Uses gold for keys and white for descriptions.
fn entity_legend_spans(level: usize) -> Vec<Span<'static>> {
    let bg = TUTORIAL_BG;
    let key_color = Color::Rgb(255, 200, 80);
    let desc_color = Color::White;

    let k = |text: &str| -> Span<'static> {
        Span::styled(
            text.to_string(),
            Style::default()
                .fg(key_color)
                .bg(bg)
                .add_modifier(Modifier::BOLD),
        )
    };
    let d = |text: &str| -> Span<'static> {
        Span::styled(
            text.to_string(),
            Style::default().fg(desc_color).bg(bg),
        )
    };

    match level {
        1 => vec![
            k("hjkl"), d("=Move  "),
            k("5l"), d("=Move 5 right  "),
            k("0"), d("/"), k("$"), d("=Line start/end  "),
            k("gg"), d("/"), k("G"), d("=Top/Bottom"),
        ],
        2 => vec![
            k("i"), d("=Insert mode  "),
            k("c"), d("=Place belt  "),
            k("Esc"), d("=Back to Normal  "),
            k("hjkl"), d("=Move (insert)"),
        ],
        3 => vec![
            k("c"), d("=Belt  "),
            k("s"), d("=Smelter (3x3)  "),
            k("k"), d("/"), k("j"), d("=Up/Down (align rows)  "),
            k("Arrows"), d("=Set facing"),
        ],
        4 => vec![
            k("c"), d("=Belt  "),
            k("s"), d("=Smelter  "),
            k("a"), d("=Assembler (3x4)  "),
            k("Arrows"), d("=Turn belts"),
        ],
        5 => vec![
            k("~"), d("=Rotate CW  "),
            k("x"), d("=Delete  "),
            k("d"), d("+motion=Range delete  "),
            k("i"), d("=Insert"),
        ],
        6 => vec![
            k("yy"), d("=Copy row  "),
            k("p"), d("=Paste  "),
            k("j"), d("=Move down  "),
            k("Esc"), d("=Normal mode"),
        ],
        7 => vec![
            k("\"a"), d("=Select register a  "),
            k("2yy"), d("=Copy 2 rows  "),
            k("p"), d("=Paste from register"),
        ],
        8 => vec![
            k("Ctrl-v"), d("=Visual Block  "),
            k("y"), d("=Copy block  "),
            k("p"), d("=Paste block"),
        ],
        9 => vec![
            k("fs"), d("=Find smelter  "),
            k("fb"), d("=Find bin  "),
            k("/"), d("=Search  "),
            k("%"), d("=Follow chain"),
        ],
        10 => vec![
            k("qa"), d("=Record macro  "),
            k("q"), d("=Stop  "),
            k("@a"), d("=Replay  "),
            k("4@a"), d("=Replay 4x"),
        ],
        11 => vec![
            k("~"), d("=Rotate CW  "),
            k("."), d("=Repeat last edit  "),
            k("l"), d("=Move right"),
        ],
        12 => vec![
            k("ma"), d("=Set mark a  "),
            k("'a"), d("=Jump to mark a  "),
            k("mb"), d("/"), k("mc"), d("/"), k("md"), d("=More marks"),
        ],
        13 => vec![
            k("Ctrl-w v"), d("=V-split  "),
            k("Ctrl-w s"), d("=H-split  "),
            k("Ctrl-w h/l"), d("=Switch panes"),
        ],
        _ => vec![],
    }
}

/// Compute progress text for the current level's completion condition.
fn progress_text(app: &AppState, tut: &TutorialState) -> String {
    let config = match get_level(tut.current_level) {
        Some(c) => c,
        None => return String::new(),
    };

    let (ore, ingots, widgets) = total_output_counts(app);

    match &config.completion {
        CompletionCondition::NavigateToAll(positions) => {
            let visited = positions
                .iter()
                .filter(|p| tut.visited_positions.contains(p))
                .count();
            format!("[{}/{} found]", visited, positions.len())
        }
        CompletionCondition::ProduceWidgets(target) => {
            format!("[{}/{} widgets]", widgets, target)
        }
        CompletionCondition::DeliverOre(target) => {
            format!("[{}/{} ore]", ore, target)
        }
        CompletionCondition::DeliverIngots(target) => {
            format!("[{}/{} ingots]", ingots, target)
        }
        _ => String::new(),
    }
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
