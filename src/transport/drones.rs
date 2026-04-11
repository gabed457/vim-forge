use serde::{Deserialize, Serialize};

use crate::resources::Resource;

// ---------------------------------------------------------------------------
// Drone port
// ---------------------------------------------------------------------------

/// A drone port that dispatches and receives drones.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct DronePort {
    pub pos: (usize, usize),
    /// Maximum range in tiles.
    pub range: u32,
    /// Maximum drones this port can have.
    pub max_drones: u32,
    /// Currently idle drones at this port.
    pub idle_drones: u32,
    /// Input buffer — items waiting to be picked up by drones.
    pub input_buffer: Vec<(Resource, u32)>,
}

impl DronePort {
    pub fn new(pos: (usize, usize)) -> Self {
        Self {
            pos,
            range: 20,
            max_drones: 5,
            idle_drones: 5,
            input_buffer: Vec::new(),
        }
    }

    /// Check if another port is within range.
    pub fn in_range(&self, other: (usize, usize)) -> bool {
        let dx = (self.pos.0 as isize - other.0 as isize).unsigned_abs() as u32;
        let dy = (self.pos.1 as isize - other.1 as isize).unsigned_abs() as u32;
        // Use euclidean distance for drone range.
        dx * dx + dy * dy <= self.range * self.range
    }

    /// Dispatch a drone if one is available.
    pub fn dispatch(&mut self) -> bool {
        if self.idle_drones > 0 {
            self.idle_drones -= 1;
            true
        } else {
            false
        }
    }

    /// Return a drone to this port.
    pub fn receive_drone(&mut self) {
        self.idle_drones = (self.idle_drones + 1).min(self.max_drones);
    }
}

// ---------------------------------------------------------------------------
// Drone
// ---------------------------------------------------------------------------

/// An individual drone in flight.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Drone {
    /// Current position (can be fractional for smooth flight, but we use grid pos).
    pub pos: (usize, usize),
    /// Target position.
    pub target: (usize, usize),
    /// Speed: 3 tiles per tick (direct flight).
    pub speed: u32,
    /// Cargo — a single item.
    pub cargo: Option<(Resource, u32)>,
    /// Maximum cargo (1 stack).
    pub capacity: u32,
    /// Port this drone belongs to.
    pub home_port: (usize, usize),
    /// Whether the drone is returning home.
    pub returning: bool,
    /// Progress toward target (tiles remaining).
    pub distance_remaining: u32,
}

impl Drone {
    pub fn new(
        home: (usize, usize),
        target: (usize, usize),
        cargo: Option<(Resource, u32)>,
    ) -> Self {
        let dx = (home.0 as isize - target.0 as isize).unsigned_abs() as u32;
        let dy = (home.1 as isize - target.1 as isize).unsigned_abs() as u32;
        let distance = ((dx * dx + dy * dy) as f64).sqrt() as u32;
        Self {
            pos: home,
            target,
            speed: 3,
            cargo,
            capacity: 1,
            home_port: home,
            returning: false,
            distance_remaining: distance,
        }
    }

    /// Tick this drone. Returns true if it reached its target.
    pub fn tick(&mut self) -> bool {
        if self.distance_remaining == 0 {
            return true;
        }
        let moved = self.speed.min(self.distance_remaining);
        self.distance_remaining -= moved;
        if self.distance_remaining == 0 {
            self.pos = self.target;
            return true;
        }
        false
    }
}

// ---------------------------------------------------------------------------
// Maintenance drone
// ---------------------------------------------------------------------------

/// A maintenance drone that auto-repairs broken machines in range.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct MaintenanceDrone {
    pub home: (usize, usize),
    pub range: u32,
    pub repair_speed: f64, // health % restored per tick
}

impl MaintenanceDrone {
    pub fn new(home: (usize, usize)) -> Self {
        Self {
            home,
            range: 20,
            repair_speed: 5.0,
        }
    }

    /// Check if a position is within repair range.
    pub fn in_range(&self, pos: (usize, usize)) -> bool {
        let dx = (self.home.0 as isize - pos.0 as isize).unsigned_abs() as u32;
        let dy = (self.home.1 as isize - pos.1 as isize).unsigned_abs() as u32;
        dx * dx + dy * dy <= self.range * self.range
    }
}
