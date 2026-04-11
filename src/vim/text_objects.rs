//! Functions to compute Range for text objects.
//! In the VimForge factory context:
//!   - "word" (iw/aw) = contiguous entity cluster on a row
//!   - "paragraph" (ip/ap) = contiguous group of rows that have entities
//!   - "block" (ib/ab, also i(/a() = enclosed area bounded by walls

use std::collections::HashSet;

use crate::commands::Range;
use crate::map::grid::Map;
use hecs::World;

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

/// Around block: enclosed area including the surrounding walls.
pub fn around_block(map: &Map, world: &World, x: usize, y: usize) -> Range {
    match map.find_inner_block_with_world(world, x, y) {
        Some(inner_positions) => {
            let walls = map.find_around_block(&inner_positions);
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
