use hecs::World;

use crate::ecs::components::*;
use crate::game::connections::{has_matching_input_port, resolve_to_anchor};
use crate::map::grid::Map;
use crate::map::multitile::building_footprint;
use crate::resources::{EntityType, Facing, Resource};

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

/// Compute the world-space positions of output ports for a building.
/// Returns (port_world_x, port_world_y, port_direction) for each output port.
fn output_port_positions(
    anchor_x: usize,
    anchor_y: usize,
    kind: EntityType,
    facing: Facing,
) -> Vec<(usize, usize, Facing)> {
    let fp = building_footprint(kind).rotate_to(facing);
    fp.ports
        .iter()
        .filter(|p| p.port_type.is_output())
        .map(|p| {
            (
                anchor_x.wrapping_add(p.offset_x as usize),
                anchor_y.wrapping_add(p.offset_y as usize),
                p.direction,
            )
        })
        .collect()
}

/// Compute the world-space positions of input ports for a building.
/// Returns (port_world_x, port_world_y, port_direction, port_index) for each input port.
fn input_port_positions(
    anchor_x: usize,
    anchor_y: usize,
    kind: EntityType,
    facing: Facing,
) -> Vec<(usize, usize, Facing, usize)> {
    let fp = building_footprint(kind).rotate_to(facing);
    fp.ports
        .iter()
        .filter(|p| p.port_type.is_input())
        .map(|p| {
            (
                anchor_x.wrapping_add(p.offset_x as usize),
                anchor_y.wrapping_add(p.offset_y as usize),
                p.direction,
                p.port_index,
            )
        })
        .collect()
}

/// Check if an adjacent entity/tile can receive from the given direction.
/// Handles both 1×1 entities and multi-tile buildings by resolving to anchor
/// and checking port definitions.
fn can_receive_from(world: &World, map: &Map, tile_x: usize, tile_y: usize, from_dir: Facing) -> bool {
    if let Some(adj_entity) = map.entity_at(tile_x, tile_y) {
        let anchor = resolve_to_anchor(world, adj_entity);
        has_matching_input_port(world, anchor, tile_x, tile_y, from_dir)
    } else {
        false
    }
}

/// Step 1: Ore deposits emit ore to adjacent tiles via output ports.
fn ore_deposit_emit(world: &mut World, map: &mut Map, _config: &SimConfig) {
    // Phase 1: Update emitter ticks and collect ready emitters.
    let mut ready: Vec<(usize, usize, EntityType, Facing)> = Vec::new();
    for (_entity, (pos, kind, emitter)) in
        world.query_mut::<(&Position, &EntityKind, &mut OreEmitter)>()
    {
        emitter.ticks_since_emit += 1;
        if emitter.ticks_since_emit < emitter.interval {
            continue;
        }
        emitter.ticks_since_emit = 0;
        let facing = Facing::Right; // OreEmitter entities always use their stored facing
        ready.push((pos.x, pos.y, kind.kind, facing));
    }

    // Re-query facing for each ready emitter (can't hold &mut and & simultaneously)
    let mut emits: Vec<(usize, usize)> = Vec::new();
    for (ax, ay, kind, _) in &ready {
        // Get actual facing from world
        let facing = if let Some(ent) = map.entity_at(*ax, *ay) {
            world
                .get::<&FacingComponent>(ent)
                .ok()
                .map(|f| f.facing)
                .unwrap_or(Facing::Right)
        } else {
            Facing::Right
        };

        let ports = output_port_positions(*ax, *ay, *kind, facing);
        for (px, py, dir) in ports {
            if let Some((nx, ny)) = map.neighbor(px, py, dir) {
                if map.resource_at(nx, ny).is_some() {
                    continue;
                }
                if can_receive_from(world, map, nx, ny, dir) {
                    emits.push((nx, ny));
                    break;
                }
            }
        }
    }

    for (x, y) in emits {
        map.set_resource(x, y, Resource::IronOre);
    }
}

/// Step 2: Machines with completed output push to adjacent tile via output ports.
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

        let ports = output_port_positions(pos.x, pos.y, kind.kind, facing.facing);
        for (px, py, dir) in ports {
            if let Some((nx, ny)) = map.neighbor(px, py, dir) {
                if map.resource_at(nx, ny).is_some() {
                    continue;
                }
                if can_receive_from(world, map, nx, ny, dir) {
                    pushes.push((entity, nx, ny, resource));
                    break;
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
/// Conveyors are 1×1, but their destination may be a multi-tile building tile.
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

            // Check if destination can receive from this direction
            if can_receive_from(world, map, nx, ny, *facing) {
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

/// Step 4: Machines consume from tiles adjacent to their input ports.
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
                // Check input port tiles for resources
                let ports = input_port_positions(pos.x, pos.y, kind.kind, facing.facing);
                for (px, py, dir, _idx) in ports {
                    // Check the port tile itself (conveyor may have pushed resource here)
                    if let Some(Resource::IronOre) = map.resource_at(px, py) {
                        consumes.push((entity, px, py, Resource::IronOre, false));
                        break;
                    }
                    // Check tile adjacent to port in the port's facing direction
                    if let Some((nx, ny)) = map.neighbor(px, py, dir) {
                        if let Some(Resource::IronOre) = map.resource_at(nx, ny) {
                            consumes.push((entity, nx, ny, Resource::IronOre, false));
                            break;
                        }
                    }
                }
            }
            EntityType::Assembler => {
                let ports = input_port_positions(pos.x, pos.y, kind.kind, facing.facing);
                for (px, py, dir, idx) in ports {
                    let is_b = idx == 1;
                    if is_b && proc.input_b.is_some() {
                        continue;
                    }
                    if !is_b && proc.input_a.is_some() {
                        continue;
                    }
                    // Check port tile
                    if let Some(Resource::IronIngot) = map.resource_at(px, py) {
                        consumes.push((entity, px, py, Resource::IronIngot, is_b));
                        continue;
                    }
                    // Check adjacent to port
                    if let Some((nx, ny)) = map.neighbor(px, py, dir) {
                        if let Some(Resource::IronIngot) = map.resource_at(nx, ny) {
                            consumes.push((entity, nx, ny, Resource::IronIngot, is_b));
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

/// Step 5a: Splitter routing — uses port definitions for 3×3 splitters.
fn splitter_process(world: &mut World, map: &mut Map) {
    let mut moves: Vec<(usize, usize, usize, usize)> = Vec::new();

    for (_entity, (pos, kind, facing, state)) in
        world.query::<(&Position, &EntityKind, &FacingComponent, &mut SplitterState)>().iter()
    {
        if kind.kind != EntityType::Splitter {
            continue;
        }

        // Find input resource via input ports
        let in_ports = input_port_positions(pos.x, pos.y, kind.kind, facing.facing);
        let resource_pos = in_ports.iter().find_map(|&(px, py, dir, _)| {
            // Check port tile
            if map.resource_at(px, py).is_some() {
                return Some((px, py));
            }
            // Check adjacent to port
            if let Some((nx, ny)) = map.neighbor(px, py, dir) {
                if map.resource_at(nx, ny).is_some() {
                    return Some((nx, ny));
                }
            }
            None
        });
        let (ix, iy) = match resource_pos {
            Some(p) => p,
            None => continue,
        };

        // Try output ports in priority order
        let out_ports = output_port_positions(pos.x, pos.y, kind.kind, facing.facing);
        let (first, second) = match state.next_output {
            SplitterOutput::A => (0usize, 1usize),
            SplitterOutput::B => (1usize, 0usize),
        };

        let indices = [first, second];
        for &idx in &indices {
            if idx >= out_ports.len() {
                continue;
            }
            let (px, py, dir) = out_ports[idx];
            if let Some((ox, oy)) = map.neighbor(px, py, dir) {
                if map.resource_at(ox, oy).is_some() {
                    continue;
                }
                if can_receive_from(world, map, ox, oy, dir) {
                    moves.push((ix, iy, ox, oy));
                    state.next_output = state.next_output.toggle();
                    break;
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

/// Step 5b: Merger routing — uses port definitions for 3×3 mergers.
fn merger_process(world: &mut World, map: &mut Map) {
    let mut moves: Vec<(usize, usize, usize, usize)> = Vec::new();

    for (_entity, (pos, kind, facing, state)) in
        world.query::<(&Position, &EntityKind, &FacingComponent, &mut MergerState)>().iter()
    {
        if kind.kind != EntityType::Merger {
            continue;
        }

        // Check output port
        let out_ports = output_port_positions(pos.x, pos.y, kind.kind, facing.facing);
        if out_ports.is_empty() {
            continue;
        }
        let (opx, opy, odir) = out_ports[0];
        let (ox, oy) = match map.neighbor(opx, opy, odir) {
            Some(p) => p,
            None => continue,
        };

        if map.resource_at(ox, oy).is_some() {
            continue; // Output blocked
        }

        // Check if output can receive
        if !can_receive_from(world, map, ox, oy, odir) {
            continue;
        }

        // Check input ports in priority order
        let in_ports = input_port_positions(pos.x, pos.y, kind.kind, facing.facing);
        let (first, second) = match state.priority {
            MergerPriority::InputA => (0usize, 1usize),
            MergerPriority::InputB => (1usize, 0usize),
        };

        let indices = [first, second];
        for &idx in &indices {
            if idx >= in_ports.len() {
                continue;
            }
            let (ipx, ipy, idir, _) = in_ports[idx];
            // Check adjacent to input port
            if let Some((ix, iy)) = map.neighbor(ipx, ipy, idir) {
                if map.resource_at(ix, iy).is_some() {
                    moves.push((ix, iy, ox, oy));
                    state.priority = state.priority.toggle();
                    break;
                }
            }
            // Also check port tile itself
            if map.resource_at(ipx, ipy).is_some() {
                moves.push((ipx, ipy, ox, oy));
                state.priority = state.priority.toggle();
                break;
            }
        }
    }

    for (sx, sy, dx, dy) in moves {
        if let Some(resource) = map.remove_resource(sx, sy) {
            map.set_resource(dx, dy, resource);
        }
    }
}

/// Step 6: Output bins consume resources at their input port tiles.
fn output_bin_consume(world: &mut World, map: &mut Map) {
    let mut consumes: Vec<(hecs::Entity, usize, usize)> = Vec::new();

    for (entity, (pos, kind)) in world.query::<(&Position, &EntityKind)>().iter() {
        if kind.kind != EntityType::OutputBin {
            continue;
        }
        // Skip secondary tiles
        if world.get::<&PartOfBuilding>(entity).is_ok() {
            continue;
        }

        let facing = world
            .get::<&FacingComponent>(entity)
            .ok()
            .map(|f| f.facing)
            .unwrap_or(Facing::Right);

        let in_ports = input_port_positions(pos.x, pos.y, kind.kind, facing);
        for (px, py, dir, _idx) in in_ports {
            // Check port tile itself (conveyor pushes resource here)
            if map.resource_at(px, py).is_some() {
                consumes.push((entity, px, py));
                continue;
            }
            // Check tile adjacent to port
            if let Some((nx, ny)) = map.neighbor(px, py, dir) {
                if map.resource_at(nx, ny).is_some() {
                    consumes.push((entity, nx, ny));
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
