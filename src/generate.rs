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

    // Validate kernel dimensions.
    if let Some(ref bonus) = config.kernel.cell_bonus {
        assert!(
            bonus.width == space.width && bonus.height == space.height,
            "cell_bonus dimensions ({}×{}) must match Space ({}×{})",
            bonus.width,
            bonus.height,
            space.width,
            space.height,
        );
    }

    // Extract flat routing cost slice (if provided).
    let routing_cost_cells: Option<Vec<f32>> = space.routing_costs.as_ref().map(|g| {
        assert!(
            g.width == space.width && g.height == space.height,
            "routing_costs dimensions ({}×{}) must match Space ({}×{})",
            g.width,
            g.height,
            space.width,
            space.height,
        );
        (0..h)
            .flat_map(|y| (0..w).map(move |x| *g.get(x, y).unwrap()))
            .collect()
    });
    let rc_slice = routing_cost_cells.as_deref();

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

    // --- PRE-COMMIT: place fixed nodes before the sequential loop ---
    for node in &graph.nodes {
        let Some(fixed_pos) = node.fixed else {
            continue;
        };
        let nid = node.id.0 as usize;
        positions[nid] = Some(fixed_pos);
        placed_soa.push(fixed_pos, node.width, node.height);
        placed_node_ids.push(nid);
        mark_footprint(&mut blocked, w, fixed_pos, node.width, node.height);
    }

    // Route edges between pairs of fixed nodes.
    for e in &graph.edges {
        let si = e.src.0 as usize;
        let di = e.dst.0 as usize;
        if graph.nodes[si].fixed.is_none() || graph.nodes[di].fixed.is_none() {
            continue;
        }
        let src = positions[si].unwrap();
        let dst = positions[di].unwrap();
        let src_hw = graph.nodes[si].width as f32 / 2.0;
        let src_hh = graph.nodes[si].height as f32 / 2.0;
        let dst_hw = graph.nodes[di].width as f32 / 2.0;
        let dst_hh = graph.nodes[di].height as f32 / 2.0;

        let passable = |p: Pos| {
            let i = (p.y * w + p.x) as usize;
            if !blocked[i] {
                return true;
            }
            let in_src = in_footprint(p, src, src_hw, src_hh);
            let in_dst = in_footprint(p, dst, dst_hw, dst_hh);
            in_src || in_dst
        };

        if let Some((path, cost)) = negotiate_route(
            w, h, src, dst, passable, &congestion, &mut dijk_buf, rc_slice,
        ) {
            for &p in &path {
                blocked[(p.y * w + p.x) as usize] = true;
            }
            congestion.add_path(&path);
            route_costs.insert(e.id, cost);
            paths.insert(e.id, path);
        }
    }

    // --- PHASE 1: sequential placement with congestion-aware routing ---
    for node in &graph.nodes {
        // Skip fixed nodes (already committed).
        if node.fixed.is_some() {
            continue;
        }

        let nid = node.id.0 as usize;
        let node_half_w = node.width as f32 / 2.0;
        let node_half_h = node.height as f32 / 2.0;

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
                            node.id,
                            node_half_w,
                            node_half_h,
                            placed_ref,
                            node_adj,
                            positions_ref,
                            space,
                            config,
                            rep_mults_ref,
                            &config.kernel,
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
                        node.id,
                        node_half_w,
                        node_half_h,
                        placed_ref,
                        node_adj,
                        positions_ref,
                        space,
                        config,
                        rep_mults_ref,
                        &config.kernel,
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
        placed_soa.push(chosen, node.width, node.height);
        placed_node_ids.push(nid);
        mark_footprint(&mut blocked, w, chosen, node.width, node.height);

        // ROUTE_NEGOTIATED: connect to already-placed neighbors.
        for &(edge_id, neighbor_id, _) in &adj[nid] {
            if positions[neighbor_id.0 as usize].is_none() {
                continue;
            }
            let src = chosen;
            let dst = positions[neighbor_id.0 as usize].unwrap();
            let src_hw = node.width as f32 / 2.0;
            let src_hh = node.height as f32 / 2.0;
            let dst_hw = graph.nodes[neighbor_id.0 as usize].width as f32 / 2.0;
            let dst_hh = graph.nodes[neighbor_id.0 as usize].height as f32 / 2.0;

            let passable = |p: Pos| {
                let i = (p.y * w + p.x) as usize;
                if !blocked[i] {
                    return true;
                }
                let in_src = in_footprint(p, src, src_hw, src_hh);
                let in_dst = in_footprint(p, dst, dst_hw, dst_hh);
                in_src || in_dst
            };

            let result = negotiate_route(
                w,
                h,
                src,
                dst,
                passable,
                &congestion,
                &mut dijk_buf,
                rc_slice,
            );

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
        struct EdgeInfo {
            eid: EdgeId,
            src: Pos,
            dst: Pos,
            weight: f32,
            src_hw: f32,
            src_hh: f32,
            dst_hw: f32,
            dst_hh: f32,
        }
        let mut edge_info: Vec<EdgeInfo> = graph
            .edges
            .iter()
            .filter_map(|e| {
                let src_pos = positions[e.src.0 as usize]?;
                let dst_pos = positions[e.dst.0 as usize]?;
                Some(EdgeInfo {
                    eid: e.id,
                    src: src_pos,
                    dst: dst_pos,
                    weight: e.weight,
                    src_hw: graph.nodes[e.src.0 as usize].width as f32 / 2.0,
                    src_hh: graph.nodes[e.src.0 as usize].height as f32 / 2.0,
                    dst_hw: graph.nodes[e.dst.0 as usize].width as f32 / 2.0,
                    dst_hh: graph.nodes[e.dst.0 as usize].height as f32 / 2.0,
                })
            })
            .collect();
        edge_info.sort_by(|a, b| b.weight.partial_cmp(&a.weight).unwrap_or(std::cmp::Ordering::Equal));

        for _ in 0..config.negotiation_iterations {
            let mut any_conflict = false;

            for ei in &edge_info {
                // Rip up
                if let Some(old_path) = paths.get(&ei.eid) {
                    congestion.remove_path(old_path);
                }

                let passable = |p: Pos| {
                    let i = (p.y * w + p.x) as usize;
                    if !blocked[i] {
                        return true;
                    }
                    let in_src = in_footprint(p, ei.src, ei.src_hw, ei.src_hh);
                    let in_dst = in_footprint(p, ei.dst, ei.dst_hw, ei.dst_hh);
                    in_src || in_dst
                };

                // Reroute
                if let Some((new_path, cost)) = negotiate_route(
                    w,
                    h,
                    ei.src,
                    ei.dst,
                    passable,
                    &congestion,
                    &mut dijk_buf,
                    rc_slice,
                ) {
                    congestion.add_path(&new_path);
                    paths.insert(ei.eid, new_path);
                    route_costs.insert(ei.eid, cost);
                } else if let Some(old_path) = paths.get(&ei.eid) {
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

/// True if `p` is within the rectangular footprint centered at `center`.
#[inline]
fn in_footprint(p: Pos, center: Pos, half_w: f32, half_h: f32) -> bool {
    (p.x as f32 - center.x as f32).abs() < half_w
        && (p.y as f32 - center.y as f32).abs() < half_h
}

/// Block grid cells within a node's rectangular footprint.
fn mark_footprint(blocked: &mut [bool], grid_w: u32, center: Pos, width: u32, height: u32) {
    let left = (width.saturating_sub(1)) / 2;
    let right = width / 2;
    let top = (height.saturating_sub(1)) / 2;
    let bottom = height / 2;
    let x0 = center.x.saturating_sub(left);
    let x1 = (center.x + right).min(grid_w - 1);
    let y0 = center.y.saturating_sub(top);
    let y1 = (center.y + bottom).min((blocked.len() as u32 / grid_w).saturating_sub(1));
    for y in y0..=y1 {
        for x in x0..=x1 {
            blocked[(y * grid_w + x) as usize] = true;
        }
    }
}
