use std::collections::{HashSet, VecDeque};

use crate::map::grid::Map;
use crate::resources::EntityType;
use hecs::World;

use crate::ecs::components::EntityKind;

impl Map {
    /// Find the next entity position scanning right then wrapping to next rows.
    pub fn find_next_entity(&self, x: usize, y: usize) -> Option<(usize, usize)> {
        // Scan right on current row from x+1
        for col in (x + 1)..self.width {
            if self.tiles[y][col].entity.is_some() {
                return Some((col, y));
            }
        }
        // Scan subsequent rows
        for row in (y + 1)..self.height {
            for col in 0..self.width {
                if self.tiles[row][col].entity.is_some() {
                    return Some((col, row));
                }
            }
        }
        // Wrap from top
        for row in 0..y {
            for col in 0..self.width {
                if self.tiles[row][col].entity.is_some() {
                    return Some((col, row));
                }
            }
        }
        // Same row, before x
        for col in 0..=x {
            if self.tiles[y][col].entity.is_some() && col != x {
                return Some((col, y));
            }
        }
        None
    }

    /// Find the next entity of a DIFFERENT type.
    pub fn find_next_entity_big(
        &self,
        world: &World,
        x: usize,
        y: usize,
    ) -> Option<(usize, usize)> {
        let current_type = self.entity_type_at(world, x, y);

        let scan = |col: usize, row: usize| -> bool {
            if let Some(entity) = self.tiles[row][col].entity {
                if let Ok(kind) = world.get::<&EntityKind>(entity) {
                    return Some(kind.kind) != current_type;
                }
            }
            false
        };

        for col in (x + 1)..self.width {
            if scan(col, y) {
                return Some((col, y));
            }
        }
        for row in (y + 1)..self.height {
            for col in 0..self.width {
                if scan(col, row) {
                    return Some((col, row));
                }
            }
        }
        for row in 0..y {
            for col in 0..self.width {
                if scan(col, row) {
                    return Some((col, row));
                }
            }
        }
        for col in 0..=x {
            if col != x && scan(col, y) {
                return Some((col, y));
            }
        }
        None
    }

    /// Find previous entity scanning left then wrapping.
    pub fn find_prev_entity(&self, x: usize, y: usize) -> Option<(usize, usize)> {
        // Scan left on current row
        if x > 0 {
            for col in (0..x).rev() {
                if self.tiles[y][col].entity.is_some() {
                    return Some((col, y));
                }
            }
        }
        // Scan previous rows from right
        for row in (0..y).rev() {
            for col in (0..self.width).rev() {
                if self.tiles[row][col].entity.is_some() {
                    return Some((col, row));
                }
            }
        }
        // Wrap from bottom
        for row in (y..self.height).rev() {
            for col in (0..self.width).rev() {
                if row == y && col <= x {
                    continue;
                }
                if self.tiles[row][col].entity.is_some() {
                    return Some((col, row));
                }
            }
        }
        None
    }

    /// Find previous entity of different type.
    pub fn find_prev_entity_big(
        &self,
        world: &World,
        x: usize,
        y: usize,
    ) -> Option<(usize, usize)> {
        let current_type = self.entity_type_at(world, x, y);

        let scan = |col: usize, row: usize| -> bool {
            if let Some(entity) = self.tiles[row][col].entity {
                if let Ok(kind) = world.get::<&EntityKind>(entity) {
                    return Some(kind.kind) != current_type;
                }
            }
            false
        };

        if x > 0 {
            for col in (0..x).rev() {
                if scan(col, y) {
                    return Some((col, y));
                }
            }
        }
        for row in (0..y).rev() {
            for col in (0..self.width).rev() {
                if scan(col, row) {
                    return Some((col, row));
                }
            }
        }
        for row in ((y + 1)..self.height).rev() {
            for col in (0..self.width).rev() {
                if scan(col, row) {
                    return Some((col, row));
                }
            }
        }
        for col in ((x + 1)..self.width).rev() {
            if scan(col, y) {
                return Some((col, y));
            }
        }
        None
    }

    /// Find end of entity cluster (contiguous horizontal entities).
    pub fn find_end_of_cluster(&self, x: usize, y: usize) -> (usize, usize) {
        let mut end_x = x;
        while end_x + 1 < self.width && self.tiles[y][end_x + 1].entity.is_some() {
            end_x += 1;
        }
        (end_x, y)
    }

    /// Find next entity of a given type, scanning forward.
    pub fn find_entity_type_forward(
        &self,
        world: &World,
        x: usize,
        y: usize,
        target: EntityType,
    ) -> Option<(usize, usize)> {
        // Scan right on current row
        for col in (x + 1)..self.width {
            if self.entity_type_at(world, col, y) == Some(target) {
                return Some((col, y));
            }
        }
        // Subsequent rows
        for row in (y + 1)..self.height {
            for col in 0..self.width {
                if self.entity_type_at(world, col, row) == Some(target) {
                    return Some((col, row));
                }
            }
        }
        // Wrap
        for row in 0..y {
            for col in 0..self.width {
                if self.entity_type_at(world, col, row) == Some(target) {
                    return Some((col, row));
                }
            }
        }
        for col in 0..=x {
            if self.entity_type_at(world, col, y) == Some(target) && col != x {
                return Some((col, y));
            }
        }
        None
    }

    /// Find next entity of given type, scanning backward.
    pub fn find_entity_type_backward(
        &self,
        world: &World,
        x: usize,
        y: usize,
        target: EntityType,
    ) -> Option<(usize, usize)> {
        if x > 0 {
            for col in (0..x).rev() {
                if self.entity_type_at(world, col, y) == Some(target) {
                    return Some((col, y));
                }
            }
        }
        for row in (0..y).rev() {
            for col in (0..self.width).rev() {
                if self.entity_type_at(world, col, row) == Some(target) {
                    return Some((col, row));
                }
            }
        }
        // Wrap
        for row in (y..self.height).rev() {
            for col in (0..self.width).rev() {
                if row == y && col <= x {
                    continue;
                }
                if self.entity_type_at(world, col, row) == Some(target) {
                    return Some((col, row));
                }
            }
        }
        None
    }

    /// Find next paragraph boundary (first empty row after current paragraph).
    pub fn find_next_paragraph(&self, y: usize) -> usize {
        let mut row = y;
        // Skip current non-empty rows
        while row < self.height && self.row_has_entities(row) {
            row += 1;
        }
        // Skip empty rows
        while row < self.height && !self.row_has_entities(row) {
            row += 1;
        }
        row.min(self.height.saturating_sub(1))
    }

    /// Find previous paragraph boundary (last empty row before current paragraph).
    pub fn find_prev_paragraph(&self, y: usize) -> usize {
        if y == 0 {
            return 0;
        }
        let mut row = y;
        // If on empty row, skip empty rows backward
        if !self.row_has_entities(row) {
            while row > 0 && !self.row_has_entities(row) {
                row -= 1;
            }
        }
        // Skip current non-empty rows backward
        while row > 0 && self.row_has_entities(row) {
            row -= 1;
        }
        row
    }

    /// Flood fill to find inner block (for `i(` / `ib` text object).
    /// Returns the set of positions inside the enclosure, NOT including walls.
    /// Returns None if cursor is not enclosed by walls.
    pub fn find_inner_block(&self, x: usize, y: usize) -> Option<HashSet<(usize, usize)>> {
        let mut visited = HashSet::new();
        let mut queue = VecDeque::new();
        let mut reached_edge = false;

        queue.push_back((x, y));
        visited.insert((x, y));

        while let Some((cx, cy)) = queue.pop_front() {
            // Check if we reached the map edge
            if cx == 0 || cy == 0 || cx == self.width - 1 || cy == self.height - 1 {
                reached_edge = true;
            }

            for &(dx, dy) in &[(0isize, -1isize), (0, 1), (-1, 0), (1, 0)] {
                let nx = cx as isize + dx;
                let ny = cy as isize + dy;
                if !self.in_bounds_signed(nx, ny) {
                    reached_edge = true;
                    continue;
                }
                let (ux, uy) = (nx as usize, ny as usize);
                if visited.contains(&(ux, uy)) {
                    continue;
                }
                // Stop at walls
                if self.entity_type_at_simple(ux, uy) == Some(EntityType::Wall) {
                    continue;
                }
                visited.insert((ux, uy));
                queue.push_back((ux, uy));
            }
        }

        if reached_edge {
            None
        } else {
            Some(visited)
        }
    }

    /// Find wall tiles surrounding an inner block.
    pub fn find_around_block(&self, inner: &HashSet<(usize, usize)>) -> HashSet<(usize, usize)> {
        let mut walls = HashSet::new();
        for &(x, y) in inner {
            for &(dx, dy) in &[(0isize, -1isize), (0, 1), (-1, 0), (1, 0)] {
                let nx = x as isize + dx;
                let ny = y as isize + dy;
                if self.in_bounds_signed(nx, ny) {
                    let (ux, uy) = (nx as usize, ny as usize);
                    if !inner.contains(&(ux, uy)) {
                        if self.entity_type_at_simple(ux, uy) == Some(EntityType::Wall) {
                            walls.insert((ux, uy));
                        }
                    }
                }
            }
        }
        walls
    }

    /// Simple entity type lookup without World (uses a cache or checks entity type from tile).
    /// This is used for flood fill where we don't want to pass World everywhere.
    /// We store entity type info redundantly for this purpose.
    fn entity_type_at_simple(&self, _x: usize, _y: usize) -> Option<EntityType> {
        // This needs an entity type cache on the map.
        // For now, return None — the full version will use World.
        None
    }

    /// Find inner block using World for entity type lookups.
    pub fn find_inner_block_with_world(
        &self,
        world: &World,
        x: usize,
        y: usize,
    ) -> Option<HashSet<(usize, usize)>> {
        let mut visited = HashSet::new();
        let mut queue = VecDeque::new();
        let mut reached_edge = false;

        queue.push_back((x, y));
        visited.insert((x, y));

        while let Some((cx, cy)) = queue.pop_front() {
            if cx == 0 || cy == 0 || cx == self.width - 1 || cy == self.height - 1 {
                reached_edge = true;
            }

            for &(dx, dy) in &[(0isize, -1isize), (0, 1), (-1, 0), (1, 0)] {
                let nx = cx as isize + dx;
                let ny = cy as isize + dy;
                if !self.in_bounds_signed(nx, ny) {
                    reached_edge = true;
                    continue;
                }
                let (ux, uy) = (nx as usize, ny as usize);
                if visited.contains(&(ux, uy)) {
                    continue;
                }
                if self.entity_type_at(world, ux, uy) == Some(EntityType::Wall) {
                    continue;
                }
                visited.insert((ux, uy));
                queue.push_back((ux, uy));
            }
        }

        if reached_edge {
            None
        } else {
            Some(visited)
        }
    }
}
