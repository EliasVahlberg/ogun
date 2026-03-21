//! Composite optimization score for a completed layout.
//!
//! Four metrics, each in [0, 1], combined with equal weights:
//! - Path efficiency: how close routed paths are to optimal length
//! - Accessibility: fraction of edges that were successfully routed
//! - Congestion: inverse of max overlap density on the grid
//! - Void ratio: penalizes too much or too little empty space

use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use crate::graph::{Graph, Space};
use crate::grid::Grid;
use crate::types::{EdgeId, NodeId, Pos};

/// Per-metric breakdown of the composite layout score.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ScoreBreakdown {
    /// How close routed paths are to optimal length. 1.0 = perfect.
    pub path_efficiency: f32,
    /// Fraction of edges that were successfully routed.
    pub accessibility: f32,
    /// Inverse of max overlap density. 1.0 = no shared cells.
    pub congestion: f32,
    /// Penalizes deviation from ~20% empty space. 1.0 = ideal.
    pub void_ratio: f32,
    /// Equal-weight average of the four metrics.
    pub composite: f32,
}

/// Compute the score breakdown for a completed layout.
pub fn score(
    positions: &HashMap<NodeId, Pos>,
    paths: &HashMap<EdgeId, Vec<Pos>>,
    graph: &Graph,
    space: &Space,
) -> ScoreBreakdown {
    let pe = path_efficiency(positions, paths, graph);
    let acc = accessibility(paths, graph);
    let cong = congestion(paths, space);
    let vr = void_ratio(positions, paths, graph, space);

    ScoreBreakdown {
        path_efficiency: pe,
        accessibility: acc,
        congestion: cong,
        void_ratio: vr,
        composite: (pe + acc + cong + vr) / 4.0,
    }
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
    let routed = graph
        .edges
        .iter()
        .filter(|e| paths.contains_key(&e.id))
        .count();
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
    let max_overlap = usage
        .positions()
        .into_iter()
        .filter_map(|p| usage.get(p.x, p.y))
        .max()
        .copied()
        .unwrap_or(0);
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
        let node = &graph.nodes[node_id.0 as usize];
        let left = (node.width.saturating_sub(1)) / 2;
        let right = node.width / 2;
        let top = (node.height.saturating_sub(1)) / 2;
        let bottom = node.height / 2;
        let x0 = pos.x.saturating_sub(left);
        let x1 = (pos.x + right).min(space.width - 1);
        let y0 = pos.y.saturating_sub(top);
        let y1 = (pos.y + bottom).min(space.height - 1);
        for y in y0..=y1 {
            for x in x0..=x1 {
                occupied.set(x, y, true);
            }
        }
    }
    for path in paths.values() {
        for &pos in path {
            occupied.set(pos.x, pos.y, true);
        }
    }

    let used = occupied
        .positions()
        .into_iter()
        .filter(|p| occupied.get(p.x, p.y) == Some(&true))
        .count() as f32;
    let ratio: f32 = 1.0 - (used / total);

    // Ideal void ratio ~0.2. Score drops as we deviate.
    let ideal: f32 = 0.2;
    1.0 - ((ratio - ideal).abs() / ideal).min(1.0)
}
