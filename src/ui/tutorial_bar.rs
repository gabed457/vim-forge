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

/// Background color for the top bar.
const TUTORIAL_BG: Color = Color::Rgb(30, 30, 60);

/// Render the 2-row top bar.
///
/// Row 1: level chip + objective + live progress (right-aligned).
/// Row 2: hint carousel (key-caps highlighted) + taught-keys strip (right).
///
/// On short terminals the layout hands us a 1-row area; only row 1 renders.
pub fn render_tutorial_bar(frame: &mut Frame, area: Rect, app: &AppState, tut: &TutorialState) {
    if area.height == 0 || area.width == 0 {
        return;
    }

    let style = Style::default().fg(Color::Rgb(200, 200, 210)).bg(TUTORIAL_BG);
    let bold = style.add_modifier(Modifier::BOLD);
    // Level chip: solid cyan block so the level number/name pops.
    let chip_style = Style::default()
        .fg(Color::Rgb(6, 22, 30))
        .bg(Color::Rgb(80, 200, 255))
        .add_modifier(Modifier::BOLD);
    let name_style = Style::default()
        .fg(Color::Rgb(80, 200, 255))
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

    // -- Row 1: level chip + objective + live progress (right-aligned) --
    let mut row1: Vec<Span> = vec![
        Span::styled(format!(" LEVEL {} ", level), chip_style),
        Span::styled(format!(" {} ", name), name_style),
        Span::styled("\u{25B8} ", style),
    ];
    let progress = progress_text(app, tut);
    // The live progress readout always stays visible: truncate the objective
    // prose (with an ellipsis) rather than letting it push progress offscreen.
    let chip_len: usize = row1.iter().map(|s| s.content.chars().count()).sum();
    let progress_len = progress.chars().count() + 2;
    let obj_budget = (area.width as usize).saturating_sub(chip_len + progress_len);
    let objective_text = truncate_ellipsis(objective, obj_budget);
    row1.extend(highlight_keys(&objective_text, bold, TUTORIAL_BG));
    let progress_span = Span::styled(format!("{} ", progress), progress_style);
    push_right_aligned(&mut row1, vec![progress_span], area.width, style);
    lines.push(Line::from(row1));

    // -- Row 2: hint carousel + taught-keys strip (right-aligned) --
    if area.height >= 2 {
        let config = get_level(level);
        let num_hints = config.as_ref().map(|c| c.hints.len()).unwrap_or(0);
        let hint_text =
            hints::get_hint(level, tut.current_hint_index).unwrap_or("Explore the level!");
        let hint_chip_style = Style::default()
            .fg(Color::Rgb(200, 160, 80))
            .bg(TUTORIAL_BG)
            .add_modifier(Modifier::BOLD);
        let counter_style = Style::default()
            .fg(Color::Rgb(110, 110, 135))
            .bg(TUTORIAL_BG);
        let counter = if num_hints > 1 {
            format!("({}/{}) ", tut.current_hint_index + 1, num_hints)
        } else {
            String::new()
        };
        let mut row2: Vec<Span> = vec![
            Span::styled(" \u{2726} HINT ", hint_chip_style),
            Span::styled(counter, counter_style),
        ];
        row2.extend(highlight_keys(hint_text, style, TUTORIAL_BG));

        // Taught-keys strip: the keys this level is teaching, as key-caps.
        let keys = taught_keys(level);
        if !keys.is_empty() {
            let mut strip: Vec<Span> = vec![Span::styled(
                "\u{2328} ",
                Style::default().fg(Color::Rgb(130, 140, 165)).bg(TUTORIAL_BG),
            )];
            let key_style = key_token_style(TUTORIAL_BG);
            for (i, key) in keys.iter().enumerate() {
                if i > 0 {
                    strip.push(Span::styled(
                        "\u{00B7}",
                        Style::default().fg(Color::Rgb(70, 75, 110)).bg(TUTORIAL_BG),
                    ));
                }
                strip.push(Span::styled(key.clone(), key_style));
            }
            strip.push(Span::styled(" ", style));
            push_right_aligned(&mut row2, strip, area.width, style);
        }
        lines.push(Line::from(row2));
    }

    let paragraph = Paragraph::new(lines).style(style);
    frame.render_widget(paragraph, area);
}

/// Append `right` spans to `row`, padded so they hug the right edge.
/// If the row is already too wide, the right group is dropped rather than
/// overflowing (the Paragraph would clip mid-span otherwise).
fn push_right_aligned<'a>(
    row: &mut Vec<Span<'a>>,
    right: Vec<Span<'a>>,
    width: u16,
    pad_style: Style,
) {
    let left_len: usize = row.iter().map(|s| s.content.chars().count()).sum();
    let right_len: usize = right.iter().map(|s| s.content.chars().count()).sum();
    let total = left_len + right_len;
    if (width as usize) > total {
        row.push(Span::styled(
            " ".repeat(width as usize - total),
            pad_style,
        ));
        row.extend(right);
    } else if (width as usize) > left_len + right_len.min(4) {
        // Tight fit: single space separator, let the edge clip.
        row.push(Span::styled(" ", pad_style));
        row.extend(right);
    }
}

/// The keys a level teaches, for the row-2 strip.
///
/// Prefers the level's `allowed_commands` whitelist (exactly the keys the
/// player may use); when a level allows everything, falls back to the key
/// tokens mentioned in its first hint.
fn taught_keys(level: usize) -> Vec<String> {
    let config = match get_level(level) {
        Some(c) => c,
        None => return Vec::new(),
    };
    if let Some(allowed) = &config.allowed_commands {
        return allowed.iter().take(10).map(|s| s.to_string()).collect();
    }
    // Fallback: pull the key tokens out of the hints (first hints first).
    let mut keys: Vec<String> = Vec::new();
    'hints: for hint in &config.hints {
        for word in hint.split_whitespace() {
            let core = word
                .trim_matches(|c: char| matches!(c, '(' | ')' | '.' | ',' | '!' | '?' | ':' | ';'));
            if !core.is_empty() && is_key_token(core) && !keys.iter().any(|k| k == core) {
                keys.push(core.to_string());
                if keys.len() >= 8 {
                    break 'hints;
                }
            }
        }
    }
    keys
}

/// Truncate `text` to at most `budget` chars, appending `…` when cut.
fn truncate_ellipsis(text: &str, budget: usize) -> String {
    if text.chars().count() <= budget {
        return text.to_string();
    }
    if budget == 0 {
        return String::new();
    }
    let cut: String = text.chars().take(budget - 1).collect();
    format!("{}\u{2026}", cut.trim_end())
}

// ---------------------------------------------------------------------------
// Key-name highlighting in hint prose
// ---------------------------------------------------------------------------

/// Style for a highlighted key name inside hint text: gold key-cap.
fn key_token_style(bg: Color) -> Style {
    Style::default()
        .fg(Color::Rgb(255, 216, 100))
        .bg(bg)
        .add_modifier(Modifier::BOLD)
}

/// Split hint prose into spans, rendering vim-key tokens (`h/j/k/l`, `5l`,
/// `Esc`, `gg`, `$`, `i`, `c`, ...) in a bright key-cap style so the keys to
/// press jump out of the sentence.
fn highlight_keys(text: &str, base: Style, bg: Color) -> Vec<Span<'static>> {
    let key_style = key_token_style(bg);
    let mut spans: Vec<Span<'static>> = Vec::new();
    let mut plain = String::new();

    for word in text.split_inclusive(' ') {
        let trimmed = word.trim_end();
        let trailing_space = word.len() - trimmed.len();
        // Strip trailing punctuation for matching, keep it as prose.
        let stripped = trimmed.trim_end_matches(['.', ',', '!', '?', ':', ';', ')']);
        let punct = &trimmed[stripped.len()..];
        // Strip a leading paren too.
        let (lead, core) = if let Some(rest) = stripped.strip_prefix('(') {
            ("(", rest)
        } else {
            ("", stripped)
        };

        if !core.is_empty() && is_key_token(core) {
            if !plain.is_empty() || !lead.is_empty() {
                spans.push(Span::styled(format!("{}{}", plain, lead), base));
                plain = String::new();
            }
            spans.push(Span::styled(core.to_string(), key_style));
            plain.push_str(punct);
            plain.push_str(&" ".repeat(trailing_space));
        } else {
            plain.push_str(word);
        }
    }
    if !plain.is_empty() {
        spans.push(Span::styled(plain, base));
    }
    spans
}

/// Heuristic: is this word a vim key (or key sequence) the player can press?
fn is_key_token(word: &str) -> bool {
    // Named keys / chords
    if matches!(
        word,
        "Esc" | "Enter" | "Space" | "Tab" | "Arrows" | "Ctrl-v" | "Ctrl-w" | "Ctrl-r"
    ) {
        return true;
    }
    // Slash-joined sequences like h/j/k/l or 0/$
    if word.contains('/') && word.len() <= 11 {
        return word.split('/').all(|p| !p.is_empty() && is_simple_key(p));
    }
    is_simple_key(word)
}

/// Single vim key or count+key combo: `h`, `gg`, `$`, `5l`, `10j`, `@a`, `qa`, `yy`.
fn is_simple_key(word: &str) -> bool {
    // Count prefix + motion: 5l, 10j, 2yy, 4@a
    let rest = word.trim_start_matches(|c: char| c.is_ascii_digit());
    if rest.len() != word.len() && !rest.is_empty() {
        return is_simple_key(rest);
    }
    matches!(
        word,
        // Motions
        "h" | "j" | "k" | "l" | "0" | "$" | "gg" | "G" | "H" | "M" | "L" | "w" | "b" | "%"
        // Editing / placement keys
        | "i" | "c" | "s" | "x" | "d" | "dd" | "p" | "P" | "yy" | "y" | "u" | "~" | "."
        // Macros, registers, marks, search
        | "q" | "qa" | "@a" | "\"a" | "ma" | "mb" | "mc" | "md" | "'a" | "'b" | "fs" | "fb"
    )
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
        CompletionCondition::UseCommands(cmds) => {
            let used = cmds
                .iter()
                .filter(|c| tut.commands_used.contains(c.as_str()))
                .count();
            if used < cmds.len() {
                // Show up to three commands still missing.
                let missing: Vec<&str> = cmds
                    .iter()
                    .filter(|c| !tut.commands_used.contains(c.as_str()))
                    .take(3)
                    .map(|c| c.as_str())
                    .collect();
                format!("[{}/{} keys — need: {}]", used, cmds.len(), missing.join(" "))
            } else {
                format!("[{}/{} keys]", used, cmds.len())
            }
        }
        CompletionCondition::ScoreInMoves(target, max_edits) => {
            let over = tut.edit_count > *max_edits;
            if over {
                format!(
                    "[{}/{} widgets | edits {}/{} OVER PAR — :restart]",
                    widgets, target, tut.edit_count, max_edits
                )
            } else {
                format!(
                    "[{}/{} widgets | edits {}/{}]",
                    widgets, target, tut.edit_count, max_edits
                )
            }
        }
        CompletionCondition::Custom(name) => match name.as_str() {
            "all_conveyors_facing_right" => {
                let mut total = 0usize;
                let mut right = 0usize;
                for (_e, (kind, facing)) in app
                    .world
                    .query::<(
                        &crate::ecs::components::EntityKind,
                        &crate::ecs::components::FacingComponent,
                    )>()
                    .iter()
                {
                    if kind.kind == crate::resources::EntityType::BasicBelt {
                        total += 1;
                        if facing.facing == crate::resources::Facing::Right {
                            right += 1;
                        }
                    }
                }
                format!("[{right}/{total} belts facing right]")
            }
            "all_5_clusters_producing" => {
                let mut bins = 0usize;
                let mut producing = 0usize;
                for (_e, c) in app
                    .world
                    .query::<&crate::ecs::components::OutputCounter>()
                    .iter()
                {
                    bins += 1;
                    if c.total() >= 1 {
                        producing += 1;
                    }
                }
                format!("[{producing}/{bins} bins flowing]")
            }
            _ => String::new(),
        },
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
