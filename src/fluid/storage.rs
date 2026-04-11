use serde::{Deserialize, Serialize};

use crate::resources::Resource;

// ---------------------------------------------------------------------------
// Tank state
// ---------------------------------------------------------------------------

/// State of a fluid storage tank entity.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TankState {
    pub fluid_type: Option<Resource>,
    pub level: u32,
    pub capacity: u32,
    /// Maximum units that can enter per tick.
    pub fill_rate: u32,
    /// Maximum units that can leave per tick.
    pub drain_rate: u32,
}

impl TankState {
    pub fn new(capacity: u32, fill_rate: u32, drain_rate: u32) -> Self {
        Self {
            fluid_type: None,
            level: 0,
            capacity,
            fill_rate,
            drain_rate,
        }
    }

    pub fn fill_fraction(&self) -> f32 {
        if self.capacity == 0 {
            return 0.0;
        }
        self.level as f32 / self.capacity as f32
    }

    /// Display character based on fill level: empty, 1/4, 1/2, 3/4, full.
    pub fn fill_glyph(&self) -> char {
        let frac = self.fill_fraction();
        if frac <= 0.0 {
            ' '
        } else if frac < 0.25 {
            '\u{2581}' // ▁
        } else if frac < 0.50 {
            '\u{2583}' // ▃
        } else if frac < 0.75 {
            '\u{2585}' // ▅
        } else {
            '\u{2587}' // ▇
        }
    }

    /// Try to fill this tank with the given resource.
    /// Returns how much was actually accepted.
    pub fn try_fill(&mut self, resource: Resource, amount: u32) -> u32 {
        if let Some(existing) = self.fluid_type {
            if existing != resource {
                return 0; // contamination: refuse mixing
            }
        }
        let space = self.capacity.saturating_sub(self.level);
        let accepted = amount.min(space).min(self.fill_rate);
        if accepted > 0 {
            self.level += accepted;
            self.fluid_type = Some(resource);
        }
        accepted
    }

    /// Try to drain from this tank.
    /// Returns (resource, amount actually drained).
    pub fn try_drain(&mut self, amount: u32) -> Option<(Resource, u32)> {
        let res = self.fluid_type?;
        let drained = amount.min(self.level).min(self.drain_rate);
        if drained == 0 {
            return None;
        }
        self.level -= drained;
        if self.level == 0 {
            self.fluid_type = None;
        }
        Some((res, drained))
    }
}
