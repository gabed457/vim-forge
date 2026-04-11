use serde::{Deserialize, Serialize};

use crate::resources::Resource;

// ---------------------------------------------------------------------------
// Waste buffer (internal machine buffer)
// ---------------------------------------------------------------------------

/// Internal waste buffer for a machine. If this fills up, the machine HALTS.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct WasteBuffer {
    /// Solid waste items buffered.
    pub solid_count: u32,
    /// Fluid waste in mL buffered.
    pub fluid_ml: u32,
    /// Solid buffer capacity (machine halts when exceeded).
    pub solid_capacity: u32,
    /// Fluid buffer capacity (machine halts when exceeded).
    pub fluid_capacity: u32,
}

impl WasteBuffer {
    pub fn new() -> Self {
        Self {
            solid_count: 0,
            fluid_ml: 0,
            solid_capacity: 10,
            fluid_capacity: 500,
        }
    }

    /// Returns true if the machine should halt due to full waste buffer.
    pub fn is_blocked(&self) -> bool {
        self.solid_count >= self.solid_capacity || self.fluid_ml >= self.fluid_capacity
    }

    /// Try to add solid waste. Returns false if buffer is full.
    pub fn add_solid(&mut self, count: u32) -> bool {
        if self.solid_count + count > self.solid_capacity {
            return false;
        }
        self.solid_count += count;
        true
    }

    /// Try to add fluid waste. Returns false if buffer is full.
    pub fn add_fluid(&mut self, ml: u32) -> bool {
        if self.fluid_ml + ml > self.fluid_capacity {
            return false;
        }
        self.fluid_ml += ml;
        true
    }

    /// Drain solid waste (e.g. picked up by belt / waste bin).
    pub fn drain_solid(&mut self, max: u32) -> u32 {
        let drained = self.solid_count.min(max);
        self.solid_count -= drained;
        drained
    }

    /// Drain fluid waste.
    pub fn drain_fluid(&mut self, max: u32) -> u32 {
        let drained = self.fluid_ml.min(max);
        self.fluid_ml -= drained;
        drained
    }
}

impl Default for WasteBuffer {
    fn default() -> Self {
        Self::new()
    }
}

// ---------------------------------------------------------------------------
// Disposal entities
// ---------------------------------------------------------------------------

/// State of a Waste Bin (simple solid waste storage).
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct WasteBinState {
    pub items: Vec<(Resource, u32)>,
    pub total_count: u32,
    pub capacity: u32,
}

impl WasteBinState {
    pub fn new(capacity: u32) -> Self {
        Self {
            items: Vec::new(),
            total_count: 0,
            capacity,
        }
    }

    pub fn is_full(&self) -> bool {
        self.total_count >= self.capacity
    }

    pub fn try_add(&mut self, resource: Resource, count: u32) -> u32 {
        let space = self.capacity.saturating_sub(self.total_count);
        let accepted = count.min(space);
        if accepted > 0 {
            if let Some(entry) = self.items.iter_mut().find(|(r, _)| *r == resource) {
                entry.1 += accepted;
            } else {
                self.items.push((resource, accepted));
            }
            self.total_count += accepted;
        }
        accepted
    }
}

/// State of a Vent Stack (destroys gas, adds pollution).
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct VentStackState {
    /// Total gas destroyed this tick (for pollution calculation).
    pub gas_vented_this_tick: u32,
}

impl VentStackState {
    pub fn new() -> Self {
        Self {
            gas_vented_this_tick: 0,
        }
    }

    /// Vent gas — returns pollution generated.
    pub fn vent(&mut self, amount: u32) -> f64 {
        self.gas_vented_this_tick += amount;
        (amount as f64 / 100.0) * super::pollution::VENT_POLLUTION_PER_100L
    }

    pub fn reset_tick(&mut self) {
        self.gas_vented_this_tick = 0;
    }
}

impl Default for VentStackState {
    fn default() -> Self {
        Self::new()
    }
}

// ---------------------------------------------------------------------------
// Recycler types
// ---------------------------------------------------------------------------

/// Describes what a recycler converts: input waste → useful output.
#[derive(Clone, Debug)]
pub struct RecyclerSpec {
    pub input: Resource,
    pub input_amount: u32,
    pub output: Resource,
    pub output_amount: u32,
    /// Ticks per conversion cycle.
    pub cycle_ticks: u32,
}

/// Get recycler specs for known recycler types.
pub fn recycler_specs() -> Vec<RecyclerSpec> {
    vec![
        // SlagRecycler: Slag → IronOre (partial recovery)
        RecyclerSpec {
            input: Resource::Slag,
            input_amount: 5,
            output: Resource::IronOre,
            output_amount: 1,
            cycle_ticks: 10,
        },
        // WastewaterTreatment: Wastewater → Water
        RecyclerSpec {
            input: Resource::Wastewater,
            input_amount: 100,
            output: Resource::Water,
            output_amount: 80,
            cycle_ticks: 15,
        },
        // AcidNeutralizer: SpentAcid → (destroyed, no useful output but clears waste)
        RecyclerSpec {
            input: Resource::SpentAcid,
            input_amount: 10,
            output: Resource::Stone, // neutralised to inert solid
            output_amount: 1,
            cycle_ticks: 8,
        },
        // MetalRecovery: MetalShavings → IronIngot
        RecyclerSpec {
            input: Resource::MetalShavings,
            input_amount: 10,
            output: Resource::IronIngot,
            output_amount: 1,
            cycle_ticks: 12,
        },
        // RedMud processing: RedMud → AluminumIngot (small yield)
        RecyclerSpec {
            input: Resource::RedMud,
            input_amount: 20,
            output: Resource::AluminumIngot,
            output_amount: 1,
            cycle_ticks: 20,
        },
    ]
}

/// Runtime state of a recycler machine.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct RecyclerState {
    pub input_buffer: u32,
    pub output_buffer: u32,
    pub ticks_remaining: u32,
    pub active: bool,
}

impl RecyclerState {
    pub fn new() -> Self {
        Self {
            input_buffer: 0,
            output_buffer: 0,
            ticks_remaining: 0,
            active: false,
        }
    }

    /// Tick the recycler given its spec. Returns true if a conversion completed.
    pub fn tick(&mut self, spec: &RecyclerSpec) -> bool {
        if self.active {
            if self.ticks_remaining > 0 {
                self.ticks_remaining -= 1;
            }
            if self.ticks_remaining == 0 {
                self.output_buffer += spec.output_amount;
                self.active = false;
                return true;
            }
        } else if self.input_buffer >= spec.input_amount {
            self.input_buffer -= spec.input_amount;
            self.ticks_remaining = spec.cycle_ticks;
            self.active = true;
        }
        false
    }
}

impl Default for RecyclerState {
    fn default() -> Self {
        Self::new()
    }
}
