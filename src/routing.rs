//! Lee algorithm: BFS maze routing on a grid.
//!
//! 1. Wave expansion from source, marking distances
//! 2. Backtrace from destination following decreasing distances

use std::collections::VecDeque;

use crate::types::Pos;

const DIRS: [(i32, i32); 4] = [(0, -1), (1, 0), (0, 1), (-1, 0)];

/// Reusable buffers for Lee routing to avoid per-call allocation.
pub struct RouteBuf {
    dist: Vec<u32>,
    queue: VecDeque<Pos>,
    width: u32,
}

impl RouteBuf {
    pub fn new(w: u32, h: u32) -> Self {
        Self {
            dist: vec![u32::MAX; (w * h) as usize],
            queue: VecDeque::with_capacity((w * h / 4) as usize),
            width: w,
        }
    }

    fn reset(&mut self) {
        self.dist.fill(u32::MAX);
        self.queue.clear();
    }

    #[inline]
    fn idx(&self, p: Pos) -> usize {
        (p.y * self.width + p.x) as usize
    }
}

/// Find the shortest path from `src` to `dst` on the grid using reusable buffers.
pub fn lee_route(
    grid_w: u32,
    grid_h: u32,
    src: Pos,
    dst: Pos,
    passable: impl Fn(Pos) -> bool,
    buf: &mut RouteBuf,
) -> Option<Vec<Pos>> {
    if src == dst {
        return Some(vec![src]);
    }

    buf.reset();
    let src_i = buf.idx(src);
    buf.dist[src_i] = 0;
    buf.queue.push_back(src);

    // Wave expansion.
    let mut found = false;
    while let Some(cur) = buf.queue.pop_front() {
        if cur == dst {
            found = true;
            break;
        }
        let d = buf.dist[buf.idx(cur)];
        for &(dx, dy) in &DIRS {
            let nx = cur.x as i32 + dx;
            let ny = cur.y as i32 + dy;
            if nx < 0 || ny < 0 || nx as u32 >= grid_w || ny as u32 >= grid_h {
                continue;
            }
            let nb = Pos::new(nx as u32, ny as u32);
            let ni = buf.idx(nb);
            if buf.dist[ni] != u32::MAX {
                continue;
            }
            if nb != dst && !passable(nb) {
                continue;
            }
            buf.dist[ni] = d + 1;
            buf.queue.push_back(nb);
        }
    }

    if !found {
        return None;
    }

    // Backtrace from dst to src.
    let end_dist = buf.dist[buf.idx(dst)];
    let mut path = Vec::with_capacity(end_dist as usize + 1);
    path.push(dst);
    let mut cur = dst;

    for expected in (0..end_dist).rev() {
        for &(dx, dy) in &DIRS {
            let nx = cur.x as i32 + dx;
            let ny = cur.y as i32 + dy;
            if nx < 0 || ny < 0 || nx as u32 >= grid_w || ny as u32 >= grid_h {
                continue;
            }
            let nb = Pos::new(nx as u32, ny as u32);
            if buf.dist[buf.idx(nb)] == expected {
                path.push(nb);
                cur = nb;
                break;
            }
        }
    }

    path.reverse();
    Some(path)
}
