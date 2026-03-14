//! Composite optimization score for a completed layout.
//!
//! Four metrics, each in [0, 1], combined with equal weights:
//! - Path efficiency: how close routed paths are to optimal length
//! - Accessibility: fraction of edges that were successfully routed
//! - Congestion: inverse of max overlap density on the grid
//! - Void ratio: penalizes too much or too little empty space

use std::collections::HashMap;

use crate::graph::{Graph, Space};
use crate::grid::Grid;
use crate::types::{EdgeId, NodeId, Pos};

/// Compute the composite optimization score in [0, 1].
pub fn score(
    positions: &HashMap<NodeId, Pos>,
    paths: &HashMap<EdgeId, Vec<Pos>>,
    graph: &Graph,
    space: &Space,
) -> f32 {
    let efficiency = path_efficiency(positions, paths, graph);
    let access = accessibility(paths, graph);
    let cong = congestion(paths, space);
    let void_r = void_ratio(positions, paths, graph, space);

    // Equal-weight average of four metrics.
    (efficiency + access + cong + void_r) / 4.0
}

/// Average (manhattan_distance / actual_path_length) over routed edges.
/// Perfect routing scores 1.0.
fn path_efficiency(
    positions: &HashMap<NodeId, Pos>,
    paths: &HashMap<EdgeId, Vec<Pos>>,
    graph: &Graph,
) -> f32 {
    if paths.is_empty() {
        return 0.0;
    }
    let mut sum = 0.0;
    let mut count = 0;
    for edge in &graph.edges {
        if let Some(path) = paths.get(&edge.id) {
            let src_pos = positions[&edge.src];
            let dst_pos = positions[&edge.dst];
            let optimal = src_pos.manhattan(dst_pos).max(1) as f32;
            let actual = (path.len().max(1) - 1) as f32; // path length = steps
            sum += (optimal / actual.max(1.0)).min(1.0);
            count += 1;
        }
    }
    if count == 0 { 0.0 } else { sum / count as f32 }
}

/// Fraction of edges that were successfully routed.
fn accessibility(paths: &HashMap<EdgeId, Vec<Pos>>, graph: &Graph) -> f32 {
    if graph.edges.is_empty() {
        return 1.0;
    }
    let routed = graph.edges.iter().filter(|e| paths.contains_key(&e.id)).count();
    routed as f32 / graph.edges.len() as f32
}

/// 1.0 minus normalized max-overlap. Lower congestion = higher score.
fn congestion(paths: &HashMap<EdgeId, Vec<Pos>>, space: &Space) -> f32 {
    if paths.is_empty() {
        return 1.0;
    }
    let mut usage: Grid<u32> = Grid::new(space.width, space.height, 0);
    for path in paths.values() {
        for &pos in path {
            if let Some(v) = usage.get(pos.x, pos.y) {
                let new = v + 1;
                usage.set(pos.x, pos.y, new);
            }
        }
    }
    // Max overlap across all cells.
    let max_overlap = usage.positions().into_iter().filter_map(|p| usage.get(p.x, p.y)).max().copied().unwrap_or(0);
    if max_overlap <= 1 {
        1.0
    } else {
        1.0 / max_overlap as f32
    }
}

/// Penalizes void ratio that deviates from an ideal ~20% empty space.
/// Returns 1.0 when void ratio is near 0.2, drops toward 0 at extremes.
fn void_ratio(
    positions: &HashMap<NodeId, Pos>,
    paths: &HashMap<EdgeId, Vec<Pos>>,
    graph: &Graph,
    space: &Space,
) -> f32 {
    let total = (space.width * space.height) as f32;
    if total == 0.0 {
        return 0.0;
    }

    // Count occupied cells (node footprints + path cells).
    let mut occupied: Grid<bool> = Grid::new(space.width, space.height, false);
    for (&node_id, &pos) in positions {
        let r = graph.nodes[node_id.0 as usize].radius;
        let r_i32 = r as i32;
        for dy in -r_i32..=r_i32 {
            for dx in -r_i32..=r_i32 {
                let nx = pos.x as i32 + dx;
                let ny = pos.y as i32 + dy;
                if nx >= 0 && ny >= 0 {
                    occupied.set(nx as u32, ny as u32, true);
                }
            }
        }
    }
    for path in paths.values() {
        for &pos in path {
            occupied.set(pos.x, pos.y, true);
        }
    }

    let used = occupied.positions().into_iter().filter(|p| occupied.get(p.x, p.y) == Some(&true)).count() as f32;
    let ratio: f32 = 1.0 - (used / total);

    // Ideal void ratio ~0.2. Score drops as we deviate.
    let ideal: f32 = 0.2;
    1.0 - ((ratio - ideal).abs() / ideal).min(1.0)
}
