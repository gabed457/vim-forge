use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use crate::resources::Resource;

/// Trade hub sells at this fraction of market price.
pub const TRADE_HUB_MULTIPLIER: f64 = 0.80;

/// Supply pressure decay per tick.
const SUPPLY_DECAY_RATE: f64 = 0.001;

/// How much each sale increases supply pressure.
const SUPPLY_PRESSURE_PER_UNIT: f64 = 0.001;

/// Maximum supply pressure (caps price reduction).
const MAX_SUPPLY_PRESSURE: f64 = 0.8;

/// Demand drift interval in ticks.
const DEMAND_DRIFT_INTERVAL: u64 = 300;

/// Maximum absolute demand modifier.
const MAX_DEMAND_MOD: f64 = 0.5;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct MarketState {
    pub supply_pressure: HashMap<Resource, f64>,
    pub demand_modifier: HashMap<Resource, f64>,
    pub last_drift_tick: u64,
    drift_seed: u64,
}

impl MarketState {
    pub fn new() -> Self {
        Self {
            supply_pressure: HashMap::new(),
            demand_modifier: HashMap::new(),
            last_drift_tick: 0,
            drift_seed: 42,
        }
    }

    /// Current market price for a resource.
    pub fn current_price(&self, resource: Resource) -> f64 {
        let base = resource.base_value();
        if base <= 0.0 {
            return 0.0;
        }
        let sp = self.supply_pressure.get(&resource).copied().unwrap_or(0.0);
        let dm = self.demand_modifier.get(&resource).copied().unwrap_or(0.0);

        let price = base * (1.0 - sp * 0.3) * (1.0 + dm);
        // Floor: 20% of base value
        price.max(base * 0.2)
    }

    /// Price at which the trade hub actually sells (80% of market).
    pub fn sell_price(&self, resource: Resource) -> f64 {
        self.current_price(resource) * TRADE_HUB_MULTIPLIER
    }

    /// Record a sale, increasing supply pressure.
    pub fn record_sale(&mut self, resource: Resource, quantity: u64) {
        let pressure = self
            .supply_pressure
            .entry(resource)
            .or_insert(0.0);
        *pressure = (*pressure + quantity as f64 * SUPPLY_PRESSURE_PER_UNIT)
            .min(MAX_SUPPLY_PRESSURE);
    }

    /// Called every tick. Decays supply pressure and drifts demand.
    pub fn update(&mut self, tick: u64) {
        // Decay supply pressure
        for pressure in self.supply_pressure.values_mut() {
            *pressure = (*pressure - SUPPLY_DECAY_RATE).max(0.0);
        }

        // Demand drift every DEMAND_DRIFT_INTERVAL ticks
        if tick >= self.last_drift_tick + DEMAND_DRIFT_INTERVAL {
            self.last_drift_tick = tick;
            self.drift_seed = self.drift_seed.wrapping_mul(6364136223846793005).wrapping_add(1);
            self.drift_demand();
        }
    }

    fn drift_demand(&mut self) {
        // Apply a small random walk to each resource's demand modifier
        // Using a simple deterministic sequence
        let mut seed = self.drift_seed;
        for (_, dm) in self.demand_modifier.iter_mut() {
            seed = seed.wrapping_mul(2862933555777941757).wrapping_add(3037000493);
            // Map to [-0.1, +0.1]
            let drift = ((seed % 201) as f64 / 1000.0) - 0.1;
            *dm = (*dm + drift).clamp(-MAX_DEMAND_MOD, MAX_DEMAND_MOD);
        }
    }

    /// Initialize demand modifiers for all known producible resources.
    pub fn init_resource(&mut self, resource: Resource) {
        self.demand_modifier.entry(resource).or_insert(0.0);
        self.supply_pressure.entry(resource).or_insert(0.0);
    }
}
