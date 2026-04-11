use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::resources::Resource;
use super::network::{CircuitNetwork, CompareOp};

// ---------------------------------------------------------------------------
// Arithmetic Combinator
// ---------------------------------------------------------------------------

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum ArithmeticOp {
    Add,
    Subtract,
    Multiply,
    Divide,
    Modulo,
    And,
    Or,
    Xor,
}

/// An arithmetic combinator: reads signals, performs an operation, outputs a signal.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ArithmeticCombinator {
    pub pos: (usize, usize),
    /// Input network ID.
    pub input_network: usize,
    /// Output network ID.
    pub output_network: usize,
    /// Left operand: either a specific resource signal or a constant.
    pub left: SignalOrConstant,
    /// Right operand.
    pub right: SignalOrConstant,
    /// Operation to perform.
    pub operation: ArithmeticOp,
    /// Output signal resource.
    pub output_signal: Resource,
}

/// Either a resource signal value or a constant integer.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum SignalOrConstant {
    Signal(Resource),
    Constant(i64),
}

impl ArithmeticCombinator {
    /// Compute the output value given the current network signals.
    pub fn compute(&self, input_signals: &HashMap<Resource, i64>) -> i64 {
        let left_val = match &self.left {
            SignalOrConstant::Signal(res) => input_signals.get(res).copied().unwrap_or(0),
            SignalOrConstant::Constant(v) => *v,
        };
        let right_val = match &self.right {
            SignalOrConstant::Signal(res) => input_signals.get(res).copied().unwrap_or(0),
            SignalOrConstant::Constant(v) => *v,
        };

        match self.operation {
            ArithmeticOp::Add => left_val.wrapping_add(right_val),
            ArithmeticOp::Subtract => left_val.wrapping_sub(right_val),
            ArithmeticOp::Multiply => left_val.wrapping_mul(right_val),
            ArithmeticOp::Divide => {
                if right_val == 0 {
                    0
                } else {
                    left_val / right_val
                }
            }
            ArithmeticOp::Modulo => {
                if right_val == 0 {
                    0
                } else {
                    left_val % right_val
                }
            }
            ArithmeticOp::And => left_val & right_val,
            ArithmeticOp::Or => left_val | right_val,
            ArithmeticOp::Xor => left_val ^ right_val,
        }
    }
}

// ---------------------------------------------------------------------------
// Decider Combinator
// ---------------------------------------------------------------------------

/// A decider combinator: reads signals, evaluates a condition, outputs a signal.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct DeciderCombinator {
    pub pos: (usize, usize),
    /// Input network ID.
    pub input_network: usize,
    /// Output network ID.
    pub output_network: usize,
    /// Left operand signal.
    pub left: Resource,
    /// Comparison operator.
    pub operator: CompareOp,
    /// Right operand: signal or constant.
    pub right: SignalOrConstant,
    /// Output signal resource.
    pub output_signal: Resource,
    /// Output value when condition is true.
    pub output_value: DeciderOutput,
}

/// What the decider outputs when condition is true.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum DeciderOutput {
    /// Output the input signal value.
    InputCount,
    /// Output a fixed value.
    Constant(i64),
}

impl DeciderCombinator {
    /// Evaluate the condition and return the output value (0 if condition is false).
    pub fn compute(&self, input_signals: &HashMap<Resource, i64>) -> i64 {
        let left_val = input_signals.get(&self.left).copied().unwrap_or(0);
        let right_val = match &self.right {
            SignalOrConstant::Signal(res) => input_signals.get(res).copied().unwrap_or(0),
            SignalOrConstant::Constant(v) => *v,
        };

        let condition_met = match self.operator {
            CompareOp::GreaterThan => left_val > right_val,
            CompareOp::LessThan => left_val < right_val,
            CompareOp::Equal => left_val == right_val,
            CompareOp::GreaterEqual => left_val >= right_val,
            CompareOp::LessEqual => left_val <= right_val,
            CompareOp::NotEqual => left_val != right_val,
        };

        if condition_met {
            match &self.output_value {
                DeciderOutput::InputCount => left_val,
                DeciderOutput::Constant(v) => *v,
            }
        } else {
            0
        }
    }
}

// ---------------------------------------------------------------------------
// Constant Combinator
// ---------------------------------------------------------------------------

/// A constant combinator: outputs fixed signal values every tick.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ConstantCombinator {
    pub pos: (usize, usize),
    /// Output network ID.
    pub output_network: usize,
    /// Fixed output signals.
    pub signals: Vec<(Resource, i64)>,
    /// Whether this combinator is active.
    pub enabled: bool,
}

impl ConstantCombinator {
    pub fn new(pos: (usize, usize), output_network: usize) -> Self {
        Self {
            pos,
            output_network,
            signals: Vec::new(),
            enabled: true,
        }
    }
}

// ---------------------------------------------------------------------------
// Combinator tick
// ---------------------------------------------------------------------------

/// Process all combinators for one tick.
///
/// Reads from input networks, computes outputs, writes to output networks.
pub fn tick_combinators(
    networks: &mut Vec<CircuitNetwork>,
    arithmetic: &[ArithmeticCombinator],
    deciders: &[DeciderCombinator],
    constants: &[ConstantCombinator],
) {
    // Snapshot input signals before writing outputs.
    let snapshots: HashMap<usize, HashMap<Resource, i64>> = networks
        .iter()
        .map(|n| (n.id, n.signals.clone()))
        .collect();

    // Process arithmetic combinators.
    for ac in arithmetic {
        let input_signals = snapshots.get(&ac.input_network);
        if let Some(signals) = input_signals {
            let value = ac.compute(signals);
            if let Some(net) = networks.iter_mut().find(|n| n.id == ac.output_network) {
                net.add_signal(ac.output_signal, value);
            }
        }
    }

    // Process decider combinators.
    for dc in deciders {
        let input_signals = snapshots.get(&dc.input_network);
        if let Some(signals) = input_signals {
            let value = dc.compute(signals);
            if value != 0 {
                if let Some(net) = networks.iter_mut().find(|n| n.id == dc.output_network) {
                    net.add_signal(dc.output_signal, value);
                }
            }
        }
    }

    // Process constant combinators.
    for cc in constants {
        if !cc.enabled {
            continue;
        }
        if let Some(net) = networks.iter_mut().find(|n| n.id == cc.output_network) {
            for &(resource, value) in &cc.signals {
                net.add_signal(resource, value);
            }
        }
    }
}
