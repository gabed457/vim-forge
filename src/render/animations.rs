/// The kind of flash animation.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FlashKind {
    /// Successful entity placement.
    Placement,
    /// Error (invalid placement, etc.).
    Error,
    /// Demolition effect.
    Demolition,
    /// Contract completion celebration.
    ContractComplete,
    /// Research completion.
    ResearchComplete,
}

/// A single flash animation on a tile.
#[derive(Clone, Copy, Debug)]
pub struct Flash {
    pub x: usize,
    pub y: usize,
    pub kind: FlashKind,
    /// Number of frames remaining.
    pub frames_remaining: u8,
}

/// Default flash duration in frames (~100ms at 30fps).
const DEFAULT_FLASH_FRAMES: u8 = 3;

/// Status bar flash overlay.
#[derive(Clone, Copy, Debug)]
pub struct StatusFlash {
    pub color: (u8, u8, u8),
    pub frames_remaining: u8,
}

/// Manages all active flash animations.
pub struct AnimationManager {
    flashes: Vec<Flash>,
    pub status_flash: Option<StatusFlash>,
    /// Global frame counter for cycling animations (belt, terrain, etc.)
    pub frame_counter: u32,
}

impl AnimationManager {
    pub fn new() -> Self {
        AnimationManager {
            flashes: Vec::new(),
            status_flash: None,
            frame_counter: 0,
        }
    }

    /// Add a placement flash at the given position.
    pub fn flash_placement(&mut self, x: usize, y: usize) {
        self.flashes.push(Flash {
            x,
            y,
            kind: FlashKind::Placement,
            frames_remaining: 5,
        });
    }

    /// Add an error flash at the given position.
    pub fn flash_error(&mut self, x: usize, y: usize) {
        self.flashes.push(Flash {
            x,
            y,
            kind: FlashKind::Error,
            frames_remaining: 4,
        });
    }

    /// Add a demolition effect at the given position.
    pub fn flash_demolition(&mut self, x: usize, y: usize) {
        self.flashes.push(Flash {
            x,
            y,
            kind: FlashKind::Demolition,
            frames_remaining: DEFAULT_FLASH_FRAMES,
        });
    }

    /// Add a contract completion burst at the given position.
    pub fn flash_contract_complete(&mut self, x: usize, y: usize) {
        self.flashes.push(Flash {
            x,
            y,
            kind: FlashKind::ContractComplete,
            frames_remaining: 6,
        });
    }

    /// Add a research completion effect at the given position.
    pub fn flash_research_complete(&mut self, x: usize, y: usize) {
        self.flashes.push(Flash {
            x,
            y,
            kind: FlashKind::ResearchComplete,
            frames_remaining: 5,
        });
    }

    /// Flash the status bar with a temporary color.
    pub fn flash_status(&mut self, color: (u8, u8, u8)) {
        self.status_flash = Some(StatusFlash {
            color,
            frames_remaining: 6,
        });
    }

    /// Flash status bar gold (success).
    pub fn flash_status_success(&mut self) {
        self.flash_status((255, 200, 60));
    }

    /// Flash status bar red (failure).
    pub fn flash_status_failure(&mut self) {
        self.flash_status((200, 40, 40));
    }

    /// Tick all animations, removing any that have expired.
    pub fn tick(&mut self) {
        self.frame_counter = self.frame_counter.wrapping_add(1);

        for flash in &mut self.flashes {
            flash.frames_remaining = flash.frames_remaining.saturating_sub(1);
        }
        self.flashes.retain(|f| f.frames_remaining > 0);

        if let Some(sf) = &mut self.status_flash {
            sf.frames_remaining = sf.frames_remaining.saturating_sub(1);
            if sf.frames_remaining == 0 {
                self.status_flash = None;
            }
        }
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
        !self.flashes.is_empty() || self.status_flash.is_some()
    }

    /// Get all active flashes (for direct iteration).
    pub fn flashes(&self) -> &[Flash] {
        &self.flashes
    }

    /// Clear all animations.
    pub fn clear(&mut self) {
        self.flashes.clear();
        self.status_flash = None;
    }
}
