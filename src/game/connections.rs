use std::collections::HashMap;

use hecs::World;

use crate::ecs::components::{EntityKind, FacingComponent, PartOfBuilding, Position};
use crate::map::grid::Map;
use crate::map::multitile::building_footprint;
use crate::resources::{EntityType, Facing};

pub struct ConnectionGraph {
    pub connections: HashMap<hecs::Entity, Vec<hecs::Entity>>,
}

/// Resolve an entity to its anchor: if the entity carries a `PartOfBuilding`
/// component, return the anchor entity; otherwise return itself.
pub fn resolve_to_anchor(world: &World, entity: hecs::Entity) -> hecs::Entity {
    if let Ok(pob) = world.get::<&PartOfBuilding>(entity) {
        pob.anchor
    } else {
        entity
    }
}

/// Check whether `anchor` has an input port at the world-space tile
/// (`tile_x`, `tile_y`) that faces `from_direction.opposite()`.
pub fn has_matching_input_port(
    world: &World,
    anchor: hecs::Entity,
    tile_x: usize,
    tile_y: usize,
    from_direction: Facing,
) -> bool {
    let pos = match world.get::<&Position>(anchor) {
        Ok(p) => *p,
        Err(_) => return false,
    };
    let kind = match world.get::<&EntityKind>(anchor) {
        Ok(k) => k.kind,
        Err(_) => return false,
    };
    let facing = world
        .get::<&FacingComponent>(anchor)
        .ok()
        .map(|f| f.facing)
        .unwrap_or(Facing::Right);

    let fp = building_footprint(kind).rotate_to(facing);
    let needed_dir = from_direction.opposite();

    fp.ports.iter().any(|port| {
        if !port.port_type.is_input() {
            return false;
        }
        let wx = pos.x.wrapping_add(port.offset_x as usize);
        let wy = pos.y.wrapping_add(port.offset_y as usize);
        wx == tile_x && wy == tile_y && port.direction == needed_dir
    })
}

impl ConnectionGraph {
    pub fn new() -> Self {
        ConnectionGraph {
            connections: HashMap::new(),
        }
    }

    /// Rebuild the entire connection graph from the current map state.
    /// Uses building port definitions so that multi-tile footprints with
    /// ports at specific tile offsets are handled correctly.
    pub fn rebuild(&mut self, world: &World, map: &Map) {
        self.connections.clear();

        // Only iterate over anchor entities (those with EntityKind but NOT PartOfBuilding).
        for (entity, (pos, kind)) in world.query::<(&Position, &EntityKind)>().iter() {
            if world.get::<&PartOfBuilding>(entity).is_ok() {
                continue;
            }

            let facing = world
                .get::<&FacingComponent>(entity)
                .ok()
                .map(|f| f.facing)
                .unwrap_or(Facing::Right);

            let fp = building_footprint(kind.kind).rotate_to(facing);

            for port in &fp.ports {
                // Only follow output (and waste-output) ports.
                if !port.port_type.is_output() && !port.port_type.is_waste() {
                    continue;
                }

                // World-space tile where this port lives.
                let wx = pos.x.wrapping_add(port.offset_x as usize);
                let wy = pos.y.wrapping_add(port.offset_y as usize);

                // The adjacent tile in the port's direction.
                if let Some((nx, ny)) = map.neighbor(wx, wy, port.direction) {
                    if let Some(adj_entity) = map.entity_at(nx, ny) {
                        let adj_anchor = resolve_to_anchor(world, adj_entity);

                        // Avoid self-connections.
                        if adj_anchor == entity {
                            continue;
                        }

                        if has_matching_input_port(world, adj_anchor, nx, ny, port.direction) {
                            self.connections
                                .entry(entity)
                                .or_default()
                                .push(adj_anchor);
                            self.connections
                                .entry(adj_anchor)
                                .or_default()
                                .push(entity);
                        }
                    }
                }
            }
        }

        // Deduplicate
        for conns in self.connections.values_mut() {
            conns.sort();
            conns.dedup();
        }
    }

    pub fn get_connections(&self, entity: hecs::Entity) -> &[hecs::Entity] {
        self.connections
            .get(&entity)
            .map(|v| v.as_slice())
            .unwrap_or(&[])
    }

    /// Follow conveyor chain to its end (for `%` motion on conveyors).
    pub fn follow_conveyor_chain_forward(
        &self,
        world: &World,
        map: &Map,
        start_x: usize,
        start_y: usize,
    ) -> Option<(usize, usize)> {
        let mut x = start_x;
        let mut y = start_y;
        let mut visited = std::collections::HashSet::new();
        visited.insert((x, y));

        loop {
            let entity = map.entity_at(x, y)?;
            let kind = world.get::<&EntityKind>(entity).ok()?;
            if kind.kind != EntityType::BasicBelt {
                return Some((x, y));
            }
            let facing = world.get::<&FacingComponent>(entity).ok()?.facing;
            let (nx, ny) = map.neighbor(x, y, facing)?;
            if visited.contains(&(nx, ny)) {
                return Some((x, y)); // Loop detected
            }
            if map.entity_at(nx, ny).is_none() {
                return Some((x, y)); // Dead end
            }
            visited.insert((nx, ny));
            x = nx;
            y = ny;
        }
    }

    /// Follow conveyor chain backward.
    pub fn follow_conveyor_chain_backward(
        &self,
        world: &World,
        map: &Map,
        start_x: usize,
        start_y: usize,
    ) -> Option<(usize, usize)> {
        let mut x = start_x;
        let mut y = start_y;
        let mut visited = std::collections::HashSet::new();
        visited.insert((x, y));

        loop {
            let entity = map.entity_at(x, y)?;
            let kind = world.get::<&EntityKind>(entity).ok()?;
            if kind.kind != EntityType::BasicBelt {
                return Some((x, y));
            }
            let facing = world.get::<&FacingComponent>(entity).ok()?.facing;
            // Go backward = opposite of facing
            let (nx, ny) = map.neighbor(x, y, facing.opposite())?;
            if visited.contains(&(nx, ny)) {
                return Some((x, y));
            }
            if map.entity_at(nx, ny).is_none() {
                return Some((x, y));
            }
            visited.insert((nx, ny));
            x = nx;
            y = ny;
        }
    }
}
