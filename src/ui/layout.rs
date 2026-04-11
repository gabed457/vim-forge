use ratatui::layout::{Constraint, Direction, Layout, Rect};

/// Minimum terminal dimensions.
pub const MIN_TERMINAL_WIDTH: u16 = 80;
pub const MIN_TERMINAL_HEIGHT: u16 = 24;

/// Sidebar width in columns.
const SIDEBAR_WIDTH: u16 = 16;

/// Tutorial hint bar height in rows.
const TUTORIAL_BAR_HEIGHT: u16 = 3;

/// Status bar height (always 1 row at the bottom).
const STATUS_BAR_HEIGHT: u16 = 1;

/// The computed ratatui areas for each part of the UI.
pub struct LayoutAreas {
    /// Tutorial hint bar at the top (2 rows). None if tutorial is hidden.
    pub tutorial_bar: Option<Rect>,
    /// Main game grid area (where tiles are rendered).
    pub game_grid: Rect,
    /// Sidebar on the right (16 cols). None if sidebar is hidden.
    pub sidebar: Option<Rect>,
    /// Status bar at the bottom (1 row).
    pub status_bar: Rect,
}

/// Compute the layout areas from the terminal frame size and display options.
pub fn compute_layout(frame_size: Rect, show_sidebar: bool, show_tutorial: bool) -> LayoutAreas {
    // Step 1: Split into main area (top) and status bar (bottom, 1 row).
    let outer_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Min(1),
            Constraint::Length(STATUS_BAR_HEIGHT),
        ])
        .split(frame_size);

    let main_area = outer_chunks[0];
    let status_bar = outer_chunks[1];

    // Step 2: Optionally split tutorial bar from the top of the main area.
    let (tutorial_bar, content_area) = if show_tutorial {
        let tutorial_chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(TUTORIAL_BAR_HEIGHT),
                Constraint::Min(1),
            ])
            .split(main_area);
        (Some(tutorial_chunks[0]), tutorial_chunks[1])
    } else {
        (None, main_area)
    };

    // Step 3: Optionally split sidebar from the right of the content area.
    let (game_grid, sidebar) = if show_sidebar {
        let sidebar_chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Min(1),
                Constraint::Length(SIDEBAR_WIDTH),
            ])
            .split(content_area);
        (sidebar_chunks[0], Some(sidebar_chunks[1]))
    } else {
        (content_area, None)
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
