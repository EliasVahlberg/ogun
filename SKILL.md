# SKILL.md

## Domain

Spatial layout generation using game-theoretic placement (sequential logit dynamics on potential games) combined with EDA-derived routing (PathFinder negotiation). The algorithm places nodes on a 2D grid via Boltzmann sampling against a potential function, then routes edges using congestion-aware Dijkstra with iterative rip-up-and-reroute.

## Core algorithms

### Boltzmann sampling (logit dynamics)
Each node selects a grid position with probability `P(p) ∝ exp(β · Φ(p))` where `Φ` is the utility (potential) function. The inverse temperature `β` controls exploration vs exploitation. Numerical stability requires subtracting `max(utilities)` before exponentiation.

### Utility / potential function
`utility()` in `potential.rs` scores a candidate position. Components:
- Overlap penalty (hard constraint — returns `None` if footprint blocked)
- Pairwise repulsion (inverse-square, with per-pair multipliers from `repulsion_pairs`)
- Edge attraction (distance to placed neighbors, weighted by edge weight)

Uses SoA (struct-of-arrays) layout (`PlacedSoa`) for SIMD auto-vectorization. The inner loop over placed nodes is the performance bottleneck.

### PathFinder negotiation routing
Two-phase routing adapted from McMurchie & Ebeling (1995):
- Phase 1: During placement, each edge is routed via Dijkstra with cost `(1 + history) × (1 + sharing)`
- Phase 2: After all nodes placed, rip-up-and-reroute iterates edges by weight (descending), rips up existing route, reroutes with updated congestion, checks convergence

`CongestionState` tracks per-cell sharing counts and historical congestion. `DijkBuf` holds reusable Dijkstra buffers (dist, prev, heap).

### Scoring
Four normalized metrics in `[0, 1]`:
- Path efficiency: `optimal_length / actual_length` averaged over routed edges
- Accessibility: fraction of edges successfully routed
- Congestion: `1 / max_overlap` (inverse of worst-case cell sharing)
- Void ratio: penalty for deviation from ~20% empty space

Composite = equal-weight average.

## Rust patterns used

- Newtype indices (`NodeId(u32)`, `EdgeId(u32)`) for type safety
- Flat `Vec<T>` grids with `y * width + x` indexing (cache-friendly, `Send + Sync`)
- SoA layout for hot loops (separate `Vec<f32>` for x, y, radius)
- Buffer reuse via `.clear()` — no per-iteration allocation
- `ChaCha8Rng` for deterministic, reproducible output
- Rayon `par_iter` for parallel utility evaluation on large grids
- `BinaryHeap<Reverse<>>` for min-heap Dijkstra
- `HashMap` for sparse output (positions, paths, route_costs)
- Serde derives on all public types for serialization
- `#[serde(default)]` on new config fields for backward compatibility

## Key references

- McMurchie & Ebeling (1995) — PathFinder negotiation-based routing
- Lee (1961) — BFS maze routing (kept as dead-code fallback)
- Blume (1993) — Logit dynamics / Boltzmann selection in potential games
