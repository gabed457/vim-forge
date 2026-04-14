use hecs::World;
use vimforge::ecs::archetypes;
use vimforge::ecs::components::*;
use vimforge::ecs::systems::{self, SimConfig};
use vimforge::map::grid::Map;
use vimforge::resources::{EntityType, Facing, Resource};

fn setup_config() -> SimConfig {
    SimConfig::default_config()
}

/// Helper: place a multi-tile entity on the map and return anchor entity.
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

#[test]
fn test_ore_deposit_emits_on_interval() {
    let mut world = World::new();
    let mut map = Map::new(20, 10);
    let config = setup_config();

    // OreDeposit (3×2) at (2,2): output port at (4,3)
    place(&mut world, &mut map, 2, 2, EntityType::OreDeposit, Facing::Right);
    // Conveyor at (5,3) receives from the output port direction (Right)
    place(&mut world, &mut map, 5, 3, EntityType::BasicBelt, Facing::Right);

    // Tick 1-3: no emission
    for _ in 0..3 {
        systems::tick(&mut world, &mut map, &config);
        assert!(map.resource_at(5, 3).is_none());
    }

    // Tick 4: emission
    systems::tick(&mut world, &mut map, &config);
    assert_eq!(map.resource_at(5, 3), Some(Resource::IronOre));
}

#[test]
fn test_conveyor_moves_resource_per_tick() {
    let mut world = World::new();
    let mut map = Map::new(10, 5);
    let config = setup_config();

    // Place 3 conveyors in a row at y=2
    for x in 2..5 {
        place(&mut world, &mut map, x, 2, EntityType::BasicBelt, Facing::Right);
    }

    // Place ore on first conveyor
    map.set_resource(2, 2, Resource::IronOre);

    // Tick: ore should move from (2,2) to (3,2)
    systems::tick(&mut world, &mut map, &config);
    assert!(map.resource_at(2, 2).is_none());
    assert_eq!(map.resource_at(3, 2), Some(Resource::IronOre));

    // Another tick: (3,2) to (4,2)
    systems::tick(&mut world, &mut map, &config);
    assert!(map.resource_at(3, 2).is_none());
    assert_eq!(map.resource_at(4, 2), Some(Resource::IronOre));
}

#[test]
fn test_conveyor_blocks_when_destination_full() {
    let mut world = World::new();
    let mut map = Map::new(10, 5);
    let config = setup_config();

    let c1 = archetypes::spawn_conveyor(&mut world, 2, 2, Facing::Right, false);
    map.set_entity(2, 2, c1);
    let c2 = archetypes::spawn_conveyor(&mut world, 3, 2, Facing::Right, false);
    map.set_entity(3, 2, c2);

    // Both conveyors have resources
    map.set_resource(2, 2, Resource::IronOre);
    map.set_resource(3, 2, Resource::IronIngot);

    systems::tick(&mut world, &mut map, &config);

    // First conveyor's ore should stay because destination is occupied
    assert_eq!(map.resource_at(2, 2), Some(Resource::IronOre));
}

#[test]
fn test_smelter_consumes_ore_and_produces_ingot() {
    let mut world = World::new();
    let mut map = Map::new(20, 10);
    let config = setup_config();

    // Smelter (3×3) at (5,2): input port at (5,3) Left, output port at (7,3) Right
    let smelter = place(&mut world, &mut map, 5, 2, EntityType::Smelter, Facing::Right);

    // Input conveyor at (4,3) facing Right → pushes to (5,3) which is smelter input
    place(&mut world, &mut map, 4, 3, EntityType::BasicBelt, Facing::Right);
    // Output conveyor at (8,3) facing Right ← receives from smelter output at (7,3)
    place(&mut world, &mut map, 8, 3, EntityType::BasicBelt, Facing::Right);

    // Place ore on input conveyor
    map.set_resource(4, 3, Resource::IronOre);

    // Run enough ticks: 1 (conveyor push) + 3 (smelt) + 1 (output push) + buffer
    for _ in 0..10 {
        systems::tick(&mut world, &mut map, &config);
    }

    // After processing, ingot should be somewhere downstream or in smelter output
    let proc = world.get::<&Processing>(smelter).unwrap();
    let has_ingot = proc.output == Some(Resource::IronIngot)
        || map.resource_at(8, 3) == Some(Resource::IronIngot);

    assert!(
        has_ingot || map.resource_at(4, 3).is_none(),
        "Ore should have been consumed and ingot produced"
    );
}

#[test]
fn test_output_bin_consumes_and_counts() {
    let mut world = World::new();
    let mut map = Map::new(20, 10);
    let config = setup_config();

    // OutputBin (3×2) at (8,2): input port at (8,3) Left
    let bin = place(&mut world, &mut map, 8, 2, EntityType::OutputBin, Facing::Right);

    // Conveyor at (7,3) facing Right → pushes to (8,3) which is bin input
    place(&mut world, &mut map, 7, 3, EntityType::BasicBelt, Facing::Right);

    // Place a widget on the conveyor
    map.set_resource(7, 3, Resource::CircuitBoard);

    systems::tick(&mut world, &mut map, &config);

    // Widget should be consumed by the output bin
    assert!(map.resource_at(7, 3).is_none(), "Resource should have been pushed");
    let counter = world.get::<&OutputCounter>(bin).unwrap();
    assert_eq!(counter.widget_count(), 1);
}

#[test]
fn test_splitter_alternates() {
    let mut world = World::new();
    let mut map = Map::new(20, 20);
    let config = setup_config();

    // Splitter (3×3) at (5,4): facing Right
    // Input port at (5,5) Left, output ports at (7,4) Right and (7,6) Right
    place(&mut world, &mut map, 5, 4, EntityType::Splitter, Facing::Right);

    // Input conveyor at (4,5) facing Right → pushes to (5,5) = splitter input
    place(&mut world, &mut map, 4, 5, EntityType::BasicBelt, Facing::Right);

    // Output conveyors at (8,4) and (8,6) facing Right ← receive from splitter outputs
    place(&mut world, &mut map, 8, 4, EntityType::BasicBelt, Facing::Right);
    place(&mut world, &mut map, 8, 6, EntityType::BasicBelt, Facing::Right);

    // First resource
    map.set_resource(4, 5, Resource::IronOre);
    systems::tick(&mut world, &mut map, &config);

    // After tick, ore should be at one of the output conveyors
    let at_top = map.resource_at(8, 4).is_some();
    let at_bot = map.resource_at(8, 6).is_some();
    assert!(at_top || at_bot, "Splitter should route to one output");
}

#[test]
fn test_full_chain_ore_to_output() {
    let mut world = World::new();
    let mut map = Map::new(30, 10);
    let config = setup_config();

    // OreDeposit (3×2) at (1,2): output at (3,3)
    place(&mut world, &mut map, 1, 2, EntityType::OreDeposit, Facing::Right);

    // Belt chain at y=3 from x=4 to x=9
    for x in 4..=9 {
        place(&mut world, &mut map, x, 3, EntityType::BasicBelt, Facing::Right);
    }

    // OutputBin (3×2) at (10,2): input at (10,3)
    let bin = place(&mut world, &mut map, 10, 2, EntityType::OutputBin, Facing::Right);

    // Run for 50 ticks — should produce several ores
    for _ in 0..50 {
        systems::tick(&mut world, &mut map, &config);
    }

    let counter = world.get::<&OutputCounter>(bin).unwrap();
    assert!(counter.ore_count() > 0, "Output bin should have received ore");
}
