use serde::{Deserialize, Serialize};

use crate::resources::Resource;

// ---------------------------------------------------------------------------
// Rail network structures
// ---------------------------------------------------------------------------

/// State of the rail network — tracks, signals, stops.
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct TrainSystem {
    /// Set of tiles that have rail track.
    pub rails: Vec<(usize, usize)>,
    /// Signal positions and types.
    pub signals: Vec<Signal>,
    /// Train stops.
    pub stops: Vec<TrainStop>,
    /// Active trains.
    pub trains: Vec<Train>,
}

/// A signal on the rail network.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Signal {
    pub pos: (usize, usize),
    pub signal_type: SignalType,
    /// Whether the block ahead is clear.
    pub clear: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum SignalType {
    /// Standard block signal — train can proceed if block ahead is clear.
    Block,
    /// Chain signal — reflects the state of the next signal in the chain.
    Chain,
}

/// A named stop where trains can load/unload.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TrainStop {
    pub pos: (usize, usize),
    pub name: String,
}

// ---------------------------------------------------------------------------
// Train
// ---------------------------------------------------------------------------

/// An individual train.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Train {
    /// Current position on the grid.
    pub pos: (usize, usize),
    /// Speed in tiles per tick (4 on straight, 2 on curves).
    pub speed: u32,
    /// Fuel remaining (abstract units).
    pub fuel: u32,
    /// Cargo manifest.
    pub cargo: Vec<(Resource, u64)>,
    /// Maximum cargo slots.
    pub cargo_capacity: u64,
    /// Current schedule.
    pub schedule: TrainSchedule,
    /// Index into the schedule for the current destination.
    pub schedule_index: usize,
    /// Current path the train is following.
    pub current_path: Vec<(usize, usize)>,
    /// Index into current_path.
    pub path_index: usize,
    /// Whether the train is currently waiting at a stop.
    pub waiting: bool,
    /// Ticks waited at current stop.
    pub wait_ticks: u32,
}

impl Train {
    pub fn new(pos: (usize, usize), cargo_capacity: u64) -> Self {
        Self {
            pos,
            speed: 4,
            fuel: 100,
            cargo: Vec::new(),
            cargo_capacity,
            schedule: TrainSchedule::default(),
            schedule_index: 0,
            current_path: Vec::new(),
            path_index: 0,
            waiting: false,
            wait_ticks: 0,
        }
    }

    /// Total items currently carried.
    pub fn cargo_count(&self) -> u64 {
        self.cargo.iter().map(|(_, n)| n).sum()
    }

    /// Load cargo onto the train. Returns how many items were loaded.
    pub fn load(&mut self, resource: Resource, amount: u64) -> u64 {
        let space = self.cargo_capacity.saturating_sub(self.cargo_count());
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

    /// Unload cargo from the train. Returns how many items were unloaded.
    pub fn unload(&mut self, resource: Resource, amount: u64) -> u64 {
        if let Some(entry) = self.cargo.iter_mut().find(|(r, _)| *r == resource) {
            let unloaded = amount.min(entry.1);
            entry.1 -= unloaded;
            if entry.1 == 0 {
                self.cargo.retain(|(_, n)| *n > 0);
            }
            unloaded
        } else {
            0
        }
    }
}

// ---------------------------------------------------------------------------
// Schedule
// ---------------------------------------------------------------------------

/// A train's schedule — a list of stops with conditions.
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct TrainSchedule {
    pub entries: Vec<ScheduleEntry>,
}

/// One entry in a train schedule.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ScheduleEntry {
    /// Name of the stop to go to.
    pub stop_name: String,
    /// Condition to wait for at this stop.
    pub wait_condition: WaitCondition,
}

/// Condition for how long a train waits at a stop.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum WaitCondition {
    /// Wait for a fixed number of ticks.
    Ticks(u32),
    /// Wait until cargo is full.
    Full,
    /// Wait until cargo is empty.
    Empty,
    /// Wait until a specific resource reaches a count.
    ItemCount(Resource, u64),
}

// ---------------------------------------------------------------------------
// Train tick
// ---------------------------------------------------------------------------

/// Advance a train by one tick. The caller is responsible for providing the
/// path (computed via A* on the rail grid).
///
/// Returns true if the train reached a stop this tick.
pub fn tick_train(train: &mut Train) -> bool {
    if train.fuel == 0 {
        return false;
    }

    if train.waiting {
        train.wait_ticks += 1;
        return false;
    }

    if train.current_path.is_empty() || train.path_index >= train.current_path.len() {
        return false;
    }

    // Move along path.
    let steps = train.speed as usize;
    for _ in 0..steps {
        if train.path_index >= train.current_path.len() {
            break;
        }
        train.pos = train.current_path[train.path_index];
        train.path_index += 1;
        train.fuel = train.fuel.saturating_sub(1);
    }

    // Check if we've reached the end of the path (i.e., a stop).
    if train.path_index >= train.current_path.len() {
        train.waiting = true;
        train.wait_ticks = 0;
        return true;
    }

    false
}
