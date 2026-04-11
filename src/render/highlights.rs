use ratatui::style::{Color, Modifier, Style};

/// The different types of highlights that can be applied to tiles.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum HighlightType {
    /// Normal mode cursor.
    Cursor,
    /// Insert mode cursor.
    CursorInsert,
    /// Visual mode selection.
    VisualSelection,
    /// Current search match (the one the cursor is on).
    SearchCurrent,
    /// Other search matches.
    SearchOther,
    /// Error flash (e.g., invalid placement).
    ErrorFlash,
    /// Placement flash (successful entity placement).
    PlacementFlash,
    /// Demolition flash.
    DemolitionFlash,
    /// Contract completion flash.
    ContractFlash,
    /// Connected chain highlight (blue glow).
    ConnectionChain,
    /// Bottleneck indicator (belt at 100% capacity).
    Bottleneck,
    /// Starved machine (waiting for input).
    Starved,
}

/// Returns the ratatui Style for a given highlight type. ONLY uses Color::Rgb.
pub fn highlight_style(ht: HighlightType) -> Style {
    match ht {
        HighlightType::Cursor => Style::default()
            .bg(Color::Rgb(80, 80, 120))
            .add_modifier(Modifier::REVERSED),
        HighlightType::CursorInsert => Style::default()
            .bg(Color::Rgb(60, 120, 60))
            .add_modifier(Modifier::REVERSED),
        HighlightType::VisualSelection => Style::default().bg(Color::Rgb(100, 80, 40)),
        HighlightType::SearchCurrent => Style::default()
            .bg(Color::Rgb(200, 150, 0))
            .add_modifier(Modifier::BOLD),
        HighlightType::SearchOther => Style::default().bg(Color::Rgb(80, 60, 0)),
        HighlightType::ErrorFlash => Style::default()
            .fg(Color::Rgb(255, 60, 60))
            .bg(Color::Rgb(80, 0, 0)),
        HighlightType::PlacementFlash => Style::default()
            .fg(Color::Rgb(80, 220, 80))
            .bg(Color::Rgb(30, 80, 30))
            .add_modifier(Modifier::BOLD),
        HighlightType::DemolitionFlash => Style::default()
            .fg(Color::Rgb(200, 140, 60))
            .bg(Color::Rgb(60, 30, 10)),
        HighlightType::ContractFlash => Style::default()
            .fg(Color::Rgb(255, 215, 0))
            .bg(Color::Rgb(60, 50, 10))
            .add_modifier(Modifier::BOLD),
        HighlightType::ConnectionChain => Style::default()
            .bg(Color::Rgb(20, 30, 60)),
        HighlightType::Bottleneck => Style::default()
            .bg(Color::Rgb(60, 40, 10)),
        HighlightType::Starved => Style::default()
            .bg(Color::Rgb(50, 50, 10)),
    }
}

/// Determine what highlight (if any) applies to a tile at (x, y), given the current
/// game state information. Returns the highest-priority highlight type.
///
/// Priority order (highest first):
/// 1. Error flash / Placement flash / Demolition flash / Contract flash (animations)
/// 2. Cursor (normal or insert)
/// 3. Search current match
/// 4. Visual selection
/// 5. Search other matches
/// 6. Connection chain / bottleneck / starved (lowest)
pub fn resolve_highlight(
    x: usize,
    y: usize,
    cursor_x: usize,
    cursor_y: usize,
    is_insert_mode: bool,
    visual_tiles: &[(usize, usize)],
    search_matches: &[(usize, usize)],
    search_current: Option<usize>,
    flash_positions: &[(usize, usize, bool)], // (x, y, is_error)
) -> Option<HighlightType> {
    // Check flashes first (highest priority)
    for &(fx, fy, is_error) in flash_positions {
        if fx == x && fy == y {
            return Some(if is_error {
                HighlightType::ErrorFlash
            } else {
                HighlightType::PlacementFlash
            });
        }
    }

    // Cursor
    if x == cursor_x && y == cursor_y {
        return Some(if is_insert_mode {
            HighlightType::CursorInsert
        } else {
            HighlightType::Cursor
        });
    }

    // Search current match
    if let Some(current_idx) = search_current {
        if let Some(&(sx, sy)) = search_matches.get(current_idx) {
            if sx == x && sy == y {
                return Some(HighlightType::SearchCurrent);
            }
        }
    }

    // Visual selection
    if visual_tiles.iter().any(|&(vx, vy)| vx == x && vy == y) {
        return Some(HighlightType::VisualSelection);
    }

    // Other search matches
    if search_matches.iter().any(|&(sx, sy)| sx == x && sy == y) {
        return Some(HighlightType::SearchOther);
    }

    None
}
