use serde::{Deserialize, Serialize};

use crate::resources::{EntityType, Resource};
use super::tree::recipe_ids;

// ---------------------------------------------------------------------------
// Science Pack Recipes
// ---------------------------------------------------------------------------

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SciencePackRecipe {
    pub id: u16,
    pub output: Resource,
    pub output_count: u32,
    pub inputs: Vec<(Resource, u32)>,
    pub machine: EntityType,
    pub craft_time: u32,
}

/// Get all five science pack recipes.
pub fn get_science_pack_recipes() -> Vec<SciencePackRecipe> {
    vec![
        // SP1: IronIngot + CopperWire -> SciencePack1 in Assembler, 5 ticks
        SciencePackRecipe {
            id: recipe_ids::SCIENCE_PACK_1,
            output: Resource::SciencePack1,
            output_count: 1,
            inputs: vec![(Resource::IronIngot, 1), (Resource::CopperWire, 1)],
            machine: EntityType::Assembler,
            craft_time: 5,
        },
        // SP2: CircuitBoard + Steel -> SciencePack2 in Assembler, 8 ticks
        SciencePackRecipe {
            id: recipe_ids::SCIENCE_PACK_2,
            output: Resource::SciencePack2,
            output_count: 1,
            inputs: vec![(Resource::CircuitBoard, 1), (Resource::Steel, 1)],
            machine: EntityType::Assembler,
            craft_time: 8,
        },
        // SP3: AdvancedCircuit + BatteryPack + Gear -> SciencePack3 in PrecisionAssembler, 12 ticks
        SciencePackRecipe {
            id: recipe_ids::SCIENCE_PACK_3,
            output: Resource::SciencePack3,
            output_count: 1,
            inputs: vec![
                (Resource::AdvancedCircuit, 1),
                (Resource::BatteryPack, 1),
                (Resource::Gear, 1),
            ],
            machine: EntityType::PrecisionAssembler,
            craft_time: 12,
        },
        // SP4: Processor + Servo + CompositePanel -> SciencePack4 in PrecisionAssembler, 15 ticks
        SciencePackRecipe {
            id: recipe_ids::SCIENCE_PACK_4,
            output: Resource::SciencePack4,
            output_count: 1,
            inputs: vec![
                (Resource::Processor, 1),
                (Resource::Servo, 1),
                (Resource::CompositePanel, 1),
            ],
            machine: EntityType::PrecisionAssembler,
            craft_time: 15,
        },
        // SP5: QuantumProcessor + FusionCore + SuperconductorWire + NanobotSwarm + RocketFuel
        //      -> SciencePack5 in Megassembler, 20 ticks
        SciencePackRecipe {
            id: recipe_ids::SCIENCE_PACK_5,
            output: Resource::SciencePack5,
            output_count: 1,
            inputs: vec![
                (Resource::QuantumProcessor, 1),
                (Resource::FusionCore, 1),
                (Resource::SuperconductorWire, 1),
                (Resource::NanobotSwarm, 1),
                (Resource::RocketFuel, 1),
            ],
            machine: EntityType::Megassembler,
            craft_time: 20,
        },
    ]
}

/// Look up a science pack recipe by its output resource.
pub fn get_recipe_for_pack(pack: Resource) -> Option<SciencePackRecipe> {
    get_science_pack_recipes()
        .into_iter()
        .find(|r| r.output == pack)
}

/// Look up a science pack recipe by recipe ID.
pub fn get_recipe_by_id(id: u16) -> Option<SciencePackRecipe> {
    get_science_pack_recipes()
        .into_iter()
        .find(|r| r.id == id)
}

/// Get the ordered list of all science pack resources (SP1 through SP5).
pub fn all_science_packs() -> Vec<Resource> {
    vec![
        Resource::SciencePack1,
        Resource::SciencePack2,
        Resource::SciencePack3,
        Resource::SciencePack4,
        Resource::SciencePack5,
    ]
}

/// Get the tier for a science pack resource (1-5).
pub fn science_pack_tier(pack: Resource) -> Option<u8> {
    match pack {
        Resource::SciencePack1 => Some(1),
        Resource::SciencePack2 => Some(2),
        Resource::SciencePack3 => Some(3),
        Resource::SciencePack4 => Some(4),
        Resource::SciencePack5 => Some(5),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn five_science_pack_recipes() {
        let recipes = get_science_pack_recipes();
        assert_eq!(recipes.len(), 5);
    }

    #[test]
    fn recipe_ids_match() {
        let recipes = get_science_pack_recipes();
        assert_eq!(recipes[0].id, recipe_ids::SCIENCE_PACK_1);
        assert_eq!(recipes[1].id, recipe_ids::SCIENCE_PACK_2);
        assert_eq!(recipes[2].id, recipe_ids::SCIENCE_PACK_3);
        assert_eq!(recipes[3].id, recipe_ids::SCIENCE_PACK_4);
        assert_eq!(recipes[4].id, recipe_ids::SCIENCE_PACK_5);
    }

    #[test]
    fn sp1_recipe() {
        let r = get_recipe_for_pack(Resource::SciencePack1).unwrap();
        assert_eq!(r.machine, EntityType::Assembler);
        assert_eq!(r.craft_time, 5);
        assert_eq!(r.inputs.len(), 2);
    }

    #[test]
    fn sp2_recipe() {
        let r = get_recipe_for_pack(Resource::SciencePack2).unwrap();
        assert_eq!(r.machine, EntityType::Assembler);
        assert_eq!(r.craft_time, 8);
        assert!(r.inputs.iter().any(|(res, _)| *res == Resource::CircuitBoard));
        assert!(r.inputs.iter().any(|(res, _)| *res == Resource::Steel));
    }

    #[test]
    fn sp3_needs_precision_assembler() {
        let r = get_recipe_for_pack(Resource::SciencePack3).unwrap();
        assert_eq!(r.machine, EntityType::PrecisionAssembler);
        assert_eq!(r.inputs.len(), 3);
    }

    #[test]
    fn sp5_needs_megassembler() {
        let r = get_recipe_for_pack(Resource::SciencePack5).unwrap();
        assert_eq!(r.machine, EntityType::Megassembler);
        assert_eq!(r.craft_time, 20);
        assert_eq!(r.inputs.len(), 5);
    }

    #[test]
    fn lookup_by_id() {
        let r = get_recipe_by_id(recipe_ids::SCIENCE_PACK_3).unwrap();
        assert_eq!(r.output, Resource::SciencePack3);
    }

    #[test]
    fn all_packs_ordered() {
        let packs = all_science_packs();
        assert_eq!(packs.len(), 5);
        assert_eq!(packs[0], Resource::SciencePack1);
        assert_eq!(packs[4], Resource::SciencePack5);
    }

    #[test]
    fn tier_mapping() {
        assert_eq!(science_pack_tier(Resource::SciencePack1), Some(1));
        assert_eq!(science_pack_tier(Resource::SciencePack5), Some(5));
        assert_eq!(science_pack_tier(Resource::IronOre), None);
    }
}
