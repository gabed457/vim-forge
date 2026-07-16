use ratatui::layout::Rect;

/// Minimum terminal dimensions.
pub const MIN_TERMINAL_WIDTH: u16 = 80;
pub const MIN_TERMINAL_HEIGHT: u16 = 24;

/// Right-hand Command Dock width in columns.
pub const DOCK_WIDTH: u16 = 30;

/// The dock only appears on wide terminals; below this it hides entirely so
/// the grid keeps every column.
pub const MIN_WIDTH_FOR_DOCK: u16 = 140;

/// Top bar height in rows (level chip + objective row, hint carousel row).
pub const TOP_BAR_HEIGHT: u16 = 2;

/// Below this terminal height the top bar collapses to a single row.
pub const MIN_HEIGHT_FOR_FULL_TOP_BAR: u16 = 30;

/// Status bar height (always 1 row at the bottom).
const STATUS_BAR_HEIGHT: u16 = 1;

/// Terminal heights below this drop the top bar so the grid keeps room.
const MIN_HEIGHT_FOR_TUTORIAL: u16 = 12;

/// The computed ratatui areas for each part of the UI.
///
/// Field names are kept stable (`tutorial_bar` = the 2-row top bar,
/// `sidebar` = the 30-col Command Dock) so existing call sites keep working.
pub struct LayoutAreas {
    /// Top bar (2 rows; 1 row on short terminals). None if hidden.
    pub tutorial_bar: Option<Rect>,
    /// Main game grid area, edge to edge (where tiles are rendered).
    pub game_grid: Rect,
    /// Command Dock on the right (30 cols, only on wide terminals). None if hidden.
    pub sidebar: Option<Rect>,
    /// Status bar at the bottom (1 row).
    pub status_bar: Rect,
}

/// Compute the layout areas from the terminal frame size and display options.
///
/// Layout contract:
/// - Top bar: 2 rows (1 row below `MIN_HEIGHT_FOR_FULL_TOP_BAR`, dropped
///   entirely below `MIN_HEIGHT_FOR_TUTORIAL`).
/// - Command Dock: 30 columns on the right, only when width >= 140.
/// - Status bar: 1 row at the bottom.
/// - The grid gets ALL remaining cells, edge to edge — no gaps.
pub fn compute_layout(frame_size: Rect, show_sidebar: bool, show_tutorial: bool) -> LayoutAreas {
    // Graceful degradation on cramped terminals.
    let show_top = show_tutorial && frame_size.height >= MIN_HEIGHT_FOR_TUTORIAL;
    let top_h = if !show_top {
        0
    } else if frame_size.height >= MIN_HEIGHT_FOR_FULL_TOP_BAR {
        TOP_BAR_HEIGHT
    } else {
        1
    };
    let show_dock = show_sidebar && frame_size.width >= MIN_WIDTH_FOR_DOCK;
    let dock_w = if show_dock { DOCK_WIDTH } else { 0 };

    let x = frame_size.x;
    let y = frame_size.y;
    let w = frame_size.width;
    let h = frame_size.height;

    let status_bar = Rect::new(x, y + h.saturating_sub(STATUS_BAR_HEIGHT), w, STATUS_BAR_HEIGHT);

    let tutorial_bar = if top_h > 0 {
        Some(Rect::new(x, y, w, top_h))
    } else {
        None
    };

    // Middle band: everything between the top bar and the status bar.
    let mid_y = y + top_h;
    let mid_h = h.saturating_sub(top_h + STATUS_BAR_HEIGHT).max(1);

    let game_grid = Rect::new(x, mid_y, w.saturating_sub(dock_w).max(1), mid_h);
    let sidebar = if show_dock {
        Some(Rect::new(x + w.saturating_sub(dock_w), mid_y, dock_w, mid_h))
    } else {
        None
    };

    LayoutAreas {
        tutorial_bar,
        game_grid,
        sidebar,
        status_bar,
    }
}

/// Check if the terminal size meets the minimum requirements.
pub fn is_terminal_too_small(frame_size: Rect) -> bool {
    frame_size.width < MIN_TERMINAL_WIDTH || frame_size.height < MIN_TERMINAL_HEIGHT
}

/// Render a "terminal too small" warning message area (centered).
pub fn too_small_area(frame_size: Rect) -> Rect {
    let msg_w = 40u16.min(frame_size.width);
    let msg_h = 3u16.min(frame_size.height);
    let x = (frame_size.width.saturating_sub(msg_w)) / 2;
    let y = (frame_size.height.saturating_sub(msg_h)) / 2;
    Rect::new(x + frame_size.x, y + frame_size.y, msg_w, msg_h)
}
