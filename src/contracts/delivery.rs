use serde::{Deserialize, Serialize};

use crate::contracts::board::ContractBoard;
use crate::market::prices::MarketState;
use crate::resources::Resource;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct DeliveryResult {
    /// Contract id that consumed items, if any.
    pub contract_matched: Option<u64>,
    /// Quantity sold at market (not consumed by contract).
    pub sold_at_market: u64,
    /// Total income from this delivery.
    pub income: i64,
}

/// Process a resource delivery at the trade hub.
///
/// First tries to match against active contracts. Any remainder is sold at market price.
pub fn process_delivery(
    resource: Resource,
    quantity: u64,
    board: &mut ContractBoard,
    market: &mut MarketState,
) -> DeliveryResult {
    let mut remaining = quantity;
    let mut contract_matched: Option<u64> = None;
    let contract_income: i64 = 0;

    // Try to deliver to active contracts
    for contract in &mut board.active {
        if remaining == 0 {
            break;
        }
        let consumed = contract.deliver(resource, remaining);
        if consumed > 0 {
            contract_matched = Some(contract.id);
            remaining -= consumed;
        }
    }

    // Sell remainder at market price (trade hub sells at 80% of market)
    let mut market_income: i64 = 0;
    if remaining > 0 {
        let sell = market.sell_price(resource);
        market_income = (sell * remaining as f64).floor() as i64;
        market.record_sale(resource, remaining);
    }

    // Check for completed contracts and collect rewards
    let tick = 0; // Caller should provide, but completions are checked separately
    let _ = tick;

    DeliveryResult {
        contract_matched,
        sold_at_market: remaining,
        income: contract_income + market_income,
    }
}
