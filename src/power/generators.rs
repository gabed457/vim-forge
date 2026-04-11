use serde::{Deserialize, Serialize};

use crate::resources::{EntityType, Resource};

// ---------------------------------------------------------------------------
// Generator spec
// ---------------------------------------------------------------------------

/// Static specification for a generator type.
#[derive(Clone, Debug)]
pub struct GeneratorSpec {
    pub entity_type: EntityType,
    /// Fuel consumed per tick (None for solar/wind).
    pub fuel_input: Option<(Resource, u32)>,
    /// Extra input required (e.g. Coolant for nuclear).
    pub extra_input: Option<(Resource, u32)>,
    /// MW output under normal conditions.
    pub base_mw: f64,
    /// Waste produced per tick (if any).
    pub waste_output: Option<(Resource, u32)>,
    /// Whether output varies (solar/wind).
    pub variable: bool,
}

/// Get the spec for a generator entity type.
pub fn generator_spec(entity_type: EntityType) -> Option<GeneratorSpec> {
    match entity_type {
        EntityType::CoalGenerator => Some(GeneratorSpec {
            entity_type,
            fuel_input: Some((Resource::Coal, 1)),      // 1 Coal per 5 ticks
            extra_input: None,
            base_mw: 5.0,
            waste_output: Some((Resource::FlyAsh, 1)),
            variable: false,
        }),
        EntityType::GasGenerator => Some(GeneratorSpec {
            entity_type,
            fuel_input: Some((Resource::NaturalGas, 1)),
            extra_input: None,
            base_mw: 8.0,
            waste_output: Some((Resource::CO2, 1)),
            variable: false,
        }),
        EntityType::SolarArray => Some(GeneratorSpec {
            entity_type,
            fuel_input: None,
            extra_input: None,
            base_mw: 3.0, // multiplied by solar_multiplier from day/night
            waste_output: None,
            variable: true,
        }),
        EntityType::WindTurbine => Some(GeneratorSpec {
            entity_type,
            fuel_input: None,
            extra_input: None,
            base_mw: 10.0, // actual output: random 5-15 MW
            waste_output: None,
            variable: true,
        }),
        EntityType::NuclearReactor => Some(GeneratorSpec {
            entity_type,
            fuel_input: Some((Resource::NuclearFuelRod, 1)),
            extra_input: Some((Resource::Coolant, 1)),
            base_mw: 50.0,
            waste_output: Some((Resource::DepletedUranium, 1)),
            variable: false,
        }),
        EntityType::FusionReactor => Some(GeneratorSpec {
            entity_type,
            fuel_input: Some((Resource::FusionCore, 1)),
            extra_input: None,
            base_mw: 100.0,
            waste_output: None,
            variable: false,
        }),
        EntityType::GeothermalPlant => Some(GeneratorSpec {
            entity_type,
            fuel_input: Some((Resource::GeothermalSteam, 1)),
            extra_input: None,
            base_mw: 12.0,
            waste_output: None,
            variable: false,
        }),
        _ => None,
    }
}

// ---------------------------------------------------------------------------
// Generator runtime state
// ---------------------------------------------------------------------------

/// Runtime state of a single generator instance.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct GeneratorState {
    pub entity_type: EntityType,
    pub pos: (usize, usize),
    /// Fuel buffer — how many units of fuel are stocked.
    pub fuel_buffer: u32,
    /// Extra input buffer (e.g. coolant).
    pub extra_buffer: u32,
    /// Waste output buffer.
    pub waste_buffer: u32,
    /// Ticks since last fuel consumption.
    pub fuel_tick_counter: u32,
    /// Whether the generator is currently producing power.
    pub active: bool,
    /// Current MW output this tick.
    pub current_mw: f64,
}

impl GeneratorState {
    pub fn new(entity_type: EntityType, pos: (usize, usize)) -> Self {
        Self {
            entity_type,
            pos,
            fuel_buffer: 0,
            extra_buffer: 0,
            waste_buffer: 0,
            fuel_tick_counter: 0,
            active: false,
            current_mw: 0.0,
        }
    }

    /// Tick this generator. Returns the MW produced this tick.
    ///
    /// `solar_mult` — 0.0 to 1.0 from day/night cycle (only affects solar).
    /// `wind_mw` — randomised MW for wind turbines (5-15 range).
    /// `fuel_consumption_interval` — ticks between fuel consumption (e.g. 5 for coal).
    pub fn tick(
        &mut self,
        solar_mult: f64,
        wind_mw: f64,
        fuel_consumption_interval: u32,
    ) -> f64 {
        let spec = match generator_spec(self.entity_type) {
            Some(s) => s,
            None => {
                self.active = false;
                self.current_mw = 0.0;
                return 0.0;
            }
        };

        // Check fuel availability.
        if spec.fuel_input.is_some() && self.fuel_buffer == 0 {
            self.active = false;
            self.current_mw = 0.0;
            return 0.0;
        }

        // Check extra input (e.g. coolant for nuclear).
        if spec.extra_input.is_some() && self.extra_buffer == 0 {
            self.active = false;
            self.current_mw = 0.0;
            return 0.0;
        }

        // Consume fuel on interval.
        if spec.fuel_input.is_some() {
            self.fuel_tick_counter += 1;
            if self.fuel_tick_counter >= fuel_consumption_interval {
                self.fuel_tick_counter = 0;
                let (_, amount) = spec.fuel_input.unwrap();
                if self.fuel_buffer >= amount {
                    self.fuel_buffer -= amount;
                } else {
                    self.active = false;
                    self.current_mw = 0.0;
                    return 0.0;
                }
                // Consume extra input at the same time.
                if let Some((_, extra_amount)) = spec.extra_input {
                    if self.extra_buffer >= extra_amount {
                        self.extra_buffer -= extra_amount;
                    }
                }
                // Produce waste.
                if spec.waste_output.is_some() {
                    let (_, waste_amount) = spec.waste_output.unwrap();
                    self.waste_buffer += waste_amount;
                }
            }
        }

        // Compute output.
        let mw = match self.entity_type {
            EntityType::SolarArray => spec.base_mw * solar_mult,
            EntityType::WindTurbine => wind_mw.clamp(5.0, 15.0),
            _ => spec.base_mw,
        };

        self.active = true;
        self.current_mw = mw;
        mw
    }
}

/// Default fuel consumption interval for a generator type (ticks per fuel unit).
pub fn fuel_interval(entity_type: EntityType) -> u32 {
    match entity_type {
        EntityType::CoalGenerator => 5,
        EntityType::GasGenerator => 3,
        EntityType::NuclearReactor => 100,
        EntityType::FusionReactor => 200,
        EntityType::GeothermalPlant => 10,
        _ => 1,
    }
}
