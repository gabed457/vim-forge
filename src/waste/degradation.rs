use serde::{Deserialize, Serialize};

// ---------------------------------------------------------------------------
// Machine health
// ---------------------------------------------------------------------------

/// Tracks the health / degradation of a machine.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct MachineHealth {
    /// Current health percentage (0.0 = broken, 100.0 = perfect).
    pub health: f64,
    /// Whether the machine is currently broken (needs repair).
    pub broken: bool,
    /// Accumulated operating ticks since last maintenance.
    pub ticks_since_maintenance: u64,
}

impl Default for MachineHealth {
    fn default() -> Self {
        Self {
            health: 100.0,
            broken: false,
            ticks_since_maintenance: 0,
        }
    }
}

impl MachineHealth {
    pub fn new() -> Self {
        Self::default()
    }

    /// Apply wear — called each tick the machine operates.
    /// Base degradation rate is very slow; pollution and scaling accelerate it.
    pub fn apply_wear(&mut self, pollution_level: f64, scaling_level: u32) {
        self.ticks_since_maintenance += 1;

        // Base wear: 0.001% per tick.
        let mut wear = 0.001;

        // Pollution 700+ : extra wear.
        if pollution_level >= 700.0 {
            wear += 0.01;
        }

        // High scaling: extra wear.
        if scaling_level >= 90 {
            wear += 0.005 * (scaling_level - 89) as f64;
        }

        self.health = (self.health - wear).max(0.0);
        if self.health <= 0.0 {
            self.broken = true;
        }
    }

    /// Repair the machine (e.g. via RepairKit or MaintenanceDrone).
    pub fn repair(&mut self) {
        self.health = 100.0;
        self.broken = false;
        self.ticks_since_maintenance = 0;
    }

    /// Partial repair — restores some health.
    pub fn partial_repair(&mut self, amount: f64) {
        self.health = (self.health + amount).min(100.0);
        if self.health > 0.0 {
            self.broken = false;
        }
    }
}

// ---------------------------------------------------------------------------
// Breakdown check
// ---------------------------------------------------------------------------

/// Check whether a random breakdown should occur this tick.
///
/// `pollution_level` — current pollution (0-1000).
/// `scaling_level` — current difficulty scaling.
/// `rand_value` — a random f64 in [0, 1) provided by the caller.
///
/// Returns true if the machine should break down this tick.
pub fn should_breakdown(pollution_level: f64, scaling_level: u32, rand_value: f64) -> bool {
    // Only possible at pollution 700+ or scaling 90+.
    if pollution_level < 700.0 && scaling_level < 90 {
        return false;
    }

    // Base chance per tick: 0.1%.
    let mut chance = 0.001;

    // Pollution scaling.
    if pollution_level >= 700.0 {
        chance += (pollution_level - 700.0) / 300_000.0; // up to ~0.001 extra at 1000
    }

    // Difficulty scaling.
    if scaling_level >= 90 {
        chance += (scaling_level - 89) as f64 * 0.0005;
    }

    rand_value < chance
}

// ---------------------------------------------------------------------------
// Maintenance cost
// ---------------------------------------------------------------------------

/// Returns the maintenance cost (credits) for repairing a machine at tier.
pub fn repair_cost(machine_tier: u8) -> f64 {
    match machine_tier {
        0 => 10.0,
        1 => 25.0,
        2 => 50.0,
        3 => 100.0,
        4 => 250.0,
        5 => 500.0,
        _ => 50.0,
    }
}

/// Fine for leaving a machine broken and unrepaired (per tick).
pub fn breakdown_fine(machine_tier: u8) -> f64 {
    repair_cost(machine_tier) * 0.01
}
