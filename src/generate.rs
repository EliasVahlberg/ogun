//! The main Ogun generation loop.
//!
//! ```text
//! INIT → for each agent in arrival order:
//!     EVAL_UTILITY over all valid positions
//!   → CHOOSE_BOLTZMANN (sample position)
//!   → COMMIT_IRREVOCABLE (place permanently)
//!   → ROUTE_NEGOTIATED edges to already-placed neighbors (Dijkstra + congestion)
//! → RIP_UP_AND_REROUTE (PathFinder Phase 2)
//! → SCORE the completed layout
//! ```

use std::collections::HashMap;

use rand::SeedableRng;
use rand_chacha::ChaCha8Rng;

use rayon::prelude::*;

use crate::graph::{Graph, OgunConfig, Space};
use crate::layout::Layout;
use crate::placement::{Candidate, boltzmann_sample};
use crate::potential::{PlacedSoa, utility};
use crate::routing::{CongestionState, DijkBuf, negotiate_route};
use crate::scoring::score;
use crate::types::{EdgeId, NodeId, Pos};

/// Generate a spatial layout from a graph and spatial domain.
///
/// Deterministic: same `graph` + `space` + `config` (including seed) = same output.
pub fn generate(graph: &Graph, space: &Space, config: &OgunConfig) -> Layout {
    let mut rng = ChaCha8Rng::seed_from_u64(config.seed);
    let mut positions: Vec<Option<Pos>> = vec![None; graph.nodes.len()];
    let mut paths: HashMap<EdgeId, Vec<Pos>> = HashMap::new();
    let mut route_costs: HashMap<EdgeId, f32> = HashMap::new();
    let adj = graph.adjacency();
    let w = space.width;
    let h = space.height;
    let sz = (w * h) as usize;

    // Track which grid cells are blocked (by node footprints or routed paths).
    let mut blocked = vec![false; sz];
    for y in 0..h {
        for x in 0..w {
            if space.is_obstacle(Pos::new(x, y)) {
                blocked[(y * w + x) as usize] = true;
            }
        }
    }

    // Pre-allocate reusable buffers.
    let mut candidates: Vec<Candidate> = Vec::with_capacity(sz);
    let mut placed_soa = PlacedSoa::new(graph.nodes.len());
    let mut unplaced: Vec<NodeId> = Vec::new();
    let mut congestion = CongestionState::new(w, h);
    let mut dijk_buf = DijkBuf::new(w, h);

    // Pre-build per-node adjacency (without EdgeId) for utility.
    let node_adjs: Vec<Vec<(NodeId, f32)>> = adj
        .iter()
        .map(|edges| edges.iter().map(|&(_, nb, wt)| (nb, wt)).collect())
        .collect();

    // Pre-build flat repulsion multiplier matrix (n×n, default 1.0).
    let n = graph.nodes.len();
    let mut repulsion_matrix = vec![1.0f32; n * n];
    for (&(a, b), &mult) in &config.repulsion_pairs {
        let ai = a.0 as usize;
        let bi = b.0 as usize;
        if ai < n && bi < n {
            repulsion_matrix[ai * n + bi] = mult;
            repulsion_matrix[bi * n + ai] = mult;
        }
    }
    let mut placed_node_ids: Vec<usize> = Vec::with_capacity(n);

    // --- PHASE 1: sequential placement with congestion-aware routing ---
    for node in &graph.nodes {
        let nid = node.id.0 as usize;
        let node_radius = node.radius as f32;

        // EVAL_UTILITY
        let node_adj = &node_adjs[nid];
        let positions_ref = &positions;
        let placed_ref = &placed_soa;
        let rep_mults: Vec<f32> = placed_node_ids
            .iter()
            .map(|&pid| repulsion_matrix[nid * n + pid])
            .collect();
        let rep_mults_ref = &rep_mults;

        if sz * placed_soa.count >= 500_000 {
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
                            rep_mults_ref,
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
                        rep_mults_ref,
                    ) {
                        candidates.push(Candidate { pos, utility: u });
                    }
                }
            }
        }

        // CHOOSE_BOLTZMANN
        let chosen = match boltzmann_sample(&candidates, config.beta, &mut rng) {
            Some(pos) => pos,
            None => {
                unplaced.push(node.id);
                continue;
            }
        };

        // COMMIT_IRREVOCABLE
        positions[nid] = Some(chosen);
        placed_soa.push(chosen, node.radius);
        placed_node_ids.push(nid);
        mark_footprint(&mut blocked, w, chosen, node.radius);

        // ROUTE_NEGOTIATED: connect to already-placed neighbors.
        for &(edge_id, neighbor_id, _) in &adj[nid] {
            if positions[neighbor_id.0 as usize].is_none() {
                continue;
            }
            let src = chosen;
            let dst = positions[neighbor_id.0 as usize].unwrap();
            let src_r = node.radius as f32;
            let dst_r = graph.nodes[neighbor_id.0 as usize].radius as f32;

            let passable = |p: Pos| {
                let i = (p.y * w + p.x) as usize;
                if !blocked[i] {
                    return true;
                }
                let in_src = p.dist_sq(src) <= src_r * src_r;
                let in_dst = p.dist_sq(dst) <= dst_r * dst_r;
                in_src || in_dst
            };

            let result = negotiate_route(w, h, src, dst, passable, &congestion, &mut dijk_buf);

            if let Some((path, cost)) = result {
                for &p in &path {
                    blocked[(p.y * w + p.x) as usize] = true;
                }
                congestion.add_path(&path);
                route_costs.insert(edge_id, cost);
                paths.insert(edge_id, path);
            }
        }
    }

    // --- PHASE 2: rip-up and reroute ---
    if config.negotiation_iterations > 0 && !paths.is_empty() {
        // Collect edge info for rerouting, sorted by weight descending.
        let mut edge_info: Vec<(EdgeId, Pos, Pos, f32, f32, f32)> = graph
            .edges
            .iter()
            .filter_map(|e| {
                let src_pos = positions[e.src.0 as usize]?;
                let dst_pos = positions[e.dst.0 as usize]?;
                let src_r = graph.nodes[e.src.0 as usize].radius as f32;
                let dst_r = graph.nodes[e.dst.0 as usize].radius as f32;
                Some((e.id, src_pos, dst_pos, e.weight, src_r, dst_r))
            })
            .collect();
        edge_info.sort_by(|a, b| b.3.partial_cmp(&a.3).unwrap_or(std::cmp::Ordering::Equal));

        for _ in 0..config.negotiation_iterations {
            let mut any_conflict = false;

            for &(eid, src, dst, _weight, src_r, dst_r) in &edge_info {
                // Rip up
                if let Some(old_path) = paths.get(&eid) {
                    congestion.remove_path(old_path);
                }

                let passable = |p: Pos| {
                    let i = (p.y * w + p.x) as usize;
                    if !blocked[i] {
                        return true;
                    }
                    let in_src = p.dist_sq(src) <= src_r * src_r;
                    let in_dst = p.dist_sq(dst) <= dst_r * dst_r;
                    in_src || in_dst
                };

                // Reroute
                if let Some((new_path, cost)) =
                    negotiate_route(w, h, src, dst, passable, &congestion, &mut dijk_buf)
                {
                    congestion.add_path(&new_path);
                    paths.insert(eid, new_path);
                    route_costs.insert(eid, cost);
                } else if let Some(old_path) = paths.get(&eid) {
                    // Couldn't reroute — restore
                    congestion.add_path(old_path);
                }
            }

            // Check convergence
            for &s in &congestion.sharing {
                if s > 1 {
                    any_conflict = true;
                    break;
                }
            }
            if !any_conflict {
                break;
            }

            congestion.update_history(config.history_increment);
        }
    }

    // SCORE the completed layout.
    let pos_map: HashMap<NodeId, Pos> = positions
        .iter()
        .enumerate()
        .filter_map(|(i, p)| p.map(|pos| (NodeId(i as u32), pos)))
        .collect();
    let s = score(&pos_map, &paths, graph, space);

    // Per-node metrics.
    let mut node_accessibility: HashMap<NodeId, f32> = HashMap::new();
    let mut node_congestion: HashMap<NodeId, f32> = HashMap::new();
    for node in &graph.nodes {
        let nid = node.id.0 as usize;
        if positions[nid].is_none() {
            continue;
        }
        let edges = &adj[nid];
        if edges.is_empty() {
            node_accessibility.insert(node.id, 1.0);
            node_congestion.insert(node.id, 0.0);
            continue;
        }
        let mut routed = 0u32;
        let mut cost_sum = 0.0f32;
        for &(eid, _, _) in edges {
            if let Some(&c) = route_costs.get(&eid) {
                routed += 1;
                cost_sum += c;
            }
        }
        let total = edges.len() as f32;
        node_accessibility.insert(node.id, routed as f32 / total);
        node_congestion.insert(
            node.id,
            if routed > 0 {
                cost_sum / routed as f32
            } else {
                0.0
            },
        );
    }

    Layout {
        positions: pos_map,
        paths,
        score: s,
        unplaced,
        route_costs,
        congestion_grid: congestion.sharing,
        node_accessibility,
        node_congestion,
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
