use serde::{Deserialize, Serialize};

// ---------------------------------------------------------------------------
// Resource Category
// ---------------------------------------------------------------------------

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ResourceCategory {
    Solid,
    Fluid,
    Gas,
    Plasma,
    Waste,
}

// ---------------------------------------------------------------------------
// Resource
// ---------------------------------------------------------------------------

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Resource {
    // Tier 0: Raw
    #[serde(alias = "Ore")]
    IronOre,
    CopperOre,
    Coal,
    Stone,
    CrudeOil,
    Water,
    NaturalGas,
    UraniumOre,
    QuartzSand,
    Sulfur,
    Bauxite,
    LithiumBrine,
    RareEarthOre,
    Biomass,
    GeothermalSteam,
    // Tier 1: Basic
    #[serde(alias = "Ingot")]
    IronIngot,
    CopperIngot,
    Steel,
    StoneBrick,
    Glass,
    CopperWire,
    IronPlate,
    CopperPlate,
    Rubber,
    Plastic,
    SulfuricAcid,
    Concrete,
    Charcoal,
    Syngas,
    Steam,
    Lubricant,
    PolymerResin,
    // Tier 2: Intermediate
    #[serde(alias = "Widget")]
    CircuitBoard,
    ElectricMotor,
    Gear,
    Bearing,
    PipeFitting,
    AluminumIngot,
    AluminumSheet,
    SiliconWafer,
    Fiberglass,
    LithiumCell,
    InsulatedWire,
    HeatExchanger,
    ConcSulfuricAcid,
    // Tier 3: Advanced
    Processor,
    BatteryPack,
    Servo,
    HydraulicCylinder,
    SolarPanel,
    TurbineBlade,
    AdvancedCircuit,
    CompositePanel,
    Coolant,
    ChemicalAdditive,
    SuperconductorWire,
    EnrichedUranium,
    NuclearFuelRod,
    HeavyWater,
    RocketFuel,
    RepairKit,
    // Tier 4: Mega
    QuantumProcessor,
    FusionCore,
    AntimatterCapsule,
    GravityLens,
    WarpDriveModule,
    DysonSpherePanel,
    SpaceElevatorCable,
    TerraformingAgent,
    NanobotSwarm,
    DarkMatterShard,
    // Tier 5: Victory
    SpaceElevatorSegment,
    DysonSwarmCluster,
    WarpGateComponent,
    // Science
    SciencePack1,
    SciencePack2,
    SciencePack3,
    SciencePack4,
    SciencePack5,
    // Waste
    Slag,
    Tailings,
    Wastewater,
    FlyAsh,
    CO2,
    RedMud,
    SpentAcid,
    MetalShavings,
    PCBEtchWaste,
    PetroleumCoke,
    DepletedUranium,
    NuclearWaste,
    ToxicSludge,
    CryoExhaust,
    CarbonMonoxide,
    AntimatterExhaust,
    DimensionalResidue,
    IrradiatedCoolant,
    QuantumDecoherence,
}

impl Resource {
    pub fn name(&self) -> &'static str {
        match self {
            Self::IronOre => "iron ore",
            Self::CopperOre => "copper ore",
            Self::Coal => "coal",
            Self::Stone => "stone",
            Self::CrudeOil => "crude oil",
            Self::Water => "water",
            Self::NaturalGas => "natural gas",
            Self::UraniumOre => "uranium ore",
            Self::QuartzSand => "quartz sand",
            Self::Sulfur => "sulfur",
            Self::Bauxite => "bauxite",
            Self::LithiumBrine => "lithium brine",
            Self::RareEarthOre => "rare earth ore",
            Self::Biomass => "biomass",
            Self::GeothermalSteam => "geothermal steam",
            Self::IronIngot => "iron ingot",
            Self::CopperIngot => "copper ingot",
            Self::Steel => "steel",
            Self::StoneBrick => "stone brick",
            Self::Glass => "glass",
            Self::CopperWire => "copper wire",
            Self::IronPlate => "iron plate",
            Self::CopperPlate => "copper plate",
            Self::Rubber => "rubber",
            Self::Plastic => "plastic",
            Self::SulfuricAcid => "sulfuric acid",
            Self::Concrete => "concrete",
            Self::Charcoal => "charcoal",
            Self::Syngas => "syngas",
            Self::Steam => "steam",
            Self::Lubricant => "lubricant",
            Self::PolymerResin => "polymer resin",
            Self::CircuitBoard => "circuit board",
            Self::ElectricMotor => "electric motor",
            Self::Gear => "gear",
            Self::Bearing => "bearing",
            Self::PipeFitting => "pipe fitting",
            Self::AluminumIngot => "aluminum ingot",
            Self::AluminumSheet => "aluminum sheet",
            Self::SiliconWafer => "silicon wafer",
            Self::Fiberglass => "fiberglass",
            Self::LithiumCell => "lithium cell",
            Self::InsulatedWire => "insulated wire",
            Self::HeatExchanger => "heat exchanger",
            Self::ConcSulfuricAcid => "conc. sulfuric acid",
            Self::Processor => "processor",
            Self::BatteryPack => "battery pack",
            Self::Servo => "servo",
            Self::HydraulicCylinder => "hydraulic cylinder",
            Self::SolarPanel => "solar panel",
            Self::TurbineBlade => "turbine blade",
            Self::AdvancedCircuit => "advanced circuit",
            Self::CompositePanel => "composite panel",
            Self::Coolant => "coolant",
            Self::ChemicalAdditive => "chemical additive",
            Self::SuperconductorWire => "superconductor wire",
            Self::EnrichedUranium => "enriched uranium",
            Self::NuclearFuelRod => "nuclear fuel rod",
            Self::HeavyWater => "heavy water",
            Self::RocketFuel => "rocket fuel",
            Self::RepairKit => "repair kit",
            Self::QuantumProcessor => "quantum processor",
            Self::FusionCore => "fusion core",
            Self::AntimatterCapsule => "antimatter capsule",
            Self::GravityLens => "gravity lens",
            Self::WarpDriveModule => "warp drive module",
            Self::DysonSpherePanel => "dyson sphere panel",
            Self::SpaceElevatorCable => "space elevator cable",
            Self::TerraformingAgent => "terraforming agent",
            Self::NanobotSwarm => "nanobot swarm",
            Self::DarkMatterShard => "dark matter shard",
            Self::SpaceElevatorSegment => "space elevator segment",
            Self::DysonSwarmCluster => "dyson swarm cluster",
            Self::WarpGateComponent => "warp gate component",
            Self::SciencePack1 => "science pack 1",
            Self::SciencePack2 => "science pack 2",
            Self::SciencePack3 => "science pack 3",
            Self::SciencePack4 => "science pack 4",
            Self::SciencePack5 => "science pack 5",
            Self::Slag => "slag",
            Self::Tailings => "tailings",
            Self::Wastewater => "wastewater",
            Self::FlyAsh => "fly ash",
            Self::CO2 => "CO2",
            Self::RedMud => "red mud",
            Self::SpentAcid => "spent acid",
            Self::MetalShavings => "metal shavings",
            Self::PCBEtchWaste => "PCB etch waste",
            Self::PetroleumCoke => "petroleum coke",
            Self::DepletedUranium => "depleted uranium",
            Self::NuclearWaste => "nuclear waste",
            Self::ToxicSludge => "toxic sludge",
            Self::CryoExhaust => "cryo exhaust",
            Self::CarbonMonoxide => "carbon monoxide",
            Self::AntimatterExhaust => "antimatter exhaust",
            Self::DimensionalResidue => "dimensional residue",
            Self::IrradiatedCoolant => "irradiated coolant",
            Self::QuantumDecoherence => "quantum decoherence",
        }
    }

    pub fn tier(&self) -> u8 {
        match self {
            Self::IronOre | Self::CopperOre | Self::Coal | Self::Stone
            | Self::CrudeOil | Self::Water | Self::NaturalGas | Self::UraniumOre
            | Self::QuartzSand | Self::Sulfur | Self::Bauxite | Self::LithiumBrine
            | Self::RareEarthOre | Self::Biomass | Self::GeothermalSteam => 0,

            Self::IronIngot | Self::CopperIngot | Self::Steel | Self::StoneBrick
            | Self::Glass | Self::CopperWire | Self::IronPlate | Self::CopperPlate
            | Self::Rubber | Self::Plastic | Self::SulfuricAcid | Self::Concrete
            | Self::Charcoal | Self::Syngas | Self::Steam | Self::Lubricant
            | Self::PolymerResin => 1,

            Self::CircuitBoard | Self::ElectricMotor | Self::Gear | Self::Bearing
            | Self::PipeFitting | Self::AluminumIngot | Self::AluminumSheet
            | Self::SiliconWafer | Self::Fiberglass | Self::LithiumCell
            | Self::InsulatedWire | Self::HeatExchanger | Self::ConcSulfuricAcid => 2,

            Self::Processor | Self::BatteryPack | Self::Servo | Self::HydraulicCylinder
            | Self::SolarPanel | Self::TurbineBlade | Self::AdvancedCircuit
            | Self::CompositePanel | Self::Coolant | Self::ChemicalAdditive
            | Self::SuperconductorWire | Self::EnrichedUranium | Self::NuclearFuelRod
            | Self::HeavyWater | Self::RocketFuel | Self::RepairKit => 3,

            Self::QuantumProcessor | Self::FusionCore | Self::AntimatterCapsule
            | Self::GravityLens | Self::WarpDriveModule | Self::DysonSpherePanel
            | Self::SpaceElevatorCable | Self::TerraformingAgent | Self::NanobotSwarm
            | Self::DarkMatterShard => 4,

            Self::SpaceElevatorSegment | Self::DysonSwarmCluster
            | Self::WarpGateComponent => 5,

            Self::SciencePack1 => 1,
            Self::SciencePack2 => 2,
            Self::SciencePack3 => 3,
            Self::SciencePack4 => 4,
            Self::SciencePack5 => 5,

            // Waste tier matches the tier that produces it
            Self::Slag | Self::Tailings | Self::Wastewater | Self::FlyAsh
            | Self::CO2 | Self::RedMud | Self::SpentAcid | Self::MetalShavings
            | Self::CarbonMonoxide => 0,
            Self::PCBEtchWaste | Self::PetroleumCoke => 1,
            Self::DepletedUranium | Self::NuclearWaste | Self::ToxicSludge
            | Self::IrradiatedCoolant => 2,
            Self::CryoExhaust | Self::AntimatterExhaust | Self::DimensionalResidue
            | Self::QuantumDecoherence => 3,
        }
    }

    pub fn category(&self) -> ResourceCategory {
        match self {
            Self::CrudeOil | Self::Water | Self::LithiumBrine | Self::SulfuricAcid
            | Self::Lubricant | Self::ConcSulfuricAcid | Self::Coolant
            | Self::HeavyWater | Self::RocketFuel => ResourceCategory::Fluid,

            Self::NaturalGas | Self::GeothermalSteam | Self::Syngas | Self::Steam
            | Self::CO2 | Self::CarbonMonoxide | Self::CryoExhaust => ResourceCategory::Gas,

            Self::AntimatterCapsule | Self::DarkMatterShard
            | Self::AntimatterExhaust | Self::QuantumDecoherence => ResourceCategory::Plasma,

            Self::Slag | Self::Tailings | Self::Wastewater | Self::FlyAsh
            | Self::RedMud | Self::SpentAcid | Self::MetalShavings | Self::PCBEtchWaste
            | Self::PetroleumCoke | Self::DepletedUranium | Self::NuclearWaste
            | Self::ToxicSludge | Self::DimensionalResidue
            | Self::IrradiatedCoolant => ResourceCategory::Waste,

            _ => ResourceCategory::Solid,
        }
    }

    pub fn is_waste(&self) -> bool {
        matches!(
            self,
            Self::Slag | Self::Tailings | Self::Wastewater | Self::FlyAsh
            | Self::CO2 | Self::RedMud | Self::SpentAcid | Self::MetalShavings
            | Self::PCBEtchWaste | Self::PetroleumCoke | Self::DepletedUranium
            | Self::NuclearWaste | Self::ToxicSludge | Self::CryoExhaust
            | Self::CarbonMonoxide | Self::AntimatterExhaust | Self::DimensionalResidue
            | Self::IrradiatedCoolant | Self::QuantumDecoherence
        )
    }

    pub fn base_value(&self) -> f64 {
        if self.is_waste() {
            return 0.0;
        }
        match self.tier() {
            0 => 1.0,
            1 => 5.0,
            2 => 25.0,
            3 => 100.0,
            4 => 500.0,
            5 => 2000.0,
            _ => 1.0,
        }
    }

    pub fn glyph(&self) -> char {
        match self {
            // Tier 0
            Self::IronOre => 'i', Self::CopperOre => 'c', Self::Coal => 'k',
            Self::Stone => 's', Self::CrudeOil => 'o', Self::Water => 'w',
            Self::NaturalGas => 'g', Self::UraniumOre => 'u', Self::QuartzSand => 'q',
            Self::Sulfur => 'z', Self::Bauxite => 'b', Self::LithiumBrine => 'l',
            Self::RareEarthOre => 'r', Self::Biomass => 'm', Self::GeothermalSteam => 'v',
            // Tier 1
            Self::IronIngot => 'I', Self::CopperIngot => 'C', Self::Steel => 'S',
            Self::StoneBrick => 'B', Self::Glass => 'G', Self::CopperWire => 'W',
            Self::IronPlate => 'P', Self::CopperPlate => 'p', Self::Rubber => 'R',
            Self::Plastic => 'L', Self::SulfuricAcid => 'A', Self::Concrete => 'N',
            Self::Charcoal => 'K', Self::Syngas => 'Y', Self::Steam => 'V',
            Self::Lubricant => 'U', Self::PolymerResin => 'Z',
            // Tier 2
            Self::CircuitBoard => '#', Self::ElectricMotor => 'M', Self::Gear => '*',
            Self::Bearing => '@', Self::PipeFitting => '=', Self::AluminumIngot => 'a',
            Self::AluminumSheet => 'h', Self::SiliconWafer => 'x', Self::Fiberglass => 'f',
            Self::LithiumCell => 'e', Self::InsulatedWire => 'n',
            Self::HeatExchanger => 'H', Self::ConcSulfuricAcid => 'X',
            // Tier 3
            Self::Processor => '~', Self::BatteryPack => '+', Self::Servo => '$',
            Self::HydraulicCylinder => '!', Self::SolarPanel => '%',
            Self::TurbineBlade => '^', Self::AdvancedCircuit => '&',
            Self::CompositePanel => ':', Self::Coolant => '?',
            Self::ChemicalAdditive => '<', Self::SuperconductorWire => '>',
            Self::EnrichedUranium => 'E', Self::NuclearFuelRod => '|',
            Self::HeavyWater => 'D', Self::RocketFuel => 'J', Self::RepairKit => 'T',
            // Tier 4
            Self::QuantumProcessor => 'Q', Self::FusionCore => 'F',
            Self::AntimatterCapsule => 'O', Self::GravityLens => 'L',
            Self::WarpDriveModule => 'W', Self::DysonSpherePanel => 'D',
            Self::SpaceElevatorCable => 'E', Self::TerraformingAgent => 'T',
            Self::NanobotSwarm => 'N', Self::DarkMatterShard => 'X',
            // Tier 5
            Self::SpaceElevatorSegment => 'E', Self::DysonSwarmCluster => 'D',
            Self::WarpGateComponent => 'W',
            // Science
            Self::SciencePack1 => '1', Self::SciencePack2 => '2',
            Self::SciencePack3 => '3', Self::SciencePack4 => '4',
            Self::SciencePack5 => '5',
            // Waste
            Self::Slag => '.', Self::Tailings => ',', Self::Wastewater => '~',
            Self::FlyAsh => ';', Self::CO2 => '"', Self::RedMud => '/',
            Self::SpentAcid => '\\', Self::MetalShavings => '`',
            Self::PCBEtchWaste => '_', Self::PetroleumCoke => '-',
            Self::DepletedUranium => '=', Self::NuclearWaste => '!',
            Self::ToxicSludge => '@', Self::CryoExhaust => '#',
            Self::CarbonMonoxide => '$', Self::AntimatterExhaust => '%',
            Self::DimensionalResidue => '^', Self::IrradiatedCoolant => '&',
            Self::QuantumDecoherence => '*',
        }
    }

    pub fn color(&self) -> (u8, u8, u8) {
        match self {
            // Tier 0
            Self::IronOre => (180, 140, 60), Self::CopperOre => (210, 120, 50),
            Self::Coal => (60, 60, 60), Self::Stone => (150, 150, 140),
            Self::CrudeOil => (30, 30, 30), Self::Water => (60, 120, 200),
            Self::NaturalGas => (180, 200, 220), Self::UraniumOre => (80, 220, 80),
            Self::QuartzSand => (220, 210, 170), Self::Sulfur => (220, 220, 50),
            Self::Bauxite => (200, 100, 80), Self::LithiumBrine => (200, 230, 255),
            Self::RareEarthOre => (180, 100, 180), Self::Biomass => (50, 180, 50),
            Self::GeothermalSteam => (200, 200, 200),
            // Tier 1
            Self::IronIngot => (200, 200, 200), Self::CopperIngot => (220, 140, 60),
            Self::Steel => (180, 190, 200), Self::StoneBrick => (180, 160, 120),
            Self::Glass => (200, 230, 255), Self::CopperWire => (220, 160, 40),
            Self::IronPlate => (190, 190, 190), Self::CopperPlate => (200, 130, 50),
            Self::Rubber => (40, 40, 40), Self::Plastic => (240, 240, 240),
            Self::SulfuricAcid => (200, 200, 40), Self::Concrete => (160, 160, 150),
            Self::Charcoal => (80, 60, 40), Self::Syngas => (150, 180, 200),
            Self::Steam => (220, 220, 220), Self::Lubricant => (100, 140, 60),
            Self::PolymerResin => (180, 160, 100),
            // Tier 2
            Self::CircuitBoard => (100, 220, 100), Self::ElectricMotor => (150, 160, 180),
            Self::Gear => (180, 180, 180), Self::Bearing => (200, 200, 200),
            Self::PipeFitting => (180, 130, 50), Self::AluminumIngot => (200, 210, 220),
            Self::AluminumSheet => (210, 220, 230), Self::SiliconWafer => (100, 100, 150),
            Self::Fiberglass => (240, 240, 200), Self::LithiumCell => (100, 200, 100),
            Self::InsulatedWire => (200, 50, 50), Self::HeatExchanger => (220, 150, 50),
            Self::ConcSulfuricAcid => (180, 180, 30),
            // Tier 3
            Self::Processor => (80, 180, 255), Self::BatteryPack => (100, 200, 100),
            Self::Servo => (170, 170, 180), Self::HydraulicCylinder => (150, 80, 40),
            Self::SolarPanel => (50, 100, 200), Self::TurbineBlade => (200, 200, 210),
            Self::AdvancedCircuit => (50, 255, 150), Self::CompositePanel => (140, 140, 160),
            Self::Coolant => (100, 200, 255), Self::ChemicalAdditive => (200, 100, 200),
            Self::SuperconductorWire => (255, 200, 50), Self::EnrichedUranium => (100, 255, 100),
            Self::NuclearFuelRod => (50, 255, 50), Self::HeavyWater => (40, 80, 180),
            Self::RocketFuel => (255, 100, 50), Self::RepairKit => (200, 200, 100),
            // Tier 4
            Self::QuantumProcessor => (150, 100, 255), Self::FusionCore => (255, 200, 100),
            Self::AntimatterCapsule => (255, 50, 255), Self::GravityLens => (200, 200, 255),
            Self::WarpDriveModule => (100, 255, 255), Self::DysonSpherePanel => (255, 220, 50),
            Self::SpaceElevatorCable => (180, 180, 200), Self::TerraformingAgent => (50, 200, 100),
            Self::NanobotSwarm => (200, 200, 200), Self::DarkMatterShard => (80, 50, 120),
            // Tier 5
            Self::SpaceElevatorSegment => (255, 215, 0),
            Self::DysonSwarmCluster => (255, 200, 50),
            Self::WarpGateComponent => (200, 100, 255),
            // Science
            Self::SciencePack1 => (255, 80, 80), Self::SciencePack2 => (80, 255, 80),
            Self::SciencePack3 => (80, 80, 255), Self::SciencePack4 => (255, 80, 255),
            Self::SciencePack5 => (255, 255, 80),
            // Waste
            Self::Slag => (100, 90, 80), Self::Tailings => (120, 110, 90),
            Self::Wastewater => (80, 100, 80), Self::FlyAsh => (130, 130, 120),
            Self::CO2 => (150, 150, 150), Self::RedMud => (160, 80, 60),
            Self::SpentAcid => (140, 140, 60), Self::MetalShavings => (160, 160, 160),
            Self::PCBEtchWaste => (80, 120, 80), Self::PetroleumCoke => (50, 50, 50),
            Self::DepletedUranium => (80, 120, 80), Self::NuclearWaste => (80, 200, 80),
            Self::ToxicSludge => (100, 80, 120), Self::CryoExhaust => (180, 200, 220),
            Self::CarbonMonoxide => (130, 130, 130), Self::AntimatterExhaust => (200, 100, 200),
            Self::DimensionalResidue => (120, 80, 160),
            Self::IrradiatedCoolant => (80, 180, 120),
            Self::QuantumDecoherence => (160, 120, 200),
        }
    }
}

// ---------------------------------------------------------------------------
// EntityType
// ---------------------------------------------------------------------------

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum EntityType {
    // Extraction (1x1, no facing)
    OreDeposit,
    CopperDeposit,
    CoalDeposit,
    StoneQuarry,
    OilWell,
    WaterPump,
    GasExtractor,
    UraniumMine,
    SandExtractor,
    SulfurMine,
    BauxiteMine,
    LithiumExtractor,
    RareEarthExtractor,
    BiomassHarvester,
    GeothermalTap,
    // Processing 1x1
    Smelter,
    Kiln,
    Press,
    WireMill,
    PlateMachine,
    RubberVulcanizer,
    PlasticMolder,
    Electrolyzer,
    Caster,
    CokeFurnace,
    Gasifier,
    Boiler,
    WaferCutter,
    // Processing 1x3
    Assembler,
    Mixer,
    ChemicalPlant,
    CircuitFabricator,
    MotorAssembly,
    CrushingMill,
    // Processing 1x5
    AdvancedAssembler,
    Refinery,
    CrackingTower,
    Cleanroom,
    EnrichmentCascade,
    CoolantProcessor,
    // Processing 1x7
    PrecisionAssembler,
    QuantumLab,
    RocketAssembly,
    // Processing 1x9
    Megassembler,
    SingularityLab,
    // Transport
    #[serde(alias = "Conveyor")]
    BasicBelt,
    FastBelt,
    ExpressBelt,
    Splitter,
    Merger,
    UndergroundEntrance,
    UndergroundExit,
    Pipe,
    PipeJunction,
    PumpStation,
    FluidTank,
    GasCompressor,
    GasPipeline,
    RailTrack,
    TrainStation,
    DronePort,
    // Power
    CoalGenerator,
    GasGenerator,
    SolarArray,
    WindTurbine,
    GeothermalPlant,
    NuclearReactor,
    FusionReactor,
    Transformer,
    PowerPole,
    Substation,
    BatteryBank,
    Accumulator,
    // Storage
    OutputBin,
    Warehouse,
    SiloHopper,
    CryoTank,
    ContainmentVault,
    // Defense
    Wall,
    ReinforcedWall,
    Turret,
    ShieldGenerator,
    // Environmental
    WasteDump,
    RecyclingPlant,
    IncinerationPlant,
    FilterStack,
    ScrubberUnit,
    ContainmentField,
    // Research
    ResearchLab,
    AdvancedLab,
    // Victory
    SpaceElevatorBase,
    DysonSwarmLauncher,
    WarpGateFrame,
}

/// Internal helper for grouping entity behavior.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum BuildingKind {
    Extractor,
    Processor1x1,
    Processor1x3,
    Processor1x5,
    Processor1x7,
    Processor1x9,
    Belt,
    SplitterKind,
    MergerKind,
    PipeTransport,
    RailTransport,
    PowerGenerator,
    PowerDistribution,
    Storage,
    Defense,
    Environmental,
    Research,
    Victory,
}

impl EntityType {
    fn building_kind(&self) -> BuildingKind {
        match self {
            Self::OreDeposit | Self::CopperDeposit | Self::CoalDeposit
            | Self::StoneQuarry | Self::OilWell | Self::WaterPump
            | Self::GasExtractor | Self::UraniumMine | Self::SandExtractor
            | Self::SulfurMine | Self::BauxiteMine | Self::LithiumExtractor
            | Self::RareEarthExtractor | Self::BiomassHarvester
            | Self::GeothermalTap => BuildingKind::Extractor,

            Self::Smelter | Self::Kiln | Self::Press | Self::WireMill
            | Self::PlateMachine | Self::RubberVulcanizer | Self::PlasticMolder
            | Self::Electrolyzer | Self::Caster | Self::CokeFurnace
            | Self::Gasifier | Self::Boiler | Self::WaferCutter => BuildingKind::Processor1x1,

            Self::Assembler | Self::Mixer | Self::ChemicalPlant
            | Self::CircuitFabricator | Self::MotorAssembly
            | Self::CrushingMill => BuildingKind::Processor1x3,

            Self::AdvancedAssembler | Self::Refinery | Self::CrackingTower
            | Self::Cleanroom | Self::EnrichmentCascade
            | Self::CoolantProcessor => BuildingKind::Processor1x5,

            Self::PrecisionAssembler | Self::QuantumLab
            | Self::RocketAssembly => BuildingKind::Processor1x7,

            Self::Megassembler | Self::SingularityLab => BuildingKind::Processor1x9,

            Self::BasicBelt | Self::FastBelt | Self::ExpressBelt
            | Self::UndergroundEntrance | Self::UndergroundExit => BuildingKind::Belt,

            Self::Splitter => BuildingKind::SplitterKind,
            Self::Merger => BuildingKind::MergerKind,

            Self::Pipe | Self::PipeJunction | Self::PumpStation
            | Self::FluidTank | Self::GasCompressor
            | Self::GasPipeline => BuildingKind::PipeTransport,

            Self::RailTrack | Self::TrainStation | Self::DronePort => BuildingKind::RailTransport,

            Self::CoalGenerator | Self::GasGenerator | Self::SolarArray
            | Self::WindTurbine | Self::GeothermalPlant | Self::NuclearReactor
            | Self::FusionReactor => BuildingKind::PowerGenerator,

            Self::Transformer | Self::PowerPole | Self::Substation
            | Self::BatteryBank | Self::Accumulator => BuildingKind::PowerDistribution,

            Self::OutputBin | Self::Warehouse | Self::SiloHopper
            | Self::CryoTank | Self::ContainmentVault => BuildingKind::Storage,

            Self::Wall | Self::ReinforcedWall | Self::Turret
            | Self::ShieldGenerator => BuildingKind::Defense,

            Self::WasteDump | Self::RecyclingPlant | Self::IncinerationPlant
            | Self::FilterStack | Self::ScrubberUnit
            | Self::ContainmentField => BuildingKind::Environmental,

            Self::ResearchLab | Self::AdvancedLab => BuildingKind::Research,

            Self::SpaceElevatorBase | Self::DysonSwarmLauncher
            | Self::WarpGateFrame => BuildingKind::Victory,
        }
    }

    pub fn from_insert_char(c: char) -> Option<EntityType> {
        match c {
            's' => Some(Self::Smelter),
            'a' => Some(Self::Assembler),
            'c' => Some(Self::BasicBelt),
            'p' => Some(Self::Splitter),
            'e' => Some(Self::Merger),
            'w' => Some(Self::Wall),
            _ => None,
        }
    }

    pub fn from_find_char(c: char) -> Option<EntityType> {
        match c {
            's' => Some(Self::Smelter),
            'a' => Some(Self::Assembler),
            'c' => Some(Self::BasicBelt),
            'p' => Some(Self::Splitter),
            'm' => Some(Self::Merger),
            'o' => Some(Self::OreDeposit),
            'b' => Some(Self::OutputBin),
            'w' => Some(Self::Wall),
            _ => None,
        }
    }

    pub fn from_search_prefix(prefix: &str) -> Option<EntityType> {
        let lower = prefix.to_lowercase();
        if "ore deposit".starts_with(&lower) || "ore".starts_with(&lower) {
            Some(Self::OreDeposit)
        } else if "smelter".starts_with(&lower) || "sme".starts_with(&lower) {
            Some(Self::Smelter)
        } else if "assembler".starts_with(&lower) || "ass".starts_with(&lower) {
            Some(Self::Assembler)
        } else if "basic belt".starts_with(&lower) || "belt".starts_with(&lower)
            || "conveyor".starts_with(&lower)
        {
            Some(Self::BasicBelt)
        } else if "splitter".starts_with(&lower) || "spl".starts_with(&lower) {
            Some(Self::Splitter)
        } else if "merger".starts_with(&lower) || "mer".starts_with(&lower) {
            Some(Self::Merger)
        } else if "output bin".starts_with(&lower) || "bin".starts_with(&lower) {
            Some(Self::OutputBin)
        } else if "wall".starts_with(&lower) {
            Some(Self::Wall)
        } else if "kiln".starts_with(&lower) {
            Some(Self::Kiln)
        } else if "press".starts_with(&lower) {
            Some(Self::Press)
        } else if "wire mill".starts_with(&lower) {
            Some(Self::WireMill)
        } else if "chemical plant".starts_with(&lower) {
            Some(Self::ChemicalPlant)
        } else if "refinery".starts_with(&lower) {
            Some(Self::Refinery)
        } else if "research lab".starts_with(&lower) {
            Some(Self::ResearchLab)
        } else {
            None
        }
    }

    pub fn name(&self) -> &'static str {
        match self {
            Self::OreDeposit => "ore deposit",
            Self::CopperDeposit => "copper deposit",
            Self::CoalDeposit => "coal deposit",
            Self::StoneQuarry => "stone quarry",
            Self::OilWell => "oil well",
            Self::WaterPump => "water pump",
            Self::GasExtractor => "gas extractor",
            Self::UraniumMine => "uranium mine",
            Self::SandExtractor => "sand extractor",
            Self::SulfurMine => "sulfur mine",
            Self::BauxiteMine => "bauxite mine",
            Self::LithiumExtractor => "lithium extractor",
            Self::RareEarthExtractor => "rare earth extractor",
            Self::BiomassHarvester => "biomass harvester",
            Self::GeothermalTap => "geothermal tap",
            Self::Smelter => "smelter",
            Self::Kiln => "kiln",
            Self::Press => "press",
            Self::WireMill => "wire mill",
            Self::PlateMachine => "plate machine",
            Self::RubberVulcanizer => "rubber vulcanizer",
            Self::PlasticMolder => "plastic molder",
            Self::Electrolyzer => "electrolyzer",
            Self::Caster => "caster",
            Self::CokeFurnace => "coke furnace",
            Self::Gasifier => "gasifier",
            Self::Boiler => "boiler",
            Self::WaferCutter => "wafer cutter",
            Self::Assembler => "assembler",
            Self::Mixer => "mixer",
            Self::ChemicalPlant => "chemical plant",
            Self::CircuitFabricator => "circuit fabricator",
            Self::MotorAssembly => "motor assembly",
            Self::CrushingMill => "crushing mill",
            Self::AdvancedAssembler => "advanced assembler",
            Self::Refinery => "refinery",
            Self::CrackingTower => "cracking tower",
            Self::Cleanroom => "cleanroom",
            Self::EnrichmentCascade => "enrichment cascade",
            Self::CoolantProcessor => "coolant processor",
            Self::PrecisionAssembler => "precision assembler",
            Self::QuantumLab => "quantum lab",
            Self::RocketAssembly => "rocket assembly",
            Self::Megassembler => "megassembler",
            Self::SingularityLab => "singularity lab",
            Self::BasicBelt => "basic belt",
            Self::FastBelt => "fast belt",
            Self::ExpressBelt => "express belt",
            Self::Splitter => "splitter",
            Self::Merger => "merger",
            Self::UndergroundEntrance => "underground entrance",
            Self::UndergroundExit => "underground exit",
            Self::Pipe => "pipe",
            Self::PipeJunction => "pipe junction",
            Self::PumpStation => "pump station",
            Self::FluidTank => "fluid tank",
            Self::GasCompressor => "gas compressor",
            Self::GasPipeline => "gas pipeline",
            Self::RailTrack => "rail track",
            Self::TrainStation => "train station",
            Self::DronePort => "drone port",
            Self::CoalGenerator => "coal generator",
            Self::GasGenerator => "gas generator",
            Self::SolarArray => "solar array",
            Self::WindTurbine => "wind turbine",
            Self::GeothermalPlant => "geothermal plant",
            Self::NuclearReactor => "nuclear reactor",
            Self::FusionReactor => "fusion reactor",
            Self::Transformer => "transformer",
            Self::PowerPole => "power pole",
            Self::Substation => "substation",
            Self::BatteryBank => "battery bank",
            Self::Accumulator => "accumulator",
            Self::OutputBin => "output bin",
            Self::Warehouse => "warehouse",
            Self::SiloHopper => "silo hopper",
            Self::CryoTank => "cryo tank",
            Self::ContainmentVault => "containment vault",
            Self::Wall => "wall",
            Self::ReinforcedWall => "reinforced wall",
            Self::Turret => "turret",
            Self::ShieldGenerator => "shield generator",
            Self::WasteDump => "waste dump",
            Self::RecyclingPlant => "recycling plant",
            Self::IncinerationPlant => "incineration plant",
            Self::FilterStack => "filter stack",
            Self::ScrubberUnit => "scrubber unit",
            Self::ContainmentField => "containment field",
            Self::ResearchLab => "research lab",
            Self::AdvancedLab => "advanced lab",
            Self::SpaceElevatorBase => "space elevator base",
            Self::DysonSwarmLauncher => "dyson swarm launcher",
            Self::WarpGateFrame => "warp gate frame",
        }
    }

    pub fn is_player_placeable(&self) -> bool {
        !matches!(
            self.building_kind(),
            BuildingKind::Extractor | BuildingKind::Victory
        ) && *self != Self::OutputBin
    }

    pub fn has_facing(&self) -> bool {
        matches!(
            self.building_kind(),
            BuildingKind::Processor1x1
                | BuildingKind::Processor1x3
                | BuildingKind::Processor1x5
                | BuildingKind::Processor1x7
                | BuildingKind::Processor1x9
                | BuildingKind::Belt
                | BuildingKind::SplitterKind
                | BuildingKind::MergerKind
                | BuildingKind::PipeTransport
                | BuildingKind::PowerGenerator
                | BuildingKind::Research
        )
    }

    pub fn glyph(&self) -> char {
        match self {
            // Original types
            Self::Smelter => 'S',
            Self::Assembler => 'A',
            Self::BasicBelt | Self::FastBelt | Self::ExpressBelt => '>',
            Self::Splitter => 'Y',
            Self::Merger => '\u{03BB}',
            Self::OutputBin => 'B',
            Self::Wall => '\u{2588}',
            Self::ReinforcedWall => '\u{2588}',
            // Extractors
            Self::OreDeposit | Self::CopperDeposit | Self::CoalDeposit
            | Self::OilWell | Self::WaterPump | Self::GasExtractor
            | Self::UraniumMine | Self::SandExtractor | Self::SulfurMine
            | Self::BauxiteMine | Self::LithiumExtractor | Self::RareEarthExtractor
            | Self::BiomassHarvester | Self::GeothermalTap => 'O',
            Self::StoneQuarry => 'Q',
            // 1x1 processors
            Self::Kiln => 'K', Self::Press => 'P', Self::WireMill => 'W',
            Self::PlateMachine => 'p', Self::RubberVulcanizer => 'V',
            Self::PlasticMolder => 'D', Self::Electrolyzer => 'E',
            Self::Caster => 'C', Self::CokeFurnace => 'F',
            Self::Gasifier => 'G', Self::Boiler => 'b', Self::WaferCutter => 'X',
            // 1x3+ processors
            Self::Mixer => 'M', Self::ChemicalPlant => 'C',
            Self::CircuitFabricator => '#', Self::MotorAssembly => 'A',
            Self::CrushingMill => 'C',
            Self::AdvancedAssembler | Self::PrecisionAssembler
            | Self::Megassembler => 'A',
            Self::Refinery => 'R', Self::CrackingTower => 'T',
            Self::Cleanroom => 'N', Self::EnrichmentCascade => 'E',
            Self::CoolantProcessor => 'C',
            Self::QuantumLab => 'Q', Self::RocketAssembly => 'R',
            Self::SingularityLab => 'S',
            // Transport (non-belt)
            Self::UndergroundEntrance => 'U', Self::UndergroundExit => 'u',
            Self::Pipe => '-', Self::PipeJunction => '+',
            Self::PumpStation => 'P', Self::FluidTank => 'T',
            Self::GasCompressor => 'G', Self::GasPipeline => '=',
            Self::RailTrack => '=', Self::TrainStation => 'T',
            Self::DronePort => 'D',
            // Power
            Self::CoalGenerator | Self::GasGenerator => 'G',
            Self::SolarArray => 'S', Self::WindTurbine => 'W',
            Self::GeothermalPlant => 'G', Self::NuclearReactor => 'N',
            Self::FusionReactor => 'F', Self::Transformer => 'T',
            Self::PowerPole => '|', Self::Substation => '#',
            Self::BatteryBank | Self::Accumulator => 'B',
            // Storage
            Self::Warehouse => 'W', Self::SiloHopper => 'H',
            Self::CryoTank => 'C', Self::ContainmentVault => 'V',
            // Defense
            Self::Turret => 'T', Self::ShieldGenerator => 'S',
            // Environmental
            Self::WasteDump => 'D', Self::RecyclingPlant => 'R',
            Self::IncinerationPlant => 'I', Self::FilterStack => 'F',
            Self::ScrubberUnit => 'S', Self::ContainmentField => 'C',
            // Research
            Self::ResearchLab | Self::AdvancedLab => 'L',
            // Victory
            Self::SpaceElevatorBase => 'E', Self::DysonSwarmLauncher => 'D',
            Self::WarpGateFrame => 'W',
        }
    }

    pub fn color(&self) -> (u8, u8, u8) {
        match self {
            // Original types (match existing render colors)
            Self::OreDeposit => (139, 119, 42),
            Self::Smelter => (255, 80, 80),
            Self::Assembler => (80, 200, 200),
            Self::BasicBelt => (200, 200, 200),
            Self::Splitter => (255, 255, 50),
            Self::Merger => (255, 255, 50),
            Self::OutputBin => (80, 220, 80),
            Self::Wall => (120, 120, 120),
            // Extractors
            Self::CopperDeposit => (210, 120, 50),
            Self::CoalDeposit => (60, 60, 60),
            Self::StoneQuarry => (150, 150, 140),
            Self::OilWell => (30, 30, 30),
            Self::WaterPump => (60, 120, 200),
            Self::GasExtractor => (180, 200, 220),
            Self::UraniumMine => (80, 220, 80),
            Self::SandExtractor => (220, 210, 170),
            Self::SulfurMine => (220, 220, 50),
            Self::BauxiteMine => (200, 100, 80),
            Self::LithiumExtractor => (200, 230, 255),
            Self::RareEarthExtractor => (180, 100, 180),
            Self::BiomassHarvester => (50, 180, 50),
            Self::GeothermalTap => (200, 100, 40),
            // 1x1 Processors
            Self::Kiln => (220, 150, 50),
            Self::Press => (180, 180, 200),
            Self::WireMill => (220, 160, 40),
            Self::PlateMachine => (190, 190, 200),
            Self::RubberVulcanizer => (80, 80, 80),
            Self::PlasticMolder => (240, 240, 240),
            Self::Electrolyzer => (100, 180, 220),
            Self::Caster => (200, 120, 40),
            Self::CokeFurnace => (160, 80, 30),
            Self::Gasifier => (150, 180, 200),
            Self::Boiler => (200, 100, 60),
            Self::WaferCutter => (100, 100, 150),
            // 1x3 Processors
            Self::Mixer => (100, 160, 200),
            Self::ChemicalPlant => (200, 200, 50),
            Self::CircuitFabricator => (80, 220, 120),
            Self::MotorAssembly => (180, 180, 200),
            Self::CrushingMill => (160, 140, 120),
            // 1x5
            Self::AdvancedAssembler => (100, 180, 220),
            Self::Refinery => (200, 160, 80),
            Self::CrackingTower => (180, 140, 100),
            Self::Cleanroom => (220, 220, 240),
            Self::EnrichmentCascade => (80, 220, 80),
            Self::CoolantProcessor => (100, 200, 255),
            // 1x7
            Self::PrecisionAssembler => (120, 160, 220),
            Self::QuantumLab => (150, 100, 255),
            Self::RocketAssembly => (255, 100, 50),
            // 1x9
            Self::Megassembler => (100, 200, 255),
            Self::SingularityLab => (200, 50, 255),
            // Transport
            Self::FastBelt => (255, 200, 50),
            Self::ExpressBelt => (100, 200, 255),
            Self::UndergroundEntrance | Self::UndergroundExit => (180, 180, 180),
            Self::Pipe | Self::PipeJunction => (60, 120, 200),
            Self::PumpStation => (80, 150, 220),
            Self::FluidTank => (60, 140, 220),
            Self::GasCompressor | Self::GasPipeline => (180, 200, 220),
            Self::RailTrack => (140, 100, 60),
            Self::TrainStation => (160, 120, 80),
            Self::DronePort => (100, 200, 200),
            // Power
            Self::CoalGenerator => (200, 100, 50),
            Self::GasGenerator => (180, 160, 100),
            Self::SolarArray => (255, 220, 50),
            Self::WindTurbine => (200, 220, 240),
            Self::GeothermalPlant => (200, 100, 40),
            Self::NuclearReactor => (80, 220, 80),
            Self::FusionReactor => (200, 150, 255),
            Self::Transformer => (255, 200, 50),
            Self::PowerPole => (140, 100, 60),
            Self::Substation => (200, 180, 50),
            Self::BatteryBank | Self::Accumulator => (100, 200, 100),
            // Storage
            Self::Warehouse => (180, 160, 120),
            Self::SiloHopper => (200, 200, 180),
            Self::CryoTank => (100, 200, 255),
            Self::ContainmentVault => (150, 150, 180),
            // Defense
            Self::ReinforcedWall => (160, 160, 160),
            Self::Turret => (200, 50, 50),
            Self::ShieldGenerator => (100, 150, 255),
            // Environmental
            Self::WasteDump => (120, 100, 80),
            Self::RecyclingPlant => (80, 200, 80),
            Self::IncinerationPlant => (200, 100, 50),
            Self::FilterStack => (160, 180, 200),
            Self::ScrubberUnit => (100, 160, 180),
            Self::ContainmentField => (200, 200, 50),
            // Research
            Self::ResearchLab => (200, 100, 200),
            Self::AdvancedLab => (255, 100, 255),
            // Victory
            Self::SpaceElevatorBase => (255, 215, 0),
            Self::DysonSwarmLauncher => (255, 200, 50),
            Self::WarpGateFrame => (200, 100, 255),
        }
    }

    pub fn tier(&self) -> u8 {
        match self.building_kind() {
            BuildingKind::Extractor => 0,
            BuildingKind::Processor1x1 | BuildingKind::Belt
            | BuildingKind::SplitterKind | BuildingKind::MergerKind => 1,
            BuildingKind::Processor1x3 | BuildingKind::PipeTransport
            | BuildingKind::Storage | BuildingKind::Defense
            | BuildingKind::PowerGenerator => 2,
            BuildingKind::Processor1x5 | BuildingKind::Environmental
            | BuildingKind::PowerDistribution | BuildingKind::RailTransport
            | BuildingKind::Research => 3,
            BuildingKind::Processor1x7 => 4,
            BuildingKind::Processor1x9 | BuildingKind::Victory => 5,
        }
    }

    pub fn power_required(&self) -> u32 {
        match self.building_kind() {
            BuildingKind::Extractor => 5,
            BuildingKind::Processor1x1 => 10,
            BuildingKind::Processor1x3 => 25,
            BuildingKind::Processor1x5 => 50,
            BuildingKind::Processor1x7 => 100,
            BuildingKind::Processor1x9 => 200,
            BuildingKind::Belt | BuildingKind::SplitterKind
            | BuildingKind::MergerKind => 2,
            BuildingKind::PipeTransport => 5,
            BuildingKind::RailTransport => 10,
            BuildingKind::PowerGenerator | BuildingKind::PowerDistribution => 0,
            BuildingKind::Storage => 3,
            BuildingKind::Defense => 15,
            BuildingKind::Environmental => 20,
            BuildingKind::Research => 50,
            BuildingKind::Victory => 500,
        }
    }

    pub fn operating_cost(&self) -> f64 {
        match self.building_kind() {
            BuildingKind::Extractor => 1.0,
            BuildingKind::Processor1x1 => 2.0,
            BuildingKind::Processor1x3 => 5.0,
            BuildingKind::Processor1x5 => 10.0,
            BuildingKind::Processor1x7 => 25.0,
            BuildingKind::Processor1x9 => 50.0,
            BuildingKind::Belt => 0.1,
            BuildingKind::SplitterKind | BuildingKind::MergerKind => 0.5,
            BuildingKind::PipeTransport => 0.5,
            BuildingKind::RailTransport => 2.0,
            BuildingKind::PowerGenerator => 5.0,
            BuildingKind::PowerDistribution => 0.5,
            BuildingKind::Storage => 0.2,
            BuildingKind::Defense => 3.0,
            BuildingKind::Environmental => 4.0,
            BuildingKind::Research => 10.0,
            BuildingKind::Victory => 100.0,
        }
    }
}

// ---------------------------------------------------------------------------
// Facing (unchanged)
// ---------------------------------------------------------------------------

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Facing {
    Up,
    Right,
    Down,
    Left,
}

impl Facing {
    pub fn opposite(&self) -> Facing {
        match self {
            Facing::Up => Facing::Down,
            Facing::Down => Facing::Up,
            Facing::Left => Facing::Right,
            Facing::Right => Facing::Left,
        }
    }

    pub fn rotate_cw(&self) -> Facing {
        match self {
            Facing::Up => Facing::Right,
            Facing::Right => Facing::Down,
            Facing::Down => Facing::Left,
            Facing::Left => Facing::Up,
        }
    }

    pub fn rotate_ccw(&self) -> Facing {
        match self {
            Facing::Up => Facing::Left,
            Facing::Left => Facing::Down,
            Facing::Down => Facing::Right,
            Facing::Right => Facing::Up,
        }
    }

    pub fn offset(&self) -> (isize, isize) {
        match self {
            Facing::Up => (0, -1),
            Facing::Down => (0, 1),
            Facing::Left => (-1, 0),
            Facing::Right => (1, 0),
        }
    }

    pub fn perpendicular(&self) -> (Facing, Facing) {
        match self {
            Facing::Up | Facing::Down => (Facing::Left, Facing::Right),
            Facing::Left | Facing::Right => (Facing::Up, Facing::Down),
        }
    }

    pub fn arrow_glyph(&self) -> char {
        match self {
            Facing::Up => '\u{2191}',
            Facing::Down => '\u{2193}',
            Facing::Left => '\u{2190}',
            Facing::Right => '\u{2192}',
        }
    }

    pub fn all() -> [Facing; 4] {
        [Facing::Up, Facing::Right, Facing::Down, Facing::Left]
    }
}

// ---------------------------------------------------------------------------
// Direction (unchanged)
// ---------------------------------------------------------------------------

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Direction {
    Up,
    Down,
    Left,
    Right,
}

impl Direction {
    pub fn to_facing(&self) -> Facing {
        match self {
            Direction::Up => Facing::Up,
            Direction::Down => Facing::Down,
            Direction::Left => Facing::Left,
            Direction::Right => Facing::Right,
        }
    }
}

impl From<Facing> for Direction {
    fn from(f: Facing) -> Self {
        match f {
            Facing::Up => Direction::Up,
            Facing::Down => Direction::Down,
            Facing::Left => Direction::Left,
            Facing::Right => Direction::Right,
        }
    }
}

// ---------------------------------------------------------------------------
// Input/output side helpers
// ---------------------------------------------------------------------------

pub fn get_input_sides(entity_type: EntityType, facing: Facing) -> Vec<Facing> {
    match entity_type.building_kind() {
        BuildingKind::Extractor => vec![],

        BuildingKind::Processor1x1 | BuildingKind::PowerGenerator
        | BuildingKind::Research => vec![facing.opposite()],

        BuildingKind::Processor1x3 | BuildingKind::Processor1x5
        | BuildingKind::Processor1x7 | BuildingKind::Processor1x9
        | BuildingKind::MergerKind => {
            let (a, b) = facing.perpendicular();
            vec![a, b]
        }

        BuildingKind::Belt => {
            let opp = facing.opposite();
            let (a, b) = facing.perpendicular();
            vec![opp, a, b]
        }

        BuildingKind::SplitterKind => vec![facing.opposite()],

        BuildingKind::PipeTransport => {
            let opp = facing.opposite();
            let (a, b) = facing.perpendicular();
            vec![opp, a, b]
        }

        BuildingKind::Storage | BuildingKind::Environmental => Facing::all().to_vec(),

        BuildingKind::Defense | BuildingKind::PowerDistribution
        | BuildingKind::RailTransport | BuildingKind::Victory => vec![],
    }
}

pub fn get_output_sides(entity_type: EntityType, facing: Facing) -> Vec<Facing> {
    match entity_type.building_kind() {
        BuildingKind::Extractor => Facing::all().to_vec(),

        BuildingKind::Processor1x1 | BuildingKind::Processor1x3
        | BuildingKind::Processor1x5 | BuildingKind::Processor1x7
        | BuildingKind::Processor1x9 | BuildingKind::MergerKind => vec![facing],

        BuildingKind::Belt | BuildingKind::PipeTransport => vec![facing],

        BuildingKind::SplitterKind => {
            let (a, b) = facing.perpendicular();
            vec![a, b]
        }

        BuildingKind::Storage | BuildingKind::Defense | BuildingKind::Environmental
        | BuildingKind::PowerGenerator | BuildingKind::PowerDistribution
        | BuildingKind::RailTransport | BuildingKind::Research
        | BuildingKind::Victory => vec![],
    }
}
