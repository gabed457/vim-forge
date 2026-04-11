use std::collections::{BinaryHeap, HashMap};
use std::cmp::Ordering;

// ---------------------------------------------------------------------------
// A* pathfinding on a 2D grid
// ---------------------------------------------------------------------------

/// A node in the A* open set.
#[derive(Clone, Debug, Eq, PartialEq)]
struct AStarNode {
    pos: (usize, usize),
    g_cost: u32,
    f_cost: u32,
}

impl Ord for AStarNode {
    fn cmp(&self, other: &Self) -> Ordering {
        // Reverse ordering so BinaryHeap acts as a min-heap.
        other.f_cost.cmp(&self.f_cost)
            .then_with(|| other.g_cost.cmp(&self.g_cost))
    }
}

impl PartialOrd for AStarNode {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

/// Result of a pathfinding query.
#[derive(Clone, Debug)]
pub struct PathResult {
    /// The path from start to end (inclusive), empty if no path found.
    pub path: Vec<(usize, usize)>,
    /// Total cost of the path.
    pub cost: u32,
}

/// Find the shortest path on a grid from `start` to `end`.
///
/// `width`, `height` — grid dimensions.
/// `passable` — returns true if the tile at (x, y) can be traversed.
///
/// Returns `None` if no path exists.
pub fn find_path(
    width: usize,
    height: usize,
    start: (usize, usize),
    end: (usize, usize),
    passable: impl Fn(usize, usize) -> bool,
) -> Option<PathResult> {
    if !passable(start.0, start.1) || !passable(end.0, end.1) {
        return None;
    }
    if start == end {
        return Some(PathResult {
            path: vec![start],
            cost: 0,
        });
    }

    let mut open = BinaryHeap::new();
    let mut came_from: HashMap<(usize, usize), (usize, usize)> = HashMap::new();
    let mut g_scores: HashMap<(usize, usize), u32> = HashMap::new();

    g_scores.insert(start, 0);
    open.push(AStarNode {
        pos: start,
        g_cost: 0,
        f_cost: heuristic(start, end),
    });

    let directions: [(isize, isize); 4] = [(0, -1), (1, 0), (0, 1), (-1, 0)];

    while let Some(current) = open.pop() {
        if current.pos == end {
            // Reconstruct path.
            let mut path = Vec::new();
            let mut cur = end;
            path.push(cur);
            while let Some(&prev) = came_from.get(&cur) {
                path.push(prev);
                cur = prev;
            }
            path.reverse();
            return Some(PathResult {
                path,
                cost: current.g_cost,
            });
        }

        let current_g = match g_scores.get(&current.pos) {
            Some(&g) => g,
            None => continue,
        };

        // Skip if we've already found a better path to this node.
        if current.g_cost > current_g {
            continue;
        }

        for &(dx, dy) in &directions {
            let nx = current.pos.0 as isize + dx;
            let ny = current.pos.1 as isize + dy;
            if nx < 0 || ny < 0 || nx as usize >= width || ny as usize >= height {
                continue;
            }
            let next = (nx as usize, ny as usize);
            if !passable(next.0, next.1) {
                continue;
            }

            let tentative_g = current_g + 1;
            let existing_g = g_scores.get(&next).copied().unwrap_or(u32::MAX);
            if tentative_g < existing_g {
                came_from.insert(next, current.pos);
                g_scores.insert(next, tentative_g);
                open.push(AStarNode {
                    pos: next,
                    g_cost: tentative_g,
                    f_cost: tentative_g + heuristic(next, end),
                });
            }
        }
    }

    None
}

/// Manhattan distance heuristic.
fn heuristic(a: (usize, usize), b: (usize, usize)) -> u32 {
    let dx = (a.0 as isize - b.0 as isize).unsigned_abs() as u32;
    let dy = (a.1 as isize - b.1 as isize).unsigned_abs() as u32;
    dx + dy
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_straight_path() {
        let result = find_path(10, 10, (0, 0), (4, 0), |_, _| true);
        assert!(result.is_some());
        let r = result.unwrap();
        assert_eq!(r.cost, 4);
        assert_eq!(r.path.first(), Some(&(0, 0)));
        assert_eq!(r.path.last(), Some(&(4, 0)));
    }

    #[test]
    fn test_no_path() {
        // Wall across middle.
        let result = find_path(5, 5, (0, 0), (4, 4), |x, _y| x != 2);
        assert!(result.is_none());
    }

    #[test]
    fn test_same_start_end() {
        let result = find_path(5, 5, (2, 2), (2, 2), |_, _| true);
        assert!(result.is_some());
        assert_eq!(result.unwrap().cost, 0);
    }
}
