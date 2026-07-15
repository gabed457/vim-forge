use crate::map::multitile::PortType;
use crate::resources::{EntityType, Resource};

#[derive(Clone, Debug)]
pub struct RecipeInput {
    pub resource: Resource,
    pub amount: u64,
    pub port_type: PortType,
}

#[derive(Clone, Debug)]
pub struct RecipeOutput {
    pub resource: Resource,
    pub amount: u64,
    pub port_type: PortType,
}

#[derive(Clone, Debug)]
pub struct Recipe {
    pub id: &'static str,
    /// Numeric id cross-referencing `research::tree::recipe_ids`.
    /// 0 = not gated by any technology (always available).
    pub numeric_id: u16,
    pub building: EntityType,
    pub inputs: Vec<RecipeInput>,
    pub outputs: Vec<RecipeOutput>,
    pub waste: Vec<RecipeOutput>,
    pub ticks: u32,
    pub power_mw: u32,
}

impl Recipe {
    fn new(
        id: &'static str,
        numeric_id: u16,
        building: EntityType,
        inputs: Vec<(Resource, u64)>,
        outputs: Vec<(Resource, u64)>,
        waste: Vec<(Resource, u64)>,
        ticks: u32,
        power_mw: u32,
    ) -> Self {
        Recipe {
            id,
            numeric_id,
            building,
            inputs: inputs
                .into_iter()
                .map(|(r, a)| RecipeInput {
                    resource: r,
                    amount: a,
                    port_type: PortType::SolidInput,
                })
                .collect(),
            outputs: outputs
                .into_iter()
                .map(|(r, a)| RecipeOutput {
                    resource: r,
                    amount: a,
                    port_type: PortType::SolidOutput,
                })
                .collect(),
            waste: waste
                .into_iter()
                .map(|(r, a)| RecipeOutput {
                    resource: r,
                    amount: a,
                    port_type: PortType::WasteSolid,
                })
                .collect(),
            ticks,
            power_mw,
        }
    }

    /// Inputs flattened to individual items, e.g. `[(IronIngot, 2)]`
    /// becomes `[IronIngot, IronIngot]`.
    ///
    /// The tile simulation moves items one at a time on belts, so every
    /// recipe is designed to need at most TWO input items total; a machine
    /// buffers them in its two Processing slots (`input_a`/`input_b`) and
    /// starts as soon as the buffered multiset equals a recipe's flat inputs.
    pub fn flat_inputs(&self) -> Vec<Resource> {
        let mut out = Vec::new();
        for i in &self.inputs {
            for _ in 0..i.amount {
                out.push(i.resource);
            }
        }
        out
    }

    /// Primary output resource.
    pub fn primary_output(&self) -> Resource {
        self.outputs[0].resource
    }

    /// Total waste units produced per completed cycle (used as pollution).
    pub fn waste_units(&self) -> u64 {
        self.waste.iter().map(|w| w.amount).sum()
    }
}

/// Get all registered recipes.
///
/// Design constraints (relied on by `ecs::systems`):
/// - every recipe needs at most 2 input items total (see `flat_inputs`)
/// - within one building, every recipe has a UNIQUE input multiset so the
///   active recipe can be re-derived from the buffered inputs
pub fn all_recipes() -> Vec<Recipe> {
    use EntityType::*;
    use Resource::*;
    // numeric ids from research::tree::recipe_ids (0 = ungated)
    const STONE_BRICK: u16 = 1;
    const SULFURIC_ACID: u16 = 2;
    const CONCRETE: u16 = 3;
    const CHARCOAL: u16 = 4;
    const PROCESSOR: u16 = 5;
    const ADVANCED_CIRCUIT: u16 = 6;
    const COOLANT: u16 = 7;
    const COMPOSITE_PANEL: u16 = 8;
    const FIBERGLASS: u16 = 9;
    const BATTERY_PACK: u16 = 10;
    const LITHIUM_CELL: u16 = 11;
    const QUANTUM_PROCESSOR: u16 = 12;
    const FUSION_CORE: u16 = 13;
    const GRAVITY_LENS: u16 = 15;
    const WARP_DRIVE_MODULE: u16 = 16;
    const WARP_GATE_COMPONENT: u16 = 17;
    const DYSON_SPHERE_PANEL: u16 = 18;
    const DYSON_SWARM_CLUSTER: u16 = 19;
    const SPACE_ELEVATOR_CABLE: u16 = 20;
    const SPACE_ELEVATOR_SEGMENT: u16 = 21;
    const SCIENCE_PACK_2: u16 = 101;
    const SCIENCE_PACK_3: u16 = 102;
    const SCIENCE_PACK_4: u16 = 103;
    const SCIENCE_PACK_5: u16 = 104;

    vec![
        // ============================ Smelter (3x3, 1 input port) =========
        Recipe::new("smelt_iron", 0, Smelter,
            vec![(IronOre, 1)], vec![(IronIngot, 1)], vec![(Slag, 1)], 3, 5),
        Recipe::new("smelt_copper", 0, Smelter,
            vec![(CopperOre, 1)], vec![(CopperIngot, 1)], vec![(Slag, 1)], 3, 5),
        Recipe::new("smelt_steel", 0, Smelter,
            vec![(IronIngot, 2)], vec![(Steel, 1)], vec![], 5, 10),
        // ============================ Kiln =================================
        Recipe::new("kiln_stone_brick", STONE_BRICK, Kiln,
            vec![(Stone, 1)], vec![(StoneBrick, 1)], vec![], 3, 3),
        Recipe::new("kiln_glass", 0, Kiln,
            vec![(QuartzSand, 1)], vec![(Glass, 1)], vec![], 4, 5),
        Recipe::new("kiln_charcoal", CHARCOAL, Kiln,
            vec![(Biomass, 2)], vec![(Charcoal, 1)], vec![(CO2, 1)], 3, 2),
        // ============================ Press ================================
        Recipe::new("press_iron_plate", 0, Press,
            vec![(IronIngot, 1)], vec![(IronPlate, 2)], vec![(MetalShavings, 1)], 2, 5),
        Recipe::new("press_copper_plate", 0, Press,
            vec![(CopperIngot, 1)], vec![(CopperPlate, 2)], vec![(MetalShavings, 1)], 2, 5),
        Recipe::new("press_aluminum_sheet", 0, Press,
            vec![(AluminumIngot, 1)], vec![(AluminumSheet, 2)], vec![], 3, 5),
        // ============================ WireMill =============================
        Recipe::new("wire_copper", 0, WireMill,
            vec![(CopperIngot, 1)], vec![(CopperWire, 3)], vec![], 2, 3),
        Recipe::new("wire_insulated", 0, WireMill,
            vec![(CopperWire, 1), (Rubber, 1)], vec![(InsulatedWire, 2)], vec![], 3, 5),
        Recipe::new("wire_superconductor", 0, WireMill,
            vec![(CopperWire, 1), (RareEarthOre, 1)], vec![(SuperconductorWire, 1)], vec![], 6, 10),
        // ============================ PlateMachine =========================
        Recipe::new("bulk_iron_plate", 0, PlateMachine,
            vec![(IronIngot, 2)], vec![(IronPlate, 5)], vec![(MetalShavings, 1)], 4, 8),
        // ============================ Caster ===============================
        Recipe::new("cast_aluminum", 0, Caster,
            vec![(Bauxite, 2)], vec![(AluminumIngot, 1)], vec![(RedMud, 1)], 4, 8),
        // ============================ CokeFurnace ==========================
        Recipe::new("coke_charcoal", 0, CokeFurnace,
            vec![(Coal, 2)], vec![(Charcoal, 2)], vec![(CO2, 1)], 3, 3),
        // ============================ Boiler ===============================
        Recipe::new("boil_steam", 0, Boiler,
            vec![(Water, 1), (Coal, 1)], vec![(Steam, 2)], vec![(CO2, 1)], 2, 0),
        // ============================ Gasifier =============================
        Recipe::new("gasify_syngas", 0, Gasifier,
            vec![(Charcoal, 1), (Water, 1)], vec![(Syngas, 2)], vec![(CO2, 1)], 4, 5),
        // ============================ Electrolyzer =========================
        Recipe::new("electrolyze_lithium", LITHIUM_CELL, Electrolyzer,
            vec![(LithiumBrine, 2)], vec![(LithiumCell, 1)], vec![(Wastewater, 1)], 5, 10),
        // ============================ WaferCutter ==========================
        Recipe::new("cut_silicon_wafer", 0, WaferCutter,
            vec![(QuartzSand, 2)], vec![(SiliconWafer, 1)], vec![(Tailings, 1)], 5, 8),
        // ============================ PlasticMolder ========================
        Recipe::new("mold_plastic", 0, PlasticMolder,
            vec![(CrudeOil, 2)], vec![(Plastic, 1)], vec![(CO2, 1)], 4, 8),
        Recipe::new("mold_polymer_resin", 0, PlasticMolder,
            vec![(Plastic, 1), (Sulfur, 1)], vec![(PolymerResin, 2)], vec![], 3, 5),
        // ============================ RubberVulcanizer =====================
        Recipe::new("vulcanize_rubber", 0, RubberVulcanizer,
            vec![(CrudeOil, 1), (Sulfur, 1)], vec![(Rubber, 2)], vec![], 4, 5),
        // ============================ Assembler (3x4, 2 input ports) ======
        // Legacy campaign recipe: two iron ingots -> circuit board ("widget").
        // The 13 campaign levels are built around this chain; DO NOT remove.
        Recipe::new("assemble_widget", 0, Assembler,
            vec![(IronIngot, 2)], vec![(CircuitBoard, 1)], vec![(MetalShavings, 1)], 5, 10),
        Recipe::new("assemble_circuit_board", 0, Assembler,
            vec![(IronPlate, 1), (CopperWire, 1)], vec![(CircuitBoard, 1)],
            vec![(MetalShavings, 1)], 5, 10),
        Recipe::new("assemble_gear", 0, Assembler,
            vec![(IronPlate, 2)], vec![(Gear, 1)], vec![(MetalShavings, 1)], 3, 5),
        Recipe::new("assemble_bearing", 0, Assembler,
            vec![(Steel, 1)], vec![(Bearing, 2)], vec![], 3, 5),
        Recipe::new("assemble_pipe_fitting", 0, Assembler,
            vec![(CopperPlate, 1)], vec![(PipeFitting, 2)], vec![], 2, 3),
        // Science pack 1 is deliberately UNGATED (tier-0) so freeplay
        // research can bootstrap without any completed technology.
        Recipe::new("science_pack_1", 0, Assembler,
            vec![(IronIngot, 1), (CopperWire, 1)], vec![(SciencePack1, 1)], vec![], 5, 5),
        Recipe::new("science_pack_2", SCIENCE_PACK_2, Assembler,
            vec![(CircuitBoard, 1), (Gear, 1)], vec![(SciencePack2, 1)], vec![], 8, 10),
        // ============================ Mixer ================================
        Recipe::new("mix_concrete", CONCRETE, Mixer,
            vec![(Stone, 1), (Water, 1)], vec![(Concrete, 2)], vec![], 4, 5),
        Recipe::new("mix_lubricant", 0, Mixer,
            vec![(CrudeOil, 1), (Water, 1)], vec![(Lubricant, 2)], vec![], 3, 5),
        // ============================ ChemicalPlant ========================
        Recipe::new("chem_sulfuric_acid", SULFURIC_ACID, ChemicalPlant,
            vec![(Sulfur, 1), (Water, 1)], vec![(SulfuricAcid, 2)], vec![(Wastewater, 1)], 5, 10),
        Recipe::new("chem_plastic", 0, ChemicalPlant,
            vec![(CrudeOil, 2)], vec![(Plastic, 1)], vec![(CO2, 1)], 4, 8),
        Recipe::new("chem_additive", 0, ChemicalPlant,
            vec![(SulfuricAcid, 1), (Charcoal, 1)], vec![(ChemicalAdditive, 1)], vec![(SpentAcid, 1)], 5, 10),
        Recipe::new("chem_coolant", COOLANT, ChemicalPlant,
            vec![(Water, 1), (ChemicalAdditive, 1)], vec![(Coolant, 2)], vec![], 4, 8),
        // ============================ CircuitFabricator ====================
        Recipe::new("fab_advanced_circuit", ADVANCED_CIRCUIT, CircuitFabricator,
            vec![(CircuitBoard, 1), (CopperWire, 1)], vec![(AdvancedCircuit, 1)],
            vec![(PCBEtchWaste, 1)], 6, 15),
        // ============================ MotorAssembly ========================
        Recipe::new("motor_electric", 0, MotorAssembly,
            vec![(Gear, 1), (CopperWire, 1)], vec![(ElectricMotor, 1)], vec![], 5, 10),
        // ============================ CrushingMill =========================
        Recipe::new("crush_sand", 0, CrushingMill,
            vec![(Stone, 2)], vec![(QuartzSand, 2)], vec![(Tailings, 1)], 2, 5),
        // ============================ AdvancedAssembler (4x4) ==============
        Recipe::new("adv_processor", PROCESSOR, AdvancedAssembler,
            vec![(CircuitBoard, 1), (SiliconWafer, 1)], vec![(Processor, 1)],
            vec![(PCBEtchWaste, 1)], 8, 25),
        Recipe::new("adv_battery_pack", BATTERY_PACK, AdvancedAssembler,
            vec![(LithiumCell, 1), (Plastic, 1)], vec![(BatteryPack, 1)], vec![], 6, 20),
        Recipe::new("adv_servo", 0, AdvancedAssembler,
            vec![(ElectricMotor, 1), (CircuitBoard, 1)], vec![(Servo, 1)], vec![], 5, 15),
        Recipe::new("adv_heat_exchanger", 0, AdvancedAssembler,
            vec![(CopperPlate, 1), (Steel, 1)], vec![(HeatExchanger, 1)], vec![], 5, 15),
        Recipe::new("adv_hydraulic", 0, AdvancedAssembler,
            vec![(Steel, 1), (Lubricant, 1)], vec![(HydraulicCylinder, 1)], vec![], 5, 15),
        Recipe::new("science_pack_3", SCIENCE_PACK_3, AdvancedAssembler,
            vec![(AdvancedCircuit, 1), (Steel, 1)], vec![(SciencePack3, 1)], vec![], 10, 20),
        // ============================ Refinery / CrackingTower =============
        Recipe::new("refine_rocket_fuel", 0, Refinery,
            vec![(CrudeOil, 1), (NaturalGas, 1)], vec![(RocketFuel, 1)], vec![(CO2, 1)], 8, 20),
        Recipe::new("refine_heavy_water", 0, Refinery,
            vec![(Water, 2)], vec![(HeavyWater, 1)], vec![(Wastewater, 1)], 6, 15),
        Recipe::new("crack_natural_gas", 0, CrackingTower,
            vec![(CrudeOil, 2)], vec![(NaturalGas, 2)], vec![(CO2, 1)], 5, 15),
        // ============================ Cleanroom ============================
        Recipe::new("clean_fiberglass", FIBERGLASS, Cleanroom,
            vec![(Glass, 1), (PolymerResin, 1)], vec![(Fiberglass, 2)], vec![], 6, 15),
        // ============================ EnrichmentCascade ====================
        Recipe::new("enrich_uranium", 0, EnrichmentCascade,
            vec![(UraniumOre, 2)], vec![(EnrichedUranium, 1)], vec![(DepletedUranium, 1)], 10, 30),
        Recipe::new("nuclear_fuel_rod", 0, EnrichmentCascade,
            vec![(EnrichedUranium, 1), (Steel, 1)], vec![(NuclearFuelRod, 1)], vec![], 8, 25),
        // ============================ CoolantProcessor =====================
        Recipe::new("process_coolant", COOLANT, CoolantProcessor,
            vec![(Water, 2)], vec![(Coolant, 1)], vec![(Wastewater, 1)], 5, 15),
        // ============================ PrecisionAssembler (5x5) =============
        Recipe::new("prec_solar_panel", 0, PrecisionAssembler,
            vec![(Glass, 1), (AdvancedCircuit, 1)], vec![(SolarPanel, 1)], vec![], 8, 30),
        Recipe::new("prec_turbine_blade", 0, PrecisionAssembler,
            vec![(Steel, 1), (Bearing, 1)], vec![(TurbineBlade, 1)], vec![], 6, 25),
        Recipe::new("prec_composite_panel", COMPOSITE_PANEL, PrecisionAssembler,
            vec![(AluminumSheet, 1), (Fiberglass, 1)], vec![(CompositePanel, 1)], vec![], 7, 25),
        Recipe::new("science_pack_4", SCIENCE_PACK_4, PrecisionAssembler,
            vec![(Processor, 1), (AdvancedCircuit, 1)], vec![(SciencePack4, 1)], vec![], 12, 30),
        // ============================ QuantumLab ===========================
        Recipe::new("quantum_processor", QUANTUM_PROCESSOR, QuantumLab,
            vec![(Processor, 1), (SuperconductorWire, 1)], vec![(QuantumProcessor, 1)],
            vec![(QuantumDecoherence, 1)], 12, 50),
        Recipe::new("science_pack_5", SCIENCE_PACK_5, QuantumLab,
            vec![(QuantumProcessor, 1), (AdvancedCircuit, 1)], vec![(SciencePack5, 1)], vec![], 12, 50),
        Recipe::new("gravity_lens", GRAVITY_LENS, QuantumLab,
            vec![(QuantumProcessor, 1), (Glass, 1)], vec![(GravityLens, 1)], vec![], 14, 60),
        // ============================ RocketAssembly =======================
        Recipe::new("space_elevator_cable", SPACE_ELEVATOR_CABLE, RocketAssembly,
            vec![(SuperconductorWire, 1), (Steel, 1)], vec![(SpaceElevatorCable, 1)], vec![], 10, 40),
        // ============================ Megassembler (6x6) ===================
        Recipe::new("fusion_core", FUSION_CORE, Megassembler,
            vec![(QuantumProcessor, 1), (HeavyWater, 1)], vec![(FusionCore, 1)], vec![], 15, 80),
        Recipe::new("warp_drive_module", WARP_DRIVE_MODULE, Megassembler,
            vec![(FusionCore, 1), (GravityLens, 1)], vec![(WarpDriveModule, 1)], vec![], 18, 100),
        Recipe::new("dyson_sphere_panel", DYSON_SPHERE_PANEL, Megassembler,
            vec![(SolarPanel, 1), (CompositePanel, 1)], vec![(DysonSpherePanel, 1)], vec![], 15, 80),
        Recipe::new("space_elevator_segment", SPACE_ELEVATOR_SEGMENT, Megassembler,
            vec![(SpaceElevatorCable, 1), (CompositePanel, 1)],
            vec![(SpaceElevatorSegment, 1)], vec![], 20, 100),
        // ============================ SingularityLab (6x7) =================
        Recipe::new("antimatter_capsule", 0, SingularityLab,
            vec![(FusionCore, 1), (EnrichedUranium, 1)], vec![(AntimatterCapsule, 1)],
            vec![(AntimatterExhaust, 1)], 20, 150),
        Recipe::new("dark_matter_shard", 0, SingularityLab,
            vec![(AntimatterCapsule, 1), (GravityLens, 1)], vec![(DarkMatterShard, 1)],
            vec![(DimensionalResidue, 1)], 25, 150),
        Recipe::new("warp_gate_component", WARP_GATE_COMPONENT, SingularityLab,
            vec![(WarpDriveModule, 1), (DarkMatterShard, 1)], vec![(WarpGateComponent, 1)], vec![], 30, 200),
        Recipe::new("dyson_swarm_cluster", DYSON_SWARM_CLUSTER, SingularityLab,
            vec![(DysonSpherePanel, 2)], vec![(DysonSwarmCluster, 1)], vec![], 25, 150),
    ]
}

/// Find all recipes for a given building type.
pub fn recipes_for(building: EntityType) -> Vec<Recipe> {
    all_recipes()
        .into_iter()
        .filter(|r| r.building == building)
        .collect()
}

/// Find a recipe by its ID.
pub fn recipe_by_id(id: &str) -> Option<Recipe> {
    all_recipes().into_iter().find(|r| r.id == id)
}
