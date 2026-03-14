//! Input types: the graph to lay out, the spatial domain, and algorithm config.

use crate::types::{EdgeId, NodeId, Pos};
use serde::{Deserialize, Serialize};

/// A node (agent) to be placed in the layout.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Node {
    pub id: NodeId,
    /// Footprint radius in grid cells (simplified to a circle).
    pub radius: u32,
}

/// A weighted edge connecting two nodes.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Edge {
    pub id: EdgeId,
    pub src: NodeId,
    pub dst: NodeId,
    /// Connection importance — higher weight = stronger attraction.
    pub weight: f32,
}

/// The graph of nodes and edges to lay out.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Graph {
    pub nodes: Vec<Node>,
    pub edges: Vec<Edge>,
}

impl Graph {
    /// Edges incident to a given node.
    pub fn edges_of(&self, node: NodeId) -> impl Iterator<Item = &Edge> {
        self.edges
            .iter()
            .filter(move |e| e.src == node || e.dst == node)
    }

    /// The other endpoint of an edge relative to `node`.
    pub fn neighbor(&self, edge: &Edge, node: NodeId) -> NodeId {
        if edge.src == node {
            edge.dst
        } else {
            edge.src
        }
    }

    /// Pre-built adjacency list: `adj[node.0]` = vec of `(EdgeId, neighbor_id, weight)`.
    /// Preserves edge iteration order for determinism.
    pub fn adjacency(&self) -> Vec<Vec<(EdgeId, NodeId, f32)>> {
        let mut adj = vec![Vec::new(); self.nodes.len()];
        for e in &self.edges {
            adj[e.src.0 as usize].push((e.id, e.dst, e.weight));
            adj[e.dst.0 as usize].push((e.id, e.src, e.weight));
        }
        adj
    }
}

/// An axis-aligned rectangle used for obstacles and keep-out zones.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Rect {
    pub x: u32,
    pub y: u32,
    pub w: u32,
    pub h: u32,
}

impl Rect {
    pub fn contains(&self, pos: Pos) -> bool {
        pos.x >= self.x
            && pos.x < self.x + self.w
            && pos.y >= self.y
            && pos.y < self.y + self.h
    }
}

/// The 2D spatial domain: boundary dimensions and obstacle list.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Space {
    pub width: u32,
    pub height: u32,
    pub obstacles: Vec<Rect>,
}

impl Space {
    /// True if the position is inside an obstacle.
    pub fn is_obstacle(&self, pos: Pos) -> bool {
        self.obstacles.iter().any(|r| r.contains(pos))
    }
}

/// Algorithm parameters.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct OgunConfig {
    /// Inverse temperature. Higher β → more deterministic (greedy).
    /// Lower β → more random (exploratory).
    pub beta: f32,
    /// RNG seed for deterministic output.
    pub seed: u64,
    /// Repulsion strength between all node pairs.
    pub repulsion_k: f32,
}

impl Default for OgunConfig {
    fn default() -> Self {
        Self {
            beta: 2.0,
            seed: 0,
            repulsion_k: 50.0,
        }
    }
}
