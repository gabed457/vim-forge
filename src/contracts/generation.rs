use crate::contracts::board::{Contract, ContractRequirement, ContractStatus, ContractTier};
use crate::resources::Resource;

/// Simple deterministic hash for pseudo-random generation from tick + seed.
fn pseudo_random(tick: u64, seed: u64) -> u64 {
    let mut x = tick.wrapping_mul(6364136223846793005).wrapping_add(seed);
    x ^= x >> 33;
    x = x.wrapping_mul(0xff51afd7ed558ccd);
    x ^= x >> 33;
    x
}

/// Pick a value in [lo, hi] using deterministic randomness.
fn pick_range(tick: u64, seed: u64, lo: u64, hi: u64) -> u64 {
    if lo >= hi {
        return lo;
    }
    lo + pseudo_random(tick, seed) % (hi - lo + 1)
}

/// Determine which tier is appropriate for the current scaling level.
pub fn tier_for_scaling(scaling_level: u32) -> ContractTier {
    match scaling_level {
        0..=4 => ContractTier::Starter,
        5..=14 => ContractTier::Basic,
        15..=29 => ContractTier::Standard,
        30..=49 => ContractTier::Advanced,
        50..=79 => ContractTier::Elite,
        _ => ContractTier::Legendary,
    }
}

/// Generate a contract name from tier and id.
fn contract_name(tier: ContractTier, id: u64) -> String {
    format!("{} Contract #{}", tier.name(), id)
}

/// Generate a contract based on scaling level and available resources.
///
/// `unlocked_resources`: resources the player can currently produce (non-waste, non-raw).
/// `tick`: current game tick for randomness seed.
/// `id`: unique contract id.
pub fn generate_contract(
    tier: ContractTier,
    scaling_level: u32,
    unlocked_resources: &[Resource],
    tick: u64,
    id: u64,
) -> Contract {
    let scale_mult = 1.0 + scaling_level as f64 * 0.15;

    // Pick 1-3 requirements depending on tier
    let num_requirements = match tier {
        ContractTier::Starter => 1,
        ContractTier::Basic | ContractTier::Standard => {
            1 + (pseudo_random(tick, id * 3) % 2) as usize
        }
        _ => 1 + (pseudo_random(tick, id * 5) % 3) as usize,
    };

    let (qty_lo, qty_hi) = tier.base_quantity_range();
    let (reward_lo, reward_hi) = tier.base_reward_range();

    // Filter resources to those matching tier-appropriate tiers
    let tier_max = match tier {
        ContractTier::Starter => 1,
        ContractTier::Basic => 2,
        ContractTier::Standard => 3,
        ContractTier::Advanced => 3,
        ContractTier::Elite => 4,
        ContractTier::Legendary => 5,
    };

    let eligible: Vec<Resource> = unlocked_resources
        .iter()
        .filter(|r| r.tier() <= tier_max && !r.is_waste())
        .copied()
        .collect();

    let mut requirements = Vec::new();
    for i in 0..num_requirements {
        let resource = if eligible.is_empty() {
            // Fallback: request iron ingots
            Resource::IronIngot
        } else {
            let idx = pseudo_random(tick, id * 7 + i as u64) as usize % eligible.len();
            eligible[idx]
        };

        let base_qty = pick_range(tick, id * 11 + i as u64, qty_lo, qty_hi);
        let quantity = (base_qty as f64 * scale_mult).ceil() as u64;

        requirements.push(ContractRequirement {
            resource,
            quantity,
            delivered: 0,
        });
    }

    let base_reward = pick_range(tick, id * 13, reward_lo as u64, reward_hi as u64) as i64;
    let reward = (base_reward as f64 * scale_mult).ceil() as i64;
    let bonus_reward = reward / 4; // 25% bonus for early completion
    let penalty = (reward as f64 * tier.penalty_fraction()).ceil() as i64;

    // Deadline: base ticks scaled down by scaling level, minimum 600 ticks
    let base_deadline = tier.base_deadline_ticks();
    let deadline_ticks = ((base_deadline as f64) * 0.95_f64.powi(scaling_level as i32))
        .ceil() as u64;
    let deadline_ticks = deadline_ticks.max(600);

    let rep_reward = tier.reputation_reward();
    let rep_penalty = rep_reward / 2;

    Contract {
        id,
        name: contract_name(tier, id),
        tier,
        requirements,
        reward,
        bonus_reward,
        penalty,
        deadline: tick + deadline_ticks,
        issued_at: tick,
        status: ContractStatus::Available,
        reputation_reward: rep_reward,
        reputation_penalty: rep_penalty,
    }
}
