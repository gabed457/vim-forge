use serde::{Deserialize, Serialize};

use crate::economy::ledger::Economy;

/// Contracts needed at current tier to trigger scaling increase.
const CONTRACTS_PER_TIER: u32 = 5;

/// Automatic scaling increase every this many cycles.
const AUTO_SCALE_CYCLES: u32 = 600;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ScalingState {
    pub level: u32,
    pub contracts_completed_at_tier: u32,
    pub cycles_since_last_increase: u32,
}

impl ScalingState {
    pub fn new() -> Self {
        Self {
            level: 0,
            contracts_completed_at_tier: 0,
            cycles_since_last_increase: 0,
        }
    }

    /// Net worth threshold for the current level.
    fn net_worth_threshold(&self) -> i64 {
        (10_000.0 * 1.5_f64.powi(self.level as i32)) as i64
    }

    /// Check if scaling should increase. Level is a one-way ratchet.
    /// `contracts_completed` is total completed contracts at the current tier.
    pub fn check_scaling_increase(
        &mut self,
        economy: &Economy,
        contracts_completed: u32,
    ) -> bool {
        self.cycles_since_last_increase += 1;

        let should_increase = contracts_completed >= self.contracts_completed_at_tier + CONTRACTS_PER_TIER
            || economy.net_worth() >= self.net_worth_threshold()
            || self.cycles_since_last_increase >= AUTO_SCALE_CYCLES;

        if should_increase {
            self.level += 1;
            self.contracts_completed_at_tier = contracts_completed;
            self.cycles_since_last_increase = 0;
            true
        } else {
            false
        }
    }

    /// Quantity multiplier based on scaling level. +15% per level.
    pub fn quantity_multiplier(&self) -> f64 {
        1.0 + self.level as f64 * 0.15
    }
}
