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

/// Keyframes for the day/night tint curve: (tick, (r_mult, g_mult, b_mult)).
///
/// The tint is linearly interpolated between successive keyframes, so dawn
/// and dusk are smooth color lerps rather than hard switches. The last
/// keyframe wraps back to the first (tick 600 == tick 0).
const TINT_KEYFRAMES: [(u32, (f64, f64, f64)); 7] = [
    (0, (1.0, 1.0, 1.0)),        // morning: full daylight
    (240, (1.0, 1.0, 1.0)),      // afternoon: still full daylight
    (340, (1.10, 0.88, 0.68)),   // golden hour: warm amber wash
    (400, (0.78, 0.68, 0.82)),   // dusk: violet fade
    (450, (0.52, 0.60, 0.92)),   // night: cool blue moonlight
    (530, (0.52, 0.60, 0.92)),   // deep night holds
    (575, (1.02, 0.86, 0.74)),   // dawn: rosy warm-up
];

/// Smoothly interpolated day/night color multiplier for a given tick.
pub fn day_night_multiplier(day_tick: u32) -> (f64, f64, f64) {
    let t = day_tick % CYCLE_LENGTH;
    let n = TINT_KEYFRAMES.len();
    for i in 0..n {
        let (t0, c0) = TINT_KEYFRAMES[i];
        // Next keyframe, wrapping to the first at tick CYCLE_LENGTH.
        let (t1, c1) = if i + 1 < n {
            TINT_KEYFRAMES[i + 1]
        } else {
            (CYCLE_LENGTH, TINT_KEYFRAMES[0].1)
        };
        if t >= t0 && t < t1 {
            let span = (t1 - t0).max(1) as f64;
            let f = (t - t0) as f64 / span;
            return (
                c0.0 + (c1.0 - c0.0) * f,
                c0.1 + (c1.1 - c0.1) * f,
                c0.2 + (c1.2 - c0.2) * f,
            );
        }
    }
    (1.0, 1.0, 1.0)
}

/// Apply the day/night color tint to a background color (full strength).
pub fn apply_day_night(color: (u8, u8, u8), day_tick: u32) -> (u8, u8, u8) {
    let (mr, mg, mb) = day_night_multiplier(day_tick);
    (
        (color.0 as f64 * mr).min(255.0) as u8,
        (color.1 as f64 * mg).min(255.0) as u8,
        (color.2 as f64 * mb).min(255.0) as u8,
    )
}

/// Apply a gentler day/night tint suited for foreground glyphs: the
/// multiplier is blended halfway toward neutral so machines stay readable
/// at night while still picking up the ambient hue.
pub fn apply_day_night_fg(color: (u8, u8, u8), day_tick: u32) -> (u8, u8, u8) {
    let (mr, mg, mb) = day_night_multiplier(day_tick);
    let soften = |m: f64| m + (1.0 - m) * 0.55;
    (
        (color.0 as f64 * soften(mr)).min(255.0) as u8,
        (color.1 as f64 * soften(mg)).min(255.0) as u8,
        (color.2 as f64 * soften(mb)).min(255.0) as u8,
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
