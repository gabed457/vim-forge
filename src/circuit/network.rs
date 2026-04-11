use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::resources::Resource;

// ---------------------------------------------------------------------------
// Wire color
// ---------------------------------------------------------------------------

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum WireColor {
    Red,
    Green,
}

// ---------------------------------------------------------------------------
// Circuit Network
// ---------------------------------------------------------------------------

/// A circuit network — a set of entities connected by wires of one color.
/// Each network carries signals: a map from Resource to i64 value.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CircuitNetwork {
    pub id: usize,
    pub color: WireColor,
    /// Current signals on this network.
    pub signals: HashMap<Resource, i64>,
    /// Positions of entities connected to this network.
    pub connected_entities: Vec<(usize, usize)>,
}

impl CircuitNetwork {
    pub fn new(id: usize, color: WireColor) -> Self {
        Self {
            id,
            color,
            signals: HashMap::new(),
            connected_entities: Vec::new(),
        }
    }

    /// Clear all signals (called at start of each tick before propagation).
    pub fn clear_signals(&mut self) {
        self.signals.clear();
    }

    /// Add a signal value (signals are summed from all sources).
    pub fn add_signal(&mut self, resource: Resource, value: i64) {
        let entry = self.signals.entry(resource).or_insert(0);
        *entry += value;
    }

    /// Get the current signal value for a resource.
    pub fn get_signal(&self, resource: Resource) -> i64 {
        self.signals.get(&resource).copied().unwrap_or(0)
    }

    /// Get all non-zero signals.
    pub fn all_signals(&self) -> &HashMap<Resource, i64> {
        &self.signals
    }
}

// ---------------------------------------------------------------------------
// Circuit condition
// ---------------------------------------------------------------------------

/// A condition that can be evaluated against network signals.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CircuitCondition {
    /// The resource signal to check.
    pub resource: Resource,
    /// Comparison operator.
    pub operator: CompareOp,
    /// Value to compare against.
    pub value: i64,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum CompareOp {
    GreaterThan,
    LessThan,
    Equal,
    GreaterEqual,
    LessEqual,
    NotEqual,
}

impl CircuitCondition {
    pub fn evaluate(&self, signals: &HashMap<Resource, i64>) -> bool {
        let signal_value = signals.get(&self.resource).copied().unwrap_or(0);
        match self.operator {
            CompareOp::GreaterThan => signal_value > self.value,
            CompareOp::LessThan => signal_value < self.value,
            CompareOp::Equal => signal_value == self.value,
            CompareOp::GreaterEqual => signal_value >= self.value,
            CompareOp::LessEqual => signal_value <= self.value,
            CompareOp::NotEqual => signal_value != self.value,
        }
    }
}

// ---------------------------------------------------------------------------
// Entity circuit connection
// ---------------------------------------------------------------------------

/// How an entity is connected to circuit networks.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct EntityCircuitConnection {
    pub pos: (usize, usize),
    /// Network IDs this entity is connected to (can be on red, green, or both).
    pub network_ids: Vec<usize>,
    /// Optional enable condition: if set, entity is only active when condition is met.
    pub enable_condition: Option<CircuitCondition>,
    /// Whether the entity is currently enabled (computed each tick).
    pub enabled: bool,
}

impl EntityCircuitConnection {
    pub fn new(pos: (usize, usize)) -> Self {
        Self {
            pos,
            network_ids: Vec::new(),
            enable_condition: None,
            enabled: true,
        }
    }
}

// ---------------------------------------------------------------------------
// Update function
// ---------------------------------------------------------------------------

/// Main per-tick circuit update.
///
/// 1. Clear all network signals.
/// 2. Collect signals from constant combinators and other output entities.
/// 3. Propagate through arithmetic/decider combinators (handled externally).
/// 4. Evaluate enable conditions on connected entities.
///
/// `signal_sources` — (network_id, resource, value) tuples from all signal-producing entities.
/// `connections` — all entity circuit connections to evaluate.
pub fn update_circuits(
    networks: &mut Vec<CircuitNetwork>,
    signal_sources: &[(usize, Resource, i64)],
    connections: &mut [EntityCircuitConnection],
) {
    // Step 1: Clear.
    for net in networks.iter_mut() {
        net.clear_signals();
    }

    // Step 2: Add signals from sources.
    for &(net_id, resource, value) in signal_sources {
        if let Some(net) = networks.iter_mut().find(|n| n.id == net_id) {
            net.add_signal(resource, value);
        }
    }

    // Step 3: Evaluate enable conditions.
    for conn in connections.iter_mut() {
        if let Some(ref condition) = conn.enable_condition {
            // Merge signals from all connected networks.
            let mut merged: HashMap<Resource, i64> = HashMap::new();
            for &net_id in &conn.network_ids {
                if let Some(net) = networks.iter().find(|n| n.id == net_id) {
                    for (&res, &val) in &net.signals {
                        *merged.entry(res).or_insert(0) += val;
                    }
                }
            }
            conn.enabled = condition.evaluate(&merged);
        } else {
            conn.enabled = true;
        }
    }
}
