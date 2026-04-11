use serde::{Deserialize, Serialize};

use crate::resources::Resource;

// ---------------------------------------------------------------------------
// Truck system
// ---------------------------------------------------------------------------

/// The truck network state.
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct TruckSystem {
    /// Road tiles.
    pub roads: Vec<(usize, usize)>,
    /// Truck depots.
    pub depots: Vec<TruckDepot>,
    /// Active trucks.
    pub trucks: Vec<Truck>,
}

/// A truck depot — trucks originate from and return to depots.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TruckDepot {
    pub pos: (usize, usize),
    pub max_trucks: u32,
    pub active_trucks: u32,
}

impl TruckDepot {
    pub fn new(pos: (usize, usize), max_trucks: u32) -> Self {
        Self {
            pos,
            max_trucks,
            active_trucks: 0,
        }
    }

    pub fn can_dispatch(&self) -> bool {
        self.active_trucks < self.max_trucks
    }
}

// ---------------------------------------------------------------------------
// Truck
// ---------------------------------------------------------------------------

/// An individual truck.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Truck {
    pub pos: (usize, usize),
    /// Destination position.
    pub destination: (usize, usize),
    /// Speed: 2 tiles per tick on roads.
    pub speed: u32,
    /// Cargo.
    pub cargo: Vec<(Resource, u32)>,
    /// Max items.
    pub capacity: u32,
    /// Current path.
    pub path: Vec<(usize, usize)>,
    /// Index into path.
    pub path_index: usize,
    /// Whether the truck has reached its destination.
    pub arrived: bool,
    /// Whether the truck is returning to depot.
    pub returning: bool,
}

impl Truck {
    pub fn new(start: (usize, usize), destination: (usize, usize)) -> Self {
        Self {
            pos: start,
            destination,
            speed: 2,
            cargo: Vec::new(),
            capacity: 25,
            path: Vec::new(),
            path_index: 0,
            arrived: false,
            returning: false,
        }
    }

    /// Total items carried.
    pub fn cargo_count(&self) -> u32 {
        self.cargo.iter().map(|(_, n)| n).sum()
    }

    /// Load cargo. Returns how many items were loaded.
    pub fn load(&mut self, resource: Resource, amount: u32) -> u32 {
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

    /// Unload all cargo. Returns the items.
    pub fn unload_all(&mut self) -> Vec<(Resource, u32)> {
        std::mem::take(&mut self.cargo)
    }
}

/// Advance a truck by one tick.
///
/// Returns true if the truck just arrived at its destination.
pub fn tick_truck(truck: &mut Truck) -> bool {
    if truck.arrived {
        return false;
    }

    if truck.path.is_empty() || truck.path_index >= truck.path.len() {
        truck.arrived = true;
        return true;
    }

    let steps = truck.speed as usize;
    for _ in 0..steps {
        if truck.path_index >= truck.path.len() {
            break;
        }
        truck.pos = truck.path[truck.path_index];
        truck.path_index += 1;
    }

    if truck.path_index >= truck.path.len() {
        truck.arrived = true;
        return true;
    }

    false
}
