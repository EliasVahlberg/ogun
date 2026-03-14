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

use std::collections::HashMap;

use rand::SeedableRng;
use rand_chacha::ChaCha8Rng;

use rayon::prelude::*;

use crate::graph::{Graph, OgunConfig, Space};
use crate::layout::Layout;
use crate::placement::{Candidate, boltzmann_sample};
use crate::potential::{PlacedSoa, utility};
use crate::routing::{RouteBuf, lee_route};
use crate::scoring::score;
use crate::types::{NodeId, Pos};

/// Generate a spatial layout from a graph and spatial domain.
///
/// Deterministic: same `graph` + `space` + `config` (including seed) = same output.
pub fn generate(graph: &Graph, space: &Space, config: &OgunConfig) -> Layout {
    let mut rng = ChaCha8Rng::seed_from_u64(config.seed);
    let mut positions: Vec<Option<Pos>> = vec![None; graph.nodes.len()];
    let mut paths = HashMap::new();
    let adj = graph.adjacency();
    let w = space.width;
    let h = space.height;

    // Track which grid cells are blocked (by node footprints or routed paths).
    let mut blocked = vec![false; (w * h) as usize];

    // Mark obstacle cells as blocked.
    for y in 0..h {
        for x in 0..w {
            if space.is_obstacle(Pos::new(x, y)) {
                blocked[(y * w + x) as usize] = true;
            }
        }
    }

    // Pre-allocate reusable buffers.
    let mut candidates: Vec<Candidate> = Vec::with_capacity((w * h) as usize);
    let mut route_buf = RouteBuf::new(w, h);
    let mut placed_soa = PlacedSoa::new(graph.nodes.len());

    // Pre-build per-node adjacency (without EdgeId) for utility.
    let node_adjs: Vec<Vec<(NodeId, f32)>> = adj
        .iter()
        .map(|edges| edges.iter().map(|&(_, nb, wt)| (nb, wt)).collect())
        .collect();

    // --- MAIN LOOP: process each agent in order ---
    for node in &graph.nodes {
        let nid = node.id.0 as usize;
        let node_radius = node.radius as f32;

        // EVAL_UTILITY: score every position.
        // Use rayon for large grids, sequential for small ones.
        let node_adj = &node_adjs[nid];
        let positions_ref = &positions;
        let placed_ref = &placed_soa;
        let grid_size = (w * h) as usize;
        if grid_size * placed_soa.count >= 500_000 {
            candidates = (0..h)
                .into_par_iter()
                .flat_map(|y| {
                    (0..w).into_par_iter().filter_map(move |x| {
                        let pos = Pos::new(x, y);
                        utility(
                            pos,
                            node_radius,
                            placed_ref,
                            node_adj,
                            positions_ref,
                            space,
                            config,
                        )
                        .map(|u| Candidate { pos, utility: u })
                    })
                })
                .collect();
        } else {
            candidates.clear();
            for y in 0..h {
                for x in 0..w {
                    let pos = Pos::new(x, y);
                    if let Some(u) = utility(
                        pos,
                        node_radius,
                        placed_ref,
                        node_adj,
                        positions_ref,
                        space,
                        config,
                    ) {
                        candidates.push(Candidate { pos, utility: u });
                    }
                }
            }
        }

        // CHOOSE_BOLTZMANN: sample a position.
        let chosen = match boltzmann_sample(&candidates, config.beta, &mut rng) {
            Some(pos) => pos,
            None => continue,
        };

        // COMMIT_IRREVOCABLE: place the node and block its footprint.
        positions[nid] = Some(chosen);
        placed_soa.push(chosen, node.radius);
        mark_footprint(&mut blocked, w, chosen, node.radius);

        // ROUTE: connect to already-placed neighbors via Lee BFS.
        for &(edge_id, neighbor_id, _) in &adj[nid] {
            if positions[neighbor_id.0 as usize].is_none() {
                continue;
            }
            let src = chosen;
            let dst = positions[neighbor_id.0 as usize].unwrap();

            let src_r = node.radius as f32;
            let dst_r = graph.nodes[neighbor_id.0 as usize].radius as f32;
            let blocked_ref = &blocked;
            let path = lee_route(
                w,
                h,
                src,
                dst,
                |p| {
                    if !blocked_ref[(p.y * w + p.x) as usize] {
                        return true;
                    }
                    let in_src = p.dist_sq(src) <= src_r * src_r;
                    let in_dst = p.dist_sq(dst) <= dst_r * dst_r;
                    in_src || in_dst
                },
                &mut route_buf,
            );

            if let Some(path) = path {
                for &p in &path {
                    blocked[(p.y * w + p.x) as usize] = true;
                }
                paths.insert(edge_id, path);
            }
        }
    }

    // SCORE the completed layout.
    let pos_map: HashMap<NodeId, Pos> = positions
        .iter()
        .enumerate()
        .filter_map(|(i, p)| p.map(|pos| (NodeId(i as u32), pos)))
        .collect();
    let s = score(&pos_map, &paths, graph, space);

    Layout {
        positions: pos_map,
        paths,
        score: s,
    }
}

/// Block grid cells within a node's footprint (square approximation).
fn mark_footprint(blocked: &mut [bool], w: u32, center: Pos, radius: u32) {
    let r = radius as i32;
    for dy in -r..=r {
        for dx in -r..=r {
            let nx = center.x as i32 + dx;
            let ny = center.y as i32 + dy;
            if nx >= 0 && ny >= 0 && (nx as u32) < w {
                let idx = ny as u32 * w + nx as u32;
                if (idx as usize) < blocked.len() {
                    blocked[idx as usize] = true;
                }
            }
        }
    }
}
