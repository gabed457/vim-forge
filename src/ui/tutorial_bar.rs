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

    let style = Style::default().fg(Color::White).bg(TUTORIAL_BG);
    let bold = style.add_modifier(Modifier::BOLD);
    let title_style = Style::default()
        .fg(Color::Yellow)
        .bg(TUTORIAL_BG)
        .add_modifier(Modifier::BOLD);
    let goal_style = Style::default()
        .fg(Color::Green)
        .bg(TUTORIAL_BG)
        .add_modifier(Modifier::BOLD);
    let progress_style = Style::default()
        .fg(Color::Cyan)
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
            .fg(Color::Magenta)
            .bg(TUTORIAL_BG)
            .add_modifier(Modifier::BOLD);
        let counter_style = Style::default()
            .fg(Color::DarkGray)
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

/// Build colored legend spans showing what entity glyphs mean.
/// Early levels show entity symbols; later levels show key commands.
fn entity_legend_spans(level: usize) -> Vec<Span<'static>> {
    let bg = TUTORIAL_BG;
    let label = |text: &str| -> Span<'static> {
        Span::styled(
            text.to_string(),
            Style::default().fg(Color::White).bg(bg),
        )
    };
    let glyph = |ch: &str, color: Color| -> Span<'static> {
        Span::styled(
            ch.to_string(),
            Style::default()
                .fg(color)
                .bg(bg)
                .add_modifier(Modifier::BOLD),
        )
    };

    match level {
        1 => vec![
            glyph("O", Color::Rgb(139, 119, 42)),
            label("=Ore  "),
            glyph("\u{2192}", Color::White),
            label("=Conveyor  "),
            glyph("S", Color::Red),
            label("=Smelter  "),
            glyph("B", Color::Green),
            label("=Output Bin"),
        ],
        2 | 3 => vec![
            glyph("O", Color::Rgb(139, 119, 42)),
            label("=Ore  "),
            glyph("\u{2192}", Color::White),
            label("=Conveyor  "),
            glyph("S", Color::Red),
            label("=Smelter  "),
            glyph("B", Color::Green),
            label("=Bin  "),
            glyph("i", Color::Rgb(100, 220, 100)),
            label("=Enter Insert mode"),
        ],
        4 => vec![
            glyph("O", Color::Rgb(139, 119, 42)),
            label("=Ore  "),
            glyph("S", Color::Red),
            label("=Smelter  "),
            glyph("A", Color::Cyan),
            label("=Assembler  "),
            glyph("B", Color::Green),
            label("=Bin"),
        ],
        5 => vec![
            label("x=Delete  d+motion=Delete range  ~=Rotate  i=Insert mode"),
        ],
        6 | 7 => vec![
            label("yy=Copy row  p=Paste  \"a=Use register a"),
        ],
        8 => vec![
            label("Ctrl-v=Block select  y=Copy  p=Paste"),
        ],
        9 => vec![
            label("f<c>=Find entity  /=Search  %=Follow connection"),
        ],
        10 => vec![
            label("qa=Record macro  q=Stop  @a=Replay  4@a=Replay 4x"),
        ],
        11 => vec![
            label("~=Rotate entity  .=Repeat last action"),
        ],
        12 => vec![
            label("ma=Set mark a  'a=Jump to mark a"),
        ],
        13 => vec![
            label("Ctrl-w v=V-split  Ctrl-w s=H-split  Ctrl-w h/j/k/l=Switch"),
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
