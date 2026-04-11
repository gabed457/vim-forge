use serde::{Deserialize, Serialize};

// ---------------------------------------------------------------------------
// Day/Night cycle
// ---------------------------------------------------------------------------

/// Total ticks in one day/night cycle.
pub const CYCLE_LENGTH: u32 = 600;

/// Day phase boundaries.
pub const DAY_START: u32 = 0;
pub const DAY_END: u32 = 350;
pub const DUSK_END: u32 = 400;
pub const NIGHT_END: u32 = 550;
// Dawn: 550 - 600 (wraps to 0)

/// Phase of the day.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum DayPhase {
    Dawn,
    Day,
    Dusk,
    Night,
}

/// Get the current phase of the day from the tick count.
pub fn day_phase(tick: u32) -> DayPhase {
    let t = tick % CYCLE_LENGTH;
    if t < DAY_END {
        DayPhase::Day
    } else if t < DUSK_END {
        DayPhase::Dusk
    } else if t < NIGHT_END {
        DayPhase::Night
    } else {
        DayPhase::Dawn
    }
}

/// Solar multiplier (0.0 at night, 1.0 during day, ramping at dawn/dusk).
/// Used to scale solar panel output.
pub fn solar_multiplier(tick: u32) -> f64 {
    let t = tick % CYCLE_LENGTH;
    match day_phase(tick) {
        DayPhase::Day => 1.0,
        DayPhase::Night => 0.0,
        DayPhase::Dusk => {
            // Ramp from 1.0 to 0.0 over dusk period (350..400).
            let progress = (t - DAY_END) as f64 / (DUSK_END - DAY_END) as f64;
            1.0 - progress
        }
        DayPhase::Dawn => {
            // Ramp from 0.0 to 1.0 over dawn period (550..600).
            let progress = (t - NIGHT_END) as f64 / (CYCLE_LENGTH - NIGHT_END) as f64;
            progress
        }
    }
}

/// RGB light multiplier for rendering.
///
/// Day: (1.0, 1.0, 1.0) — full brightness.
/// Night: (0.3, 0.3, 0.5) — blue-tinted darkness.
/// Dawn/Dusk: interpolated.
pub fn light_multiplier(tick: u32) -> (f64, f64, f64) {
    let solar = solar_multiplier(tick);

    // Day colors.
    let day_r = 1.0;
    let day_g = 1.0;
    let day_b = 1.0;

    // Night colors (blue-tinted).
    let night_r = 0.3;
    let night_g = 0.3;
    let night_b = 0.5;

    let r = night_r + (day_r - night_r) * solar;
    let g = night_g + (day_g - night_g) * solar;
    let b = night_b + (day_b - night_b) * solar;

    (r, g, b)
}

/// Day/Night cycle state for the game.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct DayNightCycle {
    /// Current tick within the cycle (0..CYCLE_LENGTH-1).
    pub tick: u32,
}

impl Default for DayNightCycle {
    fn default() -> Self {
        Self { tick: 0 }
    }
}

impl DayNightCycle {
    pub fn new() -> Self {
        Self::default()
    }

    /// Advance the cycle by one tick.
    pub fn advance(&mut self) {
        self.tick = (self.tick + 1) % CYCLE_LENGTH;
    }

    pub fn phase(&self) -> DayPhase {
        day_phase(self.tick)
    }

    pub fn solar(&self) -> f64 {
        solar_multiplier(self.tick)
    }

    pub fn light(&self) -> (f64, f64, f64) {
        light_multiplier(self.tick)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_day_phase() {
        assert_eq!(day_phase(0), DayPhase::Day);
        assert_eq!(day_phase(200), DayPhase::Day);
        assert_eq!(day_phase(349), DayPhase::Day);
        assert_eq!(day_phase(350), DayPhase::Dusk);
        assert_eq!(day_phase(399), DayPhase::Dusk);
        assert_eq!(day_phase(400), DayPhase::Night);
        assert_eq!(day_phase(549), DayPhase::Night);
        assert_eq!(day_phase(550), DayPhase::Dawn);
        assert_eq!(day_phase(599), DayPhase::Dawn);
        assert_eq!(day_phase(600), DayPhase::Day); // wraps
    }

    #[test]
    fn test_solar_multiplier() {
        assert_eq!(solar_multiplier(100), 1.0);
        assert_eq!(solar_multiplier(450), 0.0);
        // Dusk should be between 0 and 1.
        let dusk = solar_multiplier(375);
        assert!(dusk > 0.0 && dusk < 1.0);
        // Dawn should be between 0 and 1.
        let dawn = solar_multiplier(575);
        assert!(dawn > 0.0 && dawn < 1.0);
    }
}
