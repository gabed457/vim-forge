//! Helper functions that resolve a motion to a destination position
//! given the current cursor and map state. These are used by the parser
//! after it determines the motion type, and by the input handler to
//! resolve commands into concrete positions.

use crate::resources::Direction;

/// Compute a destination from a Move command, clamping to map bounds.
pub fn move_direction(
    x: usize,
    y: usize,
    dir: Direction,
    count: usize,
    map_width: usize,
    map_height: usize,
) -> (usize, usize) {
    match dir {
        Direction::Left => {
            let new_x = x.saturating_sub(count);
            (new_x, y)
        }
        Direction::Right => {
            let new_x = (x + count).min(map_width.saturating_sub(1));
            (new_x, y)
        }
        Direction::Up => {
            let new_y = y.saturating_sub(count);
            (x, new_y)
        }
        Direction::Down => {
            let new_y = (y + count).min(map_height.saturating_sub(1));
            (x, new_y)
        }
    }
}

/// Resolve line start (0 key): column 0, same row.
pub fn line_start(y: usize) -> (usize, usize) {
    (0, y)
}

/// Resolve line end ($ key): last column, same row.
pub fn line_end(y: usize, map_width: usize) -> (usize, usize) {
    (map_width.saturating_sub(1), y)
}

/// Resolve map start (gg): row 0 (or specified row), column 0.
pub fn map_start(row: Option<usize>, map_height: usize) -> (usize, usize) {
    match row {
        Some(r) => (0, r.min(map_height.saturating_sub(1))),
        None => (0, 0),
    }
}

/// Resolve map end (G): last row (or specified row), column 0.
pub fn map_end(row: Option<usize>, map_height: usize) -> (usize, usize) {
    match row {
        Some(r) => (0, r.min(map_height.saturating_sub(1))),
        None => (0, map_height.saturating_sub(1)),
    }
}

/// Resolve viewport top (H): y = viewport_top_row.
pub fn viewport_top(x: usize, viewport_top_row: usize) -> (usize, usize) {
    (x, viewport_top_row)
}

/// Resolve viewport middle (M): y = middle of viewport.
pub fn viewport_middle(x: usize, viewport_top_row: usize, viewport_height: usize) -> (usize, usize) {
    (x, viewport_top_row + viewport_height / 2)
}

/// Resolve viewport bottom (L): y = viewport_top_row + viewport_height - 1.
pub fn viewport_bottom(
    x: usize,
    viewport_top_row: usize,
    viewport_height: usize,
    map_height: usize,
) -> (usize, usize) {
    let bottom = (viewport_top_row + viewport_height.saturating_sub(1)).min(map_height.saturating_sub(1));
    (x, bottom)
}

/// Compute the range of tiles between two positions in a horizontal line.
/// Returns (start_x, end_x) where start_x <= end_x, all on the same row.
/// For motions that move within a single row.
pub fn horizontal_range(x1: usize, x2: usize) -> (usize, usize) {
    if x1 <= x2 {
        (x1, x2)
    } else {
        (x2, x1)
    }
}

/// Compute linewise row range from two y coordinates.
pub fn linewise_range(y1: usize, y2: usize) -> (usize, usize) {
    if y1 <= y2 {
        (y1, y2)
    } else {
        (y2, y1)
    }
}
