# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

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

[0.1.0]: https://github.com/EliasVahlberg/ogun/releases/tag/v0.1.0
