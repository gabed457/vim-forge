use ratatui::style::{Color, Modifier, Style};

use crate::render::animations::{Flash, FlashKind};

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
///
/// The cursor styles set BOTH fg and bg explicitly (rather than relying on
/// REVERSED), so the cursor block stays high-contrast on every possible
/// background — it can never disappear into a same-colored tile.
pub fn highlight_style(ht: HighlightType) -> Style {
    match ht {
        HighlightType::Cursor => Style::default()
            .fg(Color::Rgb(12, 14, 20))
            .bg(Color::Rgb(240, 238, 225))
            .add_modifier(Modifier::BOLD),
        HighlightType::CursorInsert => Style::default()
            .fg(Color::Rgb(8, 20, 10))
            .bg(Color::Rgb(120, 235, 130))
            .add_modifier(Modifier::BOLD),
        HighlightType::VisualSelection => Style::default().bg(Color::Rgb(72, 62, 110)),
        HighlightType::SearchCurrent => Style::default()
            .fg(Color::Rgb(25, 20, 5))
            .bg(Color::Rgb(255, 190, 40))
            .add_modifier(Modifier::BOLD),
        HighlightType::SearchOther => Style::default().bg(Color::Rgb(92, 72, 8)),
        HighlightType::ErrorFlash => Style::default()
            .fg(Color::Rgb(255, 80, 80))
            .bg(Color::Rgb(96, 8, 8))
            .add_modifier(Modifier::BOLD),
        HighlightType::PlacementFlash => Style::default()
            .fg(Color::Rgb(120, 255, 120))
            .bg(Color::Rgb(24, 88, 32))
            .add_modifier(Modifier::BOLD),
        HighlightType::DemolitionFlash => Style::default()
            .fg(Color::Rgb(230, 160, 70))
            .bg(Color::Rgb(70, 36, 12)),
        HighlightType::ContractFlash => Style::default()
            .fg(Color::Rgb(255, 225, 80))
            .bg(Color::Rgb(72, 58, 8))
            .add_modifier(Modifier::BOLD),
        HighlightType::ConnectionChain => Style::default()
            .bg(Color::Rgb(20, 30, 60)),
        HighlightType::Bottleneck => Style::default()
            .bg(Color::Rgb(60, 40, 10)),
        HighlightType::Starved => Style::default()
            .bg(Color::Rgb(50, 50, 10)),
    }
}

/// Style for a mark badge glyph (small vim-mark letter shown on its tile).
pub fn mark_badge_style() -> Style {
    Style::default()
        .fg(Color::Rgb(20, 18, 8))
        .bg(Color::Rgb(212, 175, 55))
        .add_modifier(Modifier::BOLD)
}

/// Map a flash animation kind to the highlight used to paint its tile.
pub fn flash_highlight(kind: FlashKind) -> HighlightType {
    match kind {
        FlashKind::Placement => HighlightType::PlacementFlash,
        FlashKind::Error => HighlightType::ErrorFlash,
        FlashKind::Demolition => HighlightType::DemolitionFlash,
        FlashKind::ContractComplete | FlashKind::ResearchComplete => HighlightType::ContractFlash,
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
    flashes: &[Flash],
) -> Option<HighlightType> {
    // Check flashes first (highest priority)
    for flash in flashes {
        if flash.x == x && flash.y == y {
            return Some(flash_highlight(flash.kind));
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
