use hecs::World;

use crate::ecs::components::*;
use crate::map::grid::Map;
use crate::resources::{get_input_sides, get_output_sides, EntityType, Facing, Resource};

/// Process one simulation tick in the correct order.
pub fn tick(world: &mut World, map: &mut Map, config: &SimConfig) {
    ore_deposit_emit(world, map, config);
    machine_output(world, map);
    conveyor_movement(world, map);
    splitter_process(world, map);
    merger_process(world, map);
    machine_consume(world, map, config);
    output_bin_consume(world, map);
    machine_process_tick(world);
}

pub struct SimConfig {
    pub ore_emit_interval: u32,
    pub smelter_process_ticks: u32,
    pub assembler_process_ticks: u32,
}

impl SimConfig {
    pub fn default_config() -> Self {
        SimConfig {
            ore_emit_interval: 4,
            smelter_process_ticks: 3,
            assembler_process_ticks: 5,
        }
    }
}

/// Step 1: Ore deposits emit ore to adjacent conveyors.
fn ore_deposit_emit(world: &mut World, map: &mut Map, _config: &SimConfig) {
    // Phase 1: Update emitter ticks and collect positions that are ready to emit.
    let mut ready_positions: Vec<(usize, usize)> = Vec::new();
    for (_entity, (pos, emitter)) in world.query_mut::<(&Position, &mut OreEmitter)>() {
        emitter.ticks_since_emit += 1;
        if emitter.ticks_since_emit < emitter.interval {
            continue;
        }
        emitter.ticks_since_emit = 0;
        ready_positions.push((pos.x, pos.y));
    }

    // Phase 2: For each ready emitter, find an adjacent tile to emit to.
    // Now we can borrow world immutably.
    let mut emits: Vec<(usize, usize)> = Vec::new();
    for (px, py) in ready_positions {
        for facing in Facing::all() {
            if let Some((nx, ny)) = map.neighbor(px, py, facing) {
                if map.resource_at(nx, ny).is_some() {
                    continue;
                }
                if let Some(adj_entity) = map.entity_at(nx, ny) {
                    if let Ok(kind) = world.get::<&EntityKind>(adj_entity) {
                        let adj_facing = world
                            .get::<&FacingComponent>(adj_entity)
                            .ok()
                            .map(|f| f.facing)
                            .unwrap_or(Facing::Right);
                        let input_sides = get_input_sides(kind.kind, adj_facing);
                        if input_sides.contains(&facing.opposite()) {
                            emits.push((nx, ny));
                            break;
                        }
                    }
                }
            }
        }
    }

    for (x, y) in emits {
        map.set_resource(x, y, Resource::IronOre);
    }
}

/// Step 2: Machines with completed output push to output tile.
fn machine_output(world: &mut World, map: &mut Map) {
    let mut pushes: Vec<(hecs::Entity, usize, usize, Resource)> = Vec::new();

    for (entity, (pos, kind, facing, proc)) in
        world.query::<(&Position, &EntityKind, &FacingComponent, &Processing)>().iter()
    {
        if !matches!(kind.kind, EntityType::Smelter | EntityType::Assembler) {
            continue;
        }
        let resource = match proc.output {
            Some(r) => r,
            None => continue,
        };

        let output_sides = get_output_sides(kind.kind, facing.facing);
        for side in output_sides {
            if let Some((nx, ny)) = map.neighbor(pos.x, pos.y, side) {
                if map.resource_at(nx, ny).is_some() {
                    continue;
                }
                if let Some(adj_entity) = map.entity_at(nx, ny) {
                    if let Ok(adj_kind) = world.get::<&EntityKind>(adj_entity) {
                        let adj_facing = world
                            .get::<&FacingComponent>(adj_entity)
                            .ok()
                            .map(|f| f.facing)
                            .unwrap_or(Facing::Right);
                        let input_sides = get_input_sides(adj_kind.kind, adj_facing);
                        if input_sides.contains(&side.opposite()) {
                            pushes.push((entity, nx, ny, resource));
                            break;
                        }
                    }
                }
            }
        }
    }

    for (entity, nx, ny, resource) in pushes {
        if let Ok(mut proc) = world.get::<&mut Processing>(entity) {
            proc.output = None;
        }
        map.set_resource(nx, ny, resource);
    }
}

/// Step 3: Conveyor movement — simultaneous pass.
fn conveyor_movement(world: &mut World, map: &mut Map) {
    let mut moves: Vec<(usize, usize, usize, usize, Resource)> = Vec::new();
    let mut destinations_claimed: std::collections::HashSet<(usize, usize)> =
        std::collections::HashSet::new();

    // Collect all conveyor pushes
    let mut conveyors: Vec<(hecs::Entity, usize, usize, Facing)> = Vec::new();
    for (entity, (pos, kind, facing)) in
        world.query::<(&Position, &EntityKind, &FacingComponent)>().iter()
    {
        if kind.kind != EntityType::BasicBelt {
            continue;
        }
        conveyors.push((entity, pos.x, pos.y, facing.facing));
    }

    // Sort by entity ID for determinism
    conveyors.sort_by_key(|(e, _, _, _)| *e);

    for (_entity, x, y, facing) in &conveyors {
        let resource = match map.resource_at(*x, *y) {
            Some(r) => r,
            None => continue,
        };

        if let Some((nx, ny)) = map.neighbor(*x, *y, *facing) {
            if destinations_claimed.contains(&(nx, ny)) {
                continue;
            }
            if map.resource_at(nx, ny).is_some() {
                continue;
            }

            // Check if destination can receive
            let can_receive = if let Some(adj_entity) = map.entity_at(nx, ny) {
                if let Ok(adj_kind) = world.get::<&EntityKind>(adj_entity) {
                    let adj_facing = world
                        .get::<&FacingComponent>(adj_entity)
                        .ok()
                        .map(|f| f.facing)
                        .unwrap_or(Facing::Right);
                    let input_sides = get_input_sides(adj_kind.kind, adj_facing);
                    input_sides.contains(&facing.opposite())
                } else {
                    false
                }
            } else {
                false
            };

            if can_receive {
                destinations_claimed.insert((nx, ny));
                moves.push((*x, *y, nx, ny, resource));
            }
        }
    }

    // Apply moves
    for (sx, sy, dx, dy, resource) in moves {
        map.remove_resource(sx, sy);
        map.set_resource(dx, dy, resource);
    }
}

/// Step 4: Machines consume from input tiles.
fn machine_consume(world: &mut World, map: &mut Map, config: &SimConfig) {
    let mut consumes: Vec<(hecs::Entity, usize, usize, Resource, bool)> = Vec::new();

    for (entity, (pos, kind, facing, proc)) in
        world.query::<(&Position, &EntityKind, &FacingComponent, &Processing)>().iter()
    {
        match kind.kind {
            EntityType::Smelter => {
                if proc.input_a.is_some() || proc.is_processing() || proc.output.is_some() {
                    continue;
                }
                // Check own tile first (conveyor may have pushed here)
                if let Some(Resource::IronOre) = map.resource_at(pos.x, pos.y) {
                    consumes.push((entity, pos.x, pos.y, Resource::IronOre, false));
                } else {
                    let input_side = facing.facing.opposite();
                    if let Some((nx, ny)) = map.neighbor(pos.x, pos.y, input_side) {
                        if let Some(Resource::IronOre) = map.resource_at(nx, ny) {
                            consumes.push((entity, nx, ny, Resource::IronOre, false));
                        }
                    }
                }
            }
            EntityType::Assembler => {
                let (side_a, side_b) = facing.facing.perpendicular();

                if proc.input_a.is_none() {
                    if let Some((nx, ny)) = map.neighbor(pos.x, pos.y, side_a) {
                        if let Some(Resource::IronIngot) = map.resource_at(nx, ny) {
                            consumes.push((entity, nx, ny, Resource::IronIngot, false));
                        }
                    }
                }
                if proc.input_b.is_none() {
                    if let Some((nx, ny)) = map.neighbor(pos.x, pos.y, side_b) {
                        if let Some(Resource::IronIngot) = map.resource_at(nx, ny) {
                            consumes.push((entity, nx, ny, Resource::IronIngot, true));
                        }
                    }
                }
            }
            _ => {}
        }
    }

    for (entity, nx, ny, _resource, is_input_b) in consumes {
        map.remove_resource(nx, ny);
        if let Ok(mut proc) = world.get::<&mut Processing>(entity) {
            if is_input_b {
                proc.input_b = Some(Resource::IronIngot);
            } else {
                if let Ok(kind) = world.get::<&EntityKind>(entity) {
                    match kind.kind {
                        EntityType::Smelter => {
                            proc.input_a = Some(Resource::IronOre);
                            proc.ticks_remaining = config.smelter_process_ticks;
                        }
                        EntityType::Assembler => {
                            proc.input_a = Some(Resource::IronIngot);
                            // Start processing only if both inputs are ready
                            if proc.input_b.is_some() {
                                proc.ticks_remaining = config.assembler_process_ticks;
                            }
                        }
                        _ => {}
                    }
                }
            }
            // For assembler: check if we now have both inputs
            if let Ok(kind) = world.get::<&EntityKind>(entity) {
                if kind.kind == EntityType::Assembler
                    && proc.input_a.is_some()
                    && proc.input_b.is_some()
                    && proc.ticks_remaining == 0
                {
                    proc.ticks_remaining = config.assembler_process_ticks;
                }
            }
        }
    }
}

/// Step 5a: Splitter routing.
fn splitter_process(world: &mut World, map: &mut Map) {
    let mut moves: Vec<(usize, usize, usize, usize)> = Vec::new();

    for (_entity, (pos, kind, facing, state)) in
        world.query::<(&Position, &EntityKind, &FacingComponent, &mut SplitterState)>().iter()
    {
        if kind.kind != EntityType::Splitter {
            continue;
        }
        // Check own tile first, then input side
        let input_side = facing.facing.opposite();
        let resource_pos = if map.resource_at(pos.x, pos.y).is_some() {
            Some((pos.x, pos.y))
        } else if let Some((ix, iy)) = map.neighbor(pos.x, pos.y, input_side) {
            if map.resource_at(ix, iy).is_some() {
                Some((ix, iy))
            } else {
                None
            }
        } else {
            None
        };
        let (ix, iy) = match resource_pos {
            Some(p) => p,
            None => continue,
        };

        let (perp_a, perp_b) = facing.facing.perpendicular();
        let (out_a, out_b) = match state.next_output {
            SplitterOutput::A => (perp_a, perp_b),
            SplitterOutput::B => (perp_b, perp_a),
        };

        for out_side in [out_a, out_b] {
            if let Some((ox, oy)) = map.neighbor(pos.x, pos.y, out_side) {
                if map.resource_at(ox, oy).is_some() {
                    continue;
                }
                if let Some(adj) = map.entity_at(ox, oy) {
                    if let Ok(adj_kind) = world.get::<&EntityKind>(adj) {
                        let adj_f = world
                            .get::<&FacingComponent>(adj)
                            .ok()
                            .map(|f| f.facing)
                            .unwrap_or(Facing::Right);
                        if get_input_sides(adj_kind.kind, adj_f)
                            .contains(&out_side.opposite())
                        {
                            moves.push((ix, iy, ox, oy));
                            state.next_output = state.next_output.toggle();
                            break;
                        }
                    }
                }
            }
        }
    }

    for (sx, sy, dx, dy) in moves {
        if let Some(resource) = map.remove_resource(sx, sy) {
            map.set_resource(dx, dy, resource);
        }
    }
}

/// Step 5b: Merger routing.
fn merger_process(world: &mut World, map: &mut Map) {
    let mut moves: Vec<(usize, usize, usize, usize)> = Vec::new();

    for (_entity, (pos, kind, facing, state)) in
        world.query::<(&Position, &EntityKind, &FacingComponent, &mut MergerState)>().iter()
    {
        if kind.kind != EntityType::Merger {
            continue;
        }

        let output_side = facing.facing;
        let (ox, oy) = match map.neighbor(pos.x, pos.y, output_side) {
            Some(p) => p,
            None => continue,
        };

        if map.resource_at(ox, oy).is_some() {
            continue; // Output blocked
        }

        // Check if output can receive
        let can_output = if let Some(adj) = map.entity_at(ox, oy) {
            if let Ok(adj_kind) = world.get::<&EntityKind>(adj) {
                let adj_f = world
                    .get::<&FacingComponent>(adj)
                    .ok()
                    .map(|f| f.facing)
                    .unwrap_or(Facing::Right);
                get_input_sides(adj_kind.kind, adj_f).contains(&output_side.opposite())
            } else {
                false
            }
        } else {
            false
        };

        if !can_output {
            continue;
        }

        let (perp_a, perp_b) = facing.facing.perpendicular();
        let (in_a, in_b) = match state.priority {
            MergerPriority::InputA => (perp_a, perp_b),
            MergerPriority::InputB => (perp_b, perp_a),
        };

        let mut pushed = false;
        for in_side in [in_a, in_b] {
            if let Some((ix, iy)) = map.neighbor(pos.x, pos.y, in_side) {
                if map.resource_at(ix, iy).is_some() {
                    moves.push((ix, iy, ox, oy));
                    state.priority = state.priority.toggle();
                    pushed = true;
                    break;
                }
            }
        }
        let _ = pushed;
    }

    for (sx, sy, dx, dy) in moves {
        if let Some(resource) = map.remove_resource(sx, sy) {
            map.set_resource(dx, dy, resource);
        }
    }
}

/// Step 6: Output bins consume adjacent resources.
fn output_bin_consume(world: &mut World, map: &mut Map) {
    let mut consumes: Vec<(hecs::Entity, usize, usize)> = Vec::new();

    for (entity, (pos, kind)) in world.query::<(&Position, &EntityKind)>().iter() {
        if kind.kind != EntityType::OutputBin {
            continue;
        }
        // Check own tile first (conveyors push resources here)
        if map.resource_at(pos.x, pos.y).is_some() {
            consumes.push((entity, pos.x, pos.y));
            continue;
        }
        // Then check adjacent tiles
        for facing in Facing::all() {
            if let Some((nx, ny)) = map.neighbor(pos.x, pos.y, facing) {
                if map.resource_at(nx, ny).is_some() {
                    // Check if the adjacent entity's output faces us
                    if let Some(adj) = map.entity_at(nx, ny) {
                        if let Ok(adj_kind) = world.get::<&EntityKind>(adj) {
                            let adj_f = world
                                .get::<&FacingComponent>(adj)
                                .ok()
                                .map(|f| f.facing)
                                .unwrap_or(Facing::Right);
                            let output_sides = get_output_sides(adj_kind.kind, adj_f);
                            if output_sides.contains(&facing.opposite()) {
                                consumes.push((entity, nx, ny));
                            }
                        }
                    }
                }
            }
        }
    }

    for (bin_entity, nx, ny) in consumes {
        if let Some(resource) = map.remove_resource(nx, ny) {
            if let Ok(mut counter) = world.get::<&mut OutputCounter>(bin_entity) {
                counter.add(resource);
            }
        }
    }
}

/// Decrement processing timers and produce output when done.
fn machine_process_tick(world: &mut World) {
    for (_entity, (kind, proc)) in world.query_mut::<(&EntityKind, &mut Processing)>() {
        if proc.ticks_remaining == 0 {
            continue;
        }
        proc.ticks_remaining -= 1;
        if proc.ticks_remaining == 0 {
            match kind.kind {
                EntityType::Smelter => {
                    proc.input_a = None;
                    proc.output = Some(Resource::IronIngot);
                }
                EntityType::Assembler => {
                    proc.input_a = None;
                    proc.input_b = None;
                    proc.output = Some(Resource::CircuitBoard);
                }
                _ => {}
            }
        }
    }
}
