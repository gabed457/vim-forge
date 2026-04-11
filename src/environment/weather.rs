use serde::{Deserialize, Serialize};

// ---------------------------------------------------------------------------
// Weather events
// ---------------------------------------------------------------------------

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum WeatherEventType {
    /// Solar -50%, outdoor machines slow.
    Rain,
    /// Solar -100%, wind +50%, chance of pole damage.
    Storm,
    /// Water extraction -30%.
    Drought,
    /// Random machine breaks, pipe ruptures.
    Earthquake,
    /// Destroys 5x5 area, creates RareEarthOre deposit.
    Meteor,
    /// Disables all electronics for duration.
    SolarFlare,
    /// All market prices crash.
    MarketCrash,
    /// Demand for all resources surges (2x value).
    DemandSurge,
}

/// An active weather event.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct WeatherEvent {
    pub event_type: WeatherEventType,
    /// Tick when the event started.
    pub start_tick: u64,
    /// Duration in ticks.
    pub duration: u32,
    /// Ticks remaining.
    pub remaining: u32,
}

impl WeatherEvent {
    pub fn new(event_type: WeatherEventType, start_tick: u64) -> Self {
        let duration = event_duration(&event_type);
        Self {
            event_type,
            start_tick,
            duration,
            remaining: duration,
        }
    }

    pub fn is_active(&self) -> bool {
        self.remaining > 0
    }

    pub fn tick(&mut self) {
        self.remaining = self.remaining.saturating_sub(1);
    }
}

/// Default duration for each event type (in ticks).
fn event_duration(event_type: &WeatherEventType) -> u32 {
    match event_type {
        WeatherEventType::Rain => 120,
        WeatherEventType::Storm => 60,
        WeatherEventType::Drought => 300,
        WeatherEventType::Earthquake => 1,   // instant
        WeatherEventType::Meteor => 1,        // instant
        WeatherEventType::SolarFlare => 30,
        WeatherEventType::MarketCrash => 200,
        WeatherEventType::DemandSurge => 200,
    }
}

// ---------------------------------------------------------------------------
// Weather effects
// ---------------------------------------------------------------------------

/// Solar multiplier modification from weather.
pub fn weather_solar_modifier(event_type: &WeatherEventType) -> f64 {
    match event_type {
        WeatherEventType::Rain => 0.5,      // -50%
        WeatherEventType::Storm => 0.0,     // -100%
        _ => 1.0,
    }
}

/// Wind multiplier modification from weather.
pub fn weather_wind_modifier(event_type: &WeatherEventType) -> f64 {
    match event_type {
        WeatherEventType::Storm => 1.5,     // +50%
        _ => 1.0,
    }
}

/// Water extraction multiplier from weather.
pub fn weather_water_modifier(event_type: &WeatherEventType) -> f64 {
    match event_type {
        WeatherEventType::Drought => 0.7,   // -30%
        _ => 1.0,
    }
}

/// Market price multiplier from weather.
pub fn weather_market_modifier(event_type: &WeatherEventType) -> f64 {
    match event_type {
        WeatherEventType::MarketCrash => 0.5,
        WeatherEventType::DemandSurge => 2.0,
        _ => 1.0,
    }
}

// ---------------------------------------------------------------------------
// Weather check
// ---------------------------------------------------------------------------

/// Base probabilities per tick for each event type.
/// Probabilities increase with scaling level.
fn base_probability(event_type: &WeatherEventType) -> f64 {
    match event_type {
        WeatherEventType::Rain => 0.001,
        WeatherEventType::Storm => 0.0005,
        WeatherEventType::Drought => 0.0003,
        WeatherEventType::Earthquake => 0.0001,
        WeatherEventType::Meteor => 0.00005,
        WeatherEventType::SolarFlare => 0.0002,
        WeatherEventType::MarketCrash => 0.0002,
        WeatherEventType::DemandSurge => 0.0003,
    }
}

/// Check if a weather event should trigger this tick.
///
/// `rand_value` — a random f64 in [0, 1) provided by the caller.
/// `scaling_level` — current difficulty level (higher = more events).
///
/// Returns the event type if one triggers. Only one event at a time is expected;
/// the caller should skip if an event is already active.
pub fn check_weather(
    rand_value: f64,
    scaling_level: u32,
) -> Option<WeatherEventType> {
    let scaling_mult = 1.0 + scaling_level as f64 * 0.01;

    let events = [
        WeatherEventType::Rain,
        WeatherEventType::Storm,
        WeatherEventType::Drought,
        WeatherEventType::Earthquake,
        WeatherEventType::Meteor,
        WeatherEventType::SolarFlare,
        WeatherEventType::MarketCrash,
        WeatherEventType::DemandSurge,
    ];

    let mut cumulative = 0.0;
    for event in &events {
        cumulative += base_probability(event) * scaling_mult;
        if rand_value < cumulative {
            return Some(event.clone());
        }
    }

    None
}
