//! # Ogun
//!
//! Spatial layout generation via sequential logit dynamics on potential games.
//!
//! Given a weighted graph and a 2D spatial domain, Ogun places nodes and
//! routes edges at a controllable optimization level. The inverse temperature
//! β governs output character — from near-random (low β) to near-optimal (high β).
//!
//! ## Algorithm
//!
//! ```text
//! for each agent in arrival order:
//!     1. EVAL_UTILITY  — score every valid grid position via potential Φ
//!     2. CHOOSE        — Boltzmann sample: P(p) ∝ exp(β · Φ(p))
//!     3. COMMIT        — place irrevocably, block footprint
//!     4. ROUTE         — BFS (Lee) to already-placed neighbors
//! score the completed layout
//! ```
//!
//! ## Quick start
//!
//! ```rust
//! use ogun::*;
//!
//! let graph = Graph {
//!     nodes: vec![
//!         Node { id: NodeId(0), width: 3, height: 3, fixed: None },
//!         Node { id: NodeId(1), width: 3, height: 3, fixed: None },
//!     ],
//!     edges: vec![Edge {
//!         id: EdgeId(0),
//!         src: NodeId(0),
//!         dst: NodeId(1),
//!         weight: 1.0,
//!     }],
//! };
//! let space = Space { width: 20, height: 20, obstacles: vec![], routing_costs: None };
//! let config = OgunConfig { seed: 42, ..OgunConfig::default() };
//!
//! let layout = generate(&graph, &space, &config);
//! assert!(layout.score.composite > 0.0);
//! ```

mod generate;
mod graph;
mod grid;
mod layout;
mod placement;
mod potential;
mod routing;
mod scoring;
mod types;

// Public API: input types, output type, and the generate function.
pub use generate::generate;
pub use graph::{Edge, Graph, Node, OgunConfig, PotentialKernel, Rect, Space};
pub use grid::Grid;
pub use layout::Layout;
pub use scoring::ScoreBreakdown;
pub use types::{EdgeId, NodeId, Pos};
