//! The main Ogun generation loop.
//!
//! ```text
//! INIT → for each agent in arrival order:
//!     EVAL_UTILITY over all valid positions
//!   → CHOOSE_BOLTZMANN (sample position)
//!   → COMMIT_IRREVOCABLE (place permanently)
//!   → ROUTE edges to already-placed neighbors (Lee BFS)
//! → SCORE the completed layout
//! ```
//!
//! This is a single-pass algorithm. Each agent plays once and is
//! committed irrevocably. β controls the exploration/exploitation
//! tradeoff — the "optimization level" of the output.

use std::collections::{HashMap, HashSet};

use rand::SeedableRng;
use rand_chacha::ChaCha8Rng;

use crate::graph::{Graph, OgunConfig, Space};
use crate::grid::Grid;
use crate::layout::Layout;
use crate::placement::{boltzmann_sample, Candidate};
use crate::potential::utility;
use crate::routing::lee_route;
use crate::scoring::score;
use crate::types::Pos;

/// Generate a spatial layout from a graph and spatial domain.
///
/// Deterministic: same `graph` + `space` + `config` (including seed) = same output.
pub fn generate(graph: &Graph, space: &Space, config: &OgunConfig) -> Layout {
    let mut rng = ChaCha8Rng::seed_from_u64(config.seed);
    let mut positions = HashMap::new();
    let mut paths = HashMap::new();

    // Track which grid cells are blocked (by node footprints or routed paths).
    let mut blocked: Grid<bool> = Grid::new(space.width, space.height, false);

    // Mark obstacle cells as blocked.
    for pos in blocked.positions() {
        if space.is_obstacle(pos) {
            blocked.set(pos.x, pos.y, true);
        }
    }

    // --- MAIN LOOP: process each agent in order ---
    for node in &graph.nodes {
        // EVAL_UTILITY: score every valid position.
        let candidates: Vec<Candidate> = blocked
            .positions()
            .into_iter()
            .filter_map(|pos| {
                utility(node.id, pos, &positions, graph, space, config)
                    .map(|u| Candidate { pos, utility: u })
            })
            .collect();

        // CHOOSE_BOLTZMANN: sample a position.
        let chosen = match boltzmann_sample(&candidates, config.beta, &mut rng) {
            Some(pos) => pos,
            None => continue, // no valid position — skip this agent
        };

        // COMMIT_IRREVOCABLE: place the node and block its footprint.
        positions.insert(node.id, chosen);
        mark_footprint(&mut blocked, chosen, node.radius);

        // ROUTE: connect to already-placed neighbors via Lee BFS.
        let placed_set: HashSet<_> = positions.keys().copied().collect();
        for edge in graph.edges_of(node.id) {
            let neighbor_id = graph.neighbor(edge, node.id);
            if !placed_set.contains(&neighbor_id) {
                continue; // neighbor not placed yet — route later
            }
            let src = chosen;
            let dst = positions[&neighbor_id];

            // Allow routing through the footprints of the two endpoint nodes.
            let src_r = node.radius as f32;
            let dst_r = graph.nodes[neighbor_id.0 as usize].radius as f32;
            let blocked_ref = &blocked;
            let path = lee_route(space.width, space.height, src, dst, |p| {
                if !blocked_ref.get_pos(p).copied().unwrap_or(true) {
                    return true; // not blocked at all
                }
                // Allow cells within either endpoint's footprint.
                let in_src = p.dist_sq(src) <= src_r * src_r;
                let in_dst = p.dist_sq(dst) <= dst_r * dst_r;
                in_src || in_dst
            });

            if let Some(path) = path {
                // Mark path cells as blocked for future routing.
                for &p in &path {
                    blocked.set(p.x, p.y, true);
                }
                paths.insert(edge.id, path);
            }
        }
    }

    // SCORE the completed layout.
    let s = score(&positions, &paths, graph, space);

    Layout {
        positions,
        paths,
        score: s,
    }
}

/// Block grid cells within a node's footprint (square approximation).
fn mark_footprint(blocked: &mut Grid<bool>, center: Pos, radius: u32) {
    let r = radius as i32;
    for dy in -r..=r {
        for dx in -r..=r {
            let nx = center.x as i32 + dx;
            let ny = center.y as i32 + dy;
            if nx >= 0 && ny >= 0 {
                blocked.set(nx as u32, ny as u32, true);
            }
        }
    }
}
