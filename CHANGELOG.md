# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.2.0] - 2026-03-15

### Added

- PathFinder negotiation routing — congestion-aware Dijkstra (Phase 1) with
  rip-up-and-reroute (Phase 2), replacing uniform-cost Lee BFS
- `ScoreBreakdown` — per-metric scores (path efficiency, accessibility,
  congestion, void ratio) instead of a single composite float
- Per-pair repulsion config — `repulsion_pairs: HashMap<(NodeId, NodeId), f32>`
  on `OgunConfig` for category-specific repulsion multipliers
- Per-node metrics on `Layout`: `node_accessibility` (fraction of edges routed)
  and `node_congestion` (average route cost)
- Routing metadata on `Layout`: `route_costs` (per-edge) and `congestion_grid`
  (per-cell overlap count)
- `unplaced: Vec<NodeId>` on `Layout` — reports nodes that couldn't be placed
- `negotiation_iterations` and `history_increment` on `OgunConfig`

### Changed

- `Layout.score` is now `ScoreBreakdown` (was `f32`). Use `.score.composite`
  for the previous behavior.
- Routing uses Dijkstra with congestion costs instead of uniform BFS. Routes
  compete for space, producing emergent road hierarchy.

## [0.1.0] - 2026-03-14

Initial release.

### Added

- `generate()` — sequential logit dynamics placement with Boltzmann sampling
- Potential function: overlap penalty, inverse-square repulsion, edge attraction
- Lee algorithm (BFS) maze routing for edge paths
- Composite layout scoring (path efficiency, accessibility, congestion, void ratio)
- Configurable inverse temperature β for optimization level control
- Deterministic output via seeded ChaCha8 RNG
- Obstacle and keep-out zone support
- Rayon parallel utility evaluation (adaptive threshold)
- Serde serialization for all public types

[0.2.0]: https://github.com/EliasVahlberg/ogun/releases/tag/v0.2.0
[0.1.0]: https://github.com/EliasVahlberg/ogun/releases/tag/v0.1.0
