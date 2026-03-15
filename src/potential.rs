//! Potential function Φ: evaluates the utility of placing a node at a position.
//!
//! The potential decomposes into three terms:
//! - **Attraction**: connected nodes want proximity (spring-like, Hooke's law)
//! - **Repulsion**: all placed nodes repel (electrostatic, inverse-square)
//! - **Obstacle penalty**: positions on or near obstacles are penalized
//!
//! Higher utility = better position for this agent.

use crate::graph::{OgunConfig, PotentialKernel, Space};
use crate::types::{NodeId, Pos};

/// SoA (struct-of-arrays) layout for placed nodes.
/// Contiguous f32 arrays enable SIMD auto-vectorization of the distance loop.
pub struct PlacedSoa {
    pub xs: Vec<f32>,
    pub ys: Vec<f32>,
    pub radii: Vec<f32>,
    pub count: usize,
}

impl PlacedSoa {
    pub fn new(capacity: usize) -> Self {
        Self {
            xs: Vec::with_capacity(capacity),
            ys: Vec::with_capacity(capacity),
            radii: Vec::with_capacity(capacity),
            count: 0,
        }
    }

    pub fn push(&mut self, pos: Pos, radius: u32) {
        self.xs.push(pos.x as f32);
        self.ys.push(pos.y as f32);
        self.radii.push(radius as f32);
        self.count += 1;
    }
}

/// Evaluate the utility of placing `node` at `pos` given the current state.
///
/// Returns `None` if the position is invalid (obstacle, out of bounds, or
/// overlapping an already-placed node's footprint).
#[allow(clippy::too_many_arguments)]
pub fn utility(
    pos: Pos,
    node_id: NodeId,
    node_radius: f32,
    placed: &PlacedSoa,
    adj: &[(NodeId, f32)],
    positions: &[Option<Pos>],
    space: &Space,
    config: &OgunConfig,
    repulsion_mults: &[f32],
    kernel: &PotentialKernel,
) -> Option<f32> {
    // Invalid: out of bounds or on an obstacle
    if pos.x >= space.width || pos.y >= space.height || space.is_obstacle(pos) {
        return None;
    }

    let px = pos.x as f32;
    let py = pos.y as f32;
    let mut score = 0.0f32;

    // Repulsion cutoff: skip nodes where contribution < 0.01.
    // k / dist_sq < 0.01  →  dist_sq > k * 100
    let cutoff_sq = config.repulsion_k * 100.0;

    // Fused overlap check + repulsion over SoA arrays.
    #[allow(clippy::needless_range_loop)]
    for j in 0..placed.count {
        let dx = px - placed.xs[j];
        let dy = py - placed.ys[j];
        let dist_sq = dx * dx + dy * dy;

        // Overlap rejection
        let min_dist = node_radius + placed.radii[j];
        if dist_sq < min_dist * min_dist {
            return None;
        }

        // Repulsion (skip negligible distant contributions)
        if dist_sq < cutoff_sq {
            score += config.repulsion_k * repulsion_mults[j] / dist_sq.max(1.0);
        }
    }

    // Attraction: connected placed nodes pull this node closer.
    for &(neighbor, weight) in adj {
        if let Some(neighbor_pos) = positions[neighbor.0 as usize] {
            let dist = pos.dist_sq(neighbor_pos).sqrt();
            score -= weight * dist;
        }
    }

    // Kernel: boundary affinity.
    if let Some(&affinity) = kernel.boundary_affinity.get(&node_id) {
        let dx = px.min(space.width as f32 - 1.0 - px);
        let dy = py.min(space.height as f32 - 1.0 - py);
        let dist_to_edge = dx.min(dy);
        let max_dist = (space.width.min(space.height) as f32) / 2.0;
        score += affinity * (1.0 - dist_to_edge / max_dist);
    }

    // Kernel: cell bonus.
    if let Some(ref bonus) = kernel.cell_bonus {
        if let Some(&b) = bonus.get_pos(pos) {
            score += b;
        }
    }

    Some(score)
}
