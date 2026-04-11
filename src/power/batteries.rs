use serde::{Deserialize, Serialize};

use crate::resources::EntityType;

// ---------------------------------------------------------------------------
// Battery spec
// ---------------------------------------------------------------------------

/// Static specification for battery entity types.
#[derive(Clone, Debug)]
pub struct BatterySpec {
    pub entity_type: EntityType,
    /// Maximum stored energy (MWh — but we track in MW-ticks internally).
    pub capacity_mwh: f64,
    /// Max charge rate (MW).
    pub charge_rate: f64,
    /// Max discharge rate (MW).
    pub discharge_rate: f64,
}

pub fn battery_spec(entity_type: EntityType) -> Option<BatterySpec> {
    match entity_type {
        EntityType::BatteryBank => Some(BatterySpec {
            entity_type,
            capacity_mwh: 100.0,
            charge_rate: 10.0,
            discharge_rate: 10.0,
        }),
        EntityType::Accumulator => Some(BatterySpec {
            entity_type,
            capacity_mwh: 500.0,
            charge_rate: 25.0,
            discharge_rate: 25.0,
        }),
        // FusionReactor-tier batteries could be added here later.
        _ => None,
    }
}

// ---------------------------------------------------------------------------
// Battery runtime state
// ---------------------------------------------------------------------------

/// Runtime state of a single battery instance.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct BatteryState {
    pub entity_type: EntityType,
    pub pos: (usize, usize),
    /// Maximum stored energy (MW-ticks).
    pub capacity: f64,
    /// Current stored energy (MW-ticks).
    pub current_charge: f64,
    /// Max MW that can be absorbed per tick.
    pub charge_rate: f64,
    /// Max MW that can be supplied per tick.
    pub discharge_rate: f64,
}

impl BatteryState {
    pub fn new(entity_type: EntityType, pos: (usize, usize)) -> Self {
        let spec = battery_spec(entity_type).unwrap_or(BatterySpec {
            entity_type,
            capacity_mwh: 100.0,
            charge_rate: 10.0,
            discharge_rate: 10.0,
        });
        Self {
            entity_type,
            pos,
            capacity: spec.capacity_mwh,
            current_charge: 0.0,
            charge_rate: spec.charge_rate,
            discharge_rate: spec.discharge_rate,
        }
    }

    pub fn charge_fraction(&self) -> f64 {
        if self.capacity <= 0.0 {
            return 0.0;
        }
        self.current_charge / self.capacity
    }

    /// Charge the battery with surplus power.
    /// Returns the MW actually absorbed.
    pub fn charge(&mut self, surplus_mw: f64) -> f64 {
        let space = self.capacity - self.current_charge;
        let absorbed = surplus_mw.min(self.charge_rate).min(space).max(0.0);
        self.current_charge += absorbed;
        absorbed
    }

    /// Discharge to cover a power deficit.
    /// Returns the MW actually supplied.
    pub fn discharge(&mut self, deficit_mw: f64) -> f64 {
        let supplied = deficit_mw
            .min(self.discharge_rate)
            .min(self.current_charge)
            .max(0.0);
        self.current_charge -= supplied;
        supplied
    }
}

/// Process all batteries for one tick given the grid's surplus or deficit.
///
/// If `surplus_mw > 0`, batteries charge.
/// If `surplus_mw < 0`, batteries discharge.
///
/// Returns (total_absorbed, total_supplied).
pub fn update_batteries(batteries: &mut [BatteryState], surplus_mw: f64) -> (f64, f64) {
    let mut total_absorbed = 0.0;
    let mut total_supplied = 0.0;

    if surplus_mw > 0.0 {
        let mut remaining = surplus_mw;
        for bat in batteries.iter_mut() {
            if remaining <= 0.0 {
                break;
            }
            let absorbed = bat.charge(remaining);
            remaining -= absorbed;
            total_absorbed += absorbed;
        }
    } else if surplus_mw < 0.0 {
        let mut deficit = -surplus_mw;
        for bat in batteries.iter_mut() {
            if deficit <= 0.0 {
                break;
            }
            let supplied = bat.discharge(deficit);
            deficit -= supplied;
            total_supplied += supplied;
        }
    }

    (total_absorbed, total_supplied)
}
