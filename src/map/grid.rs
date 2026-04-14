use hecs::World;

use crate::ecs::components::*;
use crate::map::multitile::building_footprint;
use crate::map::terrain::Terrain;
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
    pub terrain: Vec<Vec<Terrain>>,
}

impl Map {
    pub fn new(width: usize, height: usize) -> Self {
        let tiles = (0..height)
            .map(|_| (0..width).map(|_| Tile::empty()).collect())
            .collect();
        let terrain = (0..height)
            .map(|_| (0..width).map(|_| Terrain::default()).collect())
            .collect();
        Map {
            width,
            height,
            tiles,
            terrain,
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

    pub fn terrain_at(&self, x: usize, y: usize) -> Terrain {
        self.terrain
            .get(y)
            .and_then(|row| row.get(x))
            .copied()
            .unwrap_or_default()
    }

    pub fn set_terrain(&mut self, x: usize, y: usize, t: Terrain) {
        if let Some(row) = self.terrain.get_mut(y) {
            if let Some(cell) = row.get_mut(x) {
                *cell = t;
            }
        }
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
        self.place_multitile_entity(world, x, y, entity_type, facing, player_placed)
    }

    /// Place a (possibly multi-tile) entity on the map.
    /// For 1x1 buildings, this behaves like the old single-tile placement.
    /// For multi-tile buildings, spawns the anchor at (x,y) and PartOfBuilding
    /// entities on all secondary tiles.
    /// Returns the anchor entity, or None if placement is blocked.
    pub fn place_multitile_entity(
        &mut self,
        world: &mut World,
        x: usize,
        y: usize,
        entity_type: EntityType,
        facing: Facing,
        player_placed: bool,
    ) -> Option<hecs::Entity> {
        let fp = building_footprint(entity_type).rotate_to(facing);

        // Check all footprint tiles are in-bounds and empty
        for fy in 0..fp.height {
            for fx in 0..fp.width {
                let tx = x + fx;
                let ty = y + fy;
                if !self.in_bounds(tx, ty) {
                    return None;
                }
                if self.tiles[ty][tx].entity.is_some() {
                    return None;
                }
            }
        }

        // Spawn anchor entity at (x, y)
        let anchor =
            crate::ecs::archetypes::spawn_entity(world, entity_type, x, y, facing, player_placed);
        self.tiles[y][x].entity = Some(anchor);

        // If multi-tile, add MultiTile component and spawn secondary tiles
        if fp.width > 1 || fp.height > 1 {
            let _ = world.insert_one(anchor, MultiTile {
                width: fp.width,
                height: fp.height,
            });

            for fy in 0..fp.height {
                for fx in 0..fp.width {
                    if fx == 0 && fy == 0 {
                        continue; // skip anchor tile
                    }
                    let tx = x + fx;
                    let ty = y + fy;
                    let secondary = world.spawn((
                        Position { x: tx, y: ty },
                        PartOfBuilding { anchor },
                    ));
                    self.tiles[ty][tx].entity = Some(secondary);
                }
            }
        }

        Some(anchor)
    }

    /// Remove a (possibly multi-tile) entity from the map.
    /// If the tile at (x, y) is a PartOfBuilding, resolves to its anchor first.
    /// Removes and despawns all tiles belonging to the building.
    /// Returns true if an entity was removed.
    pub fn remove_multitile_entity(&mut self, world: &mut World, x: usize, y: usize) -> bool {
        let ent = match self.entity_at(x, y) {
            Some(e) => e,
            None => return false,
        };

        // Resolve to anchor
        let anchor = if let Ok(pob) = world.get::<&PartOfBuilding>(ent) {
            pob.anchor
        } else {
            ent
        };

        // Get anchor position and footprint size, copying values to avoid borrow conflicts
        let anchor_pos = world.get::<&Position>(anchor).ok().map(|p| (p.x, p.y));
        let (ax, ay) = match anchor_pos {
            Some(pos) => pos,
            None => {
                // Fallback: just remove the single tile
                return self.remove_entity_from_map_single(world, x, y);
            }
        };

        // Get footprint size
        let (fw, fh) = world.get::<&MultiTile>(anchor)
            .ok()
            .map(|mt| (mt.width, mt.height))
            .unwrap_or((1, 1));

        // Remove all tiles from map and despawn secondary entities
        for fy in 0..fh {
            for fx in 0..fw {
                let tx = ax + fx;
                let ty = ay + fy;
                if let Some(tile_ent) = self.remove_entity(tx, ty) {
                    if tile_ent != anchor {
                        let _ = world.despawn(tile_ent);
                    }
                }
            }
        }

        // Despawn anchor last
        let _ = world.despawn(anchor);
        true
    }

    /// Remove and despawn a single tile entity (no multi-tile resolution).
    fn remove_entity_from_map_single(&mut self, world: &mut World, x: usize, y: usize) -> bool {
        if let Some(entity) = self.remove_entity(x, y) {
            let _ = world.despawn(entity);
            true
        } else {
            false
        }
    }

    pub fn remove_entity_from_map(&mut self, world: &mut World, x: usize, y: usize) -> bool {
        self.remove_multitile_entity(world, x, y)
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
