use serde::{Deserialize, Serialize};

use crate::resources::Resource;

// ---------------------------------------------------------------------------
// Plane system
// ---------------------------------------------------------------------------

/// Top-level state for the plane/runway system.
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct PlaneSystem {
    pub runways: Vec<Runway>,
    pub hangars: Vec<Hangar>,
    pub planes: Vec<Plane>,
}

/// A runway — requires 3+ contiguous tiles in a row.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Runway {
    /// Start position of the runway.
    pub start: (usize, usize),
    /// Length in tiles (minimum 3).
    pub length: u32,
    /// Whether the runway is horizontal (true) or vertical (false).
    pub horizontal: bool,
    /// Whether a plane is currently using this runway.
    pub occupied: bool,
}

impl Runway {
    pub fn new(start: (usize, usize), length: u32, horizontal: bool) -> Self {
        Self {
            start,
            length,
            horizontal,
            occupied: false,
        }
    }

    pub fn is_valid(&self) -> bool {
        self.length >= 3
    }
}

/// A hangar where planes are stored when not in use.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Hangar {
    pub pos: (usize, usize),
    pub capacity: u32,
    pub planes_stored: u32,
}

impl Hangar {
    pub fn new(pos: (usize, usize), capacity: u32) -> Self {
        Self {
            pos,
            capacity,
            planes_stored: 0,
        }
    }
}

// ---------------------------------------------------------------------------
// Plane types
// ---------------------------------------------------------------------------

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum PlaneType {
    /// Carries 500 solid items.
    Cargo,
    /// Carries 100,000 mL of fluid.
    Tanker,
}

/// An individual plane.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Plane {
    pub plane_type: PlaneType,
    /// Current state.
    pub state: PlaneState,
    /// Cargo (solid items for Cargo, fluid for Tanker).
    pub cargo: Vec<(Resource, u64)>,
    /// Max capacity.
    pub capacity: u64,
    /// Fuel consumed per flight (mL of RocketFuel).
    pub fuel_cost: u32,
    /// Ticks remaining in current flight.
    pub flight_ticks_remaining: u32,
    /// Total flight time (ticks).
    pub flight_duration: u32,
    /// Origin runway index.
    pub origin_runway: Option<usize>,
    /// Destination runway index.
    pub destination_runway: Option<usize>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum PlaneState {
    /// Parked in hangar.
    Hangared,
    /// Loading cargo at origin.
    Loading,
    /// In flight.
    InFlight,
    /// Unloading cargo at destination.
    Unloading,
}

impl Plane {
    pub fn cargo_plane() -> Self {
        Self {
            plane_type: PlaneType::Cargo,
            state: PlaneState::Hangared,
            cargo: Vec::new(),
            capacity: 500,
            fuel_cost: 100, // 100 mL RocketFuel per flight
            flight_ticks_remaining: 0,
            flight_duration: 30, // 30 ticks flight time
            origin_runway: None,
            destination_runway: None,
        }
    }

    pub fn tanker_plane() -> Self {
        Self {
            plane_type: PlaneType::Tanker,
            state: PlaneState::Hangared,
            cargo: Vec::new(),
            capacity: 100_000,
            fuel_cost: 100,
            flight_ticks_remaining: 0,
            flight_duration: 30,
            origin_runway: None,
            destination_runway: None,
        }
    }

    /// Total cargo currently carried.
    pub fn cargo_count(&self) -> u64 {
        self.cargo.iter().map(|(_, n)| n).sum()
    }

    /// Load cargo. Returns how much was loaded.
    pub fn load(&mut self, resource: Resource, amount: u64) -> u64 {
        let space = self.capacity.saturating_sub(self.cargo_count());
        let loaded = amount.min(space);
        if loaded > 0 {
            if let Some(entry) = self.cargo.iter_mut().find(|(r, _)| *r == resource) {
                entry.1 += loaded;
            } else {
                self.cargo.push((resource, loaded));
            }
        }
        loaded
    }

    /// Unload all cargo.
    pub fn unload_all(&mut self) -> Vec<(Resource, u64)> {
        std::mem::take(&mut self.cargo)
    }
}

/// Tick a plane. Returns true if it just completed a flight.
pub fn tick_plane(plane: &mut Plane) -> bool {
    match plane.state {
        PlaneState::InFlight => {
            if plane.flight_ticks_remaining > 0 {
                plane.flight_ticks_remaining -= 1;
            }
            if plane.flight_ticks_remaining == 0 {
                plane.state = PlaneState::Unloading;
                return true;
            }
            false
        }
        _ => false,
    }
}
