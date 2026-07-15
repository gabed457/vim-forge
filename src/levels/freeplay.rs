use crate::resources::{EntityType, Facing};

use super::config::{CompletionCondition, LevelConfig, LevelEntity};

/// Freeplay mode: a rich 120x80 "Factorio start" with every raw resource
/// represented, several output bins (deliveries auto-sell at market price),
/// and one pre-placed research lab so the science loop is discoverable.
/// Uses fixed (deterministic) positions for reproducibility.
pub fn config() -> LevelConfig {
    let mut entities = Vec::new();

    let mut add = |x: usize, y: usize, entity_type: EntityType| {
        entities.push(LevelEntity {
            x,
            y,
            entity_type,
            facing: Facing::Right,
            player_placed: false,
        });
    };

    // ---- Extractors (3x2 each) --------------------------------------
    // Iron: the bread-and-butter starter patches
    for &(x, y) in &[(4, 6), (4, 20), (30, 10), (10, 52)] {
        add(x, y, EntityType::OreDeposit);
    }
    // Copper: wire/circuit chains
    for &(x, y) in &[(4, 34), (30, 40), (14, 64)] {
        add(x, y, EntityType::CopperDeposit);
    }
    // Coal: fuel for generators and steam
    for &(x, y) in &[(4, 46), (44, 6), (52, 58)] {
        add(x, y, EntityType::CoalDeposit);
    }
    // Stone: bricks, concrete, sand
    for &(x, y) in &[(18, 6), (40, 66)] {
        add(x, y, EntityType::StoneQuarry);
    }
    // Oil: plastics, rubber, fuel
    for &(x, y) in &[(70, 8), (78, 50)] {
        add(x, y, EntityType::OilWell);
    }
    // Water: chemistry, steam, coolant
    for &(x, y) in &[(60, 20), (90, 64)] {
        add(x, y, EntityType::WaterPump);
    }
    // Sand: glass and silicon wafers
    for &(x, y) in &[(70, 30), (24, 74)] {
        add(x, y, EntityType::SandExtractor);
    }
    // Late-game specialties, one patch each, far from spawn
    add(90, 12, EntityType::SulfurMine);
    add(98, 40, EntityType::BauxiteMine);
    add(108, 70, EntityType::UraniumMine);
    add(100, 24, EntityType::LithiumExtractor);
    add(110, 4, EntityType::RareEarthExtractor);
    add(54, 72, EntityType::BiomassHarvester);
    add(88, 74, EntityType::GasExtractor);

    // ---- Delivery points (3x2 output bins) --------------------------
    for &(x, y) in &[(114, 12), (114, 30), (114, 48), (114, 62), (60, 40)] {
        add(x, y, EntityType::OutputBin);
    }

    // ---- One research lab (3x3), input port on its left ------------
    add(66, 36, EntityType::ResearchLab);

    // ---- A little ruined-structure flavor ---------------------------
    for x in 46..=52 {
        add(x, 24, EntityType::Wall);
    }
    for y in 25..=29 {
        add(46, y, EntityType::Wall);
    }
    for x in 84..=88 {
        add(x, 56, EntityType::Wall);
    }

    LevelConfig {
        number: 0,
        name: "Freeplay",
        map_width: 120,
        map_height: 80,
        entities,
        objective: "Build a factory empire: research tech, fulfill contracts, stay solvent.",
        hints: vec![
            "Deliveries to output bins sell automatically at market price.",
            "Belt iron+copper into an assembler making science pack 1, then belt packs to the lab.",
            "Check :research :market :contracts :finance :recipe for the full picture.",
        ],
        allowed_commands: None,
        completion: CompletionCondition::Custom("freeplay".to_string()),
    }
}
