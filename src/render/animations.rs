/// The kind of flash animation.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FlashKind {
    /// Successful entity placement.
    Placement,
    /// Error (invalid placement, etc.).
    Error,
}

/// A single flash animation on a tile.
#[derive(Clone, Copy, Debug)]
pub struct Flash {
    pub x: usize,
    pub y: usize,
    pub kind: FlashKind,
    /// Number of frames remaining. At 30 fps, 3 frames is approximately 100ms.
    pub frames_remaining: u8,
}

/// Default flash duration in frames (~100ms at 30fps).
const DEFAULT_FLASH_FRAMES: u8 = 3;

/// Manages all active flash animations.
pub struct AnimationManager {
    flashes: Vec<Flash>,
}

impl AnimationManager {
    pub fn new() -> Self {
        AnimationManager {
            flashes: Vec::new(),
        }
    }

    /// Add a placement flash at the given position.
    pub fn flash_placement(&mut self, x: usize, y: usize) {
        self.flashes.push(Flash {
            x,
            y,
            kind: FlashKind::Placement,
            frames_remaining: DEFAULT_FLASH_FRAMES,
        });
    }

    /// Add an error flash at the given position.
    pub fn flash_error(&mut self, x: usize, y: usize) {
        self.flashes.push(Flash {
            x,
            y,
            kind: FlashKind::Error,
            frames_remaining: DEFAULT_FLASH_FRAMES,
        });
    }

    /// Tick all animations, removing any that have expired.
    pub fn tick(&mut self) {
        for flash in &mut self.flashes {
            flash.frames_remaining = flash.frames_remaining.saturating_sub(1);
        }
        self.flashes.retain(|f| f.frames_remaining > 0);
    }

    /// Returns all active flash positions as (x, y, is_error) tuples,
    /// suitable for passing to highlight resolution.
    pub fn flash_positions(&self) -> Vec<(usize, usize, bool)> {
        self.flashes
            .iter()
            .map(|f| (f.x, f.y, f.kind == FlashKind::Error))
            .collect()
    }

    /// Check if there are any active animations.
    pub fn has_active(&self) -> bool {
        !self.flashes.is_empty()
    }

    /// Get all active flashes (for direct iteration).
    pub fn flashes(&self) -> &[Flash] {
        &self.flashes
    }

    /// Clear all animations.
    pub fn clear(&mut self) {
        self.flashes.clear();
    }
}
