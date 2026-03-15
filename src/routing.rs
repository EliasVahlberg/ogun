//! Routing: Lee BFS and PathFinder negotiation-based Dijkstra.
//!
//! Phase 1 (during placement): `negotiate_route()` — Dijkstra with congestion costs.
//! Phase 2 (after placement): `rip_up_and_reroute()` — iterative rip-up to resolve conflicts.

use std::cmp::Ordering;
use std::collections::{BinaryHeap, VecDeque};

use crate::types::Pos;

const DIRS: [(i32, i32); 4] = [(0, 1), (1, 0), (0, -1), (-1, 0)];

/// Reusable buffers for Lee routing to avoid per-call allocation.
pub struct RouteBuf {
    pub dist: Vec<u32>,
    pub queue: VecDeque<Pos>,
    pub width: u32,
}

#[allow(dead_code)]
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

/// Lee BFS routing (uniform cost, kept as fallback).
#[allow(dead_code)]
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

// ── PathFinder negotiation routing ─────────────────────────────────

/// Congestion state for PathFinder negotiation routing.
pub struct CongestionState {
    /// Number of routes currently using each cell.
    pub sharing: Vec<u32>,
    /// Accumulated historical congestion penalty.
    pub history: Vec<f32>,
    pub width: u32,
}

impl CongestionState {
    pub fn new(w: u32, h: u32) -> Self {
        let sz = (w * h) as usize;
        Self {
            sharing: vec![0; sz],
            history: vec![0.0; sz],
            width: w,
        }
    }

    #[inline]
    fn idx(&self, p: Pos) -> usize {
        (p.y * self.width + p.x) as usize
    }

    /// PathFinder cost: routing_cost * (base + history) * (1 + sharing)
    #[inline]
    fn cost(&self, p: Pos, routing_costs: Option<&[f32]>) -> f32 {
        let i = self.idx(p);
        let base = match routing_costs {
            Some(rc) => rc[i],
            None => 1.0,
        };
        base * (1.0 + self.history[i]) * (1 + self.sharing[i]) as f32
    }

    /// Add a path to the sharing grid.
    pub fn add_path(&mut self, path: &[Pos]) {
        for &p in path {
            let i = (p.y * self.width + p.x) as usize;
            self.sharing[i] += 1;
        }
    }

    /// Remove a path from the sharing grid.
    pub fn remove_path(&mut self, path: &[Pos]) {
        for &p in path {
            let i = (p.y * self.width + p.x) as usize;
            self.sharing[i] = self.sharing[i].saturating_sub(1);
        }
    }

    /// Increase history for congested cells.
    pub fn update_history(&mut self, increment: f32) {
        for i in 0..self.sharing.len() {
            if self.sharing[i] > 1 {
                self.history[i] += increment;
            }
        }
    }
}

/// Reusable buffers for Dijkstra routing.
pub struct DijkBuf {
    dist: Vec<f32>,
    prev: Vec<u32>,
    heap: BinaryHeap<DijkEntry>,
}

impl DijkBuf {
    pub fn new(w: u32, h: u32) -> Self {
        let sz = (w * h) as usize;
        Self {
            dist: vec![f32::INFINITY; sz],
            prev: vec![u32::MAX; sz],
            heap: BinaryHeap::with_capacity(sz / 4),
        }
    }
}

/// Dijkstra entry for the priority queue.
#[derive(Clone, Copy)]
struct DijkEntry {
    cost: f32,
    pos: Pos,
}

impl PartialEq for DijkEntry {
    fn eq(&self, other: &Self) -> bool {
        self.cost == other.cost
    }
}
impl Eq for DijkEntry {}

impl PartialOrd for DijkEntry {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}
impl Ord for DijkEntry {
    fn cmp(&self, other: &Self) -> Ordering {
        // Reverse for min-heap
        other
            .cost
            .partial_cmp(&self.cost)
            .unwrap_or(Ordering::Equal)
    }
}

/// Congestion-aware Dijkstra routing.
///
/// Returns `(path, total_cost)` or `None` if unreachable.
#[allow(clippy::too_many_arguments)]
pub fn negotiate_route(
    grid_w: u32,
    grid_h: u32,
    src: Pos,
    dst: Pos,
    passable: impl Fn(Pos) -> bool,
    congestion: &CongestionState,
    buf: &mut DijkBuf,
    routing_costs: Option<&[f32]>,
) -> Option<(Vec<Pos>, f32)> {
    if src == dst {
        return Some((vec![src], 0.0));
    }

    let sz = (grid_w * grid_h) as usize;
    buf.dist.resize(sz, f32::INFINITY);
    buf.dist.fill(f32::INFINITY);
    buf.prev.resize(sz, u32::MAX);
    buf.prev.fill(u32::MAX);
    buf.heap.clear();

    let idx = |p: Pos| (p.y * grid_w + p.x) as usize;

    buf.dist[idx(src)] = 0.0;
    buf.heap.push(DijkEntry {
        cost: 0.0,
        pos: src,
    });

    while let Some(DijkEntry { cost, pos: cur }) = buf.heap.pop() {
        if cur == dst {
            break;
        }
        if cost > buf.dist[idx(cur)] {
            continue;
        }
        for &(dx, dy) in &DIRS {
            let nx = cur.x as i32 + dx;
            let ny = cur.y as i32 + dy;
            if nx < 0 || ny < 0 || nx as u32 >= grid_w || ny as u32 >= grid_h {
                continue;
            }
            let nb = Pos::new(nx as u32, ny as u32);
            if nb != dst && !passable(nb) {
                continue;
            }
            let edge_cost = congestion.cost(nb, routing_costs);
            let new_cost = cost + edge_cost;
            let ni = idx(nb);
            if new_cost < buf.dist[ni] {
                buf.dist[ni] = new_cost;
                buf.prev[ni] = idx(cur) as u32;
                buf.heap.push(DijkEntry {
                    cost: new_cost,
                    pos: nb,
                });
            }
        }
    }

    let di = idx(dst);
    if buf.dist[di] == f32::INFINITY {
        return None;
    }

    let total_cost = buf.dist[di];
    let mut path = vec![dst];
    let mut ci = di;
    while ci != idx(src) {
        let pi = buf.prev[ci] as usize;
        let py = (pi as u32) / grid_w;
        let px = (pi as u32) % grid_w;
        path.push(Pos::new(px, py));
        ci = pi;
    }
    path.reverse();
    Some((path, total_cost))
}
