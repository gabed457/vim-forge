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
        building: EntityType,
        inputs: Vec<(Resource, u64)>,
        outputs: Vec<(Resource, u64)>,
        waste: Vec<(Resource, u64)>,
        ticks: u32,
        power_mw: u32,
    ) -> Self {
        Recipe {
            id,
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
}

/// Get all registered recipes.
pub fn all_recipes() -> Vec<Recipe> {
    vec![
        // Smelter recipes (1x1, Tier 0 → Tier 1)
        Recipe::new(
            "smelt_iron",
            EntityType::Smelter,
            vec![(Resource::IronOre, 1)],
            vec![(Resource::IronIngot, 1)],
            vec![(Resource::Slag, 1)],
            3,
            5,
        ),
        Recipe::new(
            "smelt_copper",
            EntityType::Smelter,
            vec![(Resource::CopperOre, 1)],
            vec![(Resource::CopperIngot, 1)],
            vec![(Resource::Slag, 1)],
            3,
            5,
        ),
        Recipe::new(
            "smelt_steel",
            EntityType::Smelter,
            vec![(Resource::IronIngot, 2)],
            vec![(Resource::Steel, 1)],
            vec![],
            5,
            10,
        ),
        // Kiln recipes
        Recipe::new(
            "kiln_stone_brick",
            EntityType::Kiln,
            vec![(Resource::Stone, 1)],
            vec![(Resource::StoneBrick, 1)],
            vec![],
            3,
            3,
        ),
        Recipe::new(
            "kiln_glass",
            EntityType::Kiln,
            vec![(Resource::QuartzSand, 1)],
            vec![(Resource::Glass, 1)],
            vec![],
            4,
            5,
        ),
        Recipe::new(
            "kiln_charcoal",
            EntityType::Kiln,
            vec![(Resource::Biomass, 2)],
            vec![(Resource::Charcoal, 1)],
            vec![(Resource::CO2, 1)],
            3,
            2,
        ),
        // Press recipes
        Recipe::new(
            "press_iron_plate",
            EntityType::Press,
            vec![(Resource::IronIngot, 1)],
            vec![(Resource::IronPlate, 2)],
            vec![(Resource::MetalShavings, 1)],
            2,
            5,
        ),
        Recipe::new(
            "press_copper_plate",
            EntityType::Press,
            vec![(Resource::CopperIngot, 1)],
            vec![(Resource::CopperPlate, 2)],
            vec![(Resource::MetalShavings, 1)],
            2,
            5,
        ),
        // WireMill recipes
        Recipe::new(
            "wire_copper",
            EntityType::WireMill,
            vec![(Resource::CopperIngot, 1)],
            vec![(Resource::CopperWire, 3)],
            vec![],
            2,
            3,
        ),
        // Assembler recipes (1x3, Tier 1-2)
        Recipe::new(
            "assemble_circuit_board",
            EntityType::Assembler,
            vec![(Resource::IronPlate, 1), (Resource::CopperWire, 3)],
            vec![(Resource::CircuitBoard, 1)],
            vec![(Resource::MetalShavings, 1)],
            5,
            10,
        ),
        Recipe::new(
            "assemble_gear",
            EntityType::Assembler,
            vec![(Resource::IronIngot, 2)],
            vec![(Resource::Gear, 1)],
            vec![(Resource::MetalShavings, 1)],
            3,
            5,
        ),
        Recipe::new(
            "assemble_motor",
            EntityType::Assembler,
            vec![(Resource::Gear, 1), (Resource::CopperWire, 2)],
            vec![(Resource::ElectricMotor, 1)],
            vec![],
            6,
            15,
        ),
        Recipe::new(
            "assemble_bearing",
            EntityType::Assembler,
            vec![(Resource::Steel, 1)],
            vec![(Resource::Bearing, 2)],
            vec![],
            3,
            5,
        ),
        Recipe::new(
            "assemble_pipe_fitting",
            EntityType::Assembler,
            vec![(Resource::CopperPlate, 1)],
            vec![(Resource::PipeFitting, 2)],
            vec![],
            2,
            3,
        ),
        // Mixer recipes (1x3)
        Recipe::new(
            "mix_concrete",
            EntityType::Mixer,
            vec![(Resource::Stone, 2), (Resource::Water, 1)],
            vec![(Resource::Concrete, 2)],
            vec![],
            4,
            5,
        ),
        // Chemical plant recipes (1x3)
        Recipe::new(
            "chem_sulfuric_acid",
            EntityType::ChemicalPlant,
            vec![(Resource::Sulfur, 1), (Resource::Water, 1)],
            vec![(Resource::SulfuricAcid, 2)],
            vec![(Resource::Wastewater, 1)],
            5,
            10,
        ),
        Recipe::new(
            "chem_plastic",
            EntityType::ChemicalPlant,
            vec![(Resource::CrudeOil, 2)],
            vec![(Resource::Plastic, 1)],
            vec![(Resource::CO2, 1)],
            4,
            8,
        ),
        Recipe::new(
            "chem_rubber",
            EntityType::ChemicalPlant,
            vec![(Resource::CrudeOil, 1), (Resource::Sulfur, 1)],
            vec![(Resource::Rubber, 2)],
            vec![],
            5,
            8,
        ),
        // Advanced assembler recipes (1x5)
        Recipe::new(
            "adv_processor",
            EntityType::AdvancedAssembler,
            vec![
                (Resource::CircuitBoard, 2),
                (Resource::SiliconWafer, 1),
                (Resource::CopperWire, 4),
            ],
            vec![(Resource::Processor, 1)],
            vec![(Resource::PCBEtchWaste, 1)],
            8,
            25,
        ),
        Recipe::new(
            "adv_battery_pack",
            EntityType::AdvancedAssembler,
            vec![
                (Resource::LithiumCell, 4),
                (Resource::CopperWire, 2),
                (Resource::Plastic, 1),
            ],
            vec![(Resource::BatteryPack, 1)],
            vec![],
            6,
            20,
        ),
        Recipe::new(
            "adv_servo",
            EntityType::AdvancedAssembler,
            vec![
                (Resource::ElectricMotor, 1),
                (Resource::Gear, 2),
                (Resource::CircuitBoard, 1),
            ],
            vec![(Resource::Servo, 1)],
            vec![],
            5,
            15,
        ),
        // Science pack recipes
        Recipe::new(
            "science_pack_1",
            EntityType::Assembler,
            vec![(Resource::IronPlate, 1), (Resource::CopperWire, 1)],
            vec![(Resource::SciencePack1, 1)],
            vec![],
            5,
            5,
        ),
        Recipe::new(
            "science_pack_2",
            EntityType::Assembler,
            vec![
                (Resource::CircuitBoard, 1),
                (Resource::Gear, 1),
            ],
            vec![(Resource::SciencePack2, 1)],
            vec![],
            8,
            10,
        ),
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
