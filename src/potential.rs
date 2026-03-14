//! Potential function Φ: evaluates the utility of placing a node at a position.
//!
//! The potential decomposes into three terms:
//! - **Attraction**: connected nodes want proximity (spring-like, Hooke's law)
//! - **Repulsion**: all placed nodes repel (electrostatic, inverse-square)
//! - **Obstacle penalty**: positions on or near obstacles are penalized
//!
//! Higher utility = better position for this agent.

use std::collections::HashMap;

use crate::graph::{Graph, OgunConfig, Space};
use crate::types::{NodeId, Pos};

/// Evaluate the utility of placing `node` at `pos` given the current state.
///
/// Returns `None` if the position is invalid (obstacle, out of bounds, or
/// overlapping an already-placed node's footprint).
pub fn utility(
    node: NodeId,
    pos: Pos,
    placed: &HashMap<NodeId, Pos>,
    graph: &Graph,
    space: &Space,
    config: &OgunConfig,
) -> Option<f32> {
    // Invalid: out of bounds or on an obstacle
    if pos.x >= space.width || pos.y >= space.height || space.is_obstacle(pos) {
        return None;
    }

    // Invalid: overlaps an already-placed node (footprint check)
    let node_data = &graph.nodes[node.0 as usize];
    for (&other_id, &other_pos) in placed {
        let other_data = &graph.nodes[other_id.0 as usize];
        let min_dist = (node_data.radius + other_data.radius) as f32;
        if pos.dist_sq(other_pos) < min_dist * min_dist {
            return None;
        }
    }

    let mut score = 0.0;

    // Attraction: connected placed nodes pull this node closer.
    // u_attract = -weight * distance  (negative distance = higher utility when close)
    for edge in graph.edges_of(node) {
        let neighbor = graph.neighbor(edge, node);
        if let Some(&neighbor_pos) = placed.get(&neighbor) {
            let dist = pos.dist_sq(neighbor_pos).sqrt();
            score -= edge.weight * dist;
        }
    }

    // Repulsion: all placed nodes push this node away.
    // u_repel = +k / dist²  (positive when far, prevents crowding)
    for &other_pos in placed.values() {
        let dist_sq = pos.dist_sq(other_pos).max(1.0);
        score += config.repulsion_k / dist_sq;
    }

    Some(score)
}
