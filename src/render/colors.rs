/// Biome-specific color palette for tile rendering.
#[derive(Clone, Copy, Debug)]
pub struct BiomePalette {
    pub floor_bg: (u8, u8, u8),
    pub floor_fg: (u8, u8, u8),
    pub accent: (u8, u8, u8),
}

/// Region types that define the visual biome.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RegionType {
    Grassland,
    Desert,
    Tundra,
    Volcanic,
    OceanPlatform,
    Asteroid,
}

impl RegionType {
    pub fn palette(&self) -> BiomePalette {
        match self {
            RegionType::Grassland => BiomePalette {
                floor_bg: (12, 18, 12),
                floor_fg: (35, 50, 35),
                accent: (80, 140, 60),
            },
            RegionType::Desert => BiomePalette {
                floor_bg: (22, 18, 10),
                floor_fg: (55, 45, 30),
                accent: (180, 150, 80),
            },
            RegionType::Tundra => BiomePalette {
                floor_bg: (14, 16, 22),
                floor_fg: (40, 45, 60),
                accent: (140, 170, 220),
            },
            RegionType::Volcanic => BiomePalette {
                floor_bg: (20, 10, 8),
                floor_fg: (50, 25, 20),
                accent: (220, 80, 30),
            },
            RegionType::OceanPlatform => BiomePalette {
                floor_bg: (8, 14, 24),
                floor_fg: (25, 40, 65),
                accent: (60, 140, 200),
            },
            RegionType::Asteroid => BiomePalette {
                floor_bg: (6, 4, 10),
                floor_fg: (20, 15, 30),
                accent: (120, 80, 180),
            },
        }
    }
}

/// Returns the default (Grassland) biome palette.
pub fn default_palette() -> BiomePalette {
    RegionType::Grassland.palette()
}

/// Day phase for rendering tints.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DayPhase {
    Dawn,
    Day,
    Dusk,
    Night,
}

/// Day/night cycle length in ticks (matches environment::day_night if wired).
const CYCLE_LENGTH: u32 = 600;

/// Determine day phase from tick count.
pub fn day_phase_from_tick(tick: u32) -> DayPhase {
    let t = tick % CYCLE_LENGTH;
    if t < 350 {
        DayPhase::Day
    } else if t < 400 {
        DayPhase::Dusk
    } else if t < 550 {
        DayPhase::Night
    } else {
        DayPhase::Dawn
    }
}

/// Day/night color multiplier tuple for a given phase.
fn phase_tint(phase: DayPhase) -> (f64, f64, f64) {
    match phase {
        DayPhase::Dawn => (1.1, 1.0, 0.8),
        DayPhase::Day => (1.0, 1.0, 1.0),
        DayPhase::Dusk => (1.1, 0.9, 0.7),
        DayPhase::Night => (0.6, 0.7, 0.9),
    }
}

/// Apply the day/night color tint to a background color.
/// Machine foreground colors should NOT be passed through this.
pub fn apply_day_night(color: (u8, u8, u8), day_tick: u32) -> (u8, u8, u8) {
    let phase = day_phase_from_tick(day_tick);
    let (mr, mg, mb) = phase_tint(phase);
    (
        (color.0 as f64 * mr).min(255.0) as u8,
        (color.1 as f64 * mg).min(255.0) as u8,
        (color.2 as f64 * mb).min(255.0) as u8,
    )
}

/// Apply pollution tint to a background color.
/// Shifts toward yellow-green-brown proportional to pollution level (0.0..1.0).
pub fn apply_pollution_tint(color: (u8, u8, u8), pollution: f64) -> (u8, u8, u8) {
    let p = pollution.clamp(0.0, 1.0);
    if p < 0.01 {
        return color;
    }
    // Pollution tint target: murky yellow-brown (80, 70, 30)
    let tr = 80.0;
    let tg = 70.0;
    let tb = 30.0;
    let blend = p * 0.4; // max 40% tint at full pollution
    (
        (color.0 as f64 * (1.0 - blend) + tr * blend).min(255.0) as u8,
        (color.1 as f64 * (1.0 - blend) + tg * blend).min(255.0) as u8,
        (color.2 as f64 * (1.0 - blend) + tb * blend).min(255.0) as u8,
    )
}

/// Dim a foreground color to the given fraction (for idle state, etc.).
pub fn dim_color(color: (u8, u8, u8), factor: f64) -> (u8, u8, u8) {
    (
        (color.0 as f64 * factor).min(255.0) as u8,
        (color.1 as f64 * factor).min(255.0) as u8,
        (color.2 as f64 * factor).min(255.0) as u8,
    )
}

/// Blend a trail color into a background at a given intensity (0.0..1.0).
pub fn blend_trail(bg: (u8, u8, u8), trail_color: (u8, u8, u8), intensity: f64) -> (u8, u8, u8) {
    let t = intensity.clamp(0.0, 1.0);
    (
        (bg.0 as f64 * (1.0 - t) + trail_color.0 as f64 * t).min(255.0) as u8,
        (bg.1 as f64 * (1.0 - t) + trail_color.1 as f64 * t).min(255.0) as u8,
        (bg.2 as f64 * (1.0 - t) + trail_color.2 as f64 * t).min(255.0) as u8,
    )
}
