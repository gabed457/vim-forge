use ratatui::layout::Rect;

use crate::render::viewport::Viewport;
use crate::resources::Direction;

/// Maximum number of simultaneous panes.
const MAX_PANES: usize = 4;

/// How the panes are arranged.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum SplitLayout {
    /// Single full-screen pane.
    Single,
    /// Two panes side by side. The u16 is the percentage width of the left pane (0-100).
    Vertical(u16),
    /// Two panes stacked. The u16 is the percentage height of the top pane (0-100).
    Horizontal(u16),
    /// Four panes in a 2x2 grid.
    Grid,
}

/// A single pane within the split layout.
#[derive(Clone, Debug)]
pub struct Pane {
    /// Viewport/camera for this pane.
    pub viewport: Viewport,
    /// Computed screen area during render (set by layout pass).
    pub area: Rect,
}

impl Pane {
    pub fn new(width: usize, height: usize) -> Self {
        Pane {
            viewport: Viewport::new(width, height),
            area: Rect::default(),
        }
    }
}

/// Manages all panes and the active pane index.
pub struct SplitManager {
    pub panes: Vec<Pane>,
    pub active_pane: usize,
    pub layout: SplitLayout,
}

impl SplitManager {
    pub fn new() -> Self {
        SplitManager {
            panes: vec![Pane::new(80, 24)],
            active_pane: 0,
            layout: SplitLayout::Single,
        }
    }

    /// Get a reference to the active pane.
    pub fn active(&self) -> &Pane {
        &self.panes[self.active_pane]
    }

    /// Get a mutable reference to the active pane.
    pub fn active_mut(&mut self) -> &mut Pane {
        &mut self.panes[self.active_pane]
    }

    /// Split the active pane vertically (side by side). Returns false if already at max.
    pub fn split_vertical(&mut self) -> bool {
        if self.panes.len() >= MAX_PANES {
            return false;
        }
        let active = &self.panes[self.active_pane];
        let new_pane = Pane {
            viewport: active.viewport.clone(),
            area: Rect::default(),
        };
        self.panes.push(new_pane);
        self.layout = match self.panes.len() {
            2 => SplitLayout::Vertical(50),
            3 => {
                // Keep existing layout direction; add to it
                match &self.layout {
                    SplitLayout::Vertical(_) => SplitLayout::Vertical(33),
                    _ => SplitLayout::Vertical(50),
                }
            }
            4 => SplitLayout::Grid,
            _ => self.layout.clone(),
        };
        // Focus the new pane
        self.active_pane = self.panes.len() - 1;
        true
    }

    /// Split the active pane horizontally (stacked). Returns false if already at max.
    pub fn split_horizontal(&mut self) -> bool {
        if self.panes.len() >= MAX_PANES {
            return false;
        }
        let active = &self.panes[self.active_pane];
        let new_pane = Pane {
            viewport: active.viewport.clone(),
            area: Rect::default(),
        };
        self.panes.push(new_pane);
        self.layout = match self.panes.len() {
            2 => SplitLayout::Horizontal(50),
            3 => {
                match &self.layout {
                    SplitLayout::Horizontal(_) => SplitLayout::Horizontal(33),
                    _ => SplitLayout::Horizontal(50),
                }
            }
            4 => SplitLayout::Grid,
            _ => self.layout.clone(),
        };
        self.active_pane = self.panes.len() - 1;
        true
    }

    /// Close the active pane. If it is the last pane, does nothing and returns false.
    pub fn close_pane(&mut self) -> bool {
        if self.panes.len() <= 1 {
            return false;
        }
        self.panes.remove(self.active_pane);
        if self.active_pane >= self.panes.len() {
            self.active_pane = self.panes.len() - 1;
        }
        self.update_layout_after_remove();
        true
    }

    /// Close all panes except the active one.
    pub fn close_others(&mut self) {
        if self.panes.len() <= 1 {
            return;
        }
        let active = self.panes.remove(self.active_pane);
        self.panes.clear();
        self.panes.push(active);
        self.active_pane = 0;
        self.layout = SplitLayout::Single;
    }

    /// Equalize all pane sizes (reset split percentages to even).
    pub fn equalize(&mut self) {
        match self.panes.len() {
            1 => self.layout = SplitLayout::Single,
            2 => {
                self.layout = match &self.layout {
                    SplitLayout::Horizontal(_) => SplitLayout::Horizontal(50),
                    _ => SplitLayout::Vertical(50),
                };
            }
            3 => {
                self.layout = match &self.layout {
                    SplitLayout::Horizontal(_) => SplitLayout::Horizontal(33),
                    _ => SplitLayout::Vertical(33),
                };
            }
            _ => self.layout = SplitLayout::Grid,
        }
    }

    /// Move focus to a pane in the given direction. Uses the pane areas to determine
    /// spatial relationships.
    pub fn focus_direction(&mut self, direction: Direction) {
        if self.panes.len() <= 1 {
            return;
        }

        let current_area = self.panes[self.active_pane].area;
        let cx = current_area.x as i32 + current_area.width as i32 / 2;
        let cy = current_area.y as i32 + current_area.height as i32 / 2;

        let mut best_idx = None;
        let mut best_dist = i32::MAX;

        for (i, pane) in self.panes.iter().enumerate() {
            if i == self.active_pane {
                continue;
            }
            let px = pane.area.x as i32 + pane.area.width as i32 / 2;
            let py = pane.area.y as i32 + pane.area.height as i32 / 2;

            let valid = match direction {
                Direction::Left => px < cx,
                Direction::Right => px > cx,
                Direction::Up => py < cy,
                Direction::Down => py > cy,
            };

            if valid {
                let dist = (px - cx).abs() + (py - cy).abs();
                if dist < best_dist {
                    best_dist = dist;
                    best_idx = Some(i);
                }
            }
        }

        if let Some(idx) = best_idx {
            self.active_pane = idx;
        }
    }

    /// Compute the Rect areas for each pane given a total available area.
    pub fn compute_areas(&mut self, total: Rect) {
        match (&self.layout, self.panes.len()) {
            (SplitLayout::Single, _) | (_, 1) => {
                if let Some(pane) = self.panes.first_mut() {
                    pane.area = total;
                }
            }
            (SplitLayout::Vertical(pct), 2) => {
                let left_w = (total.width as u32 * *pct as u32 / 100) as u16;
                let right_w = total.width.saturating_sub(left_w).saturating_sub(1); // 1 for border
                self.panes[0].area = Rect::new(total.x, total.y, left_w, total.height);
                self.panes[1].area =
                    Rect::new(total.x + left_w + 1, total.y, right_w, total.height);
            }
            (SplitLayout::Horizontal(pct), 2) => {
                let top_h = (total.height as u32 * *pct as u32 / 100) as u16;
                let bot_h = total.height.saturating_sub(top_h).saturating_sub(1); // 1 for border
                self.panes[0].area = Rect::new(total.x, total.y, total.width, top_h);
                self.panes[1].area =
                    Rect::new(total.x, total.y + top_h + 1, total.width, bot_h);
            }
            (SplitLayout::Vertical(_), 3) => {
                let col_w = total.width / 3;
                let rem = total.width - col_w * 3;
                for (i, pane) in self.panes.iter_mut().enumerate() {
                    let extra = if i == 2 { rem } else { 0 };
                    pane.area = Rect::new(
                        total.x + col_w * i as u16,
                        total.y,
                        col_w + extra,
                        total.height,
                    );
                }
            }
            (SplitLayout::Horizontal(_), 3) => {
                let row_h = total.height / 3;
                let rem = total.height - row_h * 3;
                for (i, pane) in self.panes.iter_mut().enumerate() {
                    let extra = if i == 2 { rem } else { 0 };
                    pane.area = Rect::new(
                        total.x,
                        total.y + row_h * i as u16,
                        total.width,
                        row_h + extra,
                    );
                }
            }
            (SplitLayout::Grid, _) => {
                let half_w = total.width / 2;
                let half_h = total.height / 2;
                let rem_w = total.width - half_w * 2;
                let rem_h = total.height - half_h * 2;
                let positions = [
                    (0, 0, half_w, half_h),
                    (half_w, 0, half_w + rem_w, half_h),
                    (0, half_h, half_w, half_h + rem_h),
                    (half_w, half_h, half_w + rem_w, half_h + rem_h),
                ];
                for (i, pane) in self.panes.iter_mut().enumerate() {
                    if i < positions.len() {
                        let (px, py, pw, ph) = positions[i];
                        pane.area = Rect::new(total.x + px, total.y + py, pw, ph);
                    }
                }
            }
            // Fallback: give the whole area to each pane
            _ => {
                for pane in &mut self.panes {
                    pane.area = total;
                }
            }
        }

        // Update each pane's viewport dimensions to match its area.
        // We use 2 chars per tile horizontally, so tile columns = area_width / 2.
        for pane in &mut self.panes {
            let tile_cols = (pane.area.width as usize) / 2;
            let tile_rows = pane.area.height as usize;
            pane.viewport.resize(tile_cols, tile_rows);
        }
    }

    /// Number of panes.
    pub fn pane_count(&self) -> usize {
        self.panes.len()
    }

    fn update_layout_after_remove(&mut self) {
        match self.panes.len() {
            1 => self.layout = SplitLayout::Single,
            2 => {
                // Keep the current direction but reset to 50%
                self.layout = match &self.layout {
                    SplitLayout::Horizontal(_) | SplitLayout::Grid => SplitLayout::Horizontal(50),
                    _ => SplitLayout::Vertical(50),
                };
            }
            3 => {
                self.layout = match &self.layout {
                    SplitLayout::Horizontal(_) => SplitLayout::Horizontal(33),
                    _ => SplitLayout::Vertical(33),
                };
            }
            _ => {}
        }
    }
}
