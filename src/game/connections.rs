use std::collections::HashMap;

use hecs::World;

use crate::ecs::components::*;
use crate::map::grid::Map;
use crate::resources::{get_input_sides, get_output_sides, EntityType, Facing};

pub struct ConnectionGraph {
    pub connections: HashMap<hecs::Entity, Vec<hecs::Entity>>,
}

impl ConnectionGraph {
    pub fn new() -> Self {
        ConnectionGraph {
            connections: HashMap::new(),
        }
    }

    /// Rebuild the entire connection graph from the current map state.
    pub fn rebuild(&mut self, world: &World, map: &Map) {
        self.connections.clear();

        for (entity, (pos, kind)) in world.query::<(&Position, &EntityKind)>().iter() {
            let facing = world
                .get::<&FacingComponent>(entity)
                .ok()
                .map(|f| f.facing)
                .unwrap_or(Facing::Right);
            let output_sides = get_output_sides(kind.kind, facing);

            for side in output_sides {
                if let Some((nx, ny)) = map.neighbor(pos.x, pos.y, side) {
                    if let Some(adj_entity) = map.entity_at(nx, ny) {
                        if let Ok(adj_kind) = world.get::<&EntityKind>(adj_entity) {
                            let adj_facing = world
                                .get::<&FacingComponent>(adj_entity)
                                .ok()
                                .map(|f| f.facing)
                                .unwrap_or(Facing::Right);
                            let adj_inputs = get_input_sides(adj_kind.kind, adj_facing);
                            if adj_inputs.contains(&side.opposite()) {
                                self.connections
                                    .entry(entity)
                                    .or_default()
                                    .push(adj_entity);
                                self.connections
                                    .entry(adj_entity)
                                    .or_default()
                                    .push(entity);
                            }
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
