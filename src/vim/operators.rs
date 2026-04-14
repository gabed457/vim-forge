//! Helper functions that apply an operator (delete/yank/change/rotate)
//! to a Range on the map. These are used by the input handler after
//! the parser produces operator commands.

use std::collections::HashSet;

use crate::commands::{Blueprint, BlueprintEntity, Operator, Range};
use crate::ecs::components::{EntityKind, FacingComponent, MultiTile, PartOfBuilding, Position, Processing};
use crate::game::inventory::Inventory;
use crate::map::grid::Map;
use crate::map::multitile::building_footprint;
use crate::resources::Facing;
use hecs::World;

/// Yank (copy) entities in the given range into a Blueprint.
/// Resolves PartOfBuilding tiles to their anchor and deduplicates so each
/// multi-tile building is captured once at its anchor position.
pub fn yank_range(world: &World, map: &Map, range: &Range) -> Blueprint {
    if range.tiles.is_empty() {
        return Blueprint::empty();
    }

    let mut entities = Vec::new();
    let mut seen_anchors: HashSet<hecs::Entity> = HashSet::new();
    let mut min_x = usize::MAX;
    let mut min_y = usize::MAX;
    let mut max_x = 0usize;
    let mut max_y = 0usize;

    for &(x, y) in &range.tiles {
        if let Some(ecs_entity) = map.entity_at(x, y) {
            // Resolve PartOfBuilding to anchor
            let anchor = if let Ok(pob) = world.get::<&PartOfBuilding>(ecs_entity) {
                pob.anchor
            } else {
                ecs_entity
            };

            // Deduplicate by anchor
            if !seen_anchors.insert(anchor) {
                continue;
            }

            if let Ok(kind) = world.get::<&EntityKind>(anchor) {
                // Skip non-player-placeable entities (ore deposits, output bins, etc.)
                if !kind.kind.is_player_placeable() {
                    continue;
                }
                // Use anchor's position for blueprint offsets
                let (ax, ay) = world
                    .get::<&Position>(anchor)
                    .map(|p| (p.x, p.y))
                    .unwrap_or((x, y));
                let facing = world
                    .get::<&FacingComponent>(anchor)
                    .ok()
                    .map(|f| f.facing)
                    .unwrap_or(Facing::Right);
                entities.push((ax, ay, kind.kind, facing));
                min_x = min_x.min(ax);
                min_y = min_y.min(ay);
                max_x = max_x.max(ax);
                max_y = max_y.max(ay);
            }
        }
    }

    if entities.is_empty() {
        return Blueprint::empty();
    }

    let bp_entities: Vec<BlueprintEntity> = entities
        .iter()
        .map(|&(x, y, entity_type, facing)| BlueprintEntity {
            offset_x: x - min_x,
            offset_y: y - min_y,
            entity_type,
            facing,
        })
        .collect();

    Blueprint {
        entities: bp_entities,
        width: max_x - min_x + 1,
        height: max_y - min_y + 1,
        linewise: range.linewise,
    }
}

/// Collect resources at a tile into the player inventory.
/// Picks up the tile resource and any resources inside a Processing component.
/// Must be called **before** despawning the entity.
pub fn collect_resources_at(world: &World, map: &mut Map, inventory: &mut Inventory, x: usize, y: usize) {
    // Collect tile resource (carried on conveyors)
    if let Some(resource) = map.remove_resource(x, y) {
        inventory.add(resource);
    }
    // Collect resources inside Processing component (smelters/assemblers mid-process)
    if let Some(entity) = map.entity_at(x, y) {
        if let Ok(proc) = world.get::<&Processing>(entity) {
            if let Some(r) = proc.input_a {
                inventory.add(r);
            }
            if let Some(r) = proc.input_b {
                inventory.add(r);
            }
            if let Some(r) = proc.output {
                inventory.add(r);
            }
        }
    }
}

/// Delete (demolish) entities in the given range from the map.
/// Returns a Blueprint of what was deleted (for pasting back).
/// Collects resources into the player inventory before removing entities.
/// Resolves PartOfBuilding to anchors and deduplicates removals.
pub fn delete_range(world: &mut World, map: &mut Map, inventory: &mut Inventory, range: &Range) -> Blueprint {
    let bp = yank_range(world, map, range);

    // Collect resources from all tiles in the range
    for &(x, y) in &range.tiles {
        collect_resources_at(world, map, inventory, x, y);
    }

    // Remove entities, deduplicating multi-tile buildings
    let mut removed_anchors: HashSet<hecs::Entity> = HashSet::new();
    for &(x, y) in &range.tiles {
        if let Some(ent) = map.entity_at(x, y) {
            // Resolve to anchor
            let anchor = if let Ok(pob) = world.get::<&PartOfBuilding>(ent) {
                pob.anchor
            } else {
                ent
            };
            if removed_anchors.insert(anchor) {
                map.remove_multitile_entity(world, x, y);
            }
        }
    }
    bp
}

/// Rotate all entities in the range clockwise.
/// Resolves PartOfBuilding to anchors and deduplicates.
/// For non-square buildings, removes and re-places with rotated footprint.
pub fn rotate_cw_range(world: &mut World, map: &mut Map, range: &Range) {
    let mut seen: HashSet<hecs::Entity> = HashSet::new();
    for &(x, y) in &range.tiles {
        if let Some(entity) = map.entity_at(x, y) {
            let anchor = if let Ok(pob) = world.get::<&PartOfBuilding>(entity) {
                pob.anchor
            } else {
                entity
            };
            if seen.insert(anchor) {
                let old_facing = world
                    .get::<&FacingComponent>(anchor)
                    .ok()
                    .map(|f| f.facing)
                    .unwrap_or(Facing::Right);
                let new_facing = old_facing.rotate_cw();
                rotate_building(world, map, anchor, new_facing);
            }
        }
    }
}

/// Rotate all entities in the range counter-clockwise.
/// Resolves PartOfBuilding to anchors and deduplicates.
/// For non-square buildings, removes and re-places with rotated footprint.
pub fn rotate_ccw_range(world: &mut World, map: &mut Map, range: &Range) {
    let mut seen: HashSet<hecs::Entity> = HashSet::new();
    for &(x, y) in &range.tiles {
        if let Some(entity) = map.entity_at(x, y) {
            let anchor = if let Ok(pob) = world.get::<&PartOfBuilding>(entity) {
                pob.anchor
            } else {
                entity
            };
            if seen.insert(anchor) {
                let old_facing = world
                    .get::<&FacingComponent>(anchor)
                    .ok()
                    .map(|f| f.facing)
                    .unwrap_or(Facing::Right);
                let new_facing = old_facing.rotate_ccw();
                rotate_building(world, map, anchor, new_facing);
            }
        }
    }
}

/// Rotate a single building to a new facing.
/// For 1×1 and square buildings, just updates FacingComponent.
/// For non-square buildings, removes tiles from the map, checks if the
/// rotated footprint fits, and re-places with new secondary tiles.
fn rotate_building(world: &mut World, map: &mut Map, anchor: hecs::Entity, new_facing: Facing) {
    let (ax, ay) = match world.get::<&Position>(anchor).ok().map(|p| (p.x, p.y)) {
        Some(p) => p,
        None => return,
    };
    let kind = match world.get::<&EntityKind>(anchor).ok().map(|k| k.kind) {
        Some(k) => k,
        None => return,
    };
    let old_facing = world
        .get::<&FacingComponent>(anchor)
        .ok()
        .map(|f| f.facing)
        .unwrap_or(Facing::Right);

    let old_fp = building_footprint(kind).rotate_to(old_facing);
    let new_fp = building_footprint(kind).rotate_to(new_facing);

    // Square or 1×1: just update facing — footprint shape is unchanged
    if old_fp.width == old_fp.height {
        if let Ok(mut f) = world.get::<&mut FacingComponent>(anchor) {
            f.facing = new_facing;
        }
        return;
    }

    // Non-square: need to reposition tiles on the map
    // Step 1: Collect and clear all old tile entities from the map
    let mut old_secondary_ents: Vec<hecs::Entity> = Vec::new();
    for fy in 0..old_fp.height {
        for fx in 0..old_fp.width {
            let tx = ax + fx;
            let ty = ay + fy;
            if let Some(ent) = map.remove_entity(tx, ty) {
                if ent != anchor {
                    old_secondary_ents.push(ent);
                }
            }
        }
    }

    // Step 2: Check if new footprint fits at anchor position
    let fits = (0..new_fp.height).all(|fy| {
        (0..new_fp.width).all(|fx| {
            let tx = ax + fx;
            let ty = ay + fy;
            map.in_bounds(tx, ty) && map.entity_at(tx, ty).is_none()
        })
    });

    if fits {
        // Update facing and MultiTile dimensions
        if let Ok(mut f) = world.get::<&mut FacingComponent>(anchor) {
            f.facing = new_facing;
        }
        if let Ok(mut mt) = world.get::<&mut MultiTile>(anchor) {
            mt.width = new_fp.width;
            mt.height = new_fp.height;
        }

        // Despawn old secondary entities
        for ent in old_secondary_ents {
            let _ = world.despawn(ent);
        }

        // Place anchor back on map
        map.set_entity(ax, ay, anchor);

        // Spawn new secondary tiles
        for fy in 0..new_fp.height {
            for fx in 0..new_fp.width {
                if fx == 0 && fy == 0 {
                    continue;
                }
                let tx = ax + fx;
                let ty = ay + fy;
                let secondary = world.spawn((
                    Position { x: tx, y: ty },
                    PartOfBuilding { anchor },
                ));
                map.set_entity(tx, ty, secondary);
            }
        }
    } else {
        // Doesn't fit: restore old tiles at original positions
        map.set_entity(ax, ay, anchor);
        let mut sec_idx = 0;
        for fy in 0..old_fp.height {
            for fx in 0..old_fp.width {
                if fx == 0 && fy == 0 {
                    continue;
                }
                let tx = ax + fx;
                let ty = ay + fy;
                if sec_idx < old_secondary_ents.len() {
                    map.set_entity(tx, ty, old_secondary_ents[sec_idx]);
                    sec_idx += 1;
                }
            }
        }
    }
}

/// Apply an operator to a range. Returns a Blueprint if the operator
/// produces one (delete and yank do; rotate does not).
pub fn apply_operator(
    op: &Operator,
    world: &mut World,
    map: &mut Map,
    inventory: &mut Inventory,
    range: &Range,
) -> Option<Blueprint> {
    match op {
        Operator::Delete => Some(delete_range(world, map, inventory, range)),
        Operator::Yank => Some(yank_range(world, map, range)),
        Operator::Change => {
            let bp = delete_range(world, map, inventory, range);
            // Change also enters insert mode, but that is handled by the caller
            Some(bp)
        }
        Operator::RotateCW => {
            rotate_cw_range(world, map, range);
            None
        }
        Operator::RotateCCW => {
            rotate_ccw_range(world, map, range);
            None
        }
    }
}
