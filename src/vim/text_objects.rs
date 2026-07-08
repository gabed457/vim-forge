//! Functions to compute Range for text objects.
//! In the VimForge factory context:
//!   - "word" (iw/aw) = contiguous entity cluster on a row
//!   - "paragraph" (ip/ap) = contiguous group of rows that have entities
//!   - "block" (ib/ab, also all bracket flavors i(/a(, i[/a[, i{/a{,
//!     i</a<, iB/aB, and the "sentence" is/as) = enclosed area bounded
//!     by walls
//!   - "quote" (i"/a", i'/a') = belt run: the maximal contiguous straight
//!     line of same-direction belts through the cursor; a" also includes
//!     the machines at both ends when adjacent
//!   - "tag" (it/at) = the machine (multi-tile building) under the cursor:
//!     it = its footprint tiles, at = footprint plus its port tiles

use std::collections::HashSet;

use crate::commands::Range;
use crate::ecs::components::{EntityKind, FacingComponent, PartOfBuilding, Position};
use crate::map::grid::Map;
use crate::map::multitile::building_footprint;
use crate::resources::{EntityType, Facing};
use hecs::World;

/// Whether an entity type is a belt (used by the belt-run object and the
/// machine "sentence" motions).
pub fn is_belt(entity_type: EntityType) -> bool {
    matches!(
        entity_type,
        EntityType::BasicBelt
            | EntityType::FastBelt
            | EntityType::ExpressBelt
            | EntityType::UndergroundEntrance
            | EntityType::UndergroundExit
    )
}

/// Inner word: the contiguous horizontal cluster of entities around cursor.
pub fn inner_word(map: &Map, x: usize, y: usize) -> Range {
    if !map.in_bounds(x, y) {
        return Range::empty();
    }

    // If cursor is not on an entity, return just the cursor tile
    if map.entity_at(x, y).is_none() {
        return Range::single(x, y);
    }

    // Scan left to find start of cluster
    let mut start = x;
    while start > 0 && map.entity_at(start - 1, y).is_some() {
        start -= 1;
    }

    // Scan right to find end of cluster
    let mut end = x;
    while end + 1 < map.width && map.entity_at(end + 1, y).is_some() {
        end += 1;
    }

    Range::horizontal(y, start, end)
}

/// Around word: the cluster plus one tile of surrounding space.
pub fn around_word(map: &Map, x: usize, y: usize) -> Range {
    if !map.in_bounds(x, y) {
        return Range::empty();
    }

    let inner = inner_word(map, x, y);
    if inner.tiles.is_empty() {
        return inner;
    }

    // Find the bounds of the inner word
    let min_x = inner.tiles.iter().map(|&(tx, _)| tx).min().unwrap_or(x);
    let max_x = inner.tiles.iter().map(|&(tx, _)| tx).max().unwrap_or(x);

    // Extend right if there's a space, else extend left
    let start = if max_x + 1 < map.width && map.entity_at(max_x + 1, y).is_none() {
        min_x
    } else if min_x > 0 {
        min_x - 1
    } else {
        min_x
    };

    let end = if max_x + 1 < map.width && map.entity_at(max_x + 1, y).is_none() {
        max_x + 1
    } else {
        max_x
    };

    Range::horizontal(y, start, end)
}

/// Inner paragraph: all contiguous rows that have entities, around cursor row.
pub fn inner_paragraph(map: &Map, _x: usize, y: usize) -> Range {
    if y >= map.height {
        return Range::empty();
    }

    // Find the contiguous block of rows with entities around y
    let mut top = y;
    while top > 0 && map.row_has_entities(top - 1) {
        top -= 1;
    }

    let mut bottom = y;
    while bottom + 1 < map.height && map.row_has_entities(bottom + 1) {
        bottom += 1;
    }

    Range::linewise_rows(top, bottom, map.width)
}

/// Around paragraph: the paragraph plus one adjacent empty row.
pub fn around_paragraph(map: &Map, x: usize, y: usize) -> Range {
    let inner = inner_paragraph(map, x, y);
    if inner.tiles.is_empty() {
        return inner;
    }

    // Get actual row numbers from the tile coordinates
    let actual_top = inner.tiles.first().map(|&(_, ty)| ty).unwrap_or(y);
    let actual_bottom = inner.tiles.last().map(|&(_, ty)| ty).unwrap_or(y);

    // Extend down if possible, else extend up
    let ext_bottom = if actual_bottom + 1 < map.height && !map.row_has_entities(actual_bottom + 1)
    {
        actual_bottom + 1
    } else {
        actual_bottom
    };

    let ext_top = if ext_bottom == actual_bottom && actual_top > 0 {
        actual_top - 1
    } else {
        actual_top
    };

    Range::linewise_rows(ext_top, ext_bottom, map.width)
}

/// Inner block: enclosed area bounded by walls (flood fill), NOT including walls.
pub fn inner_block(map: &Map, world: &World, x: usize, y: usize) -> Range {
    match map.find_inner_block_with_world(world, x, y) {
        Some(positions) => {
            let mut tiles: Vec<(usize, usize)> = positions.into_iter().collect();
            tiles.sort_by(|a, b| a.1.cmp(&b.1).then(a.0.cmp(&b.0)));
            Range {
                tiles,
                linewise: false,
            }
        }
        None => Range::empty(),
    }
}

/// Belt run (i"/a"): the maximal contiguous straight line of belts sharing
/// the cursor belt's facing, along that facing's axis. With `around`, the
/// machines (non-belt buildings) adjacent at both ends are included too.
pub fn belt_run(map: &Map, world: &World, x: usize, y: usize, around: bool) -> Range {
    if !map.in_bounds(x, y) {
        return Range::empty();
    }
    let kind = match map.entity_type_at(world, x, y) {
        Some(k) if is_belt(k) => k,
        _ => return Range::empty(),
    };
    let _ = kind;
    let facing = map
        .entity_facing_at(world, x, y)
        .unwrap_or(Facing::Right);
    let horizontal = matches!(facing, Facing::Left | Facing::Right);
    let (dx, dy): (isize, isize) = if horizontal { (1, 0) } else { (0, 1) };

    let same_run_belt = |tx: isize, ty: isize| -> bool {
        if !map.in_bounds_signed(tx, ty) {
            return false;
        }
        let (ux, uy) = (tx as usize, ty as usize);
        match map.entity_type_at(world, ux, uy) {
            Some(k) if is_belt(k) => map.entity_facing_at(world, ux, uy) == Some(facing),
            _ => false,
        }
    };

    // Walk backwards to the start of the run
    let (mut sx, mut sy) = (x as isize, y as isize);
    while same_run_belt(sx - dx, sy - dy) {
        sx -= dx;
        sy -= dy;
    }
    // Walk forwards to the end of the run
    let (mut ex, mut ey) = (x as isize, y as isize);
    while same_run_belt(ex + dx, ey + dy) {
        ex += dx;
        ey += dy;
    }

    let mut tiles: Vec<(usize, usize)> = Vec::new();

    // a": include a machine adjacent at either end
    if around {
        let (mx, my) = (sx - dx, sy - dy);
        if map.in_bounds_signed(mx, my) {
            if let Some(k) = map.entity_type_at(world, mx as usize, my as usize) {
                if !is_belt(k) {
                    tiles.push((mx as usize, my as usize));
                }
            }
        }
    }
    let (mut cx, mut cy) = (sx, sy);
    loop {
        tiles.push((cx as usize, cy as usize));
        if (cx, cy) == (ex, ey) {
            break;
        }
        cx += dx;
        cy += dy;
    }
    if around {
        let (mx, my) = (ex + dx, ey + dy);
        if map.in_bounds_signed(mx, my) {
            if let Some(k) = map.entity_type_at(world, mx as usize, my as usize) {
                if !is_belt(k) {
                    tiles.push((mx as usize, my as usize));
                }
            }
        }
    }

    tiles.sort_by(|a, b| a.1.cmp(&b.1).then(a.0.cmp(&b.0)));
    Range {
        tiles,
        linewise: false,
    }
}

/// Machine object (it/at): the footprint of the multi-tile building under
/// the cursor. `with_ports` additionally includes the tiles just outside
/// the footprint that its ports connect to (at).
pub fn machine_object(map: &Map, world: &World, x: usize, y: usize, with_ports: bool) -> Range {
    let entity = match map.entity_at(x, y) {
        Some(e) => e,
        None => return Range::empty(),
    };
    let anchor = if let Ok(pob) = world.get::<&PartOfBuilding>(entity) {
        pob.anchor
    } else {
        entity
    };
    let (ax, ay) = match world.get::<&Position>(anchor).ok().map(|p| (p.x, p.y)) {
        Some(p) => p,
        None => return Range::empty(),
    };
    let kind = match world.get::<&EntityKind>(anchor).ok().map(|k| k.kind) {
        Some(k) => k,
        None => return Range::empty(),
    };
    let facing = world
        .get::<&FacingComponent>(anchor)
        .ok()
        .map(|f| f.facing)
        .unwrap_or(Facing::Right);
    let fp = building_footprint(kind).rotate_to(facing);

    let mut tiles: HashSet<(usize, usize)> = HashSet::new();
    for fy in 0..fp.height {
        for fx in 0..fp.width {
            let tx = ax + fx;
            let ty = ay + fy;
            if map.in_bounds(tx, ty) {
                tiles.insert((tx, ty));
            }
        }
    }

    if with_ports {
        for port in &fp.ports {
            let (dx, dy) = port.direction.offset();
            let px = ax as isize + port.offset_x as isize + dx;
            let py = ay as isize + port.offset_y as isize + dy;
            if map.in_bounds_signed(px, py) {
                tiles.insert((px as usize, py as usize));
            }
        }
    }

    let mut tiles: Vec<(usize, usize)> = tiles.into_iter().collect();
    tiles.sort_by(|a, b| a.1.cmp(&b.1).then(a.0.cmp(&b.0)));
    Range {
        tiles,
        linewise: false,
    }
}

/// Around block: enclosed area including the surrounding walls.
pub fn around_block(map: &Map, world: &World, x: usize, y: usize) -> Range {
    match map.find_inner_block_with_world(world, x, y) {
        Some(inner_positions) => {
            // Collect the wall tiles bordering the enclosure. (The map's
            // find_around_block can't see entity types without the World,
            // so the lookup happens here.)
            // 8-neighborhood so the ring's corner walls are included too.
            let mut walls: HashSet<(usize, usize)> = HashSet::new();
            for &(ix, iy) in &inner_positions {
                for &(dx, dy) in &[
                    (0isize, -1isize),
                    (0, 1),
                    (-1, 0),
                    (1, 0),
                    (-1, -1),
                    (-1, 1),
                    (1, -1),
                    (1, 1),
                ] {
                    let nx = ix as isize + dx;
                    let ny = iy as isize + dy;
                    if map.in_bounds_signed(nx, ny) {
                        let (ux, uy) = (nx as usize, ny as usize);
                        if !inner_positions.contains(&(ux, uy))
                            && map.entity_type_at(world, ux, uy) == Some(EntityType::Wall)
                        {
                            walls.insert((ux, uy));
                        }
                    }
                }
            }
            let mut all: HashSet<(usize, usize)> = inner_positions;
            all.extend(walls);
            let mut tiles: Vec<(usize, usize)> = all.into_iter().collect();
            tiles.sort_by(|a, b| a.1.cmp(&b.1).then(a.0.cmp(&b.0)));
            Range {
                tiles,
                linewise: false,
            }
        }
        None => Range::empty(),
    }
}
