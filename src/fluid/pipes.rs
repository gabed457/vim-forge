use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::resources::Resource;

// ---------------------------------------------------------------------------
// Pipe connection bitmask
// ---------------------------------------------------------------------------

/// Connection directions encoded as a bitmask.
/// Up=1, Right=2, Down=4, Left=8
pub const CONN_UP: u8 = 1;
pub const CONN_RIGHT: u8 = 2;
pub const CONN_DOWN: u8 = 4;
pub const CONN_LEFT: u8 = 8;

/// Returns the box-drawing character for the given connection bitmask (0..15).
pub fn connection_glyph(mask: u8) -> char {
    match mask & 0x0F {
        0b0000 => '\u{00B7}', // isolated: middle dot
        0b0001 => '\u{2575}', // up only
        0b0010 => '\u{2576}', // right only
        0b0011 => '\u{2514}', // up + right: └
        0b0100 => '\u{2577}', // down only
        0b0101 => '\u{2502}', // up + down: │
        0b0110 => '\u{250C}', // right + down: ┌
        0b0111 => '\u{251C}', // up + right + down: ├
        0b1000 => '\u{2574}', // left only
        0b1001 => '\u{2518}', // up + left: ┘
        0b1010 => '\u{2500}', // right + left: ─
        0b1011 => '\u{2534}', // up + right + left: ┴
        0b1100 => '\u{2510}', // down + left: ┐
        0b1101 => '\u{2524}', // up + down + left: ┤
        0b1110 => '\u{252C}', // right + down + left: ┬
        0b1111 => '\u{253C}', // all four: ┼
        _ => '?',
    }
}

/// Returns the direction bitmask for a (dx, dy) offset from center.
pub fn direction_bit(dx: isize, dy: isize) -> Option<u8> {
    match (dx, dy) {
        (0, -1) => Some(CONN_UP),
        (1, 0) => Some(CONN_RIGHT),
        (0, 1) => Some(CONN_DOWN),
        (-1, 0) => Some(CONN_LEFT),
        _ => None,
    }
}

/// Returns the opposite direction bit.
pub fn opposite_bit(bit: u8) -> u8 {
    match bit {
        CONN_UP => CONN_DOWN,
        CONN_DOWN => CONN_UP,
        CONN_LEFT => CONN_RIGHT,
        CONN_RIGHT => CONN_LEFT,
        _ => 0,
    }
}

// ---------------------------------------------------------------------------
// PipeState
// ---------------------------------------------------------------------------

/// State of a single pipe tile.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PipeState {
    /// The fluid currently in this pipe, if any.
    pub fluid_type: Option<Resource>,
    /// Current fluid level (units for solid-like, millilitres for fluid).
    pub level: u32,
    /// Maximum capacity of this pipe segment.
    pub capacity: u32,
    /// Flow coefficient — higher means fluid moves faster.
    pub flow_coefficient: f32,
    /// Connection bitmask computed from neighbours.
    pub connections: u8,
}

impl PipeState {
    pub fn new(capacity: u32) -> Self {
        Self {
            fluid_type: None,
            level: 0,
            capacity,
            flow_coefficient: 1.0,
            connections: 0,
        }
    }

    pub fn fill_fraction(&self) -> f32 {
        if self.capacity == 0 {
            return 0.0;
        }
        self.level as f32 / self.capacity as f32
    }

    /// Returns true if this pipe can accept the given fluid.
    pub fn can_accept(&self, fluid: Resource) -> bool {
        match self.fluid_type {
            None => true,
            Some(existing) => existing == fluid,
        }
    }
}

// ---------------------------------------------------------------------------
// PipeGrid — collection of pipe states on the map
// ---------------------------------------------------------------------------

/// The grid of all pipe states, keyed by (x, y) position.
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct PipeGrid {
    pub pipes: HashMap<(usize, usize), PipeState>,
}

impl PipeGrid {
    pub fn new() -> Self {
        Self {
            pipes: HashMap::new(),
        }
    }

    /// Place a new pipe at (x, y) with the given capacity, then update
    /// connections for this tile and its neighbours.
    pub fn place_pipe(&mut self, x: usize, y: usize, capacity: u32) {
        self.pipes.insert((x, y), PipeState::new(capacity));
        self.update_connections(x, y);
    }

    /// Remove a pipe at (x, y) and update neighbour connections.
    pub fn remove_pipe(&mut self, x: usize, y: usize) {
        self.pipes.remove(&(x, y));
        // Update neighbours so they no longer point toward this tile.
        for &(dx, dy) in &[(0isize, -1isize), (1, 0), (0, 1), (-1, 0)] {
            let nx = x as isize + dx;
            let ny = y as isize + dy;
            if nx >= 0 && ny >= 0 {
                let npos = (nx as usize, ny as usize);
                if self.pipes.contains_key(&npos) {
                    self.recompute_single(npos.0, npos.1);
                }
            }
        }
    }

    /// Recompute connections for a tile and its neighbours.
    fn update_connections(&mut self, x: usize, y: usize) {
        self.recompute_single(x, y);
        for &(dx, dy) in &[(0isize, -1isize), (1, 0), (0, 1), (-1, 0)] {
            let nx = x as isize + dx;
            let ny = y as isize + dy;
            if nx >= 0 && ny >= 0 {
                let npos = (nx as usize, ny as usize);
                if self.pipes.contains_key(&npos) {
                    self.recompute_single(npos.0, npos.1);
                }
            }
        }
    }

    /// Recompute the connection bitmask for a single tile.
    fn recompute_single(&mut self, x: usize, y: usize) {
        let mut mask: u8 = 0;
        for &(dx, dy) in &[(0isize, -1isize), (1, 0), (0, 1), (-1, 0)] {
            let nx = x as isize + dx;
            let ny = y as isize + dy;
            if nx >= 0 && ny >= 0 {
                if self.pipes.contains_key(&(nx as usize, ny as usize)) {
                    if let Some(bit) = direction_bit(dx, dy) {
                        mask |= bit;
                    }
                }
            }
        }
        if let Some(pipe) = self.pipes.get_mut(&(x, y)) {
            pipe.connections = mask;
        }
    }

    /// Get the display glyph for the pipe at (x, y).
    pub fn glyph_at(&self, x: usize, y: usize) -> Option<char> {
        self.pipes.get(&(x, y)).map(|p| connection_glyph(p.connections))
    }
}
