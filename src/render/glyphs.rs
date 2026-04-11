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
/// Each row corresponds to one tile of the building (top-to-bottom when facing Right).
/// Each tile has exactly 2 terminal columns: [col0, col1].
pub struct BuildingArt {
    pub rows: &'static [[char; 2]],
}

/// Returns the ASCII art definition for an entity type (in Right-facing orientation).
pub fn building_art(entity_type: EntityType) -> BuildingArt {
    use EntityType::*;
    match entity_type {
        // ── Extractors (1x1) ────────────────────────────────────────────
        OreDeposit | CopperDeposit | CoalDeposit | StoneQuarry
        | UraniumMine | SandExtractor | SulfurMine | BauxiteMine
        | LithiumExtractor | RareEarthExtractor => BuildingArt { rows: &[
            ['\u{229E}', '\u{00B7}'],  // ⊞·
        ]},
        OilWell => BuildingArt { rows: &[
            ['\u{22BC}', '~'],  // ⊼~
        ]},
        WaterPump => BuildingArt { rows: &[
            ['\u{2248}', '\u{2191}'],  // ≈↑
        ]},
        GasExtractor => BuildingArt { rows: &[
            ['\u{25CE}', '\u{00B0}'],  // ◎°
        ]},
        BiomassHarvester => BuildingArt { rows: &[
            ['\u{2320}', '\u{00A4}'],  // ⌠¤
        ]},
        GeothermalTap => BuildingArt { rows: &[
            ['\u{25BD}', '\u{25B3}'],  // ▽△
        ]},

        // ── 1x1 Processors ─────────────────────────────────────────────
        Smelter => BuildingArt { rows: &[
            ['\u{25EE}', '\u{25A3}'],  // ◮▣
        ]},
        Kiln => BuildingArt { rows: &[
            ['\u{2293}', '\u{25A5}'],  // ⊓▥
        ]},
        Press => BuildingArt { rows: &[
            ['\u{2293}', '\u{2294}'],  // ⊓⊔
        ]},
        WireMill => BuildingArt { rows: &[
            ['\u{229E}', '\u{223F}'],  // ⊞∿
        ]},
        PlateMachine => BuildingArt { rows: &[
            ['\u{229E}', '\u{25A4}'],  // ⊞▤
        ]},
        RubberVulcanizer => BuildingArt { rows: &[
            ['\u{2593}', '\u{25A4}'],  // ▓▤
        ]},
        PlasticMolder => BuildingArt { rows: &[
            ['\u{25FB}', '\u{25A4}'],  // ◻▤
        ]},
        Electrolyzer => BuildingArt { rows: &[
            ['\u{2295}', '\u{223F}'],  // ⊕∿
        ]},
        Caster => BuildingArt { rows: &[
            ['\u{25EE}', '\u{25A4}'],  // ◮▤
        ]},
        CokeFurnace => BuildingArt { rows: &[
            ['\u{2302}', '\u{2593}'],  // ⌂▓
        ]},
        Gasifier => BuildingArt { rows: &[
            ['\u{2593}', '\u{2591}'],  // ▓░
        ]},
        Boiler => BuildingArt { rows: &[
            ['\u{25EE}', '\u{2248}'],  // ◮≈
        ]},
        WaferCutter => BuildingArt { rows: &[
            ['\u{2297}', '\u{25A4}'],  // ⊗▤
        ]},

        // ── 1x3 Processors ─────────────────────────────────────────────
        Assembler => BuildingArt { rows: &[
            ['\u{2554}', '\u{25C2}'],  // ╔◂
            ['\u{229B}', '\u{25B8}'],  // ⊛▸
            ['\u{255A}', '\u{25C2}'],  // ╚◂
        ]},
        Mixer => BuildingArt { rows: &[
            ['\u{256D}', '\u{25E6}'],  // ╭◦
            ['\u{25CE}', '\u{25B8}'],  // ◎▸
            ['\u{2570}', '\u{25E6}'],  // ╰◦
        ]},
        ChemicalPlant => BuildingArt { rows: &[
            ['\u{2554}', '\u{25CE}'],  // ╔◎
            ['\u{2560}', '\u{25B8}'],  // ╠▸
            ['\u{255A}', '\u{25CE}'],  // ╚◎
        ]},
        CircuitFabricator => BuildingArt { rows: &[
            ['\u{2554}', '\u{25C2}'],  // ╔◂
            ['\u{229E}', '\u{25B8}'],  // ⊞▸
            ['\u{255A}', '\u{25C2}'],  // ╚◂
        ]},
        MotorAssembly => BuildingArt { rows: &[
            ['\u{2554}', '\u{25C2}'],  // ╔◂
            ['\u{2299}', '\u{25B8}'],  // ⊙▸
            ['\u{255A}', '\u{25C2}'],  // ╚◂
        ]},
        CrushingMill => BuildingArt { rows: &[
            ['\u{2554}', '\u{25C2}'],  // ╔◂
            ['\u{2297}', '\u{25B8}'],  // ⊗▸
            ['\u{255A}', '\u{25C2}'],  // ╚◂
        ]},

        // ── 1x5 Processors ─────────────────────────────────────────────
        AdvancedAssembler => BuildingArt { rows: &[
            ['\u{2554}', '\u{25C2}'],  // ╔◂
            ['\u{2551}', '\u{00B7}'],  // ║·
            ['\u{229B}', '\u{25B8}'],  // ⊛▸
            ['\u{2551}', '\u{00B7}'],  // ║·
            ['\u{255A}', '\u{25C2}'],  // ╚◂
        ]},
        Refinery => BuildingArt { rows: &[
            ['\u{250C}', '\u{2565}'],  // ┌╥
            ['\u{2502}', '\u{2551}'],  // │║
            ['\u{25C2}', '\u{256C}'],  // ◂╬
            ['\u{2502}', '\u{2551}'],  // │║
            ['\u{2514}', '\u{2568}'],  // └╨
        ]},
        CrackingTower => BuildingArt { rows: &[
            ['\u{2503}', '\u{25C2}'],  // ┃◂
            ['\u{2503}', '\u{00B7}'],  // ┃·
            ['\u{254B}', '\u{25B8}'],  // ╋▸
            ['\u{2503}', '\u{00B7}'],  // ┃·
            ['\u{2503}', '\u{25C2}'],  // ┃◂
        ]},
        Cleanroom => BuildingArt { rows: &[
            ['\u{2552}', '\u{25C2}'],  // ╒◂
            ['\u{2502}', '\u{00B7}'],  // │·
            ['\u{25C7}', '\u{25B8}'],  // ◇▸
            ['\u{2502}', '\u{00B7}'],  // │·
            ['\u{2558}', '\u{25C2}'],  // ╘◂
        ]},
        EnrichmentCascade => BuildingArt { rows: &[
            ['\u{2554}', '\u{25C2}'],  // ╔◂
            ['\u{2551}', '\u{00B7}'],  // ║·
            ['\u{25C9}', '\u{25B8}'],  // ◉▸
            ['\u{2551}', '\u{00B7}'],  // ║·
            ['\u{255A}', '\u{25C2}'],  // ╚◂
        ]},
        CoolantProcessor => BuildingArt { rows: &[
            ['\u{25C7}', '\u{25C2}'],  // ◇◂
            ['\u{2502}', '\u{00B7}'],  // │·
            ['\u{25C8}', '\u{25B8}'],  // ◈▸
            ['\u{2502}', '\u{00B7}'],  // │·
            ['\u{2514}', '\u{25C2}'],  // └◂
        ]},

        // ── 1x7 Processors ─────────────────────────────────────────────
        PrecisionAssembler => BuildingArt { rows: &[
            ['\u{250C}', '\u{25C2}'],  // ┌◂
            ['\u{2502}', '\u{00B7}'],  // │·
            ['\u{251C}', '\u{25C2}'],  // ├◂
            ['\u{229B}', '\u{25B8}'],  // ⊛▸
            ['\u{251C}', '\u{25C2}'],  // ├◂
            ['\u{2502}', '\u{00B7}'],  // │·
            ['\u{2514}', '\u{25C2}'],  // └◂
        ]},
        QuantumLab => BuildingArt { rows: &[
            ['\u{03A8}', '\u{25C2}'],  // Ψ◂
            ['\u{2551}', '\u{00B7}'],  // ║·
            ['\u{25C8}', '\u{25C2}'],  // ◈◂
            ['\u{22B9}', '\u{25B8}'],  // ⊹▸
            ['\u{224B}', '\u{25C2}'],  // ≋◂
            ['\u{2551}', '\u{00B7}'],  // ║·
            ['\u{2248}', '\u{25C2}'],  // ≈◂
        ]},
        RocketAssembly => BuildingArt { rows: &[
            ['\u{2554}', '\u{25C2}'],  // ╔◂
            ['\u{2551}', '\u{00B7}'],  // ║·
            ['\u{2560}', '\u{25C2}'],  // ╠◂
            ['\u{22A1}', '\u{25B8}'],  // ⊡▸
            ['\u{2560}', '\u{25C2}'],  // ╠◂
            ['\u{2551}', '\u{00B7}'],  // ║·
            ['\u{255A}', '\u{25C2}'],  // ╚◂
        ]},

        // ── 1x9 Processors ─────────────────────────────────────────────
        Megassembler => BuildingArt { rows: &[
            ['\u{2554}', '\u{25C2}'],  // ╔◂
            ['\u{2551}', '\u{00B7}'],  // ║·
            ['\u{2560}', '\u{25C2}'],  // ╠◂
            ['\u{2551}', '\u{00B7}'],  // ║·
            ['\u{22A1}', '\u{25B8}'],  // ⊡▸
            ['\u{2551}', '\u{00B7}'],  // ║·
            ['\u{2560}', '\u{25C2}'],  // ╠◂
            ['\u{2551}', '\u{00B7}'],  // ║·
            ['\u{255A}', '\u{25C2}'],  // ╚◂
        ]},
        SingularityLab => BuildingArt { rows: &[
            ['\u{229E}', '\u{25C2}'],  // ⊞◂
            ['\u{2551}', '\u{00B7}'],  // ║·
            ['\u{229E}', '\u{25C2}'],  // ⊞◂
            ['\u{2551}', '\u{00B7}'],  // ║·
            ['\u{03A9}', '\u{00B7}'],  // Ω·
            ['\u{2551}', '\u{00B7}'],  // ║·
            ['\u{229E}', '\u{25C2}'],  // ⊞◂
            ['\u{2551}', '\u{00B7}'],  // ║·
            ['\u{229E}', '\u{25C2}'],  // ⊞◂
        ]},

        // ── Belts (1x1, arrow in col0, space in col1) ───────────────────
        // Belts use entity_glyph() for directional arrows — art is a placeholder.
        BasicBelt | FastBelt | ExpressBelt => BuildingArt { rows: &[
            ['\u{2192}', ' '],  // → (placeholder, overridden by entity_glyph)
        ]},

        // ── Splitter / Merger (1x1) ─────────────────────────────────────
        Splitter => BuildingArt { rows: &[
            ['\u{254B}', '\u{25B8}'],  // ╋▸
        ]},
        Merger => BuildingArt { rows: &[
            ['\u{25C2}', '\u{254B}'],  // ◂╋
        ]},

        // ── Underground belts ───────────────────────────────────────────
        UndergroundEntrance => BuildingArt { rows: &[
            ['\u{228F}', '\u{00B7}'],  // ⊏·
        ]},
        UndergroundExit => BuildingArt { rows: &[
            ['\u{2290}', '\u{00B7}'],  // ⊐·
        ]},

        // ── Pipes & Fluid transport ─────────────────────────────────────
        Pipe => BuildingArt { rows: &[
            ['\u{2550}', '\u{00B7}'],  // ═·
        ]},
        PipeJunction => BuildingArt { rows: &[
            ['\u{256C}', '\u{00B7}'],  // ╬·
        ]},
        PumpStation => BuildingArt { rows: &[
            ['\u{25CE}', '\u{2261}'],  // ◎≡
        ]},
        FluidTank => BuildingArt { rows: &[
            ['\u{25FB}', '\u{2248}'],  // ◻≈
        ]},
        GasCompressor => BuildingArt { rows: &[
            ['\u{25CB}', '\u{00B0}'],  // ○°
        ]},
        GasPipeline => BuildingArt { rows: &[
            ['\u{2550}', '\u{00B0}'],  // ═°
        ]},

        // ── Rail & Transport ────────────────────────────────────────────
        RailTrack => BuildingArt { rows: &[
            ['\u{2550}', '\u{2550}'],  // ══
        ]},
        TrainStation => BuildingArt { rows: &[
            ['\u{25AE}', '\u{25C2}'],  // ▮◂
        ]},
        DronePort => BuildingArt { rows: &[
            ['\u{2295}', '\u{25C6}'],  // ⊕◆
        ]},

        // ── Power Generators ────────────────────────────────────────────
        CoalGenerator => BuildingArt { rows: &[
            ['\u{2302}', '\u{2593}'],  // ⌂▓
        ]},
        GasGenerator => BuildingArt { rows: &[
            ['\u{2295}', '\u{2261}'],  // ⊕≡
        ]},
        SolarArray => BuildingArt { rows: &[
            ['\u{2299}', '\u{25A6}'],  // ⊙▦
        ]},
        WindTurbine => BuildingArt { rows: &[
            ['\u{2295}', '\u{2540}'],  // ⊕╀
        ]},
        GeothermalPlant => BuildingArt { rows: &[
            ['\u{25BD}', '\u{25C9}'],  // ▽◉
        ]},
        NuclearReactor => BuildingArt { rows: &[
            ['\u{25C9}', '\u{25A3}'],  // ◉▣
        ]},
        FusionReactor => BuildingArt { rows: &[
            ['\u{25C6}', '\u{25C9}'],  // ◆◉
        ]},

        // ── Power Distribution ──────────────────────────────────────────
        Transformer => BuildingArt { rows: &[
            ['\u{2295}', '\u{223F}'],  // ⊕∿
        ]},
        PowerPole => BuildingArt { rows: &[
            ['\u{2301}', '\u{00B7}'],  // ⌁·
        ]},
        Substation => BuildingArt { rows: &[
            ['\u{2302}', '\u{223F}'],  // ⌂∿
        ]},
        BatteryBank => BuildingArt { rows: &[
            ['\u{25AE}', '\u{25AE}'],  // ▮▮
        ]},
        Accumulator => BuildingArt { rows: &[
            ['\u{2588}', '\u{25AE}'],  // █▮
        ]},

        // ── Storage ─────────────────────────────────────────────────────
        OutputBin => BuildingArt { rows: &[
            ['\u{25BC}', '\u{25A3}'],  // ▼▣
        ]},
        Warehouse => BuildingArt { rows: &[
            ['\u{25A1}', '\u{25A6}'],  // □▦
        ]},
        SiloHopper => BuildingArt { rows: &[
            ['\u{25BD}', '\u{25A6}'],  // ▽▦
        ]},
        CryoTank => BuildingArt { rows: &[
            ['\u{25FB}', '\u{25C7}'],  // ◻◇
        ]},
        ContainmentVault => BuildingArt { rows: &[
            ['\u{25FC}', '\u{25A3}'],  // ◼▣
        ]},

        // ── Defense ─────────────────────────────────────────────────────
        Wall => BuildingArt { rows: &[
            ['\u{2588}', '\u{2588}'],  // ██
        ]},
        ReinforcedWall => BuildingArt { rows: &[
            ['\u{2588}', '\u{2588}'],  // ██
        ]},
        Turret => BuildingArt { rows: &[
            ['\u{2295}', '\u{25CE}'],  // ⊕◎
        ]},
        ShieldGenerator => BuildingArt { rows: &[
            ['\u{229B}', '\u{25CE}'],  // ⊛◎
        ]},

        // ── Environmental / Waste ───────────────────────────────────────
        WasteDump => BuildingArt { rows: &[
            ['\u{25A1}', '\u{00D7}'],  // □×
        ]},
        RecyclingPlant => BuildingArt { rows: &[
            ['\u{25CE}', '\u{00D7}'],  // ◎×
        ]},
        IncinerationPlant => BuildingArt { rows: &[
            ['\u{22A0}', '\u{2593}'],  // ⊠▓
        ]},
        FilterStack => BuildingArt { rows: &[
            ['\u{224B}', '\u{25CE}'],  // ≋◎
        ]},
        ScrubberUnit => BuildingArt { rows: &[
            ['\u{224B}', '\u{25CE}'],  // ≋◎
        ]},
        ContainmentField => BuildingArt { rows: &[
            ['\u{229E}', '\u{25C9}'],  // ⊞◉
        ]},

        // ── Research ────────────────────────────────────────────────────
        ResearchLab => BuildingArt { rows: &[
            ['\u{229E}', '\u{25C6}'],  // ⊞◆
        ]},
        AdvancedLab => BuildingArt { rows: &[
            ['\u{03A8}', '\u{25C6}'],  // Ψ◆
        ]},

        // ── Victory ─────────────────────────────────────────────────────
        SpaceElevatorBase => BuildingArt { rows: &[
            ['\u{22A1}', '\u{25C6}'],  // ⊡◆
        ]},
        DysonSwarmLauncher => BuildingArt { rows: &[
            ['\u{2299}', '\u{25C6}'],  // ⊙◆
        ]},
        WarpGateFrame => BuildingArt { rows: &[
            ['\u{25C8}', '\u{25C6}'],  // ◈◆
        ]},
    }
}

/// Returns the 2-character art for a specific tile of a building, with rotation applied.
///
/// For belts, uses the existing directional arrow system.
/// For other buildings, looks up the art row for the given tile_index and applies rotation.
pub fn entity_art(entity_type: EntityType, facing: Facing, tile_index: usize) -> [char; 2] {
    // Belts use directional arrows — special handling
    if matches!(entity_type, EntityType::BasicBelt | EntityType::FastBelt | EntityType::ExpressBelt) {
        return [entity_glyph(entity_type, facing), ' '];
    }

    let art = building_art(entity_type);
    let row = art.rows.get(tile_index).copied().unwrap_or(['\u{00B7}', '\u{00B7}']);
    rotate_art(row, facing)
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
        '\u{25C2}' => '\u{25B8}', '\u{25B8}' => '\u{25C2}',  // ◂ ↔ ▸
        '\u{25B4}' => '\u{25BE}', '\u{25BE}' => '\u{25B4}',  // ▴ ↔ ▾
        '\u{2554}' => '\u{2557}', '\u{2557}' => '\u{2554}',  // ╔ ↔ ╗
        '\u{255A}' => '\u{255D}', '\u{255D}' => '\u{255A}',  // ╚ ↔ ╝
        '\u{2560}' => '\u{2563}', '\u{2563}' => '\u{2560}',  // ╠ ↔ ╣
        '\u{251C}' => '\u{2524}', '\u{2524}' => '\u{251C}',  // ├ ↔ ┤
        '\u{250C}' => '\u{2510}', '\u{2510}' => '\u{250C}',  // ┌ ↔ ┐
        '\u{2514}' => '\u{2518}', '\u{2518}' => '\u{2514}',  // └ ↔ ┘
        '\u{2552}' => '\u{2555}', '\u{2555}' => '\u{2552}',  // ╒ ↔ ╕
        '\u{2558}' => '\u{255B}', '\u{255B}' => '\u{2558}',  // ╘ ↔ ╛
        '\u{2571}' => '\u{2572}', '\u{2572}' => '\u{2571}',  // ╱ ↔ ╲
        '\u{256D}' => '\u{256E}', '\u{256E}' => '\u{256D}',  // ╭ ↔ ╮
        '\u{2570}' => '\u{256F}', '\u{256F}' => '\u{2570}',  // ╰ ↔ ╯
        '\u{228F}' => '\u{2290}', '\u{2290}' => '\u{228F}',  // ⊏ ↔ ⊐
        _ => c,
    }
}

/// Rotate a directional character 90° clockwise.
fn rotate_cw(c: char) -> char {
    match c {
        '\u{25C2}' => '\u{25B4}',  // ◂ → ▴
        '\u{25B8}' => '\u{25BE}',  // ▸ → ▾
        '\u{25B4}' => '\u{25B8}',  // ▴ → ▸
        '\u{25BE}' => '\u{25C2}',  // ▾ → ◂
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
        EntityType::BasicBelt => facing.arrow_glyph(),
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
        _ => building_art(entity_type).rows[0][0],
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
            let dimmed = dim_color(base, 0.5);
            Style::default().fg(Color::Rgb(dimmed.0, dimmed.1, dimmed.2))
        }
        MachineState::Processing => {
            // Pulse: oscillate brightness based on frame
            let pulse = ((frame % 6) as f32 / 6.0 * std::f32::consts::PI).sin();
            let pr = (base.0 as f32 + pulse * 35.0).clamp(0.0, 255.0) as u8;
            let pg = (base.1 as f32 + pulse * 35.0).clamp(0.0, 255.0) as u8;
            let pb = (base.2 as f32 + pulse * 35.0).clamp(0.0, 255.0) as u8;
            let glow = building_glow_bg(entity_type);
            Style::default()
                .fg(Color::Rgb(pr, pg, pb))
                .bg(Color::Rgb(glow.0, glow.1, glow.2))
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
        .fg(Color::Rgb(50, 50, 50))
        .add_modifier(Modifier::DIM)
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

/// Returns the style for a belt type using ONLY Color::Rgb.
pub fn belt_style(belt_type: EntityType) -> Style {
    match belt_type {
        EntityType::BasicBelt => Style::default()
            .fg(Color::Rgb(200, 200, 200))
            .bg(Color::Rgb(20, 20, 22)),
        EntityType::FastBelt => Style::default()
            .fg(Color::Rgb(220, 200, 80))
            .bg(Color::Rgb(30, 26, 8)),
        EntityType::ExpressBelt => Style::default()
            .fg(Color::Rgb(100, 160, 255))
            .bg(Color::Rgb(10, 20, 40)),
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

/// Returns the style for a resource glyph using ONLY Color::Rgb.
pub fn resource_style(resource: Resource) -> Style {
    let (r, g, b) = resource.color();
    let mut style = Style::default().fg(Color::Rgb(r, g, b));
    if resource.tier() >= 2 {
        style = style.add_modifier(Modifier::BOLD);
    }
    style
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
