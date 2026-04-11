use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use crate::resources::{EntityType, Facing, Resource};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Position {
    pub x: usize,
    pub y: usize,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct EntityKind {
    pub kind: EntityType,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct FacingComponent {
    pub facing: Facing,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Processing {
    pub ticks_remaining: u32,
    pub input_a: Option<Resource>,
    pub input_b: Option<Resource>,
    pub output: Option<Resource>,
}

impl Processing {
    pub fn new() -> Self {
        Processing {
            ticks_remaining: 0,
            input_a: None,
            input_b: None,
            output: None,
        }
    }

    pub fn is_idle(&self) -> bool {
        self.ticks_remaining == 0 && self.output.is_none()
    }

    pub fn is_processing(&self) -> bool {
        self.ticks_remaining > 0
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct OreEmitter {
    pub interval: u32,
    pub ticks_since_emit: u32,
}

impl OreEmitter {
    pub fn new(interval: u32) -> Self {
        OreEmitter {
            interval,
            ticks_since_emit: 0,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct OutputCounter {
    pub counts: HashMap<Resource, u64>,
}

impl OutputCounter {
    pub fn new() -> Self {
        OutputCounter {
            counts: HashMap::new(),
        }
    }

    pub fn add(&mut self, resource: Resource) {
        *self.counts.entry(resource).or_insert(0) += 1;
    }

    pub fn get(&self, resource: Resource) -> u64 {
        self.counts.get(&resource).copied().unwrap_or(0)
    }

    pub fn total(&self) -> u64 {
        self.counts.values().sum()
    }

    /// Backward-compat accessors used by sidebar/popup.
    pub fn ore_count(&self) -> u64 {
        self.get(Resource::IronOre)
    }

    pub fn ingot_count(&self) -> u64 {
        self.get(Resource::IronIngot)
    }

    pub fn widget_count(&self) -> u64 {
        self.get(Resource::CircuitBoard)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum SplitterOutput {
    A,
    B,
}

impl SplitterOutput {
    pub fn toggle(&self) -> SplitterOutput {
        match self {
            SplitterOutput::A => SplitterOutput::B,
            SplitterOutput::B => SplitterOutput::A,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct SplitterState {
    pub next_output: SplitterOutput,
}

impl SplitterState {
    pub fn new() -> Self {
        SplitterState {
            next_output: SplitterOutput::A,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum MergerPriority {
    InputA,
    InputB,
}

impl MergerPriority {
    pub fn toggle(&self) -> MergerPriority {
        match self {
            MergerPriority::InputA => MergerPriority::InputB,
            MergerPriority::InputB => MergerPriority::InputA,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct MergerState {
    pub priority: MergerPriority,
}

impl MergerState {
    pub fn new() -> Self {
        MergerState {
            priority: MergerPriority::InputA,
        }
    }
}

/// Marker component for entities placed by the player (as opposed to level-placed).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PlayerPlaced;

/// Component for the anchor entity of a multi-tile building.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct MultiTile {
    pub width: usize,
    pub height: usize,
}

/// Marker component for secondary tiles that belong to a multi-tile building.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PartOfBuilding {
    pub anchor: hecs::Entity,
}
