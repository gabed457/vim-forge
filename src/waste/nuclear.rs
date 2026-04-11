use serde::{Deserialize, Serialize};

// ---------------------------------------------------------------------------
// Nuclear waste storage
// ---------------------------------------------------------------------------

/// State of a nuclear waste storage facility.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct NuclearStorageState {
    /// Waste units stored.
    pub waste_level: u32,
    /// Maximum waste capacity.
    pub capacity: u32,
    /// Coolant level (mL). Requires 100 mL/min to stay safe.
    pub coolant_level: u32,
    /// Ticks without coolant.
    pub ticks_without_coolant: u32,
    /// Whether a leak is currently active.
    pub leaking: bool,
}

/// Coolant required per minute (60 ticks at ~1 tick/second).
pub const COOLANT_PER_MINUTE: u32 = 100;

/// Ticks per coolant consumption (consuming 1 mL per tick at this rate).
pub const COOLANT_CONSUME_INTERVAL: u32 = 1;

/// Warning threshold: ticks without coolant before warning.
pub const WARNING_TICKS: u32 = 30 * 60; // 30 seconds at 60 tps, but we simplify to 30 ticks

/// Leak threshold: ticks without coolant before leak starts.
pub const LEAK_TICKS: u32 = 60;

/// Contamination radius (in tiles) when a leak occurs.
pub const LEAK_RADIUS: u32 = 10;

impl NuclearStorageState {
    pub fn new(capacity: u32) -> Self {
        Self {
            waste_level: 0,
            capacity,
            coolant_level: 0,
            ticks_without_coolant: 0,
            leaking: false,
        }
    }

    pub fn is_full(&self) -> bool {
        self.waste_level >= self.capacity
    }

    /// Add waste. Returns how much was accepted.
    pub fn add_waste(&mut self, amount: u32) -> u32 {
        let space = self.capacity.saturating_sub(self.waste_level);
        let accepted = amount.min(space);
        self.waste_level += accepted;
        accepted
    }

    /// Add coolant.
    pub fn add_coolant(&mut self, ml: u32) {
        self.coolant_level += ml;
    }
}

// ---------------------------------------------------------------------------
// Nuclear leak state
// ---------------------------------------------------------------------------

/// Tracks an active nuclear leak on the map.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct NuclearLeakState {
    /// Center of the leak (the storage facility position).
    pub center: (usize, usize),
    /// Contamination radius in tiles.
    pub radius: u32,
    /// Pollution per tick from this leak.
    pub pollution_per_tick: f64,
    /// Tick when the leak started.
    pub start_tick: u64,
    /// Whether the leak has been contained.
    pub contained: bool,
}

impl NuclearLeakState {
    pub fn new(center: (usize, usize), start_tick: u64) -> Self {
        Self {
            center,
            radius: LEAK_RADIUS,
            pollution_per_tick: super::pollution::NUCLEAR_LEAK_POLLUTION,
            start_tick,
            contained: false,
        }
    }

    /// Check if a position is within the contamination zone.
    pub fn is_contaminated(&self, pos: (usize, usize)) -> bool {
        if self.contained {
            return false;
        }
        let dx = (self.center.0 as isize - pos.0 as isize).unsigned_abs() as u32;
        let dy = (self.center.1 as isize - pos.1 as isize).unsigned_abs() as u32;
        dx * dx + dy * dy <= self.radius * self.radius
    }
}

// ---------------------------------------------------------------------------
// Update function
// ---------------------------------------------------------------------------

/// Tick a nuclear storage facility.
///
/// Returns `Some(NuclearLeakState)` if a new leak just started.
pub fn update_nuclear_storage(
    state: &mut NuclearStorageState,
    pos: (usize, usize),
    current_tick: u64,
) -> Option<NuclearLeakState> {
    // No waste → no risk.
    if state.waste_level == 0 {
        state.ticks_without_coolant = 0;
        state.leaking = false;
        return None;
    }

    // Consume coolant.
    if state.coolant_level > 0 {
        let consume = COOLANT_CONSUME_INTERVAL.min(state.coolant_level);
        state.coolant_level -= consume;
        state.ticks_without_coolant = 0;
        state.leaking = false;
        return None;
    }

    // No coolant — increment danger counter.
    state.ticks_without_coolant += 1;

    // Check for leak.
    if state.ticks_without_coolant >= LEAK_TICKS && !state.leaking {
        state.leaking = true;
        return Some(NuclearLeakState::new(pos, current_tick));
    }

    None
}

/// Returns true if the storage is in warning state (no coolant but not yet leaking).
pub fn is_warning(state: &NuclearStorageState) -> bool {
    state.ticks_without_coolant >= WARNING_TICKS && !state.leaking
}
