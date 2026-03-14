//! Output type: the generated layout.

use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use crate::types::{EdgeId, NodeId, Pos};

/// The result of running the Ogun algorithm.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Layout {
    /// Final position of each placed node.
    pub positions: HashMap<NodeId, Pos>,
    /// Routed path for each edge (sequence of grid cells).
    /// Missing entries mean the edge could not be routed.
    pub paths: HashMap<EdgeId, Vec<Pos>>,
    /// Composite optimization score in [0, 1].
    pub score: f32,
}
