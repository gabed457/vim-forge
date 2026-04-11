use std::collections::{HashMap, HashSet};

use serde::{Deserialize, Serialize};

use crate::resources::{EntityType, Resource};

// ---------------------------------------------------------------------------
// TechId
// ---------------------------------------------------------------------------

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum TechId {
    // Tier 1 — Red Science only
    BasicSmelting,
    BasicBelts,
    Automation1,
    BasicMining,
    StoneworkProcessing,
    BasicPipes,
    CoalPower,
    WasteManagement1,
    BasicLogistics,
    // Tier 2 — Red + Green
    AdvancedSmelting,
    FastBelts,
    Automation2,
    FluidProcessing,
    OilRefining,
    GasPower,
    SolarPower,
    WindPower,
    CircuitNetworks,
    BasicChemistry,
    TrainTransport,
    WasteManagement2,
    AdvancedMining,
    StorageSystems,
    // Tier 3 — Red + Green + Blue
    AdvancedCircuits,
    ExpressBelts,
    Automation3,
    PrecisionManufacturing,
    NuclearPower,
    Cryogenics,
    AdvancedChemistry,
    TruckTransport,
    DroneLogistics,
    WasteManagement3,
    Composites,
    BatteryTech,
    // Tier 4 — Red + Green + Blue + Purple
    QuantumComputing,
    FusionPower,
    Nanotechnology,
    SpaceScience,
    AdvancedLogistics,
    AirTransport,
    WasteManagement4,
    Superconductors,
    QuantumBelts,
    // Tier 5 — All 5 packs
    AntimatterTech,
    DimensionalEngineering,
    GravityManipulation,
    WarpDrive,
    DysonSphere,
    SpaceElevator,
    Terraforming,
    // Infinite research
    MiningProductivity,
    BeltSpeed,
    MachineSpeed,
    EnergyEfficiency,
    PollutionReduction,
    ResearchSpeed,
}

// ---------------------------------------------------------------------------
// Technology
// ---------------------------------------------------------------------------

#[derive(Clone, Debug)]
pub struct Technology {
    pub id: TechId,
    pub name: &'static str,
    pub description: &'static str,
    pub tier: u8,
    pub science_cost: Vec<(Resource, u64)>,
    pub prerequisites: Vec<TechId>,
    pub unlocks_buildings: Vec<EntityType>,
    pub unlocks_recipes: Vec<u16>,
    pub cash_grant: u64,
    pub research_time: u32,
    pub is_infinite: bool,
    pub infinite_cost_multiplier: f64,
}

// ---------------------------------------------------------------------------
// ResearchState
// ---------------------------------------------------------------------------

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ResearchState {
    pub completed: HashSet<TechId>,
    pub current: Option<TechId>,
    pub progress: u32,
    pub infinite_levels: HashMap<TechId, u32>,
    pub queued: Vec<TechId>,
}

impl ResearchState {
    pub fn new() -> Self {
        Self {
            completed: HashSet::new(),
            current: None,
            progress: 0,
            infinite_levels: HashMap::new(),
            queued: Vec::new(),
        }
    }

    /// Start researching the given tech. Returns false if prerequisites are not met.
    pub fn start_research(&mut self, id: TechId) -> bool {
        let tech = get_tech(id);
        if !tech.is_infinite && self.completed.contains(&id) {
            return false;
        }
        if !is_available(id, &self.completed) {
            return false;
        }
        self.current = Some(id);
        self.progress = 0;
        true
    }

    /// Advance research by `ticks` amount. Returns Some(TechId) if research completed.
    pub fn tick(&mut self, ticks: u32) -> Option<TechId> {
        let id = self.current?;
        let tech = get_tech(id);
        let needed = self.effective_research_time(&tech);
        self.progress += ticks;
        if self.progress >= needed {
            return Some(self.complete_current());
        }
        None
    }

    /// Complete current research, mark it done, advance queue.
    fn complete_current(&mut self) -> TechId {
        let id = self.current.take().expect("no current research");
        let tech = get_tech(id);
        if tech.is_infinite {
            let level = self.infinite_levels.entry(id).or_insert(0);
            *level += 1;
        } else {
            self.completed.insert(id);
        }
        self.progress = 0;
        // Start next queued research if any
        while let Some(next) = self.queued.first().copied() {
            self.queued.remove(0);
            if self.start_research(next) {
                break;
            }
        }
        id
    }

    /// Effective research time accounting for infinite scaling.
    fn effective_research_time(&self, tech: &Technology) -> u32 {
        if tech.is_infinite {
            let level = self.infinite_levels.get(&tech.id).copied().unwrap_or(0);
            let multiplier = (1.0 + tech.infinite_cost_multiplier).powi(level as i32);
            (tech.research_time as f64 * multiplier) as u32
        } else {
            tech.research_time
        }
    }

    /// Check if a building is unlocked by any completed tech.
    pub fn is_building_unlocked(&self, entity_type: EntityType) -> bool {
        for tech in get_all_techs() {
            if tech.unlocks_buildings.contains(&entity_type) && self.completed.contains(&tech.id) {
                return true;
            }
        }
        false
    }

    /// Check if a recipe ID is unlocked by any completed tech.
    pub fn is_recipe_unlocked(&self, recipe_id: u16) -> bool {
        for tech in get_all_techs() {
            if tech.unlocks_recipes.contains(&recipe_id) && self.completed.contains(&tech.id) {
                return true;
            }
        }
        false
    }

    /// Queue a tech for research after the current one finishes.
    pub fn queue_research(&mut self, id: TechId) {
        if !self.queued.contains(&id) {
            self.queued.push(id);
        }
    }

    /// Get the progress fraction (0.0 - 1.0) of current research.
    pub fn progress_fraction(&self) -> f64 {
        let id = match self.current {
            Some(id) => id,
            None => return 0.0,
        };
        let tech = get_tech(id);
        let needed = self.effective_research_time(&tech);
        if needed == 0 {
            return 1.0;
        }
        self.progress as f64 / needed as f64
    }

    /// Get infinite research level for a tech.
    pub fn infinite_level(&self, id: TechId) -> u32 {
        self.infinite_levels.get(&id).copied().unwrap_or(0)
    }
}

impl Default for ResearchState {
    fn default() -> Self {
        Self::new()
    }
}

// ---------------------------------------------------------------------------
// Tech Tree — all 57 technologies
// ---------------------------------------------------------------------------

/// Recipe ID constants for cross-referencing with recipe system.
pub mod recipe_ids {
    pub const STONE_BRICK: u16 = 1;
    pub const SULFURIC_ACID: u16 = 2;
    pub const CONCRETE: u16 = 3;
    pub const CHARCOAL: u16 = 4;
    pub const PROCESSOR: u16 = 5;
    pub const ADVANCED_CIRCUIT: u16 = 6;
    pub const COOLANT: u16 = 7;
    pub const COMPOSITE_PANEL: u16 = 8;
    pub const FIBERGLASS: u16 = 9;
    pub const BATTERY_PACK: u16 = 10;
    pub const LITHIUM_CELL: u16 = 11;
    pub const QUANTUM_PROCESSOR: u16 = 12;
    pub const FUSION_CORE: u16 = 13;
    pub const NANOBOT_SWARM: u16 = 14;
    pub const GRAVITY_LENS: u16 = 15;
    pub const WARP_DRIVE_MODULE: u16 = 16;
    pub const WARP_GATE_COMPONENT: u16 = 17;
    pub const DYSON_SPHERE_PANEL: u16 = 18;
    pub const DYSON_SWARM_CLUSTER: u16 = 19;
    pub const SPACE_ELEVATOR_CABLE: u16 = 20;
    pub const SPACE_ELEVATOR_SEGMENT: u16 = 21;
    pub const TERRAFORMING_AGENT: u16 = 22;
    pub const SCIENCE_PACK_1: u16 = 100;
    pub const SCIENCE_PACK_2: u16 = 101;
    pub const SCIENCE_PACK_3: u16 = 102;
    pub const SCIENCE_PACK_4: u16 = 103;
    pub const SCIENCE_PACK_5: u16 = 104;
}

/// Get the full tech tree as a vector of all technologies.
pub fn get_all_techs() -> Vec<Technology> {
    use EntityType::*;
    use Resource::*;
    use TechId::*;

    vec![
        // =================================================================
        // TIER 1 — Red Science (SciencePack1) only
        // =================================================================
        Technology {
            id: BasicSmelting,
            name: "Basic Smelting",
            description: "Unlock furnaces for ore processing",
            tier: 1,
            science_cost: vec![(SciencePack1, 10)],
            prerequisites: vec![],
            unlocks_buildings: vec![Smelter, Kiln],
            unlocks_recipes: vec![],
            cash_grant: 500,
            research_time: 120,
            is_infinite: false,
            infinite_cost_multiplier: 0.0,
        },
        Technology {
            id: BasicBelts,
            name: "Basic Belts",
            description: "Unlock basic conveyor belts for item transport",
            tier: 1,
            science_cost: vec![(SciencePack1, 10)],
            prerequisites: vec![],
            unlocks_buildings: vec![BasicBelt],
            unlocks_recipes: vec![],
            cash_grant: 200,
            research_time: 120,
            is_infinite: false,
            infinite_cost_multiplier: 0.0,
        },
        Technology {
            id: TechId::Automation1,
            name: "Automation I",
            description: "Unlock assemblers and basic processing machines",
            tier: 1,
            science_cost: vec![(SciencePack1, 15)],
            prerequisites: vec![BasicSmelting],
            unlocks_buildings: vec![Assembler, WireMill, Press],
            unlocks_recipes: vec![recipe_ids::SCIENCE_PACK_1],
            cash_grant: 1000,
            research_time: 180,
            is_infinite: false,
            infinite_cost_multiplier: 0.0,
        },
        Technology {
            id: BasicMining,
            name: "Basic Mining",
            description: "Improve extraction efficiency from ore deposits",
            tier: 1,
            science_cost: vec![(SciencePack1, 10)],
            prerequisites: vec![],
            unlocks_buildings: vec![],
            unlocks_recipes: vec![],
            cash_grant: 300,
            research_time: 120,
            is_infinite: false,
            infinite_cost_multiplier: 0.0,
        },
        Technology {
            id: StoneworkProcessing,
            name: "Stonework Processing",
            description: "Unlock stone brick production",
            tier: 1,
            science_cost: vec![(SciencePack1, 10)],
            prerequisites: vec![BasicSmelting],
            unlocks_buildings: vec![],
            unlocks_recipes: vec![recipe_ids::STONE_BRICK],
            cash_grant: 200,
            research_time: 100,
            is_infinite: false,
            infinite_cost_multiplier: 0.0,
        },
        Technology {
            id: BasicPipes,
            name: "Basic Pipes",
            description: "Unlock fluid transport infrastructure",
            tier: 1,
            science_cost: vec![(SciencePack1, 15)],
            prerequisites: vec![],
            unlocks_buildings: vec![Pipe, PumpStation],
            unlocks_recipes: vec![],
            cash_grant: 400,
            research_time: 150,
            is_infinite: false,
            infinite_cost_multiplier: 0.0,
        },
        Technology {
            id: CoalPower,
            name: "Coal Power",
            description: "Unlock coal generators and power distribution",
            tier: 1,
            science_cost: vec![(SciencePack1, 10)],
            prerequisites: vec![BasicMining],
            unlocks_buildings: vec![CoalGenerator, PowerPole],
            unlocks_recipes: vec![],
            cash_grant: 500,
            research_time: 120,
            is_infinite: false,
            infinite_cost_multiplier: 0.0,
        },
        Technology {
            id: TechId::WasteManagement1,
            name: "Waste Management I",
            description: "Unlock basic waste disposal facilities",
            tier: 1,
            science_cost: vec![(SciencePack1, 10)],
            prerequisites: vec![BasicSmelting],
            unlocks_buildings: vec![WasteDump, FilterStack, ScrubberUnit],
            unlocks_recipes: vec![],
            cash_grant: 200,
            research_time: 100,
            is_infinite: false,
            infinite_cost_multiplier: 0.0,
        },
        Technology {
            id: TechId::BasicLogistics,
            name: "Basic Logistics",
            description: "Unlock item routing and storage buildings",
            tier: 1,
            science_cost: vec![(SciencePack1, 15)],
            prerequisites: vec![BasicBelts],
            unlocks_buildings: vec![Splitter, Merger, OutputBin],
            unlocks_recipes: vec![],
            cash_grant: 300,
            research_time: 150,
            is_infinite: false,
            infinite_cost_multiplier: 0.0,
        },

        // =================================================================
        // TIER 2 — Red + Green (SciencePack1 + SciencePack2)
        // =================================================================
        Technology {
            id: AdvancedSmelting,
            name: "Advanced Smelting",
            description: "Unlock advanced metal processing: caster, electrolyzer, plate machine",
            tier: 2,
            science_cost: vec![(SciencePack1, 20), (SciencePack2, 20)],
            prerequisites: vec![BasicSmelting],
            unlocks_buildings: vec![Caster, Electrolyzer, PlateMachine],
            unlocks_recipes: vec![],
            cash_grant: 1000,
            research_time: 240,
            is_infinite: false,
            infinite_cost_multiplier: 0.0,
        },
        Technology {
            id: TechId::FastBelts,
            name: "Fast Belts",
            description: "Unlock faster conveyor belts",
            tier: 2,
            science_cost: vec![(SciencePack1, 20), (SciencePack2, 15)],
            prerequisites: vec![BasicBelts, TechId::Automation1],
            unlocks_buildings: vec![FastBelt],
            unlocks_recipes: vec![],
            cash_grant: 500,
            research_time: 200,
            is_infinite: false,
            infinite_cost_multiplier: 0.0,
        },
        Technology {
            id: TechId::Automation2,
            name: "Automation II",
            description: "Unlock advanced assemblers and mixing",
            tier: 2,
            science_cost: vec![(SciencePack1, 25), (SciencePack2, 25)],
            prerequisites: vec![TechId::Automation1],
            unlocks_buildings: vec![AdvancedAssembler, Mixer],
            unlocks_recipes: vec![recipe_ids::SCIENCE_PACK_2],
            cash_grant: 2000,
            research_time: 300,
            is_infinite: false,
            infinite_cost_multiplier: 0.0,
        },
        Technology {
            id: TechId::FluidProcessing,
            name: "Fluid Processing",
            description: "Unlock chemical plants, pipe junctions, and boilers",
            tier: 2,
            science_cost: vec![(SciencePack1, 20), (SciencePack2, 20)],
            prerequisites: vec![BasicPipes],
            unlocks_buildings: vec![PipeJunction, ChemicalPlant, Boiler],
            unlocks_recipes: vec![],
            cash_grant: 800,
            research_time: 240,
            is_infinite: false,
            infinite_cost_multiplier: 0.0,
        },
        Technology {
            id: TechId::OilRefining,
            name: "Oil Refining",
            description: "Unlock petroleum processing infrastructure",
            tier: 2,
            science_cost: vec![(SciencePack1, 25), (SciencePack2, 20)],
            prerequisites: vec![TechId::FluidProcessing],
            unlocks_buildings: vec![Refinery, Gasifier],
            unlocks_recipes: vec![],
            cash_grant: 1500,
            research_time: 280,
            is_infinite: false,
            infinite_cost_multiplier: 0.0,
        },
        Technology {
            id: TechId::GasPower,
            name: "Gas Power",
            description: "Unlock gas generators for cleaner power",
            tier: 2,
            science_cost: vec![(SciencePack1, 15), (SciencePack2, 15)],
            prerequisites: vec![CoalPower, TechId::OilRefining],
            unlocks_buildings: vec![GasGenerator],
            unlocks_recipes: vec![],
            cash_grant: 1000,
            research_time: 180,
            is_infinite: false,
            infinite_cost_multiplier: 0.0,
        },
        Technology {
            id: TechId::SolarPower,
            name: "Solar Power",
            description: "Unlock renewable solar energy",
            tier: 2,
            science_cost: vec![(SciencePack1, 20), (SciencePack2, 20)],
            prerequisites: vec![CoalPower],
            unlocks_buildings: vec![SolarArray],
            unlocks_recipes: vec![],
            cash_grant: 800,
            research_time: 240,
            is_infinite: false,
            infinite_cost_multiplier: 0.0,
        },
        Technology {
            id: TechId::WindPower,
            name: "Wind Power",
            description: "Unlock wind turbine generators",
            tier: 2,
            science_cost: vec![(SciencePack1, 15), (SciencePack2, 15)],
            prerequisites: vec![CoalPower],
            unlocks_buildings: vec![WindTurbine],
            unlocks_recipes: vec![],
            cash_grant: 600,
            research_time: 180,
            is_infinite: false,
            infinite_cost_multiplier: 0.0,
        },
        Technology {
            id: TechId::CircuitNetworks,
            name: "Circuit Networks",
            description: "Unlock circuit fabrication and logic-controlled automation",
            tier: 2,
            science_cost: vec![(SciencePack1, 20), (SciencePack2, 20)],
            prerequisites: vec![TechId::Automation1],
            unlocks_buildings: vec![CircuitFabricator],
            unlocks_recipes: vec![],
            cash_grant: 500,
            research_time: 240,
            is_infinite: false,
            infinite_cost_multiplier: 0.0,
        },
        Technology {
            id: TechId::BasicChemistry,
            name: "Basic Chemistry",
            description: "Unlock sulfuric acid, concrete, and charcoal recipes",
            tier: 2,
            science_cost: vec![(SciencePack1, 20), (SciencePack2, 15)],
            prerequisites: vec![TechId::FluidProcessing],
            unlocks_buildings: vec![CokeFurnace],
            unlocks_recipes: vec![
                recipe_ids::SULFURIC_ACID,
                recipe_ids::CONCRETE,
                recipe_ids::CHARCOAL,
            ],
            cash_grant: 600,
            research_time: 200,
            is_infinite: false,
            infinite_cost_multiplier: 0.0,
        },
        Technology {
            id: TechId::TrainTransport,
            name: "Train Transport",
            description: "Unlock rail networks for bulk transport",
            tier: 2,
            science_cost: vec![(SciencePack1, 25), (SciencePack2, 25)],
            prerequisites: vec![TechId::BasicLogistics],
            unlocks_buildings: vec![RailTrack, TrainStation],
            unlocks_recipes: vec![],
            cash_grant: 2000,
            research_time: 300,
            is_infinite: false,
            infinite_cost_multiplier: 0.0,
        },
        Technology {
            id: TechId::WasteManagement2,
            name: "Waste Management II",
            description: "Unlock recycling plants for waste recovery",
            tier: 2,
            science_cost: vec![(SciencePack1, 15), (SciencePack2, 15)],
            prerequisites: vec![TechId::WasteManagement1],
            unlocks_buildings: vec![RecyclingPlant],
            unlocks_recipes: vec![],
            cash_grant: 500,
            research_time: 180,
            is_infinite: false,
            infinite_cost_multiplier: 0.0,
        },
        Technology {
            id: TechId::AdvancedMining,
            name: "Advanced Mining",
            description: "Unlock gas extraction and improved mining techniques",
            tier: 2,
            science_cost: vec![(SciencePack1, 20), (SciencePack2, 15)],
            prerequisites: vec![BasicMining],
            unlocks_buildings: vec![GasExtractor, GasCompressor],
            unlocks_recipes: vec![],
            cash_grant: 800,
            research_time: 200,
            is_infinite: false,
            infinite_cost_multiplier: 0.0,
        },
        Technology {
            id: TechId::StorageSystems,
            name: "Storage Systems",
            description: "Unlock large-scale storage for solids, fluids, and gases",
            tier: 2,
            science_cost: vec![(SciencePack1, 15), (SciencePack2, 10)],
            prerequisites: vec![TechId::BasicLogistics],
            unlocks_buildings: vec![Warehouse, FluidTank, GasPipeline],
            unlocks_recipes: vec![],
            cash_grant: 400,
            research_time: 150,
            is_infinite: false,
            infinite_cost_multiplier: 0.0,
        },

        // =================================================================
        // TIER 3 — Red + Green + Blue (SP1 + SP2 + SP3)
        // =================================================================
        Technology {
            id: TechId::AdvancedCircuits,
            name: "Advanced Circuits",
            description: "Unlock processor and advanced circuit fabrication",
            tier: 3,
            science_cost: vec![(SciencePack1, 30), (SciencePack2, 30), (SciencePack3, 20)],
            prerequisites: vec![TechId::CircuitNetworks, TechId::Automation2],
            unlocks_buildings: vec![],
            unlocks_recipes: vec![recipe_ids::PROCESSOR, recipe_ids::ADVANCED_CIRCUIT],
            cash_grant: 3000,
            research_time: 400,
            is_infinite: false,
            infinite_cost_multiplier: 0.0,
        },
        Technology {
            id: TechId::ExpressBelts,
            name: "Express Belts",
            description: "Unlock high-speed express conveyor belts",
            tier: 3,
            science_cost: vec![(SciencePack1, 25), (SciencePack2, 20), (SciencePack3, 15)],
            prerequisites: vec![TechId::FastBelts],
            unlocks_buildings: vec![ExpressBelt],
            unlocks_recipes: vec![],
            cash_grant: 1000,
            research_time: 300,
            is_infinite: false,
            infinite_cost_multiplier: 0.0,
        },
        Technology {
            id: TechId::Automation3,
            name: "Automation III",
            description: "Unlock precision assemblers and cleanroom manufacturing",
            tier: 3,
            science_cost: vec![(SciencePack1, 30), (SciencePack2, 30), (SciencePack3, 25)],
            prerequisites: vec![TechId::Automation2, TechId::AdvancedCircuits],
            unlocks_buildings: vec![PrecisionAssembler, Cleanroom],
            unlocks_recipes: vec![recipe_ids::SCIENCE_PACK_3],
            cash_grant: 5000,
            research_time: 450,
            is_infinite: false,
            infinite_cost_multiplier: 0.0,
        },
        Technology {
            id: TechId::PrecisionManufacturing,
            name: "Precision Manufacturing",
            description: "Unlock cracking towers, enrichment cascades, and wafer cutting",
            tier: 3,
            science_cost: vec![(SciencePack1, 25), (SciencePack2, 25), (SciencePack3, 20)],
            prerequisites: vec![TechId::Automation2, TechId::BasicChemistry],
            unlocks_buildings: vec![CrackingTower, EnrichmentCascade, WaferCutter],
            unlocks_recipes: vec![],
            cash_grant: 2000,
            research_time: 350,
            is_infinite: false,
            infinite_cost_multiplier: 0.0,
        },
        Technology {
            id: TechId::NuclearPower,
            name: "Nuclear Power",
            description: "Unlock nuclear reactors and containment facilities",
            tier: 3,
            science_cost: vec![(SciencePack1, 30), (SciencePack2, 25), (SciencePack3, 25)],
            prerequisites: vec![TechId::GasPower, TechId::PrecisionManufacturing],
            unlocks_buildings: vec![NuclearReactor, ContainmentVault],
            unlocks_recipes: vec![],
            cash_grant: 5000,
            research_time: 400,
            is_infinite: false,
            infinite_cost_multiplier: 0.0,
        },
        Technology {
            id: TechId::Cryogenics,
            name: "Cryogenics",
            description: "Unlock cryogenic processing and coolant production",
            tier: 3,
            science_cost: vec![(SciencePack1, 25), (SciencePack2, 20), (SciencePack3, 20)],
            prerequisites: vec![TechId::BasicChemistry],
            unlocks_buildings: vec![CoolantProcessor, CryoTank],
            unlocks_recipes: vec![recipe_ids::COOLANT],
            cash_grant: 2000,
            research_time: 300,
            is_infinite: false,
            infinite_cost_multiplier: 0.0,
        },
        Technology {
            id: TechId::AdvancedChemistry,
            name: "Advanced Chemistry",
            description: "Unlock rubber vulcanization and advanced chemical processing",
            tier: 3,
            science_cost: vec![(SciencePack1, 25), (SciencePack2, 25), (SciencePack3, 20)],
            prerequisites: vec![TechId::BasicChemistry, TechId::PrecisionManufacturing],
            unlocks_buildings: vec![RubberVulcanizer, PlasticMolder],
            unlocks_recipes: vec![],
            cash_grant: 2500,
            research_time: 350,
            is_infinite: false,
            infinite_cost_multiplier: 0.0,
        },
        Technology {
            id: TechId::TruckTransport,
            name: "Truck Transport",
            description: "Unlock underground transport tunnels for logistics",
            tier: 3,
            science_cost: vec![(SciencePack1, 20), (SciencePack2, 20), (SciencePack3, 15)],
            prerequisites: vec![TechId::TrainTransport],
            unlocks_buildings: vec![UndergroundEntrance, UndergroundExit],
            unlocks_recipes: vec![],
            cash_grant: 1500,
            research_time: 250,
            is_infinite: false,
            infinite_cost_multiplier: 0.0,
        },
        Technology {
            id: TechId::DroneLogistics,
            name: "Drone Logistics",
            description: "Unlock drone ports for automated aerial transport",
            tier: 3,
            science_cost: vec![(SciencePack1, 25), (SciencePack2, 25), (SciencePack3, 20)],
            prerequisites: vec![TechId::AdvancedCircuits],
            unlocks_buildings: vec![DronePort],
            unlocks_recipes: vec![],
            cash_grant: 3000,
            research_time: 350,
            is_infinite: false,
            infinite_cost_multiplier: 0.0,
        },
        Technology {
            id: TechId::WasteManagement3,
            name: "Waste Management III",
            description: "Unlock incineration and advanced scrubbing",
            tier: 3,
            science_cost: vec![(SciencePack1, 20), (SciencePack2, 20), (SciencePack3, 15)],
            prerequisites: vec![TechId::WasteManagement2],
            unlocks_buildings: vec![IncinerationPlant],
            unlocks_recipes: vec![],
            cash_grant: 1000,
            research_time: 250,
            is_infinite: false,
            infinite_cost_multiplier: 0.0,
        },
        Technology {
            id: TechId::Composites,
            name: "Composites",
            description: "Unlock composite panel and fiberglass recipes",
            tier: 3,
            science_cost: vec![(SciencePack1, 25), (SciencePack2, 20), (SciencePack3, 20)],
            prerequisites: vec![AdvancedSmelting],
            unlocks_buildings: vec![],
            unlocks_recipes: vec![recipe_ids::COMPOSITE_PANEL, recipe_ids::FIBERGLASS],
            cash_grant: 1500,
            research_time: 300,
            is_infinite: false,
            infinite_cost_multiplier: 0.0,
        },
        Technology {
            id: TechId::BatteryTech,
            name: "Battery Technology",
            description: "Unlock battery banks and battery/lithium cell production",
            tier: 3,
            science_cost: vec![(SciencePack1, 25), (SciencePack2, 25), (SciencePack3, 20)],
            prerequisites: vec![TechId::BasicChemistry],
            unlocks_buildings: vec![BatteryBank, Accumulator],
            unlocks_recipes: vec![recipe_ids::BATTERY_PACK, recipe_ids::LITHIUM_CELL],
            cash_grant: 2000,
            research_time: 350,
            is_infinite: false,
            infinite_cost_multiplier: 0.0,
        },

        // =================================================================
        // TIER 4 — R+G+B+Purple (SP1 + SP2 + SP3 + SP4)
        // =================================================================
        Technology {
            id: TechId::QuantumComputing,
            name: "Quantum Computing",
            description: "Unlock quantum labs and quantum processor fabrication",
            tier: 4,
            science_cost: vec![
                (SciencePack1, 40), (SciencePack2, 35),
                (SciencePack3, 30), (SciencePack4, 25),
            ],
            prerequisites: vec![TechId::AdvancedCircuits, TechId::Automation3],
            unlocks_buildings: vec![QuantumLab],
            unlocks_recipes: vec![recipe_ids::QUANTUM_PROCESSOR, recipe_ids::SCIENCE_PACK_4],
            cash_grant: 10000,
            research_time: 600,
            is_infinite: false,
            infinite_cost_multiplier: 0.0,
        },
        Technology {
            id: TechId::FusionPower,
            name: "Fusion Power",
            description: "Unlock fusion reactors and fusion core production",
            tier: 4,
            science_cost: vec![
                (SciencePack1, 35), (SciencePack2, 30),
                (SciencePack3, 30), (SciencePack4, 25),
            ],
            prerequisites: vec![TechId::NuclearPower, TechId::Cryogenics],
            unlocks_buildings: vec![FusionReactor],
            unlocks_recipes: vec![recipe_ids::FUSION_CORE],
            cash_grant: 10000,
            research_time: 550,
            is_infinite: false,
            infinite_cost_multiplier: 0.0,
        },
        Technology {
            id: TechId::Nanotechnology,
            name: "Nanotechnology",
            description: "Unlock nanobot swarm fabrication",
            tier: 4,
            science_cost: vec![
                (SciencePack1, 35), (SciencePack2, 30),
                (SciencePack3, 25), (SciencePack4, 20),
            ],
            prerequisites: vec![TechId::QuantumComputing],
            unlocks_buildings: vec![],
            unlocks_recipes: vec![recipe_ids::NANOBOT_SWARM],
            cash_grant: 8000,
            research_time: 500,
            is_infinite: false,
            infinite_cost_multiplier: 0.0,
        },
        Technology {
            id: TechId::SpaceScience,
            name: "Space Science",
            description: "Unlock singularity lab for tier-5 science pack production",
            tier: 4,
            science_cost: vec![
                (SciencePack1, 40), (SciencePack2, 35),
                (SciencePack3, 30), (SciencePack4, 25),
            ],
            prerequisites: vec![TechId::QuantumComputing],
            unlocks_buildings: vec![SingularityLab],
            unlocks_recipes: vec![recipe_ids::SCIENCE_PACK_5],
            cash_grant: 15000,
            research_time: 600,
            is_infinite: false,
            infinite_cost_multiplier: 0.0,
        },
        Technology {
            id: TechId::AdvancedLogistics,
            name: "Advanced Logistics",
            description: "Unlock high-capacity storage and distribution systems",
            tier: 4,
            science_cost: vec![
                (SciencePack1, 30), (SciencePack2, 25),
                (SciencePack3, 25), (SciencePack4, 20),
            ],
            prerequisites: vec![TechId::ExpressBelts, TechId::DroneLogistics],
            unlocks_buildings: vec![SiloHopper, Substation],
            unlocks_recipes: vec![],
            cash_grant: 5000,
            research_time: 450,
            is_infinite: false,
            infinite_cost_multiplier: 0.0,
        },
        Technology {
            id: TechId::AirTransport,
            name: "Air Transport",
            description: "Unlock rocket assembly for orbital logistics",
            tier: 4,
            science_cost: vec![
                (SciencePack1, 30), (SciencePack2, 30),
                (SciencePack3, 25), (SciencePack4, 20),
            ],
            prerequisites: vec![TechId::TruckTransport, TechId::AdvancedChemistry],
            unlocks_buildings: vec![RocketAssembly],
            unlocks_recipes: vec![],
            cash_grant: 5000,
            research_time: 450,
            is_infinite: false,
            infinite_cost_multiplier: 0.0,
        },
        Technology {
            id: TechId::WasteManagement4,
            name: "Waste Management IV",
            description: "Unlock containment fields for hazardous materials",
            tier: 4,
            science_cost: vec![
                (SciencePack1, 25), (SciencePack2, 25),
                (SciencePack3, 20), (SciencePack4, 15),
            ],
            prerequisites: vec![TechId::WasteManagement3, TechId::QuantumComputing],
            unlocks_buildings: vec![ContainmentField],
            unlocks_recipes: vec![],
            cash_grant: 3000,
            research_time: 350,
            is_infinite: false,
            infinite_cost_multiplier: 0.0,
        },
        Technology {
            id: TechId::Superconductors,
            name: "Superconductors",
            description: "Unlock superconductor wire production and power transformers",
            tier: 4,
            science_cost: vec![
                (SciencePack1, 30), (SciencePack2, 30),
                (SciencePack3, 25), (SciencePack4, 20),
            ],
            prerequisites: vec![TechId::Cryogenics, TechId::AdvancedCircuits],
            unlocks_buildings: vec![Transformer],
            unlocks_recipes: vec![],
            cash_grant: 5000,
            research_time: 450,
            is_infinite: false,
            infinite_cost_multiplier: 0.0,
        },
        Technology {
            id: TechId::QuantumBelts,
            name: "Quantum Belts",
            description: "Unlock megassembler for ultra-high-throughput production",
            tier: 4,
            science_cost: vec![
                (SciencePack1, 35), (SciencePack2, 30),
                (SciencePack3, 25), (SciencePack4, 20),
            ],
            prerequisites: vec![TechId::AdvancedLogistics, TechId::QuantumComputing],
            unlocks_buildings: vec![Megassembler],
            unlocks_recipes: vec![],
            cash_grant: 8000,
            research_time: 500,
            is_infinite: false,
            infinite_cost_multiplier: 0.0,
        },

        // =================================================================
        // TIER 5 — All 5 packs
        // =================================================================
        Technology {
            id: TechId::AntimatterTech,
            name: "Antimatter Technology",
            description: "Unlock antimatter capsule production",
            tier: 5,
            science_cost: vec![
                (SciencePack1, 50), (SciencePack2, 45),
                (SciencePack3, 40), (SciencePack4, 35), (SciencePack5, 30),
            ],
            prerequisites: vec![TechId::FusionPower, TechId::QuantumComputing],
            unlocks_buildings: vec![],
            unlocks_recipes: vec![],
            cash_grant: 20000,
            research_time: 800,
            is_infinite: false,
            infinite_cost_multiplier: 0.0,
        },
        Technology {
            id: TechId::DimensionalEngineering,
            name: "Dimensional Engineering",
            description: "Unlock dimensional manipulation technologies",
            tier: 5,
            science_cost: vec![
                (SciencePack1, 50), (SciencePack2, 45),
                (SciencePack3, 40), (SciencePack4, 35), (SciencePack5, 30),
            ],
            prerequisites: vec![TechId::AntimatterTech],
            unlocks_buildings: vec![],
            unlocks_recipes: vec![],
            cash_grant: 20000,
            research_time: 800,
            is_infinite: false,
            infinite_cost_multiplier: 0.0,
        },
        Technology {
            id: TechId::GravityManipulation,
            name: "Gravity Manipulation",
            description: "Unlock gravity lens fabrication",
            tier: 5,
            science_cost: vec![
                (SciencePack1, 45), (SciencePack2, 40),
                (SciencePack3, 35), (SciencePack4, 30), (SciencePack5, 25),
            ],
            prerequisites: vec![TechId::QuantumComputing],
            unlocks_buildings: vec![],
            unlocks_recipes: vec![recipe_ids::GRAVITY_LENS],
            cash_grant: 15000,
            research_time: 700,
            is_infinite: false,
            infinite_cost_multiplier: 0.0,
        },
        Technology {
            id: TechId::WarpDrive,
            name: "Warp Drive",
            description: "Unlock warp drive module and warp gate component fabrication",
            tier: 5,
            science_cost: vec![
                (SciencePack1, 60), (SciencePack2, 55),
                (SciencePack3, 50), (SciencePack4, 45), (SciencePack5, 40),
            ],
            prerequisites: vec![TechId::DimensionalEngineering, TechId::GravityManipulation],
            unlocks_buildings: vec![WarpGateFrame],
            unlocks_recipes: vec![recipe_ids::WARP_DRIVE_MODULE, recipe_ids::WARP_GATE_COMPONENT],
            cash_grant: 50000,
            research_time: 1000,
            is_infinite: false,
            infinite_cost_multiplier: 0.0,
        },
        Technology {
            id: TechId::DysonSphere,
            name: "Dyson Sphere",
            description: "Unlock dyson sphere panel and swarm cluster fabrication",
            tier: 5,
            science_cost: vec![
                (SciencePack1, 60), (SciencePack2, 55),
                (SciencePack3, 50), (SciencePack4, 45), (SciencePack5, 40),
            ],
            prerequisites: vec![TechId::SpaceScience, TechId::FusionPower],
            unlocks_buildings: vec![DysonSwarmLauncher],
            unlocks_recipes: vec![recipe_ids::DYSON_SPHERE_PANEL, recipe_ids::DYSON_SWARM_CLUSTER],
            cash_grant: 50000,
            research_time: 1000,
            is_infinite: false,
            infinite_cost_multiplier: 0.0,
        },
        Technology {
            id: TechId::SpaceElevator,
            name: "Space Elevator",
            description: "Unlock space elevator cable and segment fabrication",
            tier: 5,
            science_cost: vec![
                (SciencePack1, 60), (SciencePack2, 55),
                (SciencePack3, 50), (SciencePack4, 45), (SciencePack5, 40),
            ],
            prerequisites: vec![TechId::SpaceScience, TechId::Nanotechnology],
            unlocks_buildings: vec![SpaceElevatorBase],
            unlocks_recipes: vec![
                recipe_ids::SPACE_ELEVATOR_CABLE,
                recipe_ids::SPACE_ELEVATOR_SEGMENT,
            ],
            cash_grant: 50000,
            research_time: 1000,
            is_infinite: false,
            infinite_cost_multiplier: 0.0,
        },
        Technology {
            id: TechId::Terraforming,
            name: "Terraforming",
            description: "Unlock terraforming agent and hazmat shielding production",
            tier: 5,
            science_cost: vec![
                (SciencePack1, 50), (SciencePack2, 45),
                (SciencePack3, 40), (SciencePack4, 35), (SciencePack5, 30),
            ],
            prerequisites: vec![TechId::AntimatterTech, TechId::Nanotechnology],
            unlocks_buildings: vec![ShieldGenerator],
            unlocks_recipes: vec![recipe_ids::TERRAFORMING_AGENT],
            cash_grant: 25000,
            research_time: 800,
            is_infinite: false,
            infinite_cost_multiplier: 0.0,
        },

        // =================================================================
        // INFINITE RESEARCH — All 5 packs, +20% cost per level
        // =================================================================
        Technology {
            id: TechId::MiningProductivity,
            name: "Mining Productivity",
            description: "+5% ore output per level",
            tier: 5,
            science_cost: vec![
                (SciencePack1, 50), (SciencePack2, 45),
                (SciencePack3, 40), (SciencePack4, 35), (SciencePack5, 30),
            ],
            prerequisites: vec![TechId::SpaceScience],
            unlocks_buildings: vec![],
            unlocks_recipes: vec![],
            cash_grant: 0,
            research_time: 600,
            is_infinite: true,
            infinite_cost_multiplier: 0.2,
        },
        Technology {
            id: TechId::BeltSpeed,
            name: "Belt Speed",
            description: "+10% belt throughput per level",
            tier: 5,
            science_cost: vec![
                (SciencePack1, 40), (SciencePack2, 35),
                (SciencePack3, 30), (SciencePack4, 25), (SciencePack5, 20),
            ],
            prerequisites: vec![TechId::SpaceScience],
            unlocks_buildings: vec![],
            unlocks_recipes: vec![],
            cash_grant: 0,
            research_time: 500,
            is_infinite: true,
            infinite_cost_multiplier: 0.2,
        },
        Technology {
            id: TechId::MachineSpeed,
            name: "Machine Speed",
            description: "+5% machine processing speed per level",
            tier: 5,
            science_cost: vec![
                (SciencePack1, 50), (SciencePack2, 45),
                (SciencePack3, 40), (SciencePack4, 35), (SciencePack5, 30),
            ],
            prerequisites: vec![TechId::SpaceScience],
            unlocks_buildings: vec![],
            unlocks_recipes: vec![],
            cash_grant: 0,
            research_time: 600,
            is_infinite: true,
            infinite_cost_multiplier: 0.2,
        },
        Technology {
            id: TechId::EnergyEfficiency,
            name: "Energy Efficiency",
            description: "-5% power consumption per level",
            tier: 5,
            science_cost: vec![
                (SciencePack1, 40), (SciencePack2, 35),
                (SciencePack3, 30), (SciencePack4, 25), (SciencePack5, 20),
            ],
            prerequisites: vec![TechId::SpaceScience],
            unlocks_buildings: vec![],
            unlocks_recipes: vec![],
            cash_grant: 0,
            research_time: 500,
            is_infinite: true,
            infinite_cost_multiplier: 0.2,
        },
        Technology {
            id: TechId::PollutionReduction,
            name: "Pollution Reduction",
            description: "-5% pollution output per level",
            tier: 5,
            science_cost: vec![
                (SciencePack1, 40), (SciencePack2, 35),
                (SciencePack3, 30), (SciencePack4, 25), (SciencePack5, 20),
            ],
            prerequisites: vec![TechId::SpaceScience],
            unlocks_buildings: vec![],
            unlocks_recipes: vec![],
            cash_grant: 0,
            research_time: 500,
            is_infinite: true,
            infinite_cost_multiplier: 0.2,
        },
        Technology {
            id: TechId::ResearchSpeed,
            name: "Research Speed",
            description: "+10% research speed per level",
            tier: 5,
            science_cost: vec![
                (SciencePack1, 50), (SciencePack2, 45),
                (SciencePack3, 40), (SciencePack4, 35), (SciencePack5, 30),
            ],
            prerequisites: vec![TechId::SpaceScience],
            unlocks_buildings: vec![],
            unlocks_recipes: vec![],
            cash_grant: 0,
            research_time: 600,
            is_infinite: true,
            infinite_cost_multiplier: 0.2,
        },
    ]
}

/// Look up a single technology by ID.
pub fn get_tech(id: TechId) -> Technology {
    get_all_techs()
        .into_iter()
        .find(|t| t.id == id)
        .unwrap_or_else(|| panic!("Unknown TechId: {:?}", id))
}

/// Check if a tech is available to research (all prerequisites completed, not already done).
pub fn is_available(id: TechId, completed: &HashSet<TechId>) -> bool {
    let tech = get_tech(id);
    if !tech.is_infinite && completed.contains(&id) {
        return false;
    }
    tech.prerequisites.iter().all(|prereq| completed.contains(prereq))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn all_techs_have_unique_ids() {
        let techs = get_all_techs();
        let mut seen = HashSet::new();
        for t in &techs {
            assert!(seen.insert(t.id), "Duplicate TechId: {:?}", t.id);
        }
    }

    #[test]
    fn tech_count_at_least_57() {
        assert!(get_all_techs().len() >= 57);
    }

    #[test]
    fn prerequisites_reference_valid_ids() {
        let all_ids: HashSet<TechId> = get_all_techs().iter().map(|t| t.id).collect();
        for tech in get_all_techs() {
            for prereq in &tech.prerequisites {
                assert!(
                    all_ids.contains(prereq),
                    "Tech {:?} has invalid prerequisite {:?}",
                    tech.id, prereq
                );
            }
        }
    }

    #[test]
    fn tier1_only_needs_sp1() {
        for tech in get_all_techs() {
            if tech.tier == 1 {
                for (res, _) in &tech.science_cost {
                    assert_eq!(
                        *res,
                        Resource::SciencePack1,
                        "Tier 1 tech {:?} uses non-SP1 resource {:?}",
                        tech.id, res
                    );
                }
            }
        }
    }

    #[test]
    fn infinite_techs_flagged() {
        let infinite_ids = [
            TechId::MiningProductivity,
            TechId::BeltSpeed,
            TechId::MachineSpeed,
            TechId::EnergyEfficiency,
            TechId::PollutionReduction,
            TechId::ResearchSpeed,
        ];
        for id in &infinite_ids {
            let tech = get_tech(*id);
            assert!(tech.is_infinite, "{:?} should be infinite", id);
            assert!(
                tech.infinite_cost_multiplier > 0.0,
                "{:?} should have positive cost multiplier", id
            );
        }
    }

    #[test]
    fn research_state_basic_flow() {
        let mut state = ResearchState::new();
        // Start with a tier-1 tech with no prereqs
        assert!(state.start_research(TechId::BasicSmelting));
        assert_eq!(state.current, Some(TechId::BasicSmelting));

        // Tick until done (120 ticks)
        assert!(state.tick(119).is_none());
        let completed = state.tick(1);
        assert_eq!(completed, Some(TechId::BasicSmelting));
        assert!(state.completed.contains(&TechId::BasicSmelting));
        assert!(state.current.is_none());
    }

    #[test]
    fn cannot_start_without_prereqs() {
        let mut state = ResearchState::new();
        // Automation1 requires BasicSmelting
        assert!(!state.start_research(TechId::Automation1));
    }

    #[test]
    fn infinite_research_scales_cost() {
        let mut state = ResearchState::new();
        // Fulfill all prereqs by marking SpaceScience + chain as complete
        state.completed.insert(TechId::BasicSmelting);
        state.completed.insert(TechId::BasicBelts);
        state.completed.insert(TechId::BasicMining);
        state.completed.insert(TechId::BasicPipes);
        state.completed.insert(TechId::Automation1);
        state.completed.insert(TechId::CircuitNetworks);
        state.completed.insert(TechId::Automation2);
        state.completed.insert(TechId::AdvancedCircuits);
        state.completed.insert(TechId::Automation3);
        state.completed.insert(TechId::QuantumComputing);
        state.completed.insert(TechId::SpaceScience);

        assert!(state.start_research(TechId::MiningProductivity));
        // First level: 600 ticks
        let tech = get_tech(TechId::MiningProductivity);
        assert_eq!(state.effective_research_time(&tech), 600);

        // Complete level 1
        state.tick(600);
        assert_eq!(state.infinite_level(TechId::MiningProductivity), 1);

        // Level 2 costs 20% more = 720
        assert!(state.start_research(TechId::MiningProductivity));
        assert_eq!(state.effective_research_time(&tech), 720);
    }

    #[test]
    fn building_unlock_check() {
        let mut state = ResearchState::new();
        assert!(!state.is_building_unlocked(EntityType::Smelter));
        state.completed.insert(TechId::BasicSmelting);
        assert!(state.is_building_unlocked(EntityType::Smelter));
        assert!(state.is_building_unlocked(EntityType::Kiln));
    }

    #[test]
    fn queue_auto_advances() {
        let mut state = ResearchState::new();
        assert!(state.start_research(TechId::BasicSmelting));
        state.queue_research(TechId::StoneworkProcessing);
        // Complete BasicSmelting
        state.tick(120);
        // Should auto-start StoneworkProcessing
        assert_eq!(state.current, Some(TechId::StoneworkProcessing));
    }
}
