use serde::{Deserialize, Serialize};

use crate::resources::Resource;

/// Ticks before an available contract expires if not accepted.
pub const AVAILABLE_EXPIRY_TICKS: u64 = 600;

/// Ticks between new contract generation.
pub const REFRESH_INTERVAL_TICKS: u64 = 300;

/// Maximum contracts shown in the available board.
pub const MAX_AVAILABLE: usize = 5;

/// Maximum simultaneously active contracts.
pub const MAX_ACTIVE: usize = 3;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ContractTier {
    Starter,
    Basic,
    Standard,
    Advanced,
    Elite,
    Legendary,
}

impl ContractTier {
    pub fn name(&self) -> &'static str {
        match self {
            Self::Starter => "Starter",
            Self::Basic => "Basic",
            Self::Standard => "Standard",
            Self::Advanced => "Advanced",
            Self::Elite => "Elite",
            Self::Legendary => "Legendary",
        }
    }

    pub fn penalty_fraction(&self) -> f64 {
        match self {
            Self::Starter => 0.30,
            Self::Basic => 0.40,
            Self::Standard => 0.50,
            Self::Advanced => 0.60,
            Self::Elite => 0.70,
            Self::Legendary => 0.80,
        }
    }

    pub fn base_quantity_range(&self) -> (u64, u64) {
        match self {
            Self::Starter => (5, 20),
            Self::Basic => (15, 50),
            Self::Standard => (30, 100),
            Self::Advanced => (50, 200),
            Self::Elite => (100, 500),
            Self::Legendary => (200, 1000),
        }
    }

    pub fn base_reward_range(&self) -> (i64, i64) {
        match self {
            Self::Starter => (100, 500),
            Self::Basic => (400, 1500),
            Self::Standard => (1000, 5000),
            Self::Advanced => (3000, 15000),
            Self::Elite => (10000, 50000),
            Self::Legendary => (30000, 200000),
        }
    }

    pub fn base_deadline_ticks(&self) -> u64 {
        match self {
            Self::Starter => 600,
            Self::Basic => 900,
            Self::Standard => 1200,
            Self::Advanced => 1800,
            Self::Elite => 2400,
            Self::Legendary => 3600,
        }
    }

    pub fn reputation_reward(&self) -> i64 {
        match self {
            Self::Starter => 5,
            Self::Basic => 10,
            Self::Standard => 20,
            Self::Advanced => 40,
            Self::Elite => 80,
            Self::Legendary => 200,
        }
    }

    pub fn all() -> &'static [ContractTier] {
        &[
            Self::Starter,
            Self::Basic,
            Self::Standard,
            Self::Advanced,
            Self::Elite,
            Self::Legendary,
        ]
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ContractStatus {
    Available,
    Active,
    Completed,
    Failed,
    Expired,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ContractRequirement {
    pub resource: Resource,
    pub quantity: u64,
    pub delivered: u64,
}

impl ContractRequirement {
    pub fn is_fulfilled(&self) -> bool {
        self.delivered >= self.quantity
    }

    pub fn remaining(&self) -> u64 {
        self.quantity.saturating_sub(self.delivered)
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Contract {
    pub id: u64,
    pub name: String,
    pub tier: ContractTier,
    pub requirements: Vec<ContractRequirement>,
    pub reward: i64,
    pub bonus_reward: i64,
    pub penalty: i64,
    pub deadline: u64,
    pub issued_at: u64,
    pub status: ContractStatus,
    pub reputation_reward: i64,
    pub reputation_penalty: i64,
}

impl Contract {
    pub fn is_complete(&self) -> bool {
        self.requirements.iter().all(|r| r.is_fulfilled())
    }

    pub fn is_expired(&self, current_tick: u64) -> bool {
        current_tick >= self.deadline
    }

    /// Deliver resources to this contract. Returns quantity consumed.
    pub fn deliver(&mut self, resource: Resource, quantity: u64) -> u64 {
        let mut consumed = 0;
        for req in &mut self.requirements {
            if req.resource == resource && !req.is_fulfilled() {
                let can_accept = req.remaining().min(quantity - consumed);
                req.delivered += can_accept;
                consumed += can_accept;
                if consumed >= quantity {
                    break;
                }
            }
        }
        consumed
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ContractBoard {
    pub available: Vec<Contract>,
    pub active: Vec<Contract>,
    pub completed_count: u64,
    pub failed_count: u64,
    pub reputation: i64,
    next_id: u64,
    last_refresh_tick: u64,
}

impl ContractBoard {
    pub fn new() -> Self {
        Self {
            available: Vec::new(),
            active: Vec::new(),
            completed_count: 0,
            failed_count: 0,
            reputation: 0,
            next_id: 1,
            last_refresh_tick: 0,
        }
    }

    pub fn next_id(&mut self) -> u64 {
        let id = self.next_id;
        self.next_id += 1;
        id
    }

    /// Accept a contract from available, moving it to active.
    pub fn accept(&mut self, contract_id: u64, current_tick: u64) -> bool {
        if self.active.len() >= MAX_ACTIVE {
            return false;
        }
        if let Some(idx) = self.available.iter().position(|c| c.id == contract_id) {
            let mut contract = self.available.remove(idx);
            contract.status = ContractStatus::Active;
            // Set deadline relative to acceptance
            let time_budget = contract.deadline - contract.issued_at;
            contract.deadline = current_tick + time_budget;
            self.active.push(contract);
            true
        } else {
            false
        }
    }

    /// Check active contract deadlines, failing any that are overdue.
    pub fn check_deadlines(&mut self, current_tick: u64) {
        for contract in &mut self.active {
            if contract.status == ContractStatus::Active && contract.is_expired(current_tick) {
                contract.status = ContractStatus::Failed;
                self.reputation -= contract.reputation_penalty;
                self.failed_count += 1;
            }
        }
        // Remove failed contracts from active list
        self.active.retain(|c| c.status == ContractStatus::Active);
    }

    /// Complete any contracts that have all requirements fulfilled.
    /// Returns total reward earned.
    pub fn check_completions(&mut self, current_tick: u64) -> i64 {
        let mut total_reward = 0;
        for contract in &mut self.active {
            if contract.status == ContractStatus::Active && contract.is_complete() {
                contract.status = ContractStatus::Completed;
                self.reputation += contract.reputation_reward;
                self.completed_count += 1;

                total_reward += contract.reward;
                // Bonus if completed before half the deadline
                let halfway = contract.issued_at
                    + (contract.deadline - contract.issued_at) / 2;
                if current_tick < halfway {
                    total_reward += contract.bonus_reward;
                }
            }
        }
        self.active.retain(|c| c.status == ContractStatus::Active);
        total_reward
    }

    /// Remove expired available contracts, return true if refresh is due.
    pub fn refresh_available(&mut self, current_tick: u64) -> bool {
        // Expire old available contracts
        for contract in &mut self.available {
            if current_tick >= contract.issued_at + AVAILABLE_EXPIRY_TICKS {
                contract.status = ContractStatus::Expired;
            }
        }
        self.available
            .retain(|c| c.status == ContractStatus::Available);

        // Check if it's time to generate new ones
        if current_tick >= self.last_refresh_tick + REFRESH_INTERVAL_TICKS
            && self.available.len() < MAX_AVAILABLE
        {
            self.last_refresh_tick = current_tick;
            true
        } else {
            false
        }
    }

    /// Add a newly generated contract to the available board.
    pub fn add_available(&mut self, contract: Contract) {
        if self.available.len() < MAX_AVAILABLE {
            self.available.push(contract);
        }
    }
}
