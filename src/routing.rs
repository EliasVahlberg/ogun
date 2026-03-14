//! Lee algorithm: BFS maze routing on a grid.
//!
//! 1. Wave expansion from source, marking distances
//! 2. Backtrace from destination following decreasing distances
//!
//! This is the simplest correct routing algorithm. It guarantees the
//! shortest path if one exists.

use std::collections::VecDeque;

use crate::grid::Grid;
use crate::types::Pos;

/// Find the shortest path from `src` to `dst` on the grid.
///
/// A cell is passable if `passable(pos)` returns true.
/// Both `src` and `dst` are always treated as passable regardless of the predicate.
///
/// Returns `None` if no path exists.
pub fn lee_route(
    grid_w: u32,
    grid_h: u32,
    src: Pos,
    dst: Pos,
    passable: impl Fn(Pos) -> bool,
) -> Option<Vec<Pos>> {
    if src == dst {
        return Some(vec![src]);
    }

    // Distance grid: None = unvisited.
    let mut dist: Grid<Option<u32>> = Grid::new(grid_w, grid_h, None);
    let mut queue = VecDeque::new();

    dist.set(src.x, src.y, Some(0));
    queue.push_back(src);

    // Wave expansion.
    let mut found = false;
    while let Some(cur) = queue.pop_front() {
        if cur == dst {
            found = true;
            break;
        }
        let d = dist.get(cur.x, cur.y).unwrap().unwrap();
        for nb in dist.neighbors4(cur.x, cur.y) {
            if dist.get(nb.x, nb.y).unwrap().is_some() {
                continue; // already visited
            }
            // src/dst are always passable; other cells check the predicate
            if nb != dst && !passable(nb) {
                continue;
            }
            dist.set(nb.x, nb.y, Some(d + 1));
            queue.push_back(nb);
        }
    }

    if !found {
        return None;
    }

    // Backtrace from dst to src.
    let end_dist = dist.get(dst.x, dst.y).unwrap().unwrap();
    let mut path = vec![dst];
    let mut cur = dst;

    for expected in (0..end_dist).rev() {
        let next = dist
            .neighbors4(cur.x, cur.y)
            .into_iter()
            .find(|nb| dist.get(nb.x, nb.y).and_then(|d| *d) == Some(expected))
            .expect("backtrace should always find a predecessor");
        path.push(next);
        cur = next;
    }

    path.reverse();
    Some(path)
}
