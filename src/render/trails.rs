/// A brief color trail left when a resource moves off a belt tile.
#[derive(Clone, Debug)]
pub struct Trail {
    pub x: usize,
    pub y: usize,
    pub color: (u8, u8, u8),
    pub frames_remaining: u8,
}

/// Manages all active trails.
pub struct TrailSystem {
    trails: Vec<Trail>,
}

impl TrailSystem {
    pub fn new() -> Self {
        TrailSystem {
            trails: Vec::new(),
        }
    }

    /// Add a 2-frame trail at the given position with the resource's color.
    pub fn add_trail(&mut self, x: usize, y: usize, resource_color: (u8, u8, u8)) {
        // Replace any existing trail at this position
        self.trails.retain(|t| !(t.x == x && t.y == y));
        self.trails.push(Trail {
            x,
            y,
            color: resource_color,
            frames_remaining: 2,
        });
    }

    /// Tick all trails, removing expired ones.
    pub fn tick(&mut self) {
        for trail in &mut self.trails {
            trail.frames_remaining = trail.frames_remaining.saturating_sub(1);
        }
        self.trails.retain(|t| t.frames_remaining > 0);
    }

    /// Get the trail at a position, if any.
    pub fn get_at(&self, x: usize, y: usize) -> Option<&Trail> {
        self.trails.iter().find(|t| t.x == x && t.y == y)
    }

    /// Clear all trails.
    pub fn clear(&mut self) {
        self.trails.clear();
    }
}
