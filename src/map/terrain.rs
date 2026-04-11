use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Terrain {
    Ground,
    Water,
    DeepWater,
    Forest,
    Mountain,
    Desert,
    Swamp,
    Lava,
    Ice,
    RadioactiveZone,
}

impl Terrain {
    pub fn is_buildable(&self) -> bool {
        matches!(self, Terrain::Ground | Terrain::Desert | Terrain::Ice)
    }

    pub fn build_cost_multiplier(&self) -> f64 {
        match self {
            Terrain::Ground => 1.0,
            Terrain::Desert => 1.5,
            Terrain::Ice => 2.0,
            Terrain::Forest => 3.0,
            Terrain::Swamp => 4.0,
            _ => f64::INFINITY,
        }
    }

    pub fn glyph(&self) -> char {
        match self {
            Terrain::Ground => '.',
            Terrain::Water => '~',
            Terrain::DeepWater => '\u{2248}', // approximately equal
            Terrain::Forest => '\u{2663}',    // club suit (tree)
            Terrain::Mountain => '\u{25B2}',  // triangle up
            Terrain::Desert => '\u{2237}',    // proportion
            Terrain::Swamp => '\u{2235}',     // because
            Terrain::Lava => '\u{2593}',      // dark shade
            Terrain::Ice => '\u{2022}',       // bullet
            Terrain::RadioactiveZone => '\u{2622}', // radioactive
        }
    }

    pub fn fg_color(&self) -> (u8, u8, u8) {
        match self {
            Terrain::Ground => (60, 60, 60),
            Terrain::Water => (60, 120, 200),
            Terrain::DeepWater => (30, 60, 150),
            Terrain::Forest => (30, 120, 30),
            Terrain::Mountain => (140, 140, 140),
            Terrain::Desert => (200, 180, 100),
            Terrain::Swamp => (80, 100, 60),
            Terrain::Lava => (255, 80, 20),
            Terrain::Ice => (180, 220, 255),
            Terrain::RadioactiveZone => (80, 255, 80),
        }
    }

    pub fn bg_color(&self) -> Option<(u8, u8, u8)> {
        match self {
            Terrain::Lava => Some((80, 20, 0)),
            Terrain::RadioactiveZone => Some((10, 30, 10)),
            _ => None,
        }
    }
}

impl Default for Terrain {
    fn default() -> Self {
        Terrain::Ground
    }
}
