use serde::{Deserialize, Serialize};

// ---------------------------------------------------------------------------
// Pollution state
// ---------------------------------------------------------------------------

/// Global pollution level for the factory.
/// Range: 0.0 to 1000.0 — at 1000 it's game over.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PollutionState {
    pub level: f64,
}

impl Default for PollutionState {
    fn default() -> Self {
        Self { level: 0.0 }
    }
}

/// Threshold effects.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PollutionEffect {
    /// No penalty.
    None,
    /// 200+ : regulatory fines.
    Fines,
    /// 500+ : efficiency -10%.
    EfficiencyPenalty,
    /// 700+ : random machine breakdowns.
    Breakdowns,
    /// 1000 : game over.
    GameOver,
}

impl PollutionState {
    pub fn new() -> Self {
        Self::default()
    }

    /// Returns the most severe effect that currently applies.
    pub fn current_effect(&self) -> PollutionEffect {
        if self.level >= 1000.0 {
            PollutionEffect::GameOver
        } else if self.level >= 700.0 {
            PollutionEffect::Breakdowns
        } else if self.level >= 500.0 {
            PollutionEffect::EfficiencyPenalty
        } else if self.level >= 200.0 {
            PollutionEffect::Fines
        } else {
            PollutionEffect::None
        }
    }

    /// Efficiency multiplier from pollution (1.0 = normal, 0.9 at 500+).
    pub fn efficiency_multiplier(&self) -> f64 {
        if self.level >= 500.0 {
            0.9
        } else {
            1.0
        }
    }

    /// Add pollution from a source.
    pub fn add(&mut self, amount: f64) {
        self.level = (self.level + amount).min(1000.0);
    }

    /// Remove pollution (from sinks like forest tiles, scrubbers, natural decay).
    pub fn remove(&mut self, amount: f64) {
        self.level = (self.level - amount).max(0.0);
    }
}

// ---------------------------------------------------------------------------
// Pollution sources — rates per tick
// ---------------------------------------------------------------------------

/// Pollution per tick from venting gas (per 100 L vented).
pub const VENT_POLLUTION_PER_100L: f64 = 1.0;

/// Pollution per tick from tailings (per 1000 mL).
pub const TAILINGS_POLLUTION_PER_1000ML: f64 = 2.0;

/// Pollution per item incinerated.
pub const INCINERATOR_POLLUTION_PER_ITEM: f64 = 3.0;

/// Pollution per tick from an unscrubbed generator.
pub const UNSCRUBBED_GENERATOR_POLLUTION: f64 = 0.5;

/// Pollution per tick from a nuclear leak.
pub const NUCLEAR_LEAK_POLLUTION: f64 = 50.0;

// ---------------------------------------------------------------------------
// Pollution sinks — rates per tick
// ---------------------------------------------------------------------------

/// Pollution absorbed per forest tile per tick.
pub const FOREST_ABSORPTION: f64 = 0.1;

/// Scrubber efficiency (fraction of source pollution removed).
pub const SCRUBBER_EFFICIENCY: f64 = 0.9;

/// Natural pollution decay per tick.
pub const NATURAL_DECAY: f64 = 0.5;

// ---------------------------------------------------------------------------
// Update function
// ---------------------------------------------------------------------------

/// Main per-tick update for pollution.
///
/// `source_total` — total pollution added from all sources this tick.
/// `forest_tile_count` — number of forest tiles on the map.
/// `scrubber_count` — number of active scrubber units.
///
/// Returns the new pollution effect after the update.
pub fn update_pollution(
    state: &mut PollutionState,
    source_total: f64,
    forest_tile_count: u32,
    scrubber_count: u32,
) -> PollutionEffect {
    // Add source pollution (scrubbers reduce it at the source).
    let scrubbed = if scrubber_count > 0 {
        source_total * (1.0 - SCRUBBER_EFFICIENCY * scrubber_count as f64).max(0.0)
    } else {
        source_total
    };
    state.add(scrubbed);

    // Subtract sinks.
    let forest_sink = forest_tile_count as f64 * FOREST_ABSORPTION;
    state.remove(forest_sink);
    state.remove(NATURAL_DECAY);

    state.current_effect()
}
