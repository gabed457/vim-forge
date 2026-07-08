//! Wiring tests: prove each progression loop is actually LIVE in the
//! simulation — recipes, belt tiers, research, auto-sell economy,
//! contracts, power, and pollution.

use hecs::World;
use vimforge::ecs::components::*;
use vimforge::ecs::recipes::recipe_by_id;
use vimforge::ecs::systems::{self, SimConfig};
use vimforge::game::session::GameSession;
use vimforge::map::grid::Map;
use vimforge::resources::{EntityType, Facing, Resource};

fn place(
    world: &mut World,
    map: &mut Map,
    x: usize,
    y: usize,
    kind: EntityType,
    facing: Facing,
) -> hecs::Entity {
    map.place_multitile_entity(world, x, y, kind, facing, false)
        .expect("placement should succeed")
}

fn freeplay_session() -> GameSession {
    let mut s = GameSession::new(160, 48);
    s.start_freeplay();
    s
}

// ---------------------------------------------------------------------------
// (a) Recipe-driven production: copper ore -> copper ingot -> copper wire
// ---------------------------------------------------------------------------

#[test]
fn copper_chain_produces_copper_wire_via_recipes() {
    let mut world = World::new();
    let mut map = Map::new(30, 10);
    let config = SimConfig::default_config();

    // CopperDeposit (3x2) at (1,2): output port (3,3) emits to (4,3)
    place(&mut world, &mut map, 1, 2, EntityType::CopperDeposit, Facing::Right);
    for x in 4..=6 {
        place(&mut world, &mut map, x, 3, EntityType::BasicBelt, Facing::Right);
    }
    // Smelter (3x3) at (7,2): in (7,3), out (9,3) -- smelt_copper
    place(&mut world, &mut map, 7, 2, EntityType::Smelter, Facing::Right);
    for x in 10..=12 {
        place(&mut world, &mut map, x, 3, EntityType::BasicBelt, Facing::Right);
    }
    // WireMill (3x3) at (13,2): in (13,3), out (15,3) -- wire_copper
    place(&mut world, &mut map, 13, 2, EntityType::WireMill, Facing::Right);
    place(&mut world, &mut map, 16, 3, EntityType::BasicBelt, Facing::Right);
    // OutputBin (3x2) at (17,2): input port (17,3)
    let bin = place(&mut world, &mut map, 17, 2, EntityType::OutputBin, Facing::Right);

    for i in 0..200u64 {
        systems::tick_ex(&mut world, &mut map, &config, i);
    }

    let counter = world.get::<&OutputCounter>(bin).unwrap();
    assert!(
        counter.get(Resource::CopperWire) >= 3,
        "copper chain should deliver copper wire (got {:?})",
        counter.counts
    );
}

#[test]
fn iron_extractor_still_emits_iron_ore() {
    let mut world = World::new();
    let mut map = Map::new(20, 10);
    let config = SimConfig::default_config();

    place(&mut world, &mut map, 2, 2, EntityType::OreDeposit, Facing::Right);
    place(&mut world, &mut map, 5, 3, EntityType::BasicBelt, Facing::Right);

    systems::tick(&mut world, &mut map, &config);
    systems::tick(&mut world, &mut map, &config);
    systems::tick(&mut world, &mut map, &config);
    systems::tick(&mut world, &mut map, &config);
    assert_eq!(map.resource_at(5, 3), Some(Resource::IronOre));
}

#[test]
fn coal_and_stone_extractors_emit_their_resources() {
    let mut world = World::new();
    let mut map = Map::new(20, 12);
    let config = SimConfig::default_config();

    place(&mut world, &mut map, 2, 2, EntityType::CoalDeposit, Facing::Right);
    place(&mut world, &mut map, 5, 3, EntityType::BasicBelt, Facing::Right);
    place(&mut world, &mut map, 2, 6, EntityType::StoneQuarry, Facing::Right);
    place(&mut world, &mut map, 5, 7, EntityType::BasicBelt, Facing::Right);

    for i in 0..4 {
        systems::tick_ex(&mut world, &mut map, &config, i);
    }
    assert_eq!(map.resource_at(5, 3), Some(Resource::Coal));
    assert_eq!(map.resource_at(5, 7), Some(Resource::Stone));
}

// ---------------------------------------------------------------------------
// (b) Belt tiers: Fast and Express are strictly faster than Basic
// ---------------------------------------------------------------------------

/// Ticks for one item to traverse a 12-belt run of the given tier.
fn traverse_ticks(tier: EntityType) -> u64 {
    let mut world = World::new();
    let mut map = Map::new(20, 5);
    let config = SimConfig::default_config();

    for x in 2..=14 {
        place(&mut world, &mut map, x, 2, tier, Facing::Right);
    }
    map.set_resource(2, 2, Resource::IronOre);

    for i in 0..60u64 {
        systems::tick_ex(&mut world, &mut map, &config, i);
        if map.resource_at(14, 2).is_some() {
            return i + 1;
        }
    }
    panic!("item never reached the end of the {:?} run", tier);
}

#[test]
fn fast_and_express_belts_strictly_faster_than_basic() {
    let basic = traverse_ticks(EntityType::BasicBelt);
    let fast = traverse_ticks(EntityType::FastBelt);
    let express = traverse_ticks(EntityType::ExpressBelt);

    assert_eq!(basic, 12, "BasicBelt timing must stay exactly 1 tile/tick");
    assert!(fast < basic, "FastBelt ({fast}) must beat BasicBelt ({basic})");
    assert!(
        express < fast,
        "ExpressBelt ({express}) must beat FastBelt ({fast})"
    );
}

// ---------------------------------------------------------------------------
// (c) Research: lab consumes packs, progress accrues, completion unlocks
//     a recipe (recipe gating in action)
// ---------------------------------------------------------------------------

#[test]
fn lab_consumes_packs_research_completes_and_unlocks_recipe() {
    use vimforge::research::tree::TechId;

    let mut s = freeplay_session();
    // The freeplay map pre-places a ResearchLab at (66,36); its input port
    // tile is (66,37).
    assert_eq!(
        s.entity_type_at(66, 36),
        Some(EntityType::ResearchLab),
        "freeplay should pre-place a research lab"
    );

    // Target a tech that unlocks a recipe: StoneworkProcessing -> stone brick.
    s.app.research.completed.insert(TechId::BasicSmelting);
    assert!(s.app.research.start_research(TechId::StoneworkProcessing));

    // Recipe gating: stone brick recipe is LOCKED before the tech completes.
    let brick = recipe_by_id("kiln_stone_brick").unwrap();
    assert!(
        !s.app.simulation.config.recipe_unlocked(&brick),
        "stone brick must be locked before research"
    );

    // Feed science packs onto the lab's input port tile, one per tick.
    let mut progressed = false;
    for _ in 0..40 {
        s.app.map.set_resource(66, 37, Resource::SciencePack1);
        s.tick(1);
        if s.app.research.progress > 0 {
            progressed = true;
        }
        if s.app.research.completed.contains(&TechId::StoneworkProcessing) {
            break;
        }
    }

    assert!(progressed, "research progress should accrue from consumed packs");
    assert!(
        s.app.research.completed.contains(&TechId::StoneworkProcessing),
        "research should complete from delivered science packs"
    );
    assert!(
        s.app.simulation.config.recipe_unlocked(&brick),
        "completing StoneworkProcessing must unlock the stone brick recipe"
    );
    // Auto-select picked the next cheapest available tech.
    assert!(
        s.app.research.current.is_some(),
        "a new research should be auto-selected after completion"
    );
}

#[test]
fn locked_recipe_does_not_run_until_researched() {
    let mut world = World::new();
    let mut map = Map::new(20, 10);
    let mut config = SimConfig::default_config();
    // Freeplay-style gating with nothing researched.
    config.unlocked_recipes = Some(std::collections::HashSet::new());

    // Kiln (3x3) at (5,2): input port (5,3)
    let kiln = place(&mut world, &mut map, 5, 2, EntityType::Kiln, Facing::Right);
    map.set_resource(5, 3, Resource::Stone);

    for i in 0..20u64 {
        systems::tick_ex(&mut world, &mut map, &config, i);
    }
    {
        let proc = world.get::<&Processing>(kiln).unwrap();
        assert!(
            proc.output.is_none() && proc.input_a.is_none(),
            "locked stone-brick recipe must not consume or run"
        );
    }

    // Unlock it (STONE_BRICK = 1) and the kiln springs to life.
    config.unlocked_recipes.as_mut().unwrap().insert(1);
    for i in 0..20u64 {
        systems::tick_ex(&mut world, &mut map, &config, i);
    }
    assert!(
        map.resource_at(5, 3).is_none(),
        "stone should be consumed once the recipe is unlocked"
    );
    let proc = world.get::<&Processing>(kiln).unwrap();
    assert_eq!(
        proc.output,
        Some(Resource::StoneBrick),
        "kiln should have produced a stone brick (no belt to push it out)"
    );
}

// ---------------------------------------------------------------------------
// (d) Economy: output-bin delivery credits cash and records a market sale
// ---------------------------------------------------------------------------

#[test]
fn output_delivery_credits_cash_and_records_market_sale() {
    let mut s = freeplay_session();
    let cash_before = s.app.economy.cash;
    assert!(cash_before > 0, "freeplay should start with meaningful cash");

    // Freeplay bin at (60,40): its left input port tile is (60,41).
    assert_eq!(s.entity_type_at(60, 40), Some(EntityType::OutputBin));
    s.app.map.set_resource(60, 41, Resource::IronIngot);
    s.tick(1);

    let (_, ingots, _, _) = s.output_totals();
    assert_eq!(ingots, 1, "bin should have consumed the ingot");
    assert!(
        s.app.economy.cash > cash_before,
        "auto-sell should credit cash ({} -> {})",
        cash_before,
        s.app.economy.cash
    );
    assert!(
        s.app
            .market
            .supply_pressure
            .get(&Resource::IronIngot)
            .copied()
            .unwrap_or(0.0)
            > 0.0,
        "sale should be recorded in the market (supply pressure)"
    );
    assert_eq!(
        s.app.delivered_lifetime.get(&Resource::IronIngot).copied(),
        Some(1)
    );
}

#[test]
fn cycle_expenses_are_deducted_and_cash_never_goes_negative() {
    let mut s = freeplay_session();
    let cash_before = s.app.economy.cash;
    s.tick(60); // one full economy cycle, no income
    assert!(
        s.app.economy.cash < cash_before,
        "upkeep expenses should be deducted every cycle"
    );
    assert!(s.app.last_expense_report.total > 0.0);

    // Drain the treasury and confirm the clamp-at-zero warning behavior.
    s.app.economy.cash = 1;
    s.tick(60);
    assert!(
        s.app.economy.cash >= 0,
        "cash must clamp at 0 instead of going bankrupt"
    );
}

// ---------------------------------------------------------------------------
// (e) Contracts: generated, auto-accepted, delivered against, rewarded
// ---------------------------------------------------------------------------

#[test]
fn contract_generated_auto_accepted_delivered_and_rewarded() {
    let mut s = freeplay_session();

    // Establish IronIngot as a known-produced resource (drives generation
    // eligibility and auto-accept).
    s.app.map.set_resource(60, 41, Resource::IronIngot);
    s.tick(1);
    assert!(s.app.delivered_lifetime.contains_key(&Resource::IronIngot));

    // Contracts are generated on a cycle boundary once 300 ticks have
    // passed; run to tick 360 to cross it.
    while s.app.simulation.tick_count < 360 {
        s.tick(1);
    }
    assert!(
        !s.app.contract_board.active.is_empty(),
        "an iron-ingot contract should be generated and auto-accepted"
    );
    let needed: u64 = s.app.contract_board.active[0]
        .requirements
        .iter()
        .map(|r| r.remaining())
        .sum();
    assert!(needed > 0);
    let reward = s.app.contract_board.active[0].reward;
    let earned_before = s.app.economy.total_earned;

    // Deliver enough ingots (one per tick through the bin port).
    for _ in 0..needed {
        s.app.map.set_resource(60, 41, Resource::IronIngot);
        s.tick(1);
    }
    // Run to the next cycle boundary so completion is processed.
    while s.app.simulation.tick_count % 60 != 0 {
        s.tick(1);
    }

    assert_eq!(
        s.app.contract_board.completed_count, 1,
        "contract should complete after delivering its quantity"
    );
    assert!(s.app.contract_board.reputation > 0, "reputation should rise");
    assert!(
        s.app.economy.total_earned >= earned_before + reward as u64,
        "contract reward should be credited to the treasury"
    );
}

// ---------------------------------------------------------------------------
// (f) Power: coal generator powers machines; missing power halves speed
// ---------------------------------------------------------------------------

/// Ticks for a CoolantProcessor (tier-3 building) to finish one batch under
/// freeplay power rules, optionally with a fueled coal generator.
fn coolant_batch_ticks(with_fueled_generator: bool) -> u64 {
    let mut world = World::new();
    let mut map = Map::new(30, 20);
    let mut config = SimConfig::default_config();
    config.freeplay_power = true;

    // CoolantProcessor (4x4) at (5,5): input ports (5,6) and (5,7).
    let machine = place(&mut world, &mut map, 5, 5, EntityType::CoolantProcessor, Facing::Right);

    if with_fueled_generator {
        // CoalGenerator (3x3) at (15,5): fuel input port (15,6).
        place(&mut world, &mut map, 15, 5, EntityType::CoalGenerator, Facing::Right);
        map.set_resource(15, 6, Resource::Coal);
        // Let the generator pick up fuel and start burning.
        systems::tick_ex(&mut world, &mut map, &config, 0);
    }

    // Feed the recipe (process_coolant: Water x2).
    map.set_resource(5, 6, Resource::Water);
    map.set_resource(5, 7, Resource::Water);

    for i in 1..100u64 {
        let report = systems::tick_ex(&mut world, &mut map, &config, i);
        if with_fueled_generator {
            assert!(report.generators_present > 0);
        }
        let proc = world.get::<&Processing>(machine).unwrap();
        if proc.output.is_some() {
            return i;
        }
    }
    panic!("coolant batch never finished");
}

#[test]
fn fueled_generator_powers_machines_absence_halves_speed() {
    let unpowered = coolant_batch_ticks(false);
    let powered = coolant_batch_ticks(true);
    assert!(
        powered < unpowered,
        "a fueled coal generator must speed up tier-3 machines ({powered} vs {unpowered})"
    );
}

#[test]
fn generator_consumes_coal_and_reports_supply() {
    let mut world = World::new();
    let mut map = Map::new(20, 10);
    let config = SimConfig::default_config();

    let gen = place(&mut world, &mut map, 5, 2, EntityType::CoalGenerator, Facing::Right);
    map.set_resource(5, 3, Resource::Coal); // fuel input port tile

    let report = systems::tick_ex(&mut world, &mut map, &config, 0);
    assert!(map.resource_at(5, 3).is_none(), "coal should be consumed as fuel");
    assert_eq!(report.generators_present, 1);
    assert_eq!(report.generators_active, 1);
    assert!(report.power_supply > 0.0);

    let fuel = world.get::<&FuelStore>(gen).unwrap();
    assert!(fuel.is_burning());
}

#[test]
fn campaign_machines_unaffected_without_generators() {
    // Grandfather clause: no generators + no freeplay flag = full speed.
    let mut world = World::new();
    let mut map = Map::new(20, 10);
    let config = SimConfig::default_config();

    let smelter = place(&mut world, &mut map, 5, 2, EntityType::Smelter, Facing::Right);
    map.set_resource(5, 3, Resource::IronOre);

    // smelt_iron takes 3 ticks: consumed on tick 1, done by tick 3.
    for i in 0..3u64 {
        systems::tick_ex(&mut world, &mut map, &config, i);
    }
    let proc = world.get::<&Processing>(smelter).unwrap();
    assert_eq!(
        proc.output,
        Some(Resource::IronIngot),
        "smelter must run at full speed with no generators on the map"
    );
}

// ---------------------------------------------------------------------------
// (g) Pollution: recipes emit waste as pollution; fines hit the treasury
// ---------------------------------------------------------------------------

#[test]
fn processing_emits_pollution() {
    let mut world = World::new();
    let mut map = Map::new(20, 10);
    let config = SimConfig::default_config();

    place(&mut world, &mut map, 5, 2, EntityType::Smelter, Facing::Right);
    map.set_resource(5, 3, Resource::IronOre);

    let mut total_pollution = 0.0;
    for i in 0..10u64 {
        let report = systems::tick_ex(&mut world, &mut map, &config, i);
        total_pollution += report.pollution;
    }
    assert!(
        total_pollution > 0.0,
        "completing smelt_iron (slag waste) must emit pollution"
    );
}

#[test]
fn high_pollution_incurs_fines_in_expenses() {
    let mut s = freeplay_session();
    s.app.pollution.level = 300.0; // above the 200 fine threshold
    let cash_before = s.app.economy.cash;
    s.tick(60); // one economy cycle
    assert!(
        s.app.last_expense_report.pollution_fine > 0.0,
        "pollution above threshold must produce fines"
    );
    assert!(s.app.economy.cash < cash_before);
}

// ---------------------------------------------------------------------------
// Freeplay world sanity
// ---------------------------------------------------------------------------

#[test]
fn freeplay_world_is_seeded_with_variety() {
    let s = freeplay_session();
    let mut kinds = std::collections::HashSet::new();
    for (_e, kind) in s.app.world.query::<&EntityKind>().iter() {
        kinds.insert(kind.kind);
    }
    for required in [
        EntityType::OreDeposit,
        EntityType::CopperDeposit,
        EntityType::CoalDeposit,
        EntityType::StoneQuarry,
        EntityType::OilWell,
        EntityType::WaterPump,
        EntityType::SandExtractor,
        EntityType::OutputBin,
        EntityType::ResearchLab,
    ] {
        assert!(kinds.contains(&required), "freeplay map missing {:?}", required);
    }
    assert_eq!(s.app.map.width, 120);
    assert_eq!(s.app.map.height, 80);
    assert!(s.app.economy.cash >= 25_000);
    // Research auto-selected and recipe gating active.
    assert!(s.app.research.current.is_some());
    assert!(s.app.simulation.config.unlocked_recipes.is_some());
}
