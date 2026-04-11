//! Helper functions that apply an operator (delete/yank/change/rotate)
//! to a Range on the map. These are used by the input handler after
//! the parser produces operator commands.

use crate::commands::{Blueprint, BlueprintEntity, Operator, Range};
use crate::ecs::components::{EntityKind, FacingComponent, Processing};
use crate::game::inventory::Inventory;
use crate::map::grid::Map;
use crate::resources::Facing;
use hecs::World;

/// Yank (copy) entities in the given range into a Blueprint.
pub fn yank_range(world: &World, map: &Map, range: &Range) -> Blueprint {
    if range.tiles.is_empty() {
        return Blueprint::empty();
    }

    let mut entities = Vec::new();
    let mut min_x = usize::MAX;
    let mut min_y = usize::MAX;
    let mut max_x = 0usize;
    let mut max_y = 0usize;

    for &(x, y) in &range.tiles {
        if let Some(ecs_entity) = map.entity_at(x, y) {
            if let Ok(kind) = world.get::<&EntityKind>(ecs_entity) {
                // Skip non-player-placeable entities (ore deposits, output bins, etc.)
                if !kind.kind.is_player_placeable() {
                    continue;
                }
                let facing = world
                    .get::<&FacingComponent>(ecs_entity)
                    .ok()
                    .map(|f| f.facing)
                    .unwrap_or(Facing::Right);
                entities.push((x, y, kind.kind, facing));
                min_x = min_x.min(x);
                min_y = min_y.min(y);
                max_x = max_x.max(x);
                max_y = max_y.max(y);
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
pub fn delete_range(world: &mut World, map: &mut Map, inventory: &mut Inventory, range: &Range) -> Blueprint {
    let bp = yank_range(world, map, range);
    for &(x, y) in &range.tiles {
        collect_resources_at(world, map, inventory, x, y);
        map.remove_entity_from_map(world, x, y);
    }
    bp
}

/// Rotate all entities in the range clockwise.
pub fn rotate_cw_range(world: &mut World, map: &Map, range: &Range) {
    for &(x, y) in &range.tiles {
        if let Some(entity) = map.entity_at(x, y) {
            if let Ok(mut facing) = world.get::<&mut FacingComponent>(entity) {
                facing.facing = facing.facing.rotate_cw();
            }
        }
    }
}

/// Rotate all entities in the range counter-clockwise.
pub fn rotate_ccw_range(world: &mut World, map: &Map, range: &Range) {
    for &(x, y) in &range.tiles {
        if let Some(entity) = map.entity_at(x, y) {
            if let Ok(mut facing) = world.get::<&mut FacingComponent>(entity) {
                facing.facing = facing.facing.rotate_ccw();
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
