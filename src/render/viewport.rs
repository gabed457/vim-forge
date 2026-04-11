/// Number of tiles to keep between the cursor and viewport edge before scrolling.
const SCROLL_MARGIN: usize = 5;

/// A camera/viewport into the game map. Tracks which portion of the map is visible.
#[derive(Clone, Debug)]
pub struct Viewport {
    /// Horizontal tile offset (leftmost visible column).
    pub offset_x: usize,
    /// Vertical tile offset (topmost visible row).
    pub offset_y: usize,
    /// Number of visible tile columns.
    pub width: usize,
    /// Number of visible tile rows.
    pub height: usize,
}

impl Viewport {
    /// Create a new viewport at the origin with the given dimensions.
    pub fn new(width: usize, height: usize) -> Self {
        Viewport {
            offset_x: 0,
            offset_y: 0,
            width,
            height,
        }
    }

    /// Scroll the viewport so that the cursor at (cx, cy) remains within SCROLL_MARGIN
    /// of the viewport edges. Clamps to the map bounds.
    pub fn follow_cursor(
        &mut self,
        cursor_x: usize,
        cursor_y: usize,
        map_width: usize,
        map_height: usize,
    ) {
        // Horizontal scrolling
        if self.width > 0 {
            let margin = SCROLL_MARGIN.min(self.width / 2);

            // Scroll right if cursor is too close to the right edge
            if cursor_x >= self.offset_x + self.width.saturating_sub(margin) {
                self.offset_x = cursor_x
                    .saturating_sub(self.width.saturating_sub(margin).saturating_sub(1));
            }
            // Scroll left if cursor is too close to the left edge
            if cursor_x < self.offset_x + margin {
                self.offset_x = cursor_x.saturating_sub(margin);
            }
        }

        // Vertical scrolling
        if self.height > 0 {
            let margin = SCROLL_MARGIN.min(self.height / 2);

            // Scroll down if cursor is too close to the bottom edge
            if cursor_y >= self.offset_y + self.height.saturating_sub(margin) {
                self.offset_y = cursor_y
                    .saturating_sub(self.height.saturating_sub(margin).saturating_sub(1));
            }
            // Scroll up if cursor is too close to the top edge
            if cursor_y < self.offset_y + margin {
                self.offset_y = cursor_y.saturating_sub(margin);
            }
        }

        self.clamp_to_map(map_width, map_height);
    }

    /// Clamp the viewport offset so it does not extend past the map.
    pub fn clamp_to_map(&mut self, map_width: usize, map_height: usize) {
        if map_width > self.width {
            self.offset_x = self.offset_x.min(map_width - self.width);
        } else {
            self.offset_x = 0;
        }
        if map_height > self.height {
            self.offset_y = self.offset_y.min(map_height - self.height);
        } else {
            self.offset_y = 0;
        }
    }

    /// Resize the viewport (e.g., when the terminal or pane size changes).
    pub fn resize(&mut self, width: usize, height: usize) {
        self.width = width;
        self.height = height;
    }

    /// Check if a map coordinate (x, y) is within the visible viewport.
    pub fn contains(&self, x: usize, y: usize) -> bool {
        x >= self.offset_x
            && x < self.offset_x + self.width
            && y >= self.offset_y
            && y < self.offset_y + self.height
    }

    /// Convert a map coordinate to a screen-relative coordinate, if visible.
    pub fn to_screen(&self, x: usize, y: usize) -> Option<(u16, u16)> {
        if self.contains(x, y) {
            Some((
                (x - self.offset_x) as u16,
                (y - self.offset_y) as u16,
            ))
        } else {
            None
        }
    }

    /// Map position of the top row of the viewport (for H command).
    pub fn top_row(&self) -> usize {
        self.offset_y
    }

    /// Map position of the middle row of the viewport (for M command).
    pub fn middle_row(&self, map_height: usize) -> usize {
        let max_row = map_height.saturating_sub(1);
        let mid = self.offset_y + self.height / 2;
        mid.min(max_row)
    }

    /// Map position of the bottom row of the viewport (for L command).
    pub fn bottom_row(&self, map_height: usize) -> usize {
        let max_row = map_height.saturating_sub(1);
        let bottom = self.offset_y + self.height.saturating_sub(1);
        bottom.min(max_row)
    }

    /// Center the viewport on a specific coordinate.
    pub fn center_on(&mut self, x: usize, y: usize, map_width: usize, map_height: usize) {
        self.offset_x = x.saturating_sub(self.width / 2);
        self.offset_y = y.saturating_sub(self.height / 2);
        self.clamp_to_map(map_width, map_height);
    }

    /// Scroll up by a given number of lines.
    pub fn scroll_up(&mut self, lines: usize) {
        self.offset_y = self.offset_y.saturating_sub(lines);
    }

    /// Scroll down by a given number of lines, clamping to map bounds.
    pub fn scroll_down(&mut self, lines: usize, map_height: usize) {
        self.offset_y += lines;
        if map_height > self.height {
            self.offset_y = self.offset_y.min(map_height - self.height);
        } else {
            self.offset_y = 0;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_follow_cursor_scroll_right() {
        let mut vp = Viewport::new(20, 10);
        vp.follow_cursor(18, 5, 40, 20);
        assert!(vp.offset_x > 0);
        assert!(vp.contains(18, 5));
    }

    #[test]
    fn test_clamp_to_map() {
        let mut vp = Viewport::new(20, 10);
        vp.offset_x = 100;
        vp.offset_y = 100;
        vp.clamp_to_map(30, 15);
        assert_eq!(vp.offset_x, 10);
        assert_eq!(vp.offset_y, 5);
    }

    #[test]
    fn test_viewport_positions() {
        let vp = Viewport {
            offset_x: 0,
            offset_y: 5,
            width: 20,
            height: 10,
        };
        assert_eq!(vp.top_row(), 5);
        assert_eq!(vp.middle_row(30), 10);
        assert_eq!(vp.bottom_row(30), 14);
    }
}
