use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum PrestigeBonus {
    StartingCash,
    BuildingDiscount,
    ResearchSpeed,
    MachineSpeed,
    PollutionReduction,
    ExtraContractSlots,
}

impl PrestigeBonus {
    pub fn name(&self) -> &'static str {
        match self {
            Self::StartingCash => "Starting Cash Bonus",
            Self::BuildingDiscount => "Building Discount",
            Self::ResearchSpeed => "Research Speed",
            Self::MachineSpeed => "Machine Speed",
            Self::PollutionReduction => "Pollution Reduction",
            Self::ExtraContractSlots => "Extra Contract Slots",
        }
    }

    pub fn description(&self) -> &'static str {
        match self {
            Self::StartingCash => "+10% starting cash per level",
            Self::BuildingDiscount => "-5% building costs per level",
            Self::ResearchSpeed => "+10% research speed per level",
            Self::MachineSpeed => "+5% machine throughput per level",
            Self::PollutionReduction => "-10% pollution output per level",
            Self::ExtraContractSlots => "+1 active contract slot per level",
        }
    }

    pub fn all() -> &'static [PrestigeBonus] {
        &[
            Self::StartingCash,
            Self::BuildingDiscount,
            Self::ResearchSpeed,
            Self::MachineSpeed,
            Self::PollutionReduction,
            Self::ExtraContractSlots,
        ]
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PrestigeState {
    pub level: u32,
    pub points: u64,
    pub bonuses: Vec<(PrestigeBonus, u32)>,
}

impl PrestigeState {
    pub fn new() -> Self {
        Self {
            level: 0,
            points: 0,
            bonuses: Vec::new(),
        }
    }

    /// Get the level of a specific bonus.
    pub fn bonus_level(&self, bonus: PrestigeBonus) -> u32 {
        self.bonuses
            .iter()
            .find(|(b, _)| *b == bonus)
            .map(|(_, lvl)| *lvl)
            .unwrap_or(0)
    }

    /// Try to purchase a bonus level. Returns true on success.
    pub fn purchase(&mut self, bonus: PrestigeBonus) -> bool {
        let current = self.bonus_level(bonus);
        let cost = prestige_cost(bonus, current);
        if self.points < cost {
            return false;
        }
        self.points -= cost;
        if let Some(entry) = self.bonuses.iter_mut().find(|(b, _)| *b == bonus) {
            entry.1 += 1;
        } else {
            self.bonuses.push((bonus, 1));
        }
        true
    }

    /// Perform a prestige reset. Grants points based on achievements.
    /// Returns the points earned from this prestige.
    pub fn prestige(&mut self, net_worth: i64, scaling_level: u32) -> u64 {
        let earned = (net_worth.max(0) as u64 / 1000) + scaling_level as u64 * 10;
        self.points += earned;
        self.level += 1;
        earned
    }

    /// Cost multiplier applied to the game after prestige.
    /// Each prestige adds +20% to all costs.
    pub fn cost_multiplier(&self) -> f64 {
        1.0 + self.level as f64 * 0.20
    }

    /// Starting scaling level after prestige.
    pub fn starting_scaling(&self) -> u32 {
        self.level * 2
    }
}

/// Cost in prestige points to buy the next level of a bonus.
pub fn prestige_cost(bonus: PrestigeBonus, current_level: u32) -> u64 {
    let base = match bonus {
        PrestigeBonus::StartingCash => 10,
        PrestigeBonus::BuildingDiscount => 15,
        PrestigeBonus::ResearchSpeed => 20,
        PrestigeBonus::MachineSpeed => 25,
        PrestigeBonus::PollutionReduction => 15,
        PrestigeBonus::ExtraContractSlots => 50,
    };
    base * (current_level as u64 + 1)
}

/// Computed modifiers from all prestige bonuses.
#[derive(Clone, Debug)]
pub struct PrestigeModifiers {
    pub starting_cash_bonus: f64,
    pub building_discount: f64,
    pub research_speed_bonus: f64,
    pub machine_speed_bonus: f64,
    pub pollution_reduction: f64,
    pub extra_contract_slots: u32,
}

impl Default for PrestigeModifiers {
    fn default() -> Self {
        Self {
            starting_cash_bonus: 0.0,
            building_discount: 0.0,
            research_speed_bonus: 0.0,
            machine_speed_bonus: 0.0,
            pollution_reduction: 0.0,
            extra_contract_slots: 0,
        }
    }
}

/// Compute prestige modifiers from current state.
pub fn apply_prestige_bonuses(state: &PrestigeState) -> PrestigeModifiers {
    PrestigeModifiers {
        starting_cash_bonus: state.bonus_level(PrestigeBonus::StartingCash) as f64 * 0.10,
        building_discount: state.bonus_level(PrestigeBonus::BuildingDiscount) as f64 * 0.05,
        research_speed_bonus: state.bonus_level(PrestigeBonus::ResearchSpeed) as f64 * 0.10,
        machine_speed_bonus: state.bonus_level(PrestigeBonus::MachineSpeed) as f64 * 0.05,
        pollution_reduction: state.bonus_level(PrestigeBonus::PollutionReduction) as f64 * 0.10,
        extra_contract_slots: state.bonus_level(PrestigeBonus::ExtraContractSlots),
    }
}
