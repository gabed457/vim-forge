use hecs::World;

use crate::ecs::components::*;
use crate::resources::{EntityType, Facing};

pub fn spawn_ore_deposit(world: &mut World, x: usize, y: usize, interval: u32) -> hecs::Entity {
    world.spawn((
        Position { x, y },
        EntityKind {
            kind: EntityType::OreDeposit,
        },
        OreEmitter::new(interval),
    ))
}

pub fn spawn_smelter(
    world: &mut World,
    x: usize,
    y: usize,
    facing: Facing,
    player_placed: bool,
) -> hecs::Entity {
    let mut builder = hecs::EntityBuilder::new();
    builder.add(Position { x, y });
    builder.add(EntityKind {
        kind: EntityType::Smelter,
    });
    builder.add(FacingComponent { facing });
    builder.add(Processing::new());
    if player_placed {
        builder.add(PlayerPlaced);
    }
    world.spawn(builder.build())
}

pub fn spawn_assembler(
    world: &mut World,
    x: usize,
    y: usize,
    facing: Facing,
    player_placed: bool,
) -> hecs::Entity {
    let mut builder = hecs::EntityBuilder::new();
    builder.add(Position { x, y });
    builder.add(EntityKind {
        kind: EntityType::Assembler,
    });
    builder.add(FacingComponent { facing });
    builder.add(Processing::new());
    if player_placed {
        builder.add(PlayerPlaced);
    }
    world.spawn(builder.build())
}

pub fn spawn_conveyor(
    world: &mut World,
    x: usize,
    y: usize,
    facing: Facing,
    player_placed: bool,
) -> hecs::Entity {
    let mut builder = hecs::EntityBuilder::new();
    builder.add(Position { x, y });
    builder.add(EntityKind {
        kind: EntityType::Conveyor,
    });
    builder.add(FacingComponent { facing });
    if player_placed {
        builder.add(PlayerPlaced);
    }
    world.spawn(builder.build())
}

pub fn spawn_splitter(
    world: &mut World,
    x: usize,
    y: usize,
    facing: Facing,
    player_placed: bool,
) -> hecs::Entity {
    let mut builder = hecs::EntityBuilder::new();
    builder.add(Position { x, y });
    builder.add(EntityKind {
        kind: EntityType::Splitter,
    });
    builder.add(FacingComponent { facing });
    builder.add(SplitterState::new());
    if player_placed {
        builder.add(PlayerPlaced);
    }
    world.spawn(builder.build())
}

pub fn spawn_merger(
    world: &mut World,
    x: usize,
    y: usize,
    facing: Facing,
    player_placed: bool,
) -> hecs::Entity {
    let mut builder = hecs::EntityBuilder::new();
    builder.add(Position { x, y });
    builder.add(EntityKind {
        kind: EntityType::Merger,
    });
    builder.add(FacingComponent { facing });
    builder.add(MergerState::new());
    if player_placed {
        builder.add(PlayerPlaced);
    }
    world.spawn(builder.build())
}

pub fn spawn_output_bin(world: &mut World, x: usize, y: usize) -> hecs::Entity {
    world.spawn((
        Position { x, y },
        EntityKind {
            kind: EntityType::OutputBin,
        },
        OutputCounter::new(),
    ))
}

pub fn spawn_wall(
    world: &mut World,
    x: usize,
    y: usize,
    player_placed: bool,
) -> hecs::Entity {
    let mut builder = hecs::EntityBuilder::new();
    builder.add(Position { x, y });
    builder.add(EntityKind {
        kind: EntityType::Wall,
    });
    if player_placed {
        builder.add(PlayerPlaced);
    }
    world.spawn(builder.build())
}

pub fn spawn_entity(
    world: &mut World,
    entity_type: EntityType,
    x: usize,
    y: usize,
    facing: Facing,
    player_placed: bool,
) -> hecs::Entity {
    match entity_type {
        EntityType::OreDeposit => spawn_ore_deposit(world, x, y, 4),
        EntityType::Smelter => spawn_smelter(world, x, y, facing, player_placed),
        EntityType::Assembler => spawn_assembler(world, x, y, facing, player_placed),
        EntityType::Conveyor => spawn_conveyor(world, x, y, facing, player_placed),
        EntityType::Splitter => spawn_splitter(world, x, y, facing, player_placed),
        EntityType::Merger => spawn_merger(world, x, y, facing, player_placed),
        EntityType::OutputBin => spawn_output_bin(world, x, y),
        EntityType::Wall => spawn_wall(world, x, y, player_placed),
    }
}
