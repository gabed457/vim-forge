use serde::{Deserialize, Serialize};
use std::collections::VecDeque;

use crate::resources::EntityType;

// ---------------------------------------------------------------------------
// Power pole / substation
// ---------------------------------------------------------------------------

/// Radius within which a power distribution entity connects to buildings.
pub fn connection_radius(entity_type: EntityType) -> u32 {
    match entity_type {
        EntityType::PowerPole => 8,
        EntityType::Substation => 16,
        _ => 0,
    }
}

/// Maximum wire distance between two power distribution entities.
pub fn wire_range(entity_type: EntityType) -> u32 {
    match entity_type {
        EntityType::PowerPole => 12,
        EntityType::Substation => 24,
        _ => 0,
    }
}

// ---------------------------------------------------------------------------
// PowerGrid
// ---------------------------------------------------------------------------

/// Represents one connected subgraph of power infrastructure.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PowerGrid {
    /// Unique grid id (index in the Vec<PowerGrid>).
    pub id: usize,
    /// Positions of power poles / substations in this grid.
    pub poles: Vec<(usize, usize)>,
    /// Positions of generators (producers) attached to this grid.
    pub generators: Vec<(usize, usize)>,
    /// Positions of consumers attached to this grid.
    pub consumers: Vec<(usize, usize)>,
    /// Total power available this tick (MW).
    pub available_mw: f64,
    /// Total power consumed this tick (MW).
    pub consumed_mw: f64,
    /// Efficiency: min(1.0, available / consumed).
    /// Machines on this grid run at this fraction of their normal speed.
    pub efficiency: f64,
}

impl PowerGrid {
    pub fn new(id: usize) -> Self {
        Self {
            id,
            poles: Vec::new(),
            generators: Vec::new(),
            consumers: Vec::new(),
            available_mw: 0.0,
            consumed_mw: 0.0,
            efficiency: 1.0,
        }
    }

    /// Recalculate efficiency from current available / consumed.
    pub fn recalculate_efficiency(&mut self) {
        if self.consumed_mw <= 0.0 {
            self.efficiency = 1.0;
        } else if self.available_mw >= self.consumed_mw {
            self.efficiency = 1.0;
        } else {
            self.efficiency = self.available_mw / self.consumed_mw;
        }
    }
}

// ---------------------------------------------------------------------------
// Pole graph
// ---------------------------------------------------------------------------

/// A node in the power pole graph.
#[derive(Clone, Debug)]
pub struct PoleNode {
    pub pos: (usize, usize),
    pub entity_type: EntityType,
}

/// Build connected subgraphs from a set of power poles / substations.
///
/// Two poles are connected if their manhattan distance is within both of their
/// wire ranges.
///
/// Returns a list of `PowerGrid` (one per subgraph). The caller should then
/// attach generators and consumers to each grid based on proximity to poles.
pub fn build_power_grids(poles: &[PoleNode]) -> Vec<PowerGrid> {
    if poles.is_empty() {
        return Vec::new();
    }

    // Build adjacency.
    let n = poles.len();
    let mut adj: Vec<Vec<usize>> = vec![Vec::new(); n];

    for i in 0..n {
        for j in (i + 1)..n {
            let dist = manhattan(poles[i].pos, poles[j].pos);
            let range = wire_range(poles[i].entity_type).min(wire_range(poles[j].entity_type));
            if dist <= range {
                adj[i].push(j);
                adj[j].push(i);
            }
        }
    }

    // BFS to find connected components.
    let mut visited = vec![false; n];
    let mut grids: Vec<PowerGrid> = Vec::new();

    for start in 0..n {
        if visited[start] {
            continue;
        }
        let grid_id = grids.len();
        let mut grid = PowerGrid::new(grid_id);
        let mut queue = VecDeque::new();
        queue.push_back(start);
        visited[start] = true;
        while let Some(idx) = queue.pop_front() {
            grid.poles.push(poles[idx].pos);
            for &next in &adj[idx] {
                if !visited[next] {
                    visited[next] = true;
                    queue.push_back(next);
                }
            }
        }
        grids.push(grid);
    }

    grids
}

fn manhattan(a: (usize, usize), b: (usize, usize)) -> u32 {
    let dx = (a.0 as isize - b.0 as isize).unsigned_abs() as u32;
    let dy = (a.1 as isize - b.1 as isize).unsigned_abs() as u32;
    dx + dy
}

/// Assign a building at `pos` to the nearest grid whose pole is within
/// `max_radius`. Returns the grid index if found.
pub fn find_grid_for_building(
    pos: (usize, usize),
    grids: &[PowerGrid],
    max_radius: u32,
) -> Option<usize> {
    let mut best: Option<(usize, u32)> = None;
    for (gi, grid) in grids.iter().enumerate() {
        for &pole_pos in &grid.poles {
            let dist = manhattan(pos, pole_pos);
            if dist <= max_radius {
                match best {
                    None => best = Some((gi, dist)),
                    Some((_, d)) if dist < d => best = Some((gi, dist)),
                    _ => {}
                }
            }
        }
    }
    best.map(|(gi, _)| gi)
}

/// Top-level update: rebuild power grids from current pole positions,
/// attach generators / consumers, sum power, compute efficiency.
///
/// `pole_nodes` — all power poles and substations on the map.
/// `generator_positions` — (pos, mw_output) for each generator.
/// `consumer_positions` — (pos, mw_required) for each consumer.
///
/// Returns the list of grids with efficiency computed.
pub fn update_power_grids(
    pole_nodes: &[PoleNode],
    generator_positions: &[((usize, usize), f64)],
    consumer_positions: &[((usize, usize), f64)],
) -> Vec<PowerGrid> {
    let mut grids = build_power_grids(pole_nodes);

    // The maximum connection radius across all pole types.
    let max_radius = 16_u32;

    for &(pos, mw) in generator_positions {
        if let Some(gi) = find_grid_for_building(pos, &grids, max_radius) {
            grids[gi].generators.push(pos);
            grids[gi].available_mw += mw;
        }
    }

    for &(pos, mw) in consumer_positions {
        if let Some(gi) = find_grid_for_building(pos, &grids, max_radius) {
            grids[gi].consumers.push(pos);
            grids[gi].consumed_mw += mw;
        }
    }

    for grid in &mut grids {
        grid.recalculate_efficiency();
    }

    grids
}
