# Ogun
[[github]](https://github.com/EliasVahlberg/ogun)
[[crates.io]](https://crates.io/crates/ogun)
[[docs.rs]](https://docs.rs/ogun)

Spatial layout generation via sequential logit dynamics on potential games.

Given a weighted graph and a 2D grid, Ogun places nodes and routes edges at a
controllable optimization level. Agents arrive sequentially and select positions
via Boltzmann sampling against a potential function encoding overlap, repulsion,
and attraction. The inverse temperature β governs output character — from
near-random (low β) to near-optimal (high β).

Named after the Yoruba deity of iron, metalwork, and pathfinding — a god who
routes paths through uncharted space.

```toml
[dependencies]
ogun = "0.3"
```

## Example

```rust
use ogun::*;

let graph = Graph {
    nodes: vec![
        Node { id: NodeId(0), radius: 1 },
        Node { id: NodeId(1), radius: 1 },
    ],
    edges: vec![Edge {
        id: EdgeId(0),
        src: NodeId(0),
        dst: NodeId(1),
        weight: 1.0,
    }],
};
let space = Space { width: 20, height: 20, obstacles: vec![], routing_costs: None };
let config = OgunConfig { seed: 42, ..OgunConfig::default() };

let layout = generate(&graph, &space, &config);
assert!(layout.score.composite > 0.0);
```

## Algorithm

```text
for each agent in arrival order:
    1. EVAL  — score every grid position via potential Φ
    2. CHOOSE — Boltzmann sample: P(p) ∝ exp(β · Φ(p))
    3. COMMIT — place irrevocably, block footprint
    4. ROUTE  — negotiated Dijkstra to already-placed neighbors
score the completed layout
```

The potential Φ combines overlap penalty, pairwise repulsion (inverse-square),
and edge attraction (distance to placed neighbors). See
[`docs/research/`](docs/research/) for theoretical foundations and complexity
analysis.

## Performance

Benchmarks across problem sizes, measured in `--release` mode. The stress test
(500 nodes on a 500×500 grid) is the primary target.

| Nodes | Grid | Naive | Optimized | Speedup |
|---|---|---|---|---|
| 50 | 100×100 | 139 ms | 44 ms | 3.2× |
| 100 | 250×110 | 1.62 s | 323 ms | 5.0× |
| 200 | 250×250 | 12.2 s | 1.28 s | 9.5× |
| 500 | 500×500 | 280 s | 11.3 s | 24.8× |

Key optimizations: pre-built adjacency lists, SoA placed-node layout, fused
overlap/repulsion pass, buffer reuse, Rayon parallel utility evaluation, and
spatial repulsion cutoff. Full breakdown in [`docs/BENCHMARKS.md`](docs/BENCHMARKS.md).

You can run the benchmarks with:

```console
$ cargo test --release --test perf -- --ignored --nocapture
```

## Paper

The algorithm is described in detail in [docs/paper/ogun_paper.pdf](docs/paper/ogun_paper.pdf):

> Elias Vahlberg. *Ogun: Spatial Layout Generation via Sequential Logit Dynamics
> on Potential Games.* March 2026.

## License

MIT
