use ratatui::style::{Color, Modifier, Style};

use crate::ecs::components::Processing;
use crate::render::colors::dim_color;
use crate::resources::{EntityType, Facing, Resource};

/// Machine state for per-state styling.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MachineState {
    Idle,
    Processing,
    Blocked,
    Broken,
}

// ---------------------------------------------------------------------------
// BuildingArt — multi-character ASCII art per building
// ---------------------------------------------------------------------------

/// ASCII art definition for a building.
/// Tiles are stored in row-major order: `tiles[row * width + col]`.
/// Each tile has exactly 2 terminal columns: [col0, col1].
pub struct BuildingArt {
    pub width: usize,
    pub height: usize,
    pub tiles: &'static [[char; 2]],
}

impl BuildingArt {
    /// Look up the art for a specific (row, col) within this building.
    /// Returns a default dot pair if out of bounds.
    pub fn tile_at(&self, row: usize, col: usize) -> [char; 2] {
        if row < self.height && col < self.width {
            self.tiles[row * self.width + col]
        } else {
            ['\u{00B7}', '\u{00B7}']
        }
    }
}

/// Returns the ASCII art definition for an entity type (in Right-facing orientation).
pub fn building_art(entity_type: EntityType) -> BuildingArt {
    use EntityType::*;
    match entity_type {
        // ══════════════════════════════════════════════════════════════════
        // Extractors (3×2) — output port on right edge, row 1
        // ══════════════════════════════════════════════════════════════════
        OreDeposit => BuildingArt { width: 3, height: 2, tiles: &[
            // Row 0: ╔═ ══ ═╗
            ['╔','═'], ['═','═'], ['═','╗'],
            // Row 1: ║⊞ ·· ·▸
            ['║','⊞'], ['·','·'], ['·','▸'],
        ]},
        CopperDeposit => BuildingArt { width: 3, height: 2, tiles: &[
            ['╔','═'], ['═','═'], ['═','╗'],
            ['║','⊞'], ['·','☼'], ['·','▸'],
        ]},
        CoalDeposit => BuildingArt { width: 3, height: 2, tiles: &[
            ['╔','═'], ['═','═'], ['═','╗'],
            ['║','▓'], ['▓','·'], ['·','▸'],
        ]},
        StoneQuarry => BuildingArt { width: 3, height: 2, tiles: &[
            ['╔','═'], ['═','═'], ['═','╗'],
            ['║','░'], ['▒','·'], ['·','▸'],
        ]},
        UraniumMine => BuildingArt { width: 3, height: 2, tiles: &[
            ['╔','═'], ['═','═'], ['═','╗'],
            ['║','☢'], ['·','·'], ['·','▸'],
        ]},
        SandExtractor => BuildingArt { width: 3, height: 2, tiles: &[
            ['╔','═'], ['═','═'], ['═','╗'],
            ['║','░'], ['·','·'], ['·','▸'],
        ]},
        SulfurMine => BuildingArt { width: 3, height: 2, tiles: &[
            ['╔','═'], ['═','═'], ['═','╗'],
            ['║','§'], ['·','·'], ['·','▸'],
        ]},
        BauxiteMine => BuildingArt { width: 3, height: 2, tiles: &[
            ['╔','═'], ['═','═'], ['═','╗'],
            ['║','▤'], ['·','·'], ['·','▸'],
        ]},
        LithiumExtractor => BuildingArt { width: 3, height: 2, tiles: &[
            ['╔','═'], ['═','═'], ['═','╗'],
            ['║','⊕'], ['·','·'], ['·','▸'],
        ]},
        RareEarthExtractor => BuildingArt { width: 3, height: 2, tiles: &[
            ['╔','═'], ['═','═'], ['═','╗'],
            ['║','◇'], ['·','·'], ['·','▸'],
        ]},
        OilWell => BuildingArt { width: 3, height: 2, tiles: &[
            ['╔','═'], ['═','═'], ['═','╗'],
            ['║','⊼'], ['~','·'], ['·','▸'],
        ]},
        WaterPump => BuildingArt { width: 3, height: 2, tiles: &[
            ['╔','═'], ['═','═'], ['═','╗'],
            ['║','≈'], ['↑','·'], ['·','▸'],
        ]},
        GasExtractor => BuildingArt { width: 3, height: 2, tiles: &[
            ['╔','═'], ['═','═'], ['═','╗'],
            ['║','◎'], ['°','·'], ['·','▸'],
        ]},
        BiomassHarvester => BuildingArt { width: 3, height: 2, tiles: &[
            ['╔','═'], ['═','═'], ['═','╗'],
            ['║','⌠'], ['¤','·'], ['·','▸'],
        ]},
        GeothermalTap => BuildingArt { width: 3, height: 2, tiles: &[
            ['╔','═'], ['═','═'], ['═','╗'],
            ['║','▽'], ['△','·'], ['·','▸'],
        ]},

        // ══════════════════════════════════════════════════════════════════
        // 1-input processors (3×3) — input left-center, output right-center,
        // waste bottom-center
        // ══════════════════════════════════════════════════════════════════
        Smelter => BuildingArt { width: 3, height: 3, tiles: &[
            // Row 0: ╔═ ▓▓ ═╗
            ['╔','═'], ['▓','▓'], ['═','╗'],
            // Row 1: ◂▓ ▓▓ ▓▸   (input left, fire interior, output right)
            ['◂','▓'], ['▓','▓'], ['▓','▸'],
            // Row 2: ╚═ ═▾ ═╝   (waste port bottom center)
            ['╚','═'], ['═','▾'], ['═','╝'],
        ]},
        Kiln => BuildingArt { width: 3, height: 3, tiles: &[
            ['╔','═'], ['▒','▒'], ['═','╗'],
            ['◂','▒'], ['⊓','▥'], ['▒','▸'],
            ['╚','═'], ['═','▾'], ['═','╝'],
        ]},
        Press => BuildingArt { width: 3, height: 3, tiles: &[
            ['╔','═'], ['⊓','⊓'], ['═','╗'],
            ['◂','·'], ['⊓','⊔'], ['·','▸'],
            ['╚','═'], ['═','▾'], ['═','╝'],
        ]},
        WireMill => BuildingArt { width: 3, height: 3, tiles: &[
            ['╔','═'], ['∿','∿'], ['═','╗'],
            ['◂','·'], ['⊞','∿'], ['·','▸'],
            ['╚','═'], ['═','▾'], ['═','╝'],
        ]},
        PlateMachine => BuildingArt { width: 3, height: 3, tiles: &[
            ['╔','═'], ['▤','▤'], ['═','╗'],
            ['◂','·'], ['⊞','▤'], ['·','▸'],
            ['╚','═'], ['═','▾'], ['═','╝'],
        ]},
        RubberVulcanizer => BuildingArt { width: 3, height: 3, tiles: &[
            ['╔','═'], ['▓','▓'], ['═','╗'],
            ['◂','·'], ['▓','▤'], ['·','▸'],
            ['╚','═'], ['═','▾'], ['═','╝'],
        ]},
        PlasticMolder => BuildingArt { width: 3, height: 3, tiles: &[
            ['╔','═'], ['◻','◻'], ['═','╗'],
            ['◂','·'], ['◻','▤'], ['·','▸'],
            ['╚','═'], ['═','▾'], ['═','╝'],
        ]},
        Electrolyzer => BuildingArt { width: 3, height: 3, tiles: &[
            ['╔','═'], ['⊕','∿'], ['═','╗'],
            ['◂','·'], ['⊕','∿'], ['·','▸'],
            ['╚','═'], ['═','▾'], ['═','╝'],
        ]},
        Caster => BuildingArt { width: 3, height: 3, tiles: &[
            ['╔','═'], ['▓','▓'], ['═','╗'],
            ['◂','·'], ['◮','▤'], ['·','▸'],
            ['╚','═'], ['═','▾'], ['═','╝'],
        ]},
        CokeFurnace => BuildingArt { width: 3, height: 3, tiles: &[
            ['╔','═'], ['▓','▓'], ['═','╗'],
            ['◂','·'], ['⌂','▓'], ['·','▸'],
            ['╚','═'], ['═','▾'], ['═','╝'],
        ]},
        Gasifier => BuildingArt { width: 3, height: 3, tiles: &[
            ['╔','═'], ['▓','░'], ['═','╗'],
            ['◂','·'], ['▓','░'], ['·','▸'],
            ['╚','═'], ['═','▾'], ['═','╝'],
        ]},
        Boiler => BuildingArt { width: 3, height: 3, tiles: &[
            ['╔','═'], ['≈','≈'], ['═','╗'],
            ['◂','·'], ['◮','≈'], ['·','▸'],
            ['╚','═'], ['═','▾'], ['═','╝'],
        ]},
        WaferCutter => BuildingArt { width: 3, height: 3, tiles: &[
            ['╔','═'], ['⊗','·'], ['═','╗'],
            ['◂','·'], ['⊗','▤'], ['·','▸'],
            ['╚','═'], ['═','▾'], ['═','╝'],
        ]},

        // ══════════════════════════════════════════════════════════════════
        // 2-input processors (3×4) — 2 inputs left rows 1-2, output right
        // row 2, waste bottom-center
        // ══════════════════════════════════════════════════════════════════
        Assembler => BuildingArt { width: 3, height: 4, tiles: &[
            // Row 0: ╔═ ══ ═╗
            ['╔','═'], ['═','═'], ['═','╗'],
            // Row 1: ◂· ⊛· ║║   (input 0)
            ['◂','·'], ['⊛','·'], ['║','║'],
            // Row 2: ◂· ·⊛ ·▸   (input 1, output)
            ['◂','·'], ['·','⊛'], ['·','▸'],
            // Row 3: ╚═ ═▾ ═╝   (waste)
            ['╚','═'], ['═','▾'], ['═','╝'],
        ]},
        Mixer => BuildingArt { width: 3, height: 4, tiles: &[
            ['╔','═'], ['═','═'], ['═','╗'],
            ['◂','·'], ['◎','◦'], ['║','║'],
            ['◂','·'], ['◦','◎'], ['·','▸'],
            ['╚','═'], ['═','▾'], ['═','╝'],
        ]},
        ChemicalPlant => BuildingArt { width: 3, height: 4, tiles: &[
            ['╔','═'], ['═','═'], ['═','╗'],
            ['◂','·'], ['◎','⊕'], ['║','║'],
            ['◂','·'], ['⊕','◎'], ['·','▸'],
            ['╚','═'], ['═','▾'], ['═','╝'],
        ]},
        CircuitFabricator => BuildingArt { width: 3, height: 4, tiles: &[
            ['╔','═'], ['═','═'], ['═','╗'],
            ['◂','·'], ['⊞','·'], ['║','║'],
            ['◂','·'], ['·','⊞'], ['·','▸'],
            ['╚','═'], ['═','▾'], ['═','╝'],
        ]},
        MotorAssembly => BuildingArt { width: 3, height: 4, tiles: &[
            ['╔','═'], ['═','═'], ['═','╗'],
            ['◂','·'], ['⊙','·'], ['║','║'],
            ['◂','·'], ['·','⊙'], ['·','▸'],
            ['╚','═'], ['═','▾'], ['═','╝'],
        ]},
        CrushingMill => BuildingArt { width: 3, height: 4, tiles: &[
            ['╔','═'], ['═','═'], ['═','╗'],
            ['◂','·'], ['⊗','·'], ['║','║'],
            ['◂','·'], ['·','⊗'], ['·','▸'],
            ['╚','═'], ['═','▾'], ['═','╝'],
        ]},

        // ══════════════════════════════════════════════════════════════════
        // Tier-2 processors (4×4) — 2 inputs left rows 1-2, output right
        // row 2
        // ══════════════════════════════════════════════════════════════════
        AdvancedAssembler => BuildingArt { width: 4, height: 4, tiles: &[
            // Row 0
            ['╔','═'], ['═','═'], ['═','═'], ['═','╗'],
            // Row 1: input 0
            ['◂','·'], ['⊛','·'], ['·','·'], ['·','║'],
            // Row 2: input 1, output
            ['◂','·'], ['·','⊛'], ['·','·'], ['·','▸'],
            // Row 3
            ['╚','═'], ['═','═'], ['═','═'], ['═','╝'],
        ]},
        Refinery => BuildingArt { width: 4, height: 4, tiles: &[
            ['╔','═'], ['═','═'], ['═','═'], ['═','╗'],
            ['◂','·'], ['╥','║'], ['║','╥'], ['·','║'],
            ['◂','·'], ['╨','╬'], ['╬','╨'], ['·','▸'],
            ['╚','═'], ['═','═'], ['═','═'], ['═','╝'],
        ]},
        CrackingTower => BuildingArt { width: 4, height: 4, tiles: &[
            ['╔','═'], ['═','═'], ['═','═'], ['═','╗'],
            ['◂','·'], ['┃','·'], ['·','┃'], ['·','║'],
            ['◂','·'], ['╋','·'], ['·','╋'], ['·','▸'],
            ['╚','═'], ['═','═'], ['═','═'], ['═','╝'],
        ]},
        Cleanroom => BuildingArt { width: 4, height: 4, tiles: &[
            ['╔','═'], ['═','═'], ['═','═'], ['═','╗'],
            ['◂','·'], ['◇','·'], ['·','◇'], ['·','║'],
            ['◂','·'], ['·','◇'], ['◇','·'], ['·','▸'],
            ['╚','═'], ['═','═'], ['═','═'], ['═','╝'],
        ]},
        EnrichmentCascade => BuildingArt { width: 4, height: 4, tiles: &[
            ['╔','═'], ['═','═'], ['═','═'], ['═','╗'],
            ['◂','·'], ['◉','·'], ['·','◉'], ['·','║'],
            ['◂','·'], ['·','◉'], ['◉','·'], ['·','▸'],
            ['╚','═'], ['═','═'], ['═','═'], ['═','╝'],
        ]},
        CoolantProcessor => BuildingArt { width: 4, height: 4, tiles: &[
            ['╔','═'], ['═','═'], ['═','═'], ['═','╗'],
            ['◂','·'], ['◈','·'], ['·','◈'], ['·','║'],
            ['◂','·'], ['·','◈'], ['◈','·'], ['·','▸'],
            ['╚','═'], ['═','═'], ['═','═'], ['═','╝'],
        ]},

        // ══════════════════════════════════════════════════════════════════
        // Tier-3 processors (5×5) — 3 inputs left rows 1-3, output right
        // row 2, waste bottom-center
        // ══════════════════════════════════════════════════════════════════
        PrecisionAssembler => BuildingArt { width: 5, height: 5, tiles: &[
            // Row 0
            ['╔','═'], ['═','═'], ['═','═'], ['═','═'], ['═','╗'],
            // Row 1: input 0
            ['◂','·'], ['⊛','·'], ['·','·'], ['·','·'], ['·','║'],
            // Row 2: input 1, output
            ['◂','·'], ['·','⊛'], ['⊛','·'], ['·','·'], ['·','▸'],
            // Row 3: input 2
            ['◂','·'], ['·','·'], ['·','⊛'], ['·','·'], ['·','║'],
            // Row 4: waste bottom-center
            ['╚','═'], ['═','═'], ['═','▾'], ['═','═'], ['═','╝'],
        ]},
        QuantumLab => BuildingArt { width: 5, height: 5, tiles: &[
            ['╔','═'], ['═','═'], ['═','═'], ['═','═'], ['═','╗'],
            ['◂','·'], ['Ψ','·'], ['·','≋'], ['·','·'], ['·','║'],
            ['◂','·'], ['·','⊹'], ['⊹','·'], ['·','·'], ['·','▸'],
            ['◂','·'], ['·','·'], ['≋','Ψ'], ['·','·'], ['·','║'],
            ['╚','═'], ['═','═'], ['═','▾'], ['═','═'], ['═','╝'],
        ]},
        RocketAssembly => BuildingArt { width: 5, height: 5, tiles: &[
            ['╔','═'], ['═','═'], ['═','═'], ['═','═'], ['═','╗'],
            ['◂','·'], ['⊡','·'], ['·','·'], ['·','·'], ['·','║'],
            ['◂','·'], ['·','⊡'], ['⊡','·'], ['·','·'], ['·','▸'],
            ['◂','·'], ['·','·'], ['·','⊡'], ['·','·'], ['·','║'],
            ['╚','═'], ['═','═'], ['═','▾'], ['═','═'], ['═','╝'],
        ]},

        // ══════════════════════════════════════════════════════════════════
        // Tier-4 processors — Megassembler (6×6), SingularityLab (6×7)
        // ══════════════════════════════════════════════════════════════════
        Megassembler => BuildingArt { width: 6, height: 6, tiles: &[
            // Row 0
            ['╔','═'], ['═','═'], ['═','═'], ['═','═'], ['═','═'], ['═','╗'],
            // Row 1: input 0
            ['◂','·'], ['⊡','·'], ['·','·'], ['·','·'], ['·','·'], ['·','║'],
            // Row 2: input 1
            ['◂','·'], ['·','⊡'], ['·','·'], ['·','·'], ['·','·'], ['·','║'],
            // Row 3: input 2, output
            ['◂','·'], ['·','·'], ['⊡','⊡'], ['⊡','·'], ['·','·'], ['·','▸'],
            // Row 4: input 3
            ['◂','·'], ['·','·'], ['·','·'], ['·','⊡'], ['·','·'], ['·','║'],
            // Row 5
            ['╚','═'], ['═','═'], ['═','═'], ['═','═'], ['═','═'], ['═','╝'],
        ]},
        SingularityLab => BuildingArt { width: 6, height: 7, tiles: &[
            // Row 0
            ['╔','═'], ['═','═'], ['═','═'], ['═','═'], ['═','═'], ['═','╗'],
            // Row 1: input 0
            ['◂','·'], ['Ω','·'], ['·','·'], ['·','·'], ['·','·'], ['·','║'],
            // Row 2: input 1
            ['◂','·'], ['·','Ω'], ['·','·'], ['·','·'], ['·','·'], ['·','║'],
            // Row 3: input 2, output
            ['◂','·'], ['·','·'], ['Ω','·'], ['·','Ω'], ['·','·'], ['·','▸'],
            // Row 4: input 3
            ['◂','·'], ['·','·'], ['·','·'], ['Ω','·'], ['·','·'], ['·','║'],
            // Row 5: input 4
            ['◂','·'], ['·','·'], ['·','·'], ['·','Ω'], ['·','·'], ['·','║'],
            // Row 6
            ['╚','═'], ['═','═'], ['═','═'], ['═','═'], ['═','═'], ['═','╝'],
        ]},

        // ══════════════════════════════════════════════════════════════════
        // Belts (1×1) — placeholder, overridden by entity_glyph
        // ══════════════════════════════════════════════════════════════════
        BasicBelt | FastBelt | ExpressBelt => BuildingArt { width: 1, height: 1, tiles: &[
            ['→',' '],
        ]},

        // ══════════════════════════════════════════════════════════════════
        // Splitter (3×3): 1 input left-center, 2 outputs right rows 0,2
        // ══════════════════════════════════════════════════════════════════
        Splitter => BuildingArt { width: 3, height: 3, tiles: &[
            ['╔','═'], ['═','═'], ['═','▸'],
            ['◂','·'], ['╋','·'], ['·','║'],
            ['╚','═'], ['═','═'], ['═','▸'],
        ]},

        // ══════════════════════════════════════════════════════════════════
        // Merger (3×3): 2 inputs left rows 0,2, 1 output right-center
        // ══════════════════════════════════════════════════════════════════
        Merger => BuildingArt { width: 3, height: 3, tiles: &[
            ['◂','═'], ['═','═'], ['═','╗'],
            ['║','·'], ['·','╋'], ['·','▸'],
            ['◂','═'], ['═','═'], ['═','╝'],
        ]},

        // ══════════════════════════════════════════════════════════════════
        // Underground belts (1×1)
        // ══════════════════════════════════════════════════════════════════
        UndergroundEntrance => BuildingArt { width: 1, height: 1, tiles: &[
            ['⊏','·'],
        ]},
        UndergroundExit => BuildingArt { width: 1, height: 1, tiles: &[
            ['⊐','·'],
        ]},

        // ══════════════════════════════════════════════════════════════════
        // Pipes & Fluid transport — 1×1: Pipe, PipeJunction, GasPipeline
        //                          3×2: PumpStation, FluidTank, GasCompressor
        // ══════════════════════════════════════════════════════════════════
        Pipe => BuildingArt { width: 1, height: 1, tiles: &[
            ['═','·'],
        ]},
        PipeJunction => BuildingArt { width: 1, height: 1, tiles: &[
            ['╬','·'],
        ]},
        GasPipeline => BuildingArt { width: 1, height: 1, tiles: &[
            ['═','°'],
        ]},
        PumpStation => BuildingArt { width: 3, height: 2, tiles: &[
            ['╔','═'], ['▴','≡'], ['═','╗'],
            ['◂','·'], ['◎','≡'], ['·','▸'],
        ]},
        FluidTank => BuildingArt { width: 3, height: 2, tiles: &[
            ['╔','═'], ['▴','≈'], ['═','╗'],
            ['◂','·'], ['◻','≈'], ['·','▸'],
        ]},
        GasCompressor => BuildingArt { width: 3, height: 2, tiles: &[
            ['╔','═'], ['▴','°'], ['═','╗'],
            ['◂','·'], ['○','°'], ['·','▸'],
        ]},

        // ══════════════════════════════════════════════════════════════════
        // Rail & Transport — 1×1: RailTrack; 3×3: TrainStation, DronePort
        // ══════════════════════════════════════════════════════════════════
        RailTrack => BuildingArt { width: 1, height: 1, tiles: &[
            ['═','═'],
        ]},
        TrainStation => BuildingArt { width: 3, height: 3, tiles: &[
            ['╔','═'], ['═','═'], ['═','╗'],
            ['◂','·'], ['▮','═'], ['·','▸'],
            ['╚','═'], ['═','═'], ['═','╝'],
        ]},
        DronePort => BuildingArt { width: 3, height: 3, tiles: &[
            ['╔','═'], ['═','═'], ['═','╗'],
            ['◂','·'], ['⊕','◆'], ['·','▸'],
            ['╚','═'], ['═','═'], ['═','╝'],
        ]},

        // ══════════════════════════════════════════════════════════════════
        // Power Generators — 3×3: Coal, Gas, Solar, Wind, Geothermal
        //                   5×5: Nuclear, Fusion
        // ══════════════════════════════════════════════════════════════════
        CoalGenerator => BuildingArt { width: 3, height: 3, tiles: &[
            ['╔','═'], ['═','═'], ['═','╗'],
            ['◂','·'], ['⌂','▓'], ['·','║'],
            ['╚','═'], ['═','═'], ['═','╝'],
        ]},
        GasGenerator => BuildingArt { width: 3, height: 3, tiles: &[
            ['╔','═'], ['═','═'], ['═','╗'],
            ['◂','·'], ['⊕','≡'], ['·','║'],
            ['╚','═'], ['═','═'], ['═','╝'],
        ]},
        SolarArray => BuildingArt { width: 3, height: 3, tiles: &[
            ['╔','═'], ['═','═'], ['═','╗'],
            ['◂','·'], ['⊙','▦'], ['·','║'],
            ['╚','═'], ['═','═'], ['═','╝'],
        ]},
        WindTurbine => BuildingArt { width: 3, height: 3, tiles: &[
            ['╔','═'], ['═','═'], ['═','╗'],
            ['◂','·'], ['⊕','╀'], ['·','║'],
            ['╚','═'], ['═','═'], ['═','╝'],
        ]},
        GeothermalPlant => BuildingArt { width: 3, height: 3, tiles: &[
            ['╔','═'], ['═','═'], ['═','╗'],
            ['◂','·'], ['▽','◉'], ['·','║'],
            ['╚','═'], ['═','═'], ['═','╝'],
        ]},
        NuclearReactor => BuildingArt { width: 5, height: 5, tiles: &[
            ['╔','═'], ['═','═'], ['═','═'], ['═','═'], ['═','╗'],
            ['◂','·'], ['◉','·'], ['·','▣'], ['·','·'], ['·','║'],
            ['║','·'], ['·','◉'], ['▣','·'], ['·','·'], ['·','║'],
            ['◂','·'], ['·','·'], ['·','◉'], ['·','·'], ['·','║'],
            ['╚','═'], ['═','═'], ['═','▾'], ['═','═'], ['═','╝'],
        ]},
        FusionReactor => BuildingArt { width: 5, height: 5, tiles: &[
            ['╔','═'], ['═','═'], ['═','═'], ['═','═'], ['═','╗'],
            ['◂','·'], ['◆','·'], ['·','◉'], ['·','·'], ['·','║'],
            ['║','·'], ['·','◆'], ['◉','·'], ['·','·'], ['·','║'],
            ['◂','·'], ['·','·'], ['·','◆'], ['·','·'], ['·','║'],
            ['╚','═'], ['═','═'], ['═','▾'], ['═','═'], ['═','╝'],
        ]},

        // ══════════════════════════════════════════════════════════════════
        // Power Distribution (1×1)
        // ══════════════════════════════════════════════════════════════════
        Transformer => BuildingArt { width: 1, height: 1, tiles: &[
            ['⊕','∿'],
        ]},
        PowerPole => BuildingArt { width: 1, height: 1, tiles: &[
            ['⌁','·'],
        ]},
        Substation => BuildingArt { width: 1, height: 1, tiles: &[
            ['⌂','∿'],
        ]},
        BatteryBank => BuildingArt { width: 1, height: 1, tiles: &[
            ['▮','▮'],
        ]},
        Accumulator => BuildingArt { width: 1, height: 1, tiles: &[
            ['█','▮'],
        ]},

        // ══════════════════════════════════════════════════════════════════
        // Storage — 3×2: OutputBin, SiloHopper
        //          3×3: Warehouse, CryoTank, ContainmentVault
        // ══════════════════════════════════════════════════════════════════
        OutputBin => BuildingArt { width: 3, height: 2, tiles: &[
            ['╔','═'], ['▴','▼'], ['═','╗'],
            ['◂','·'], ['▣','▣'], ['·','▸'],
        ]},
        SiloHopper => BuildingArt { width: 3, height: 2, tiles: &[
            ['╔','═'], ['▴','▽'], ['═','╗'],
            ['◂','·'], ['▦','▦'], ['·','▸'],
        ]},
        Warehouse => BuildingArt { width: 3, height: 3, tiles: &[
            ['╔','═'], ['▴','═'], ['═','╗'],
            ['◂','·'], ['□','▦'], ['·','▸'],
            ['╚','═'], ['▾','═'], ['═','╝'],
        ]},
        CryoTank => BuildingArt { width: 3, height: 3, tiles: &[
            ['╔','═'], ['▴','═'], ['═','╗'],
            ['◂','·'], ['◻','◇'], ['·','▸'],
            ['╚','═'], ['▾','═'], ['═','╝'],
        ]},
        ContainmentVault => BuildingArt { width: 3, height: 3, tiles: &[
            ['╔','═'], ['▴','═'], ['═','╗'],
            ['◂','·'], ['◼','▣'], ['·','▸'],
            ['╚','═'], ['▾','═'], ['═','╝'],
        ]},

        // ══════════════════════════════════════════════════════════════════
        // Defense (1×1)
        // ══════════════════════════════════════════════════════════════════
        Wall => BuildingArt { width: 1, height: 1, tiles: &[
            ['█','█'],
        ]},
        ReinforcedWall => BuildingArt { width: 1, height: 1, tiles: &[
            ['█','█'],
        ]},
        Turret => BuildingArt { width: 1, height: 1, tiles: &[
            ['⊕','◎'],
        ]},
        ShieldGenerator => BuildingArt { width: 1, height: 1, tiles: &[
            ['⊛','◎'],
        ]},

        // ══════════════════════════════════════════════════════════════════
        // Environmental / Waste (3×3) — input left-center, waste
        // bottom-center
        // ══════════════════════════════════════════════════════════════════
        WasteDump => BuildingArt { width: 3, height: 3, tiles: &[
            ['╔','═'], ['═','═'], ['═','╗'],
            ['◂','·'], ['□','×'], ['·','║'],
            ['╚','═'], ['═','▾'], ['═','╝'],
        ]},
        RecyclingPlant => BuildingArt { width: 3, height: 3, tiles: &[
            ['╔','═'], ['═','═'], ['═','╗'],
            ['◂','·'], ['◎','×'], ['·','║'],
            ['╚','═'], ['═','▾'], ['═','╝'],
        ]},
        IncinerationPlant => BuildingArt { width: 3, height: 3, tiles: &[
            ['╔','═'], ['═','═'], ['═','╗'],
            ['◂','·'], ['⊠','▓'], ['·','║'],
            ['╚','═'], ['═','▾'], ['═','╝'],
        ]},
        FilterStack => BuildingArt { width: 3, height: 3, tiles: &[
            ['╔','═'], ['═','═'], ['═','╗'],
            ['◂','·'], ['≋','◎'], ['·','║'],
            ['╚','═'], ['═','▾'], ['═','╝'],
        ]},
        ScrubberUnit => BuildingArt { width: 3, height: 3, tiles: &[
            ['╔','═'], ['═','═'], ['═','╗'],
            ['◂','·'], ['≋','≈'], ['·','║'],
            ['╚','═'], ['═','▾'], ['═','╝'],
        ]},
        ContainmentField => BuildingArt { width: 3, height: 3, tiles: &[
            ['╔','═'], ['═','═'], ['═','╗'],
            ['◂','·'], ['⊞','◉'], ['·','║'],
            ['╚','═'], ['═','▾'], ['═','╝'],
        ]},

        // ══════════════════════════════════════════════════════════════════
        // Research — ResearchLab (3×3), AdvancedLab (4×4)
        // ══════════════════════════════════════════════════════════════════
        ResearchLab => BuildingArt { width: 3, height: 3, tiles: &[
            ['╔','═'], ['═','═'], ['═','╗'],
            ['◂','·'], ['⊞','◆'], ['·','▸'],
            ['╚','═'], ['═','═'], ['═','╝'],
        ]},
        AdvancedLab => BuildingArt { width: 4, height: 4, tiles: &[
            ['╔','═'], ['═','═'], ['═','═'], ['═','╗'],
            ['◂','·'], ['Ψ','·'], ['·','◆'], ['·','║'],
            ['◂','·'], ['·','Ψ'], ['◆','·'], ['·','▸'],
            ['╚','═'], ['═','═'], ['═','═'], ['═','╝'],
        ]},

        // ══════════════════════════════════════════════════════════════════
        // Victory (6×6) — 4 inputs left rows 1-4, output right row 3
        // ══════════════════════════════════════════════════════════════════
        SpaceElevatorBase => BuildingArt { width: 6, height: 6, tiles: &[
            ['╔','═'], ['═','═'], ['═','═'], ['═','═'], ['═','═'], ['═','╗'],
            ['◂','·'], ['⊡','·'], ['·','·'], ['·','◆'], ['·','·'], ['·','║'],
            ['◂','·'], ['·','⊡'], ['·','·'], ['·','·'], ['·','·'], ['·','║'],
            ['◂','·'], ['·','·'], ['⊡','◆'], ['◆','⊡'], ['·','·'], ['·','▸'],
            ['◂','·'], ['·','·'], ['·','·'], ['·','⊡'], ['·','·'], ['·','║'],
            ['╚','═'], ['═','═'], ['═','═'], ['═','═'], ['═','═'], ['═','╝'],
        ]},
        DysonSwarmLauncher => BuildingArt { width: 6, height: 6, tiles: &[
            ['╔','═'], ['═','═'], ['═','═'], ['═','═'], ['═','═'], ['═','╗'],
            ['◂','·'], ['⊙','·'], ['·','·'], ['·','◆'], ['·','·'], ['·','║'],
            ['◂','·'], ['·','⊙'], ['·','·'], ['·','·'], ['·','·'], ['·','║'],
            ['◂','·'], ['·','·'], ['⊙','◆'], ['◆','⊙'], ['·','·'], ['·','▸'],
            ['◂','·'], ['·','·'], ['·','·'], ['·','⊙'], ['·','·'], ['·','║'],
            ['╚','═'], ['═','═'], ['═','═'], ['═','═'], ['═','═'], ['═','╝'],
        ]},
        WarpGateFrame => BuildingArt { width: 6, height: 6, tiles: &[
            ['╔','═'], ['═','═'], ['═','═'], ['═','═'], ['═','═'], ['═','╗'],
            ['◂','·'], ['◈','·'], ['·','·'], ['·','◆'], ['·','·'], ['·','║'],
            ['◂','·'], ['·','◈'], ['·','·'], ['·','·'], ['·','·'], ['·','║'],
            ['◂','·'], ['·','·'], ['◈','◆'], ['◆','◈'], ['·','·'], ['·','▸'],
            ['◂','·'], ['·','·'], ['·','·'], ['·','◈'], ['·','·'], ['·','║'],
            ['╚','═'], ['═','═'], ['═','═'], ['═','═'], ['═','═'], ['═','╝'],
        ]},
    }
}

/// Returns the 2-character art for a specific tile of a building, with rotation applied.
///
/// For belts, uses the existing directional arrow system.
/// For other buildings, looks up the art tile for the given (tile_row, tile_col) and applies rotation.
pub fn entity_art(entity_type: EntityType, facing: Facing, tile_row: usize, tile_col: usize) -> [char; 2] {
    // Belts use directional arrows — special handling.
    // Cell 0 always carries the direction arrow (never obscured by items);
    // cell 1 carries the track lane, which items/animation render over.
    if matches!(entity_type, EntityType::BasicBelt | EntityType::FastBelt | EntityType::ExpressBelt) {
        return [entity_glyph(entity_type, facing), belt_track_glyph(facing)];
    }

    let art = building_art(entity_type);
    let (ar, ac) = rotated_art_coords(tile_row, tile_col, facing, art.width, art.height);
    let chars = art.tile_at(ar, ac);
    rotate_art(chars, facing)
}

/// Transform screen-relative (row, col) within a rotated building back to the
/// art-definition coordinate system (base facing = Right).
pub fn rotated_art_coords(row: usize, col: usize, facing: Facing, w: usize, h: usize) -> (usize, usize) {
    match facing {
        Facing::Right => (row, col),
        Facing::Down  => (col, h.saturating_sub(1).saturating_sub(row)),
        Facing::Left  => (h.saturating_sub(1).saturating_sub(row), w.saturating_sub(1).saturating_sub(col)),
        Facing::Up    => (w.saturating_sub(1).saturating_sub(col), row),
    }
}

// ---------------------------------------------------------------------------
// Rotation helpers
// ---------------------------------------------------------------------------

/// Rotate/mirror a 2-char art row based on facing direction.
fn rotate_art(row: [char; 2], facing: Facing) -> [char; 2] {
    match facing {
        Facing::Right => row,
        Facing::Left => [mirror_h(row[1]), mirror_h(row[0])],
        Facing::Down => [rotate_cw(row[0]), rotate_cw(row[1])],
        Facing::Up => [mirror_h(rotate_cw(row[1])), mirror_h(rotate_cw(row[0]))],
    }
}

/// Mirror a character horizontally (left/right swap).
fn mirror_h(c: char) -> char {
    match c {
        // Directional arrows
        '◂' => '▸', '▸' => '◂',  // ◂ ↔ ▸
        '▴' => '▾', '▾' => '▴',  // ▴ ↔ ▾
        '←' => '→', '→' => '←',  // ← ↔ →
        '↑' => '↓', '↓' => '↑',  // ↑ ↔ ↓  (vertical stays same for H-mirror, but include for completeness)
        // Double-line box corners
        '╔' => '╗', '╗' => '╔',  // ╔ ↔ ╗
        '╚' => '╝', '╝' => '╚',  // ╚ ↔ ╝
        // Double-line T-junctions
        '╠' => '╣', '╣' => '╠',  // ╠ ↔ ╣
        '╥' => '╥',              // ╥ is symmetric
        '╨' => '╨',              // ╨ is symmetric
        // Single-line corners
        '┌' => '┐', '┐' => '┌',  // ┌ ↔ ┐
        '└' => '┘', '┘' => '└',  // └ ↔ ┘
        // Single-line T-junctions
        '├' => '┤', '┤' => '├',  // ├ ↔ ┤
        '┬' => '┬',              // ┬ is symmetric
        '┴' => '┴',              // ┴ is symmetric
        // Mixed double/single corners
        '╒' => '╕', '╕' => '╒',  // ╒ ↔ ╕
        '╘' => '╛', '╛' => '╘',  // ╘ ↔ ╛
        '╓' => '╖', '╖' => '╓',  // ╓ ↔ ╖
        '╙' => '╜', '╜' => '╙',  // ╙ ↔ ╜
        // Rounded corners
        '╭' => '╮', '╮' => '╭',  // ╭ ↔ ╮
        '╰' => '╯', '╯' => '╰',  // ╰ ↔ ╯
        // Diagonal lines
        '╱' => '╲', '╲' => '╱',  // ╱ ↔ ╲
        // Underground entrance/exit
        '⊏' => '⊐', '⊐' => '⊏',  // ⊏ ↔ ⊐
        // Symmetric characters (no change needed): ═ ║ ╬ ╋ ┃ ╀ · etc.
        _ => c,
    }
}

/// Rotate a directional character 90° clockwise.
fn rotate_cw(c: char) -> char {
    match c {
        // Triangular directional arrows
        '◂' => '▴',  // ◂ → ▴
        '▴' => '▸',  // ▴ → ▸
        '▸' => '▾',  // ▸ → ▾
        '▾' => '◂',  // ▾ → ◂
        // Unicode arrows
        '←' => '↑',  // ← → ↑
        '↑' => '→',  // ↑ → →
        '→' => '↓',  // → → ↓
        '↓' => '←',  // ↓ → ←
        // Double-line box corners (rotate CW: top-left→top-right→bottom-right→bottom-left)
        '╔' => '╗',  // ╔ → ╗
        '╗' => '╝',  // ╗ → ╝
        '╝' => '╚',  // ╝ → ╚
        '╚' => '╔',  // ╚ → ╔
        // Double-line edges swap orientation
        '═' => '║',  // ═ → ║
        '║' => '═',  // ║ → ═
        // Double-line T-junctions
        '╠' => '╦',  // ╠ → ╦
        '╦' => '╣',  // ╦ → ╣
        '╣' => '╩',  // ╣ → ╩
        '╩' => '╠',  // ╩ → ╠
        '╥' => '╢',  // ╥ → ╢ (single-top-T rotates)
        '╢' => '╨',  // ╢ → ╨
        '╨' => '╟',  // ╨ → ╟
        '╟' => '╥',  // ╟ → ╥
        // Single-line corners
        '┌' => '┐',  // ┌ → ┐
        '┐' => '┘',  // ┐ → ┘
        '┘' => '└',  // ┘ → └
        '└' => '┌',  // └ → ┌
        // Single-line T-junctions
        '├' => '┬',  // ├ → ┬
        '┬' => '┤',  // ┬ → ┤
        '┤' => '┴',  // ┤ → ┴
        '┴' => '├',  // ┴ → ├
        // Single-line edges
        '─' => '│',  // ─ → │
        '│' => '─',  // │ → ─
        // Heavy single-line edges
        '━' => '┃',  // ━ → ┃
        '┃' => '━',  // ┃ → ━
        // Mixed double/single corners
        '╒' => '╕',  // ╒ → ╕
        '╕' => '╛',  // ╕ → ╛
        '╛' => '╘',  // ╛ → ╘
        '╘' => '╒',  // ╘ → ╒
        // Rounded corners
        '╭' => '╮',  // ╭ → ╮
        '╮' => '╯',  // ╮ → ╯
        '╯' => '╰',  // ╯ → ╰
        '╰' => '╭',  // ╰ → ╭
        // Underground entrance/exit
        '⊏' => '⊏',  // not rotational, keep same
        '⊐' => '⊐',  // not rotational, keep same
        // Symmetric characters (╬, ╋, etc.) stay the same
        _ => c,
    }
}

// ---------------------------------------------------------------------------
// Building colors (updated from spec)
// ---------------------------------------------------------------------------

/// Returns the base foreground color for a building type from the visual spec.
pub fn building_fg(entity_type: EntityType) -> (u8, u8, u8) {
    use EntityType::*;
    match entity_type {
        // Extractors
        OreDeposit => (160, 120, 60),
        CopperDeposit => (210, 120, 50),
        CoalDeposit => (60, 60, 60),
        StoneQuarry => (150, 150, 140),
        OilWell => (40, 40, 45),
        WaterPump => (64, 164, 223),
        GasExtractor => (200, 200, 150),
        UraniumMine => (80, 220, 80),
        SandExtractor => (220, 210, 170),
        SulfurMine => (220, 220, 50),
        BauxiteMine => (200, 100, 80),
        LithiumExtractor => (200, 230, 255),
        RareEarthExtractor => (180, 100, 180),
        BiomassHarvester => (60, 140, 40),
        GeothermalTap => (220, 100, 40),

        // 1x1 Processors
        Smelter => (220, 60, 40),
        Kiln => (230, 140, 40),
        Press => (160, 165, 175),
        WireMill => (210, 150, 80),
        PlateMachine => (190, 190, 200),
        RubberVulcanizer => (80, 80, 80),
        PlasticMolder => (240, 240, 240),
        Electrolyzer => (100, 180, 240),
        Caster => (200, 120, 40),
        CokeFurnace => (160, 80, 30),
        Gasifier => (100, 100, 110),
        Boiler => (220, 100, 40),
        WaferCutter => (100, 100, 150),

        // 1x3 Processors
        Assembler => (60, 200, 220),
        Mixer => (80, 200, 180),
        ChemicalPlant => (50, 200, 80),
        CircuitFabricator => (80, 220, 120),
        MotorAssembly => (180, 180, 200),
        CrushingMill => (160, 140, 120),

        // 1x5 Processors
        AdvancedAssembler => (80, 220, 240),
        Refinery => (120, 120, 130),
        CrackingTower => (100, 80, 60),
        Cleanroom => (230, 235, 240),
        EnrichmentCascade => (80, 220, 80),
        CoolantProcessor => (160, 220, 255),

        // 1x7 Processors
        PrecisionAssembler => (100, 230, 255),
        QuantumLab => (180, 80, 255),
        RocketAssembly => (255, 100, 50),

        // 1x9 Processors
        Megassembler => (240, 240, 250),
        SingularityLab => (255, 200, 60),

        // Belts
        BasicBelt => (200, 200, 200),
        FastBelt => (255, 220, 50),
        ExpressBelt => (60, 140, 255),

        // Splitter/Merger
        Splitter | Merger => (230, 200, 50),

        // Underground
        UndergroundEntrance | UndergroundExit => (180, 180, 180),

        // Pipes
        Pipe | PipeJunction => (120, 120, 125),
        PumpStation => (80, 150, 220),
        FluidTank => (100, 160, 200),
        GasCompressor => (180, 180, 120),
        GasPipeline => (180, 200, 220),

        // Rail/Transport
        RailTrack => (100, 95, 85),
        TrainStation => (220, 200, 60),
        DronePort => (200, 200, 210),

        // Power Generators
        CoalGenerator => (180, 80, 30),
        GasGenerator => (160, 170, 200),
        SolarArray => (40, 60, 200),
        WindTurbine => (200, 200, 210),
        GeothermalPlant => (200, 100, 40),
        NuclearReactor => (80, 220, 80),
        FusionReactor => (255, 200, 60),

        // Power Distribution
        Transformer => (255, 200, 50),
        PowerPole => (180, 180, 60),
        Substation => (200, 200, 80),
        BatteryBank => (60, 100, 200),
        Accumulator => (40, 80, 180),

        // Storage
        OutputBin => (60, 200, 80),
        Warehouse => (160, 120, 60),
        SiloHopper => (200, 200, 180),
        CryoTank => (100, 200, 255),
        ContainmentVault => (150, 150, 180),

        // Defense
        Wall => (80, 80, 85),
        ReinforcedWall => (100, 100, 105),
        Turret => (200, 50, 50),
        ShieldGenerator => (100, 150, 255),

        // Environmental
        WasteDump => (120, 100, 70),
        RecyclingPlant => (140, 120, 80),
        IncinerationPlant => (220, 80, 30),
        FilterStack => (100, 200, 100),
        ScrubberUnit => (80, 160, 180),
        ContainmentField => (200, 200, 60),

        // Research
        ResearchLab => (180, 80, 200),
        AdvancedLab => (200, 100, 230),

        // Victory
        SpaceElevatorBase => (255, 215, 0),
        DysonSwarmLauncher => (255, 200, 50),
        WarpGateFrame => (200, 100, 255),
    }
}

/// Returns the background glow color for when a building is processing.
pub fn building_glow_bg(entity_type: EntityType) -> (u8, u8, u8) {
    let (r, g, b) = building_fg(entity_type);
    (r / 5, g / 5, b / 5)
}

// ---------------------------------------------------------------------------
// Entity glyph (backward-compat: returns col0 art character for non-belts)
// ---------------------------------------------------------------------------

/// Returns the display character for an entity type, respecting facing for belts.
pub fn entity_glyph(entity_type: EntityType, facing: Facing) -> char {
    match entity_type {
        // Solid triangles read far more crisply than thin arrows at cell size.
        EntityType::BasicBelt => match facing {
            Facing::Up => '\u{25B2}',    // ▲
            Facing::Down => '\u{25BC}',  // ▼
            Facing::Left => '\u{25C0}',  // ◀
            Facing::Right => '\u{25B6}', // ▶
        },
        EntityType::FastBelt => match facing {
            Facing::Up => '\u{21D1}',
            Facing::Down => '\u{21D3}',
            Facing::Left => '\u{21D0}',
            Facing::Right => '\u{21D2}',
        },
        EntityType::ExpressBelt => match facing {
            Facing::Up => '\u{21E7}',
            Facing::Down => '\u{21E9}',
            Facing::Left => '\u{21E6}',
            Facing::Right => '\u{21E8}',
        },
        _ => building_art(entity_type).tile_at(0, 0)[0],
    }
}

// ---------------------------------------------------------------------------
// Entity styles
// ---------------------------------------------------------------------------

/// Returns the ratatui Style for an entity type using spec colors.
pub fn entity_style(entity_type: EntityType) -> Style {
    let (r, g, b) = building_fg(entity_type);
    let mut style = Style::default().fg(Color::Rgb(r, g, b));
    match entity_type {
        EntityType::BasicBelt | EntityType::FastBelt | EntityType::ExpressBelt => {}
        EntityType::Wall | EntityType::ReinforcedWall => {}
        _ => {
            style = style.add_modifier(Modifier::BOLD);
        }
    }
    style
}

/// Returns style for an entity based on its machine state and animation frame.
pub fn entity_style_for_state(
    entity_type: EntityType,
    state: MachineState,
    frame: u32,
) -> Style {
    let base = building_fg(entity_type);
    match state {
        MachineState::Idle => {
            let dimmed = dim_color(base, 0.65);
            let glow = building_glow_bg(entity_type);
            let bg = (glow.0 / 2, glow.1 / 2, glow.2 / 2);
            Style::default()
                .fg(Color::Rgb(dimmed.0, dimmed.1, dimmed.2))
                .bg(Color::Rgb(bg.0.max(12), bg.1.max(12), bg.2.max(12)))
        }
        MachineState::Processing => {
            // Pulse: oscillate brightness based on frame
            let pulse = ((frame % 6) as f32 / 6.0 * std::f32::consts::PI).sin();
            let pr = (base.0 as f32 + pulse * 50.0).clamp(0.0, 255.0) as u8;
            let pg = (base.1 as f32 + pulse * 50.0).clamp(0.0, 255.0) as u8;
            let pb = (base.2 as f32 + pulse * 50.0).clamp(0.0, 255.0) as u8;
            let glow = building_glow_bg(entity_type);
            // BG also pulses for stronger effect
            let bg_pulse = (pulse * 0.5 + 0.5).max(0.0);
            let bgr = (glow.0 as f32 * (1.0 + bg_pulse)).min(255.0) as u8;
            let bgg = (glow.1 as f32 * (1.0 + bg_pulse)).min(255.0) as u8;
            let bgb = (glow.2 as f32 * (1.0 + bg_pulse)).min(255.0) as u8;
            Style::default()
                .fg(Color::Rgb(pr, pg, pb))
                .bg(Color::Rgb(bgr, bgg, bgb))
                .add_modifier(Modifier::BOLD)
        }
        MachineState::Blocked => {
            let blink = (frame / 15) % 2 == 0;
            if blink {
                Style::default()
                    .fg(Color::Rgb(220, 180, 40))
                    .add_modifier(Modifier::BOLD)
            } else {
                let dimmed = dim_color(base, 0.5);
                Style::default().fg(Color::Rgb(dimmed.0, dimmed.1, dimmed.2))
            }
        }
        MachineState::Broken => {
            let blink = (frame / 5) % 2 == 0;
            if blink {
                Style::default()
                    .fg(Color::Rgb(200, 0, 0))
                    .bg(Color::Rgb(40, 0, 0))
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(Color::Rgb(80, 0, 0))
            }
        }
    }
}

// ---------------------------------------------------------------------------
// Conveyor / belt helpers
// ---------------------------------------------------------------------------

/// Returns a dimmed conveyor style (when idle / not carrying a resource).
pub fn conveyor_idle_style() -> Style {
    Style::default()
        .fg(Color::Rgb(120, 120, 130))
        .bg(Color::Rgb(22, 26, 34))
}

/// Belt animation: get the animated glyph for an empty belt based on frame counter.
pub fn belt_animated_glyph(belt_type: EntityType, facing: Facing, frame: u32) -> char {
    match belt_type {
        EntityType::BasicBelt => {
            if (frame / 4) % 2 == 0 {
                '\u{00B7}'
            } else {
                match facing {
                    Facing::Right => '\u{203A}',
                    Facing::Left => '\u{2039}',
                    Facing::Up => '\u{02C4}',
                    Facing::Down => '\u{02C5}',
                }
            }
        }
        EntityType::FastBelt => {
            if (frame / 2) % 2 == 0 {
                '\u{00B7}'
            } else {
                match facing {
                    Facing::Right => '\u{00BB}',
                    Facing::Left => '\u{00AB}',
                    Facing::Up => '\u{02C4}',
                    Facing::Down => '\u{02C5}',
                }
            }
        }
        EntityType::ExpressBelt => {
            if frame % 2 == 0 {
                '\u{00B7}'
            } else {
                match facing {
                    Facing::Right => '\u{21D2}',
                    Facing::Left => '\u{21D0}',
                    Facing::Up => '\u{21D1}',
                    Facing::Down => '\u{21D3}',
                }
            }
        }
        _ => '\u{00B7}',
    }
}

/// Track-lane glyph for the second cell of a belt tile: shows the rail the
/// items ride on (double-line to match the machine housings).
pub fn belt_track_glyph(facing: Facing) -> char {
    match facing {
        Facing::Left | Facing::Right => '\u{2550}', // ═
        Facing::Up | Facing::Down => '\u{2551}',    // ║
    }
}

/// Returns the style for a belt type using ONLY Color::Rgb.
pub fn belt_style(belt_type: EntityType) -> Style {
    match belt_type {
        EntityType::BasicBelt => Style::default()
            .fg(Color::Rgb(220, 220, 230))
            .bg(Color::Rgb(22, 26, 34))
            .add_modifier(Modifier::BOLD),
        EntityType::FastBelt => Style::default()
            .fg(Color::Rgb(255, 220, 50))
            .bg(Color::Rgb(30, 28, 14))
            .add_modifier(Modifier::BOLD),
        EntityType::ExpressBelt => Style::default()
            .fg(Color::Rgb(80, 160, 255))
            .bg(Color::Rgb(12, 22, 40))
            .add_modifier(Modifier::BOLD),
        _ => Style::default().fg(Color::Rgb(200, 200, 200)),
    }
}

// ---------------------------------------------------------------------------
// Resource display
// ---------------------------------------------------------------------------

/// Returns the display character for a resource floating on the grid.
pub fn resource_glyph(resource: Resource) -> char {
    resource.glyph()
}

/// Returns the background glow color for a resource (resource color at ~20% brightness).
pub fn resource_glow_bg(resource: Resource) -> Color {
    let (r, g, b) = resource.color();
    Color::Rgb(r / 5 + 8, g / 5 + 8, b / 5 + 8)
}

/// Returns the style for a resource glyph using ONLY Color::Rgb.
pub fn resource_style(resource: Resource) -> Style {
    let (r, g, b) = resource.color();
    Style::default()
        .fg(Color::Rgb(r, g, b))
        .bg(resource_glow_bg(resource))
        .add_modifier(Modifier::BOLD)
}

// ---------------------------------------------------------------------------
// Empty tile
// ---------------------------------------------------------------------------

/// Returns the glyph for an empty tile.
pub fn empty_tile_glyph() -> char {
    '\u{00B7}'
}

/// Returns the style for an empty tile using ONLY Color::Rgb.
pub fn empty_tile_style() -> Style {
    Style::default().fg(Color::Rgb(60, 60, 60))
}

// ---------------------------------------------------------------------------
// Processing indicator
// ---------------------------------------------------------------------------

/// Format a processing indicator for machines.
/// Returns None if the machine is idle.
pub fn processing_indicator(_entity_type: EntityType, processing: &Processing) -> Option<char> {
    if !processing.is_processing() {
        return None;
    }
    let ticks = processing.ticks_remaining.min(9) as u8;
    Some((b'0' + ticks) as char)
}

// ---------------------------------------------------------------------------
// Ports
// ---------------------------------------------------------------------------

/// Whether an art character is an input/output/waste port arrow.
/// (After rotation these all stay within this set.)
pub fn is_port_char(c: char) -> bool {
    matches!(c, '\u{25C2}' | '\u{25B8}' | '\u{25B4}' | '\u{25BE}') // ◂ ▸ ▴ ▾
}

/// Accent style for a machine's port arrows so its I/O edges pop out of the
/// housing. Bright warm foreground over the machine's ambient glow.
pub fn port_style(entity_type: EntityType) -> Style {
    let glow = building_glow_bg(entity_type);
    Style::default()
        .fg(Color::Rgb(255, 232, 150))
        .bg(Color::Rgb(glow.0.max(14), glow.1.max(14), glow.2.max(14)))
        .add_modifier(Modifier::BOLD)
}
