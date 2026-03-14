//! # Ogun
//!
//! Spatial layout generation via sequential logit dynamics on potential games.
//!
//! ## Overview
//!
//! Ogun generates 2D spatial layouts at a controllable optimization level.
//! Agents arrive sequentially, select positions via Boltzmann-weighted sampling
//! against a potential function, and commit irrevocably. The inverse temperature
//! β controls output character from near-random to near-optimal.
//!
//! ## Crate structure (planned)
//!
//! - `domain` — spatial grid, boundaries, obstacles
//! - `agent` — agent types, utility functions, arrival ordering
//! - `potential` — potential function definition and evaluation
//! - `placement` — logit choice, segment tree sampling
//! - `routing` — congestion-aware pathfinding (A*, HPA*)
//! - `config` — algorithm parameters (β, kernel, etc.)
