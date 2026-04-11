use serde::{Deserialize, Serialize};

/// Cycles between regulation tightenings.
const REGULATION_INTERVAL: u64 = 500;

/// Cycles between safety inspections.
const INSPECTION_INTERVAL: u64 = 200;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Regulations {
    pub carbon_tax_multiplier: f64,
    pub pollution_fine_multiplier: f64,
    pub pollution_threshold_offset: f64,
    pub waste_disposal_multiplier: f64,
    pub last_tightening_cycle: u64,
    pub last_inspection_cycle: u64,
}

impl Regulations {
    pub fn new() -> Self {
        Self {
            carbon_tax_multiplier: 1.0,
            pollution_fine_multiplier: 1.0,
            pollution_threshold_offset: 0.0,
            waste_disposal_multiplier: 1.0,
            last_tightening_cycle: 0,
            last_inspection_cycle: 0,
        }
    }

    /// Update regulations at the given economic cycle. Called once per cycle.
    pub fn update(&mut self, cycle: u64) {
        if cycle >= self.last_tightening_cycle + REGULATION_INTERVAL {
            self.last_tightening_cycle = cycle;
            self.carbon_tax_multiplier *= 1.10;       // +10%
            self.pollution_fine_multiplier *= 1.15;    // +15%
            self.pollution_threshold_offset -= 5.0;    // -5 units
            self.waste_disposal_multiplier *= 1.05;    // +5%
        }
    }

    /// Check if safety inspection is due.
    pub fn inspection_due(&self, cycle: u64) -> bool {
        cycle >= self.last_inspection_cycle + INSPECTION_INTERVAL
    }

    pub fn mark_inspected(&mut self, cycle: u64) {
        self.last_inspection_cycle = cycle;
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Violation {
    pub description: String,
    pub fine: i64,
}

/// Snapshot of factory conditions for safety inspection.
pub struct InspectionSnapshot {
    pub pollution_over_threshold: bool,
    pub nuclear_waste_stored: u64,
    pub machines_without_power: u32,
    pub waste_dumps_full: u32,
    pub containment_breaches: u32,
}

/// Run a safety inspection. Returns list of violations with fines.
pub fn run_safety_inspection(
    snapshot: &InspectionSnapshot,
    _cycle: u64,
) -> Vec<Violation> {
    let mut violations = Vec::new();

    if snapshot.pollution_over_threshold {
        violations.push(Violation {
            description: "Pollution exceeds regulatory threshold".into(),
            fine: 500,
        });
    }

    if snapshot.nuclear_waste_stored > 100 {
        violations.push(Violation {
            description: format!(
                "Excessive nuclear waste storage ({} units)",
                snapshot.nuclear_waste_stored
            ),
            fine: 1000,
        });
    }

    if snapshot.machines_without_power > 0 {
        violations.push(Violation {
            description: format!(
                "{} machines operating without adequate power",
                snapshot.machines_without_power
            ),
            fine: 200,
        });
    }

    if snapshot.waste_dumps_full > 0 {
        violations.push(Violation {
            description: format!(
                "{} waste dumps at capacity",
                snapshot.waste_dumps_full
            ),
            fine: 300,
        });
    }

    if snapshot.containment_breaches > 0 {
        violations.push(Violation {
            description: format!(
                "{} containment breaches detected",
                snapshot.containment_breaches
            ),
            fine: 2000,
        });
    }

    violations
}
