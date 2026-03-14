use ogun::*;

/// Build a small test graph: 4 nodes in a diamond pattern with edges.
fn test_graph() -> Graph {
    Graph {
        nodes: vec![
            Node { id: NodeId(0), radius: 1 },
            Node { id: NodeId(1), radius: 1 },
            Node { id: NodeId(2), radius: 1 },
            Node { id: NodeId(3), radius: 1 },
        ],
        edges: vec![
            Edge { id: EdgeId(0), src: NodeId(0), dst: NodeId(1), weight: 1.0 },
            Edge { id: EdgeId(1), src: NodeId(1), dst: NodeId(2), weight: 1.0 },
            Edge { id: EdgeId(2), src: NodeId(2), dst: NodeId(3), weight: 1.0 },
            Edge { id: EdgeId(3), src: NodeId(3), dst: NodeId(0), weight: 0.5 },
        ],
    }
}

fn test_space() -> Space {
    Space {
        width: 30,
        height: 30,
        obstacles: vec![],
    }
}

#[test]
fn deterministic_output() {
    let graph = test_graph();
    let space = test_space();
    let config = OgunConfig { seed: 42, ..OgunConfig::default() };

    let a = generate(&graph, &space, &config);
    let b = generate(&graph, &space, &config);

    // Same seed → identical positions.
    assert_eq!(a.positions, b.positions);
    // Same seed → identical paths.
    assert_eq!(a.paths, b.paths);
    // Same seed → identical score.
    assert_eq!(a.score, b.score);
}

#[test]
fn different_seeds_differ() {
    let graph = test_graph();
    let space = test_space();

    let a = generate(&graph, &space, &OgunConfig { seed: 1, ..OgunConfig::default() });
    let b = generate(&graph, &space, &OgunConfig { seed: 2, ..OgunConfig::default() });

    // Different seeds should (almost certainly) produce different layouts.
    assert_ne!(a.positions, b.positions);
}

#[test]
fn all_nodes_placed() {
    let graph = test_graph();
    let space = test_space();
    let config = OgunConfig::default();

    let layout = generate(&graph, &space, &config);
    assert_eq!(layout.positions.len(), graph.nodes.len());
}

#[test]
fn score_in_range() {
    let graph = test_graph();
    let space = test_space();
    let config = OgunConfig::default();

    let layout = generate(&graph, &space, &config);
    assert!(layout.score >= 0.0 && layout.score <= 1.0, "score={}", layout.score);
}

#[test]
fn positions_within_bounds() {
    let graph = test_graph();
    let space = test_space();
    let config = OgunConfig::default();

    let layout = generate(&graph, &space, &config);
    for &pos in layout.positions.values() {
        assert!(pos.x < space.width, "x={} >= width={}", pos.x, space.width);
        assert!(pos.y < space.height, "y={} >= height={}", pos.y, space.height);
    }
}

#[test]
fn obstacles_respected() {
    let graph = Graph {
        nodes: vec![
            Node { id: NodeId(0), radius: 0 },
            Node { id: NodeId(1), radius: 0 },
        ],
        edges: vec![],
    };
    // Fill most of the grid with an obstacle, leaving only a narrow strip.
    let space = Space {
        width: 20,
        height: 20,
        obstacles: vec![Rect { x: 0, y: 0, w: 20, h: 18 }],
    };
    let config = OgunConfig { beta: 0.0, seed: 99, ..OgunConfig::default() };

    let layout = generate(&graph, &space, &config);
    for &pos in layout.positions.values() {
        assert!(!space.is_obstacle(pos), "node placed on obstacle at {:?}", pos);
    }
}
