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

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct OutputCounter {
    pub ore_count: u64,
    pub ingot_count: u64,
    pub widget_count: u64,
}

impl OutputCounter {
    pub fn new() -> Self {
        OutputCounter {
            ore_count: 0,
            ingot_count: 0,
            widget_count: 0,
        }
    }

    pub fn add(&mut self, resource: Resource) {
        match resource {
            Resource::Ore => self.ore_count += 1,
            Resource::Ingot => self.ingot_count += 1,
            Resource::Widget => self.widget_count += 1,
        }
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
