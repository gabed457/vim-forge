use serde::{Deserialize, Serialize};

/// Economic cycle length in ticks.
pub const CYCLE_TICKS: u64 = 60;

/// Consecutive negative-cash cycles before bankruptcy.
pub const BANKRUPTCY_THRESHOLD: u32 = 20;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Difficulty {
    Tutorial,
    Easy,
    Normal,
    Hard,
    Brutal,
}

impl Difficulty {
    pub fn name(&self) -> &'static str {
        match self {
            Self::Tutorial => "Tutorial",
            Self::Easy => "Easy",
            Self::Normal => "Normal",
            Self::Hard => "Hard",
            Self::Brutal => "Brutal",
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Economy {
    pub cash: i64,
    pub total_earned: u64,
    pub total_spent: u64,
    pub asset_value: u64,
    pub bankruptcy_counter: u32,
    pub credit_rating: f64,
    pub insurance_active: bool,
    pub difficulty: Difficulty,
    pub cycle: u64,
}

impl Economy {
    pub fn new(difficulty: Difficulty) -> Self {
        let cash = match difficulty {
            Difficulty::Tutorial => i64::MAX / 2,
            Difficulty::Easy => 50_000,
            Difficulty::Normal => 25_000,
            Difficulty::Hard => 10_000,
            Difficulty::Brutal => 5_000,
        };

        Self {
            cash,
            total_earned: 0,
            total_spent: 0,
            asset_value: 0,
            bankruptcy_counter: 0,
            credit_rating: 1.0,
            insurance_active: false,
            difficulty,
            cycle: 0,
        }
    }

    pub fn can_afford(&self, amount: i64) -> bool {
        if self.difficulty == Difficulty::Tutorial {
            return true;
        }
        self.cash >= amount
    }

    pub fn deduct(&mut self, amount: i64) {
        if self.difficulty == Difficulty::Tutorial {
            return;
        }
        self.cash -= amount;
        self.total_spent = self.total_spent.saturating_add(amount as u64);
    }

    pub fn credit(&mut self, amount: i64) {
        if self.difficulty == Difficulty::Tutorial {
            return;
        }
        self.cash += amount;
        self.total_earned = self.total_earned.saturating_add(amount as u64);
    }

    pub fn net_worth(&self) -> i64 {
        self.cash + self.asset_value as i64
    }

    /// Update bankruptcy counter. Returns true if game over.
    pub fn update_bankruptcy(&mut self) -> bool {
        if self.difficulty == Difficulty::Tutorial {
            return false;
        }
        if self.cash < 0 {
            self.bankruptcy_counter += 1;
        } else {
            self.bankruptcy_counter = 0;
        }
        self.bankruptcy_counter >= BANKRUPTCY_THRESHOLD
    }

    pub fn is_bankrupt(&self) -> bool {
        self.bankruptcy_counter >= BANKRUPTCY_THRESHOLD
    }

    /// Add building cost to asset tracking. Call on placement.
    pub fn add_asset(&mut self, cost: i64) {
        self.asset_value = self.asset_value.saturating_add((cost as u64) / 2);
    }

    /// Remove building from asset tracking. Call on demolition.
    pub fn remove_asset(&mut self, cost: i64) {
        self.asset_value = self.asset_value.saturating_sub((cost as u64) / 2);
    }

    /// Advance one economic cycle. Returns true if bankrupt.
    pub fn advance_cycle(&mut self) -> bool {
        self.cycle += 1;
        self.update_bankruptcy()
    }
}
