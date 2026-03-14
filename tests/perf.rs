//! Large-scale performance tests.
//!
//! Run with: cargo test --release --test perf -- --ignored --nocapture
//!
//! These are `#[ignore]`d so they don't slow down `cargo test`.

use ogun::*;
use rand::Rng;
use rand::SeedableRng;
use rand_chacha::ChaCha8Rng;
use std::time::Instant;

/// Build a reproducible graph with `n` nodes, ~`edges_per_node` edges on average,
/// scattered across a `w × h` grid with `num_obstacles` rectangular obstacles.
fn build_scenario(
    n: u32,
    edges_per_node: u32,
    w: u32,
    h: u32,
    num_obstacles: u32,
    seed: u64,
) -> (Graph, Space) {
    let mut rng = ChaCha8Rng::seed_from_u64(seed);

    let nodes: Vec<Node> = (0..n)
        .map(|i| Node {
            id: NodeId(i),
            radius: rng.random_range(1..=2),
        })
        .collect();

    // Connect each node to `edges_per_node` random earlier nodes (ensures connectivity-ish).
    let mut edges = Vec::new();
    let mut eid = 0u32;
    for i in 1..n {
        let count = edges_per_node.min(i);
        let mut targets = std::collections::HashSet::new();
        while targets.len() < count as usize {
            targets.insert(rng.random_range(0..i));
        }
        for t in targets {
            edges.push(Edge {
                id: EdgeId(eid),
                src: NodeId(i),
                dst: NodeId(t),
                weight: rng.random_range(0.5..3.0_f32),
            });
            eid += 1;
        }
    }

    let obstacles: Vec<Rect> = (0..num_obstacles)
        .map(|_| {
            let ow = rng.random_range(2..=6);
            let oh = rng.random_range(2..=6);
            Rect {
                x: rng.random_range(0..w.saturating_sub(ow)),
                y: rng.random_range(0..h.saturating_sub(oh)),
                w: ow,
                h: oh,
            }
        })
        .collect();

    (
        Graph { nodes, edges },
        Space {
            width: w,
            height: h,
            obstacles,
        },
    )
}

fn run_perf(label: &str, graph: &Graph, space: &Space, config: &OgunConfig) {
    let start = Instant::now();
    let layout = generate(graph, space, config);
    let elapsed = start.elapsed();

    let placed = layout.positions.len();
    let routed = layout.paths.len();
    let total_edges = graph.edges.len();

    println!(
        "{label:30} | {elapsed:>10.2?} | placed {placed:>4}/{:>4} | routed {routed:>4}/{total_edges:>4} | score {:.3}",
        graph.nodes.len(),
        layout.score,
    );
}

// ── Medium: 50 nodes, 100×100 ──────────────────────────────────────

#[test]
#[ignore]
fn perf_medium_50n_100x100() {
    let (graph, space) = build_scenario(50, 2, 100, 100, 10, 1000);
    let config = OgunConfig {
        beta: 2.0,
        seed: 42,
        repulsion_k: 50.0,
    };
    run_perf("medium 50n 100x100 β=2", &graph, &space, &config);
}

// ── Large: 200 nodes, 250×250 ──────────────────────────────────────

#[test]
#[ignore]
fn perf_large_200n_250x250() {
    let (graph, space) = build_scenario(200, 3, 250, 250, 30, 2000);
    let config = OgunConfig {
        beta: 2.0,
        seed: 42,
        repulsion_k: 50.0,
    };
    run_perf("large 200n 250x250 β=2", &graph, &space, &config);
}

// ── Saltglass scale: 100 nodes, 250×110 ────────────────────────────

#[test]
#[ignore]
fn perf_saltglass_100n_250x110() {
    let (graph, space) = build_scenario(100, 3, 250, 110, 15, 3000);
    let config = OgunConfig {
        beta: 2.0,
        seed: 42,
        repulsion_k: 50.0,
    };
    run_perf("saltglass 100n 250x110 β=2", &graph, &space, &config);
}

// ── Stress: 500 nodes, 500×500 ─────────────────────────────────────

#[test]
#[ignore]
fn perf_stress_500n_500x500() {
    let (graph, space) = build_scenario(500, 3, 500, 500, 50, 4000);
    let config = OgunConfig {
        beta: 2.0,
        seed: 42,
        repulsion_k: 50.0,
    };
    run_perf("stress 500n 500x500 β=2", &graph, &space, &config);
}

// ── Beta sweep: same graph at different β values ───────────────────

#[test]
#[ignore]
fn perf_beta_sweep() {
    let (graph, space) = build_scenario(100, 3, 200, 200, 20, 5000);
    println!();
    for &beta in &[0.1, 1.0, 2.0, 5.0, 10.0, 50.0] {
        let config = OgunConfig {
            beta,
            seed: 42,
            repulsion_k: 50.0,
        };
        run_perf(
            &format!("beta_sweep 100n 200x200 β={beta}"),
            &graph,
            &space,
            &config,
        );
    }
}

// ── Dense: many edges per node ─────────────────────────────────────

#[test]
#[ignore]
fn perf_dense_100n_6edges() {
    let (graph, space) = build_scenario(100, 6, 200, 200, 15, 6000);
    let config = OgunConfig {
        beta: 2.0,
        seed: 42,
        repulsion_k: 50.0,
    };
    run_perf("dense 100n 6e/n 200x200", &graph, &space, &config);
}

// ── Obstacle-heavy ─────────────────────────────────────────────────

#[test]
#[ignore]
fn perf_obstacle_heavy() {
    let (graph, space) = build_scenario(100, 3, 200, 200, 100, 7000);
    let config = OgunConfig {
        beta: 2.0,
        seed: 42,
        repulsion_k: 50.0,
    };
    run_perf("obstacle-heavy 100n 200x200", &graph, &space, &config);
}
