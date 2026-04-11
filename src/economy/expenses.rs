use serde::{Deserialize, Serialize};

use crate::economy::ledger::{Difficulty, Economy};
use crate::scaling::regulations::Regulations;

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct ExpenseReport {
    pub power_cost: f64,
    pub waste_disposal: f64,
    pub pollution_fine: f64,
    pub loan_interest: f64,
    pub land_lease: f64,
    pub maintenance: f64,
    pub wages: f64,
    pub carbon_tax: f64,
    pub transport: f64,
    pub total: f64,
}

impl ExpenseReport {
    pub fn format_line(&self, label: &str, amount: f64) -> String {
        if amount > 0.0 {
            format!("  {:<20} ${:.0}", label, amount)
        } else {
            String::new()
        }
    }

    pub fn summary_lines(&self) -> Vec<String> {
        let mut lines = Vec::new();
        let items = [
            ("Power", self.power_cost),
            ("Waste Disposal", self.waste_disposal),
            ("Pollution Fines", self.pollution_fine),
            ("Loan Interest", self.loan_interest),
            ("Land Lease", self.land_lease),
            ("Maintenance", self.maintenance),
            ("Wages", self.wages),
            ("Carbon Tax", self.carbon_tax),
            ("Transport", self.transport),
        ];
        for (label, amount) in &items {
            if *amount > 0.0 {
                lines.push(format!("  {:<20} ${:.0}", label, amount));
            }
        }
        lines.push(format!("  {:<20} ${:.0}", "TOTAL", self.total));
        lines
    }
}

/// Per-waste-type disposal cost.
pub fn waste_disposal_cost(tier: u8) -> f64 {
    match tier {
        0 => 0.50,
        1 => 2.0,
        2 => 8.0,
        3 => 25.0,
        _ => 50.0,
    }
}

/// Transport cost per active vehicle per cycle.
pub fn transport_vehicle_cost(vehicle_kind: &str) -> f64 {
    match vehicle_kind {
        "train" => 5.0,
        "truck" => 2.0,
        "drone" => 3.0,
        "plane" => 20.0,
        _ => 1.0,
    }
}

/// Snapshot of factory state needed to compute expenses.
pub struct FactorySnapshot {
    pub mw_consumed: f64,
    pub waste_disposed: Vec<(u8, u64)>, // (waste tier, quantity)
    pub pollution_level: f64,
    pub pollution_threshold: f64,
    pub fine_rate: f64,
    pub built_tiles: u64,
    pub operating_machines: u64,
    pub co2_vented: u64,
    pub active_trains: u32,
    pub active_trucks: u32,
    pub active_drones: u32,
    pub active_planes: u32,
}

/// Compute all expenses for one economic cycle.
pub fn compute_cycle_expenses(
    snapshot: &FactorySnapshot,
    economy: &Economy,
    regulations: &Regulations,
    loan_interest_due: f64,
) -> ExpenseReport {
    if economy.difficulty == Difficulty::Tutorial {
        return ExpenseReport::default();
    }

    let power_cost = snapshot.mw_consumed * 0.01;

    let waste_disposal: f64 = snapshot
        .waste_disposed
        .iter()
        .map(|(tier, qty)| waste_disposal_cost(*tier) * (*qty as f64) * regulations.waste_disposal_multiplier)
        .sum();

    let pollution_excess = snapshot.pollution_level
        - (snapshot.pollution_threshold + regulations.pollution_threshold_offset);
    let pollution_fine = if pollution_excess > 0.0 {
        pollution_excess * snapshot.fine_rate * regulations.pollution_fine_multiplier
    } else {
        0.0
    };

    let land_lease = snapshot.built_tiles as f64 * 0.10;
    let maintenance = snapshot.operating_machines as f64 * 0.50;
    let wages = snapshot.operating_machines as f64 * 1.0;
    let carbon_tax =
        snapshot.co2_vented as f64 * 0.50 * regulations.carbon_tax_multiplier;

    let transport = snapshot.active_trains as f64 * transport_vehicle_cost("train")
        + snapshot.active_trucks as f64 * transport_vehicle_cost("truck")
        + snapshot.active_drones as f64 * transport_vehicle_cost("drone")
        + snapshot.active_planes as f64 * transport_vehicle_cost("plane");

    let total = power_cost
        + waste_disposal
        + pollution_fine
        + loan_interest_due
        + land_lease
        + maintenance
        + wages
        + carbon_tax
        + transport;

    ExpenseReport {
        power_cost,
        waste_disposal,
        pollution_fine,
        loan_interest: loan_interest_due,
        land_lease,
        maintenance,
        wages,
        carbon_tax,
        transport,
        total,
    }
}

/// Building placement cost by entity tier.
pub fn building_cost(entity_tier: u8) -> i64 {
    match entity_tier {
        0 => 0,   // extractors are pre-placed
        1 => 50,
        2 => 200,
        3 => 500,
        4 => 2000,
        5 => 10000,
        _ => 100,
    }
}

/// Demolition refund: 50% of placement cost.
pub fn demolition_refund(entity_tier: u8) -> i64 {
    building_cost(entity_tier) / 2
}
