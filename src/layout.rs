//! Output type: the generated layout.

use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use crate::scoring::ScoreBreakdown;
use crate::types::{EdgeId, NodeId, Pos};

/// The result of running the Ogun algorithm.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Layout {
    /// Final position of each placed node.
    pub positions: HashMap<NodeId, Pos>,
    /// Routed path for each edge (sequence of grid cells).
    /// Missing entries mean the edge could not be routed.
    pub paths: HashMap<EdgeId, Vec<Pos>>,
    /// Per-metric score breakdown.
    pub score: ScoreBreakdown,
    /// Nodes that could not be placed (no valid candidate positions).
    pub unplaced: Vec<NodeId>,
    /// Per-edge routing cost (sum of congestion-weighted cell costs along path).
    pub route_costs: HashMap<EdgeId, f32>,
    /// Per-cell route overlap count (number of routes using each cell).
    /// Flat vec of size `width * height`, row-major order.
    pub congestion_grid: Vec<u32>,
    /// Per-node accessibility: fraction of incident edges that were routed.
    pub node_accessibility: HashMap<NodeId, f32>,
    /// Per-node congestion: average route cost across this node's routed edges.
    pub node_congestion: HashMap<NodeId, f32>,
}
