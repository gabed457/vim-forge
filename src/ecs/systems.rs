use std::collections::HashSet;

use hecs::World;

use crate::ecs::components::*;
use crate::ecs::recipes::{recipes_for, Recipe};
use crate::game::connections::{has_matching_input_port, resolve_to_anchor};
use crate::map::grid::Map;
use crate::map::multitile::building_footprint;
use crate::research::labs::get_lab_spec;
use crate::resources::{EntityType, Facing, Resource};

/// Process one simulation tick in the correct order (legacy signature,
/// kept for existing tests). Uses tick index 0 — fine for factories without
/// fast belts or power generators.
pub fn tick(world: &mut World, map: &mut Map, config: &SimConfig) {
    let _ = tick_ex(world, map, config, 0);
}

/// Per-tick side effects reported back to the game session.
#[derive(Clone, Debug, Default)]
pub struct TickReport {
    /// Pollution units emitted this tick (recipe waste + generator smoke).
    pub pollution: f64,
    /// Effective MW available from active (fueled + burning) generators.
    pub power_supply: f64,
    /// MW demanded by machines that were processing this tick.
    pub power_demand: f64,
    /// Fueled generator buildings on the map (burning or not).
    pub generators_present: u32,
    /// Generators actively burning fuel this tick.
    pub generators_active: u32,
    /// Whether machines ran at full speed this tick.
    pub powered: bool,
}

/// Process one simulation tick.
///
/// `tick_index` drives fast-belt cadence and half-speed processing when
/// underpowered.
pub fn tick_ex(world: &mut World, map: &mut Map, config: &SimConfig, tick_index: u64) -> TickReport {
    let mut report = TickReport {
        powered: true,
        ..TickReport::default()
    };

    ore_deposit_emit(world, map, config);
    machine_output(world, map);

    // Belt tier scheme (BasicBelt timing is the campaign regression gate and
    // is EXACTLY unchanged):
    //   pass 1: every belt tier moves one tile        -> Basic = 1 tile/tick
    //   pass 2: ExpressBelt moves again every tick    -> Express = 2 tiles/tick
    //           FastBelt moves again on even ticks    -> Fast = 1.5 tiles/tick
    conveyor_movement(
        world,
        map,
        &[EntityType::BasicBelt, EntityType::FastBelt, EntityType::ExpressBelt],
    );
    let mut second_pass = vec![EntityType::ExpressBelt];
    if tick_index % 2 == 0 {
        second_pass.push(EntityType::FastBelt);
    }
    conveyor_movement(world, map, &second_pass);

    splitter_process(world, map);
    merger_process(world, map);
    machine_consume(world, map, config);
    lab_consume(world, map);
    generator_tick(world, map, &mut report);
    output_bin_consume(world, map);
    machine_process_tick(world, config, tick_index, &mut report);

    report
}

pub struct SimConfig {
    pub ore_emit_interval: u32,
    pub smelter_process_ticks: u32,
    pub assembler_process_ticks: u32,
    /// Recipes available to machines. `None` = everything unlocked (campaign
    /// levels and unit tests). Freeplay sets `Some(set)` of unlocked numeric
    /// recipe ids; recipes with `numeric_id == 0` are always available.
    pub unlocked_recipes: Option<HashSet<u16>>,
    /// Freeplay power pressure: when no generators exist, advanced machines
    /// (building tier >= 3, i.e. 4x4 footprints and larger) run at half
    /// speed. Campaign levels leave this false (grandfather clause).
    pub freeplay_power: bool,
}

impl SimConfig {
    pub fn default_config() -> Self {
        SimConfig {
            ore_emit_interval: 4,
            smelter_process_ticks: 3,
            assembler_process_ticks: 5,
            unlocked_recipes: None,
            freeplay_power: false,
        }
    }

    /// Whether a recipe may run under the current research gating.
    pub fn recipe_unlocked(&self, recipe: &Recipe) -> bool {
        if recipe.numeric_id == 0 {
            return true;
        }
        match &self.unlocked_recipes {
            None => true,
            Some(set) => set.contains(&recipe.numeric_id),
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
fn can_receive_from(world: &World, map: &Map, tile_x: usize, tile_y: usize, from_dir: Facing) -> bool {
    if let Some(adj_entity) = map.entity_at(tile_x, tile_y) {
        let anchor = resolve_to_anchor(world, adj_entity);
        has_matching_input_port(world, anchor, tile_x, tile_y, from_dir)
    } else {
        false
    }
}

/// Entity facing looked up from the world (falls back to Right).
fn facing_of(world: &World, entity: hecs::Entity) -> Facing {
    world
        .get::<&FacingComponent>(entity)
        .ok()
        .map(|f| f.facing)
        .unwrap_or(Facing::Right)
}

/// Buffered inputs of a Processing component as a small vec.
fn buffered_inputs(proc: &Processing) -> Vec<Resource> {
    let mut v = Vec::with_capacity(2);
    if let Some(a) = proc.input_a {
        v.push(a);
    }
    if let Some(b) = proc.input_b {
        v.push(b);
    }
    v
}

/// True if `needed` (a recipe's flat inputs) contains the multiset `have`.
fn multiset_contains(needed: &[Resource], have: &[Resource]) -> bool {
    let mut pool = needed.to_vec();
    for h in have {
        match pool.iter().position(|r| r == h) {
            Some(i) => {
                pool.swap_remove(i);
            }
            None => return false,
        }
    }
    true
}

/// True if `needed` equals the multiset `have`.
fn multiset_equals(needed: &[Resource], have: &[Resource]) -> bool {
    needed.len() == have.len() && multiset_contains(needed, have)
}

/// The recipe whose flat inputs exactly match `have` (unique per building
/// by recipe-book design).
fn exact_recipe(kind: EntityType, have: &[Resource], config: &SimConfig) -> Option<Recipe> {
    recipes_for(kind)
        .into_iter()
        .filter(|r| config.recipe_unlocked(r))
        .find(|r| multiset_equals(&r.flat_inputs(), have))
}

/// Step 1: Extractors emit their resource to adjacent tiles via output ports.
fn ore_deposit_emit(world: &mut World, map: &mut Map, _config: &SimConfig) {
    // Phase 1: Update emitter ticks and collect ready emitters.
    let mut ready: Vec<(usize, usize, EntityType)> = Vec::new();
    for (_entity, (pos, kind, emitter)) in
        world.query_mut::<(&Position, &EntityKind, &mut OreEmitter)>()
    {
        emitter.ticks_since_emit += 1;
        if emitter.ticks_since_emit < emitter.interval {
            continue;
        }
        emitter.ticks_since_emit = 0;
        ready.push((pos.x, pos.y, kind.kind));
    }

    let mut emits: Vec<(usize, usize, Resource)> = Vec::new();
    for (ax, ay, kind) in &ready {
        let resource = match kind.extracted_resource() {
            Some(r) => r,
            None => continue,
        };
        let facing = if let Some(ent) = map.entity_at(*ax, *ay) {
            facing_of(world, ent)
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
                    emits.push((nx, ny, resource));
                    break;
                }
            }
        }
    }

    for (x, y, resource) in emits {
        map.set_resource(x, y, resource);
    }
}

/// Step 2: Machines with completed output push to adjacent tile via output ports.
fn machine_output(world: &mut World, map: &mut Map) {
    let mut pushes: Vec<(hecs::Entity, usize, usize, Resource)> = Vec::new();

    for (entity, (pos, kind, facing, proc)) in
        world.query::<(&Position, &EntityKind, &FacingComponent, &Processing)>().iter()
    {
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
            if proc.output_remaining > 1 {
                proc.output_remaining -= 1;
            } else {
                proc.output_remaining = 0;
                proc.output = None;
            }
        }
        map.set_resource(nx, ny, resource);
    }
}

/// Step 3: Conveyor movement — simultaneous pass over the given belt tiers.
fn conveyor_movement(world: &mut World, map: &mut Map, tiers: &[EntityType]) {
    let mut moves: Vec<(usize, usize, usize, usize, Resource)> = Vec::new();
    let mut destinations_claimed: std::collections::HashSet<(usize, usize)> =
        std::collections::HashSet::new();

    // Collect all conveyor pushes
    let mut conveyors: Vec<(hecs::Entity, usize, usize, Facing)> = Vec::new();
    for (entity, (pos, kind, facing)) in
        world.query::<(&Position, &EntityKind, &FacingComponent)>().iter()
    {
        if !tiers.contains(&kind.kind) {
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
///
/// Recipe-driven: a machine consumes an item only when, combined with what it
/// has already buffered, the item still fits some unlocked recipe of that
/// building. When the buffered inputs exactly match a recipe, processing
/// starts for `recipe.ticks`.
fn machine_consume(world: &mut World, map: &mut Map, config: &SimConfig) {
    // (machine, tile_x, tile_y, resource, into_slot_b)
    let mut consumes: Vec<(hecs::Entity, usize, usize, Resource, bool)> = Vec::new();

    for (entity, (pos, kind, facing, proc)) in
        world.query::<(&Position, &EntityKind, &FacingComponent, &Processing)>().iter()
    {
        // Machines buffer inputs only while idle (not processing, no output
        // waiting) — same one-batch-at-a-time model as before.
        if proc.is_processing() || proc.output.is_some() {
            continue;
        }

        let recipes: Vec<Recipe> = recipes_for(kind.kind)
            .into_iter()
            .filter(|r| config.recipe_unlocked(r))
            .collect();
        if recipes.is_empty() {
            continue;
        }

        let mut buffered = buffered_inputs(proc);
        if buffered.len() >= 2 {
            continue;
        }

        let ports = input_port_positions(pos.x, pos.y, kind.kind, facing.facing);
        for (px, py, dir, _idx) in ports {
            if buffered.len() >= 2 {
                break;
            }
            // Check the port tile itself, then the tile adjacent to the port.
            let mut candidates: Vec<(usize, usize)> = vec![(px, py)];
            if let Some((nx, ny)) = map.neighbor(px, py, dir) {
                candidates.push((nx, ny));
            }
            for (tx, ty) in candidates {
                if let Some(r) = map.resource_at(tx, ty) {
                    let mut want = buffered.clone();
                    want.push(r);
                    let fits = recipes
                        .iter()
                        .any(|rec| multiset_contains(&rec.flat_inputs(), &want));
                    if fits {
                        consumes.push((entity, tx, ty, r, !buffered.is_empty()));
                        buffered.push(r);
                    }
                    break; // at most one item per port per tick
                }
            }
        }
    }

    for (entity, tx, ty, resource, into_b) in consumes {
        map.remove_resource(tx, ty);
        if let Ok(mut proc) = world.get::<&mut Processing>(entity) {
            if into_b {
                proc.input_b = Some(resource);
            } else {
                proc.input_a = Some(resource);
            }
        }
    }

    // Start processing wherever the buffered inputs complete a recipe.
    for (_entity, (kind, proc)) in world.query_mut::<(&EntityKind, &mut Processing)>() {
        if proc.is_processing() || proc.output.is_some() {
            continue;
        }
        let buffered = buffered_inputs(proc);
        if buffered.is_empty() {
            continue;
        }
        if let Some(recipe) = exact_recipe(kind.kind, &buffered, config) {
            proc.ticks_remaining = recipe.ticks;
        }
    }
}

/// Step 4b: Research labs pull science packs from their input ports into
/// their LabStock. The research loop (GameSession::post_tick) drains the
/// stock into research progress.
fn lab_consume(world: &mut World, map: &mut Map) {
    let mut consumes: Vec<(hecs::Entity, usize, usize, Resource)> = Vec::new();

    for (entity, (pos, kind, stock)) in
        world.query::<(&Position, &EntityKind, &LabStock)>().iter()
    {
        let spec = match get_lab_spec(kind.kind) {
            Some(s) => s,
            None => continue,
        };
        let facing = facing_of(world, entity);

        for (px, py, dir, _idx) in input_port_positions(pos.x, pos.y, kind.kind, facing) {
            let mut candidates: Vec<(usize, usize)> = vec![(px, py)];
            if let Some((nx, ny)) = map.neighbor(px, py, dir) {
                candidates.push((nx, ny));
            }
            for (tx, ty) in candidates {
                if let Some(r) = map.resource_at(tx, ty) {
                    if spec.accepted_packs.contains(&r) && stock.get(r) < LabStock::CAP {
                        consumes.push((entity, tx, ty, r));
                    }
                    break;
                }
            }
        }
    }

    for (entity, tx, ty, resource) in consumes {
        map.remove_resource(tx, ty);
        if let Ok(mut stock) = world.get::<&mut LabStock>(entity) {
            stock.add(resource);
        }
    }
}

/// Step 4c: Power generators consume fuel deliveries and burn fuel.
/// Each active generator contributes `base_mw * 4` effective MW (one coal
/// generator comfortably powers a handful of small machines).
fn generator_tick(world: &mut World, map: &mut Map, report: &mut TickReport) {
    use crate::power::generators::generator_spec;

    // Fuel pickup from input ports.
    let mut consumes: Vec<(hecs::Entity, usize, usize)> = Vec::new();
    for (entity, (pos, kind, fuel)) in
        world.query::<(&Position, &EntityKind, &FuelStore)>().iter()
    {
        if fuel.units >= FuelStore::CAP {
            continue;
        }
        let spec = match generator_spec(kind.kind) {
            Some(s) => s,
            None => continue,
        };
        let fuel_resource = match spec.fuel_input {
            Some((r, _)) => r,
            None => continue,
        };
        let facing = facing_of(world, entity);
        for (px, py, dir, _idx) in input_port_positions(pos.x, pos.y, kind.kind, facing) {
            let mut candidates: Vec<(usize, usize)> = vec![(px, py)];
            if let Some((nx, ny)) = map.neighbor(px, py, dir) {
                candidates.push((nx, ny));
            }
            for (tx, ty) in candidates {
                if map.resource_at(tx, ty) == Some(fuel_resource) {
                    consumes.push((entity, tx, ty));
                    break;
                }
            }
        }
    }
    for (entity, tx, ty) in consumes {
        if let Ok(mut fuel) = world.get::<&mut FuelStore>(entity) {
            if fuel.units < FuelStore::CAP {
                map.remove_resource(tx, ty);
                fuel.units += 1;
            }
        }
    }

    // Burn fuel and tally supply.
    for (_entity, (kind, fuel)) in world.query_mut::<(&EntityKind, &mut FuelStore)>() {
        report.generators_present += 1;
        if fuel.burn_ticks_remaining == 0 && fuel.units > 0 {
            fuel.units -= 1;
            fuel.burn_ticks_remaining = FuelStore::BURN_TICKS_PER_UNIT;
        }
        if fuel.burn_ticks_remaining > 0 {
            fuel.burn_ticks_remaining -= 1;
            report.generators_active += 1;
            if let Some(spec) = generator_spec(kind.kind) {
                report.power_supply += spec.base_mw * 4.0;
                if spec.waste_output.is_some() {
                    report.pollution += crate::waste::pollution::UNSCRUBBED_GENERATOR_POLLUTION;
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

        let facing = facing_of(world, entity);

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

/// Step 7: Decrement processing timers and produce output when done.
///
/// Power model (light touch):
/// - No fueled generators on the map -> full speed everywhere, EXCEPT that
///   with `freeplay_power` set, advanced machines (building tier >= 3) run
///   at half speed until a generator supplies power.
/// - Generators present -> all machines run at full speed while
///   supply >= demand, half speed otherwise. Demand is the summed
///   `power_mw` of the recipes currently being processed.
fn machine_process_tick(
    world: &mut World,
    config: &SimConfig,
    tick_index: u64,
    report: &mut TickReport,
) {
    // Demand from machines that are mid-process.
    let mut demand = 0.0;
    for (_e, (kind, proc)) in world.query::<(&EntityKind, &Processing)>().iter() {
        if proc.ticks_remaining == 0 {
            continue;
        }
        let buffered = buffered_inputs(proc);
        if let Some(recipe) = exact_recipe(kind.kind, &buffered, config) {
            demand += recipe.power_mw as f64;
        }
    }
    report.power_demand = demand;

    let grid_powered = report.generators_present == 0 || report.power_supply >= demand;
    report.powered = grid_powered;

    let half_tick = tick_index % 2 == 1;

    for (_entity, (kind, proc)) in world.query_mut::<(&EntityKind, &mut Processing)>() {
        if proc.ticks_remaining == 0 {
            continue;
        }

        let full_speed = if report.generators_present > 0 {
            grid_powered
        } else {
            !(config.freeplay_power && kind.kind.tier() >= 3)
        };
        if !full_speed && half_tick {
            continue; // half speed: only advance on even ticks
        }

        proc.ticks_remaining -= 1;
        if proc.ticks_remaining == 0 {
            let buffered = buffered_inputs(proc);
            proc.input_a = None;
            proc.input_b = None;
            if let Some(recipe) = exact_recipe(kind.kind, &buffered, config) {
                proc.output = Some(recipe.primary_output());
                proc.output_remaining = recipe.outputs[0].amount.max(1) as u32;
                report.pollution += recipe.waste_units() as f64;
            }
        }
    }
}
