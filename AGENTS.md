# AGENTS.md

## Project

Ogun is a Rust library for 2D spatial layout generation via sequential logit dynamics on potential games. It adapts PCB design algorithms (force-directed placement, negotiation-based routing) for procedural city/town generation. Domain-agnostic — operates on abstract nodes and edges, not buildings and roads.

Published on [crates.io](https://crates.io/crates/ogun). Downstream consumer: [oku](https://github.com/EliasVahlberg/oku).

## Architecture

Single-crate library. One public entry point: `generate(&Graph, &Space, &OgunConfig) -> Layout`.

```
src/
├── lib.rs          — public API re-exports
├── types.rs        — NodeId, EdgeId, Pos (newtype indices, grid coords)
├── graph.rs        — Graph, Node, Edge, Space, Rect, OgunConfig (input types)
├── layout.rs       — Layout (output type)
├── grid.rs         — Grid<T> (flat Vec with 2D indexing)
├── generate.rs     — main loop: placement → routing → scoring
├── placement.rs    — Boltzmann sampling (Candidate, boltzmann_sample)
├── potential.rs    — utility function (PlacedSoa, SoA layout for SIMD)
├── routing.rs      — CongestionState, DijkBuf, negotiate_route (Dijkstra + PathFinder)
└── scoring.rs      — ScoreBreakdown, score() (4 metrics + composite)
```

### Data flow

1. `generate()` iterates nodes sequentially
2. For each node: evaluate `utility()` over all grid cells (parallel via Rayon for large inputs)
3. `boltzmann_sample()` selects a position proportional to `exp(β · utility)`
4. Route edges to placed neighbors via `negotiate_route()` (congestion-aware Dijkstra)
5. Phase 2: rip-up-and-reroute (PathFinder negotiation) to resolve congestion
6. `score()` computes `ScoreBreakdown` from the completed layout

### Key design decisions

- SoA (struct-of-arrays) layout in `PlacedSoa` for SIMD auto-vectorization of the utility hot loop
- Flat `Vec<T>` grids with `y * width + x` indexing — no 2D arrays
- Buffer reuse everywhere: `DijkBuf`, `candidates` vec, `blocked` grid are pre-allocated and cleared per iteration
- Deterministic output via `ChaCha8Rng` seeded from `config.seed`
- `lee_route` / `RouteBuf` kept with `#[allow(dead_code)]` as BFS fallback

## Conventions

- Rust 2024 edition, MSRV 1.85
- `cargo fmt` and `cargo clippy --all-targets -- -D warnings` must pass clean
- Two intentional clippy allows: `needless_range_loop` (SoA index access), `too_many_arguments` (utility hot path)
- All `OgunConfig` struct literals in tests use `..Default::default()`
- Serde derives on all public types. New `OgunConfig` fields must have `#[serde(default)]`

## Testing

```
tests/
├── smoke.rs              — 6 property tests (determinism, bounds, obstacles, etc.)
├── regression.rs         — 6 fixture-based regression tests (JSON snapshots)
├── generate_fixtures.rs  — fixture regeneration (run with --ignored)
├── perf.rs               — performance benchmarks (run with --ignored)
└── fixtures/             — JSON fixture files
```

- `cargo test --release` runs smoke + regression + doc tests (13 total)
- `cargo test --release --test generate_fixtures -- --ignored` regenerates fixtures
- `cargo test --release --test perf -- --ignored` runs perf benchmarks
- Fixtures must be regenerated when output-affecting code changes

## Release process

1. Update `CHANGELOG.md`
2. Bump version in `Cargo.toml`
3. `cargo fmt && cargo clippy --all-targets -- -D warnings && cargo test --release`
4. Commit, tag (`git tag vX.Y.Z`), push tag
5. Tag push triggers `.github/workflows/release.yml` (builds 3 platform artifacts, creates GitHub release)
6. `cargo publish`

## Performance

Hot path is `utility()` in `potential.rs` — called `nodes × grid_cells` times. Optimizations:
- SoA placed-node layout for SIMD
- Rayon parallel evaluation when `grid_size × placed_count ≥ 500,000`
- Spatial repulsion cutoff (skip distant nodes)
- Pre-built adjacency lists and flat repulsion matrix

Current benchmarks (v0.2.0, `--release`):

| Nodes | Grid | Time |
|-------|------|------|
| 50 | 100×100 | 131 ms |
| 100 | 250×110 | 797 ms |
| 200 | 250×250 | 2.87 s |
