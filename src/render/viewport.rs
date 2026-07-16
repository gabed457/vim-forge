/// Number of tiles to keep between the cursor and viewport edge before scrolling.
const SCROLL_MARGIN: usize = 5;

/// Minimum / maximum tile scale. At scale S one map tile renders as
/// (2*S columns x S rows) of terminal cells.
pub const MIN_SCALE: u8 = 1;
pub const MAX_SCALE: u8 = 3;

/// A camera/viewport into the game map. Tracks which portion of the map is
/// visible and at what tile scale it is drawn.
///
/// The viewport stores the grid rect dimensions in terminal CELLS
/// (`rect_cols` x `rect_rows`) and derives its `width`/`height` in TILES from
/// them: `width = rect_cols / (2*scale)`, `height = rect_rows / scale`.
#[derive(Clone, Debug)]
pub struct Viewport {
    /// Horizontal tile offset (leftmost visible column).
    pub offset_x: usize,
    /// Vertical tile offset (topmost visible row).
    pub offset_y: usize,
    /// Number of visible tile columns (at the current scale).
    pub width: usize,
    /// Number of visible tile rows (at the current scale).
    pub height: usize,
    /// Terminal-cell padding for centering (applied in render_grid).
    pub pad_left: u16,
    pub pad_top: u16,
    /// Tile scale (1..=3). One tile = 2*scale cols x scale rows.
    pub scale: u8,
    /// Set when the player zoomed manually (zi/zo). While set, auto-fit
    /// (level load / terminal resize) leaves the scale alone. Cleared by
    /// `zoom_fit` and on level change (via `reset_zoom_for_level`).
    pub manual_zoom: bool,
    /// Grid rect width in terminal cells.
    pub rect_cols: u16,
    /// Grid rect height in terminal rows.
    pub rect_rows: u16,
}

impl Viewport {
    /// Create a new viewport at the origin with the given dimensions.
    ///
    /// For backward compatibility `width`/`height` are TILE dimensions at
    /// scale 1 (i.e. rect_cols/2 x rect_rows), which is what all existing
    /// callers pass.
    pub fn new(width: usize, height: usize) -> Self {
        Viewport {
            offset_x: 0,
            offset_y: 0,
            width,
            height,
            pad_left: 0,
            pad_top: 0,
            scale: 1,
            manual_zoom: false,
            rect_cols: (width * 2) as u16,
            rect_rows: height as u16,
        }
    }

    // -----------------------------------------------------------------------
    // Scale / zoom
    // -----------------------------------------------------------------------

    /// The size of one map tile in terminal cells at the current scale:
    /// (columns, rows).
    pub fn tile_cell_size(&self) -> (u16, u16) {
        (2 * self.scale as u16, self.scale as u16)
    }

    /// Largest scale S in MIN_SCALE..=MAX_SCALE at which the ENTIRE map
    /// (map_w x map_h tiles) fits inside a grid rect of rect_cols x rect_rows
    /// terminal cells. Returns MIN_SCALE if even that doesn't fit.
    pub fn fit_scale(map_w: usize, map_h: usize, rect_cols: u16, rect_rows: u16) -> u8 {
        let cols = rect_cols as usize;
        let rows = rect_rows as usize;
        for s in (MIN_SCALE..=MAX_SCALE).rev() {
            let s_us = s as usize;
            if map_w * 2 * s_us <= cols && map_h * s_us <= rows {
                return s;
            }
        }
        MIN_SCALE
    }

    /// Set the scale (clamped to 1..=3) and rederive tile width/height from
    /// the stored rect dimensions. Does NOT touch `manual_zoom`.
    pub fn set_scale(&mut self, scale: u8) {
        self.scale = scale.clamp(MIN_SCALE, MAX_SCALE);
        self.recompute_tile_dims();
    }

    /// Zoom in one step (player action: `zi`). Sets the manual-zoom flag so
    /// later auto-fits keep their hands off.
    pub fn zoom_in(&mut self, map_width: usize, map_height: usize) {
        self.manual_zoom = true;
        self.set_scale(self.scale.saturating_add(1));
        self.clamp_to_map(map_width, map_height);
    }

    /// Zoom out one step (player action: `zo`). Sets the manual-zoom flag.
    pub fn zoom_out(&mut self, map_width: usize, map_height: usize) {
        self.manual_zoom = true;
        self.set_scale(self.scale.saturating_sub(1).max(MIN_SCALE));
        self.clamp_to_map(map_width, map_height);
    }

    /// Re-fit the map (player action: `zf`). Clears the manual-zoom flag and
    /// picks the largest scale that shows the whole map.
    pub fn zoom_fit(&mut self, map_width: usize, map_height: usize) {
        self.manual_zoom = false;
        self.set_scale(Self::fit_scale(map_width, map_height, self.rect_cols, self.rect_rows));
        self.clamp_to_map(map_width, map_height);
    }

    /// Auto-fit for level load / terminal resize: refits ONLY when the
    /// player hasn't zoomed manually.
    pub fn auto_fit(&mut self, map_width: usize, map_height: usize) {
        if !self.manual_zoom {
            self.set_scale(Self::fit_scale(
                map_width, map_height, self.rect_cols, self.rect_rows,
            ));
        }
        self.clamp_to_map(map_width, map_height);
    }

    /// Level change: clear the manual-zoom flag, then fit the new map.
    pub fn reset_zoom_for_level(&mut self, map_width: usize, map_height: usize) {
        self.manual_zoom = false;
        self.offset_x = 0;
        self.offset_y = 0;
        self.auto_fit(map_width, map_height);
    }

    /// Rederive tile width/height from rect dims / scale.
    fn recompute_tile_dims(&mut self) {
        let s = self.scale.max(1) as usize;
        self.width = (self.rect_cols as usize / (2 * s)).max(1);
        self.height = (self.rect_rows as usize / s).max(1);
    }

    // -----------------------------------------------------------------------
    // Scrolling / clamping (all in TILE units at the current scale)
    // -----------------------------------------------------------------------

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

    /// Clamp the viewport offset so it does not extend past the map. When the
    /// map is smaller than the viewport, compute cell padding that centers it
    /// within the grid rect (scale-aware).
    pub fn clamp_to_map(&mut self, map_width: usize, map_height: usize) {
        let s = self.scale.max(1) as usize;
        if map_width > self.width {
            self.offset_x = self.offset_x.min(map_width - self.width);
            self.pad_left = 0;
        } else {
            self.offset_x = 0;
            // Center the map horizontally: each tile = 2*scale terminal cols.
            let map_cell_width = map_width * 2 * s;
            let rect_cols = self.rect_cols as usize;
            self.pad_left = (rect_cols.saturating_sub(map_cell_width) / 2) as u16;
        }
        if map_height > self.height {
            self.offset_y = self.offset_y.min(map_height - self.height);
            self.pad_top = 0;
        } else {
            self.offset_y = 0;
            let map_cell_height = map_height * s;
            let rect_rows = self.rect_rows as usize;
            self.pad_top = (rect_rows.saturating_sub(map_cell_height) / 2) as u16;
        }
    }

    /// Resize the viewport (e.g., when the terminal or pane size changes).
    ///
    /// Backward-compatible signature: `width`/`height` are TILE dimensions at
    /// scale 1 (rect_cols/2 x rect_rows), which is what existing callers
    /// compute. Stores the equivalent cell dims and rederives the tile dims
    /// at the CURRENT scale.
    pub fn resize(&mut self, width: usize, height: usize) {
        self.rect_cols = (width * 2) as u16;
        self.rect_rows = height as u16;
        self.recompute_tile_dims();
    }

    /// Resize using exact terminal-cell dimensions of the grid rect.
    pub fn resize_cells(&mut self, rect_cols: u16, rect_rows: u16) {
        self.rect_cols = rect_cols;
        self.rect_rows = rect_rows;
        self.recompute_tile_dims();
    }

    /// Check if a map coordinate (x, y) is within the visible viewport.
    pub fn contains(&self, x: usize, y: usize) -> bool {
        x >= self.offset_x
            && x < self.offset_x + self.width
            && y >= self.offset_y
            && y < self.offset_y + self.height
    }

    /// Convert a map coordinate to a screen-relative TILE coordinate, if visible.
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

    /// zt: put the cursor row at the top of the viewport (respecting the
    /// scroll margin, like vim with 'scrolloff' set).
    pub fn cursor_to_top(&mut self, y: usize, map_width: usize, map_height: usize) {
        let margin = SCROLL_MARGIN.min(self.height / 2);
        self.offset_y = y.saturating_sub(margin);
        self.clamp_to_map(map_width, map_height);
    }

    /// zb: put the cursor row at the bottom of the viewport (respecting
    /// the scroll margin).
    pub fn cursor_to_bottom(&mut self, y: usize, map_width: usize, map_height: usize) {
        let margin = SCROLL_MARGIN.min(self.height / 2);
        self.offset_y = (y + margin + 1).saturating_sub(self.height);
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
        let mut vp = Viewport::new(20, 10);
        vp.offset_y = 5;
        assert_eq!(vp.top_row(), 5);
        assert_eq!(vp.middle_row(30), 10);
        assert_eq!(vp.bottom_row(30), 14);
    }

    #[test]
    fn test_fit_scale_prefers_largest_fitting() {
        // 88x22 tile map in a 210x52 grid rect: S=1 (176<=210, 22<=52) fits,
        // S=2 needs 352 cols -> no. But a 30x15 map: S=3 needs 180x45 -> fits.
        assert_eq!(Viewport::fit_scale(88, 22, 210, 52), 1);
        assert_eq!(Viewport::fit_scale(30, 15, 210, 52), 3);
        assert_eq!(Viewport::fit_scale(40, 20, 210, 52), 2);
        // Nothing fits: min 1.
        assert_eq!(Viewport::fit_scale(500, 500, 80, 24), 1);
    }

    #[test]
    fn test_set_scale_rederives_tile_dims() {
        let mut vp = Viewport::new(100, 48); // rect = 200x48 cells
        assert_eq!((vp.width, vp.height), (100, 48));
        vp.set_scale(2);
        assert_eq!((vp.width, vp.height), (50, 24));
        vp.set_scale(3);
        assert_eq!((vp.width, vp.height), (33, 16));
        vp.set_scale(9); // clamps to 3
        assert_eq!(vp.scale, 3);
        vp.set_scale(0); // clamps to 1
        assert_eq!(vp.scale, 1);
        assert_eq!((vp.width, vp.height), (100, 48));
    }

    #[test]
    fn test_manual_zoom_blocks_auto_fit() {
        let mut vp = Viewport::new(105, 50); // rect = 210x50
        vp.zoom_in(30, 15); // scale 2, manual
        assert_eq!(vp.scale, 2);
        assert!(vp.manual_zoom);
        vp.auto_fit(30, 15); // would pick 3, but manual zoom holds
        assert_eq!(vp.scale, 2);
        vp.zoom_fit(30, 15); // explicit re-fit clears the flag
        assert_eq!(vp.scale, 3);
        assert!(!vp.manual_zoom);
        vp.zoom_out(30, 15);
        assert_eq!(vp.scale, 2);
        vp.reset_zoom_for_level(30, 15); // level change: flag cleared, refit
        assert_eq!(vp.scale, 3);
        assert!(!vp.manual_zoom);
    }

    #[test]
    fn test_resize_keeps_scale_and_rederives() {
        let mut vp = Viewport::new(105, 50);
        vp.set_scale(2);
        vp.resize(80, 40); // legacy S=1 tile dims -> rect 160x40
        assert_eq!(vp.scale, 2);
        assert_eq!((vp.width, vp.height), (40, 20));
        vp.resize_cells(210, 52);
        assert_eq!((vp.width, vp.height), (52, 26));
    }

    #[test]
    fn test_clamp_centers_small_map_per_scale() {
        let mut vp = Viewport::new(105, 50); // rect 210x50
        vp.set_scale(3);
        vp.clamp_to_map(20, 10); // map 20x10 tiles = 120x30 cells at S=3
        assert_eq!(vp.pad_left, (210 - 120) / 2);
        assert_eq!(vp.pad_top, (50 - 30) / 2);
        // Follow-cursor still clamps correctly at scale.
        vp.set_scale(2); // 52x25 tiles visible
        vp.follow_cursor(19, 9, 20, 10);
        assert_eq!((vp.offset_x, vp.offset_y), (0, 0));
    }
}
