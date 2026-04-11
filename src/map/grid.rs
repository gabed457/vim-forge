use hecs::World;

use crate::ecs::components::*;
use crate::resources::{EntityType, Facing, Resource};

#[derive(Clone, Debug)]
pub struct Tile {
    pub entity: Option<hecs::Entity>,
    pub resource: Option<Resource>,
}

impl Tile {
    pub fn empty() -> Self {
        Tile {
            entity: None,
            resource: None,
        }
    }

    pub fn has_entity(&self) -> bool {
        self.entity.is_some()
    }

    pub fn has_resource(&self) -> bool {
        self.resource.is_some()
    }

    pub fn is_empty(&self) -> bool {
        self.entity.is_none() && self.resource.is_none()
    }
}

#[derive(Clone, Debug)]
pub struct Map {
    pub width: usize,
    pub height: usize,
    pub tiles: Vec<Vec<Tile>>,
}

impl Map {
    pub fn new(width: usize, height: usize) -> Self {
        let tiles = (0..height)
            .map(|_| (0..width).map(|_| Tile::empty()).collect())
            .collect();
        Map {
            width,
            height,
            tiles,
        }
    }

    pub fn in_bounds(&self, x: usize, y: usize) -> bool {
        x < self.width && y < self.height
    }

    pub fn in_bounds_signed(&self, x: isize, y: isize) -> bool {
        x >= 0 && y >= 0 && (x as usize) < self.width && (y as usize) < self.height
    }

    pub fn get_tile(&self, x: usize, y: usize) -> Option<&Tile> {
        self.tiles.get(y).and_then(|row| row.get(x))
    }

    pub fn get_tile_mut(&mut self, x: usize, y: usize) -> Option<&mut Tile> {
        self.tiles.get_mut(y).and_then(|row| row.get_mut(x))
    }

    pub fn entity_at(&self, x: usize, y: usize) -> Option<hecs::Entity> {
        self.get_tile(x, y).and_then(|t| t.entity)
    }

    pub fn resource_at(&self, x: usize, y: usize) -> Option<Resource> {
        self.get_tile(x, y).and_then(|t| t.resource)
    }

    pub fn set_entity(&mut self, x: usize, y: usize, entity: hecs::Entity) {
        if let Some(tile) = self.get_tile_mut(x, y) {
            tile.entity = Some(entity);
        }
    }

    pub fn remove_entity(&mut self, x: usize, y: usize) -> Option<hecs::Entity> {
        self.get_tile_mut(x, y).and_then(|tile| tile.entity.take())
    }

    pub fn set_resource(&mut self, x: usize, y: usize, resource: Resource) {
        if let Some(tile) = self.get_tile_mut(x, y) {
            tile.resource = Some(resource);
        }
    }

    pub fn remove_resource(&mut self, x: usize, y: usize) -> Option<Resource> {
        self.get_tile_mut(x, y).and_then(|tile| tile.resource.take())
    }

    pub fn place_entity_on_map(
        &mut self,
        world: &mut World,
        x: usize,
        y: usize,
        entity_type: EntityType,
        facing: Facing,
        player_placed: bool,
    ) -> Option<hecs::Entity> {
        if !self.in_bounds(x, y) {
            return None;
        }
        if self.tiles[y][x].entity.is_some() {
            return None;
        }
        let entity =
            crate::ecs::archetypes::spawn_entity(world, entity_type, x, y, facing, player_placed);
        self.tiles[y][x].entity = Some(entity);
        Some(entity)
    }

    pub fn remove_entity_from_map(&mut self, world: &mut World, x: usize, y: usize) -> bool {
        if let Some(entity) = self.remove_entity(x, y) {
            let _ = world.despawn(entity);
            true
        } else {
            false
        }
    }

    /// Get entity type at position by looking up in ECS world.
    pub fn entity_type_at(&self, world: &World, x: usize, y: usize) -> Option<EntityType> {
        let entity = self.entity_at(x, y)?;
        world.get::<&EntityKind>(entity).ok().map(|k| k.kind)
    }

    /// Get entity facing at position.
    pub fn entity_facing_at(&self, world: &World, x: usize, y: usize) -> Option<Facing> {
        let entity = self.entity_at(x, y)?;
        world
            .get::<&FacingComponent>(entity)
            .ok()
            .map(|f| f.facing)
    }

    /// Check if a row has any entities.
    pub fn row_has_entities(&self, y: usize) -> bool {
        if y >= self.height {
            return false;
        }
        self.tiles[y].iter().any(|t| t.entity.is_some())
    }

    /// Find first entity in row.
    pub fn first_entity_in_row(&self, y: usize) -> Option<usize> {
        if y >= self.height {
            return None;
        }
        self.tiles[y]
            .iter()
            .position(|t| t.entity.is_some())
    }

    /// Neighbor position in direction from (x, y).
    pub fn neighbor(&self, x: usize, y: usize, facing: Facing) -> Option<(usize, usize)> {
        let (dx, dy) = facing.offset();
        let nx = x as isize + dx;
        let ny = y as isize + dy;
        if self.in_bounds_signed(nx, ny) {
            Some((nx as usize, ny as usize))
        } else {
            None
        }
    }

    /// Clamp coordinates to map bounds.
    pub fn clamp(&self, x: usize, y: usize) -> (usize, usize) {
        (
            x.min(self.width.saturating_sub(1)),
            y.min(self.height.saturating_sub(1)),
        )
    }
}
