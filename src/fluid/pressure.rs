use std::collections::HashMap;

use crate::resources::Resource;
use super::pipes::PipeGrid;

// ---------------------------------------------------------------------------
// Pump override
// ---------------------------------------------------------------------------

/// A pump forces fluid in one direction regardless of pressure.
#[derive(Clone, Debug)]
pub struct FluidPump {
    pub pos: (usize, usize),
    /// Direction the pump pushes: (dx, dy) where one of them is ±1.
    pub push_dir: (isize, isize),
    /// Flow rate the pump can sustain (units/tick).
    pub flow_rate: u32,
}

// ---------------------------------------------------------------------------
// Flow calculation
// ---------------------------------------------------------------------------

/// Compute how many units flow between two adjacent pipe tiles.
///
/// `level_a` / `level_b` — current levels.
/// `coefficient` — the lower of the two pipes' flow coefficients.
/// `distance_penalty` — 1.0 for first 20 tiles, then -5% per 10 tiles
///   (caller tracks pipe run lengths).
///
/// Returns a signed value: positive means A→B, negative means B→A.
pub fn flow_per_tick(level_a: u32, level_b: u32, coefficient: f32, distance_penalty: f32) -> i32 {
    let diff = level_a as f32 - level_b as f32;
    (diff * coefficient * distance_penalty) as i32
}

/// Compute the distance-based throughput penalty.
///
/// For the first 20 tiles without a pump the penalty is 1.0.
/// After that it drops by 5% per 10 additional tiles.
pub fn throughput_penalty(tiles_since_pump: u32) -> f32 {
    if tiles_since_pump <= 20 {
        1.0
    } else {
        let extra = tiles_since_pump - 20;
        let steps = extra / 10;
        (1.0 - 0.05 * steps as f32).max(0.1)
    }
}

// ---------------------------------------------------------------------------
// Fluid flow update
// ---------------------------------------------------------------------------

/// Per-tick fluid transfers computed during the update pass.
#[derive(Clone, Debug)]
pub struct FluidTransfer {
    pub from: (usize, usize),
    pub to: (usize, usize),
    pub amount: u32,
    pub resource: Resource,
}

/// Main tick function — compute and apply pressure-based flow across all pipes.
///
/// `pumps` lists active pump entities that override natural flow.
/// Returns the list of transfers applied (useful for rendering / debugging).
pub fn update_fluid_flow(
    pipe_grid: &mut PipeGrid,
    pumps: &[FluidPump],
) -> Vec<FluidTransfer> {
    let mut transfers: Vec<FluidTransfer> = Vec::new();

    // Build a snapshot of levels so we read consistent state.
    let snapshot: HashMap<(usize, usize), (Option<Resource>, u32, u32, f32)> = pipe_grid
        .pipes
        .iter()
        .map(|(&pos, p)| (pos, (p.fluid_type, p.level, p.capacity, p.flow_coefficient)))
        .collect();

    // Set of pump-forced edges: (from, to) → forced flow rate.
    let mut pump_edges: HashMap<((usize, usize), (usize, usize)), u32> = HashMap::new();
    for pump in pumps {
        let nx = pump.pos.0 as isize + pump.push_dir.0;
        let ny = pump.pos.1 as isize + pump.push_dir.1;
        if nx >= 0 && ny >= 0 {
            let target = (nx as usize, ny as usize);
            if snapshot.contains_key(&target) {
                pump_edges.insert((pump.pos, target), pump.flow_rate);
            }
        }
    }

    // For each pipe, consider flow to each connected neighbour.
    let adjacencies: [(isize, isize); 4] = [(0, -1), (1, 0), (0, 1), (-1, 0)];

    for (&pos_a, &(fluid_a, level_a, _cap_a, coeff_a)) in &snapshot {
        for &(dx, dy) in &adjacencies {
            let nx = pos_a.0 as isize + dx;
            let ny = pos_a.1 as isize + dy;
            if nx < 0 || ny < 0 {
                continue;
            }
            let pos_b = (nx as usize, ny as usize);
            // Only process each edge once (A < B lexicographically).
            if pos_a >= pos_b {
                continue;
            }
            let (_fluid_b, level_b, _cap_b, coeff_b) = match snapshot.get(&pos_b) {
                Some(v) => *v,
                None => continue,
            };

            // Check for pump override.
            if let Some(&rate) = pump_edges.get(&(pos_a, pos_b)) {
                if let Some(res) = fluid_a {
                    let actual = rate.min(level_a);
                    if actual > 0 {
                        transfers.push(FluidTransfer {
                            from: pos_a,
                            to: pos_b,
                            amount: actual,
                            resource: res,
                        });
                    }
                }
                continue;
            }
            if let Some(&rate) = pump_edges.get(&(pos_b, pos_a)) {
                if let Some(res) = _fluid_b {
                    let actual = rate.min(level_b);
                    if actual > 0 {
                        transfers.push(FluidTransfer {
                            from: pos_b,
                            to: pos_a,
                            amount: actual,
                            resource: res,
                        });
                    }
                }
                continue;
            }

            // Natural pressure-based flow.
            let coeff = coeff_a.min(coeff_b);
            let penalty = 1.0_f32; // caller should track run lengths for penalty
            let flow = flow_per_tick(level_a, level_b, coeff, penalty);

            if flow > 0 {
                if let Some(res) = fluid_a {
                    let actual = (flow as u32).min(level_a);
                    if actual > 0 {
                        transfers.push(FluidTransfer {
                            from: pos_a,
                            to: pos_b,
                            amount: actual,
                            resource: res,
                        });
                    }
                }
            } else if flow < 0 {
                if let Some(res) = _fluid_b {
                    let actual = ((-flow) as u32).min(level_b);
                    if actual > 0 {
                        transfers.push(FluidTransfer {
                            from: pos_b,
                            to: pos_a,
                            amount: actual,
                            resource: res,
                        });
                    }
                }
            }
        }
    }

    // Apply transfers.
    for t in &transfers {
        if let Some(src) = pipe_grid.pipes.get_mut(&t.from) {
            src.level = src.level.saturating_sub(t.amount);
            if src.level == 0 {
                src.fluid_type = None;
            }
        }
        if let Some(dst) = pipe_grid.pipes.get_mut(&t.to) {
            let space = dst.capacity.saturating_sub(dst.level);
            let added = t.amount.min(space);
            dst.level += added;
            if added > 0 {
                dst.fluid_type = Some(t.resource);
            }
        }
    }

    transfers
}
