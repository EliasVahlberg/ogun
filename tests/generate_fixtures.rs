//! Generates baseline JSON fixtures for regression testing.
//!
//! Run with: cargo test --test generate_fixtures -- --ignored
//!
//! This writes tests/fixtures/*.json. Commit these files — they are the
//! regression baseline. Future runs of the regression test compare against them.

use ogun::*;
use serde::{Deserialize, Serialize};
use std::fs;

#[derive(Serialize, Deserialize)]
struct Fixture {
    name: String,
    description: String,
    input: FixtureInput,
    expected: Layout,
}

#[derive(Serialize, Deserialize)]
struct FixtureInput {
    graph: Graph,
    space: Space,
    config: OgunConfig,
}

fn scenarios() -> Vec<(String, String, Graph, Space, OgunConfig)> {
    vec![
        (
            "trivial".into(),
            "2 nodes, 1 edge, 20x20 empty grid — sanity check".into(),
            Graph {
                nodes: vec![
                    Node {
                        id: NodeId(0),
                        radius: 1,
                    },
                    Node {
                        id: NodeId(1),
                        radius: 1,
                    },
                ],
                edges: vec![Edge {
                    id: EdgeId(0),
                    src: NodeId(0),
                    dst: NodeId(1),
                    weight: 1.0,
                }],
            },
            Space {
                width: 20,
                height: 20,
                obstacles: vec![],
            },
            OgunConfig {
                beta: 2.0,
                seed: 42,
                repulsion_k: 50.0,
                ..Default::default()
            },
        ),
        (
            "small_cluster".into(),
            "5 nodes, 6 edges, 30x30 grid — basic clustering behavior".into(),
            Graph {
                nodes: vec![
                    Node {
                        id: NodeId(0),
                        radius: 1,
                    },
                    Node {
                        id: NodeId(1),
                        radius: 1,
                    },
                    Node {
                        id: NodeId(2),
                        radius: 1,
                    },
                    Node {
                        id: NodeId(3),
                        radius: 1,
                    },
                    Node {
                        id: NodeId(4),
                        radius: 2,
                    },
                ],
                edges: vec![
                    Edge {
                        id: EdgeId(0),
                        src: NodeId(0),
                        dst: NodeId(1),
                        weight: 1.0,
                    },
                    Edge {
                        id: EdgeId(1),
                        src: NodeId(1),
                        dst: NodeId(2),
                        weight: 1.0,
                    },
                    Edge {
                        id: EdgeId(2),
                        src: NodeId(2),
                        dst: NodeId(3),
                        weight: 1.0,
                    },
                    Edge {
                        id: EdgeId(3),
                        src: NodeId(3),
                        dst: NodeId(0),
                        weight: 0.5,
                    },
                    Edge {
                        id: EdgeId(4),
                        src: NodeId(4),
                        dst: NodeId(0),
                        weight: 2.0,
                    },
                    Edge {
                        id: EdgeId(5),
                        src: NodeId(4),
                        dst: NodeId(2),
                        weight: 2.0,
                    },
                ],
            },
            Space {
                width: 30,
                height: 30,
                obstacles: vec![],
            },
            OgunConfig {
                beta: 2.0,
                seed: 100,
                repulsion_k: 50.0,
                ..Default::default()
            },
        ),
        (
            "with_obstacles".into(),
            "3 nodes, 2 edges, 25x25 grid with obstacle wall — tests routing around obstacles"
                .into(),
            Graph {
                nodes: vec![
                    Node {
                        id: NodeId(0),
                        radius: 1,
                    },
                    Node {
                        id: NodeId(1),
                        radius: 1,
                    },
                    Node {
                        id: NodeId(2),
                        radius: 1,
                    },
                ],
                edges: vec![
                    Edge {
                        id: EdgeId(0),
                        src: NodeId(0),
                        dst: NodeId(1),
                        weight: 1.0,
                    },
                    Edge {
                        id: EdgeId(1),
                        src: NodeId(1),
                        dst: NodeId(2),
                        weight: 1.0,
                    },
                ],
            },
            Space {
                width: 25,
                height: 25,
                obstacles: vec![
                    Rect {
                        x: 10,
                        y: 0,
                        w: 2,
                        h: 20,
                    }, // vertical wall with gap at bottom
                ],
            },
            OgunConfig {
                beta: 3.0,
                seed: 77,
                repulsion_k: 50.0,
                ..Default::default()
            },
        ),
        (
            "linear_chain".into(),
            "6 nodes in a chain, 40x20 grid — tests sequential attraction along a line".into(),
            Graph {
                nodes: (0..6)
                    .map(|i| Node {
                        id: NodeId(i),
                        radius: 1,
                    })
                    .collect(),
                edges: (0..5)
                    .map(|i| Edge {
                        id: EdgeId(i),
                        src: NodeId(i),
                        dst: NodeId(i + 1),
                        weight: 1.5,
                    })
                    .collect(),
            },
            Space {
                width: 40,
                height: 20,
                obstacles: vec![],
            },
            OgunConfig {
                beta: 2.0,
                seed: 55,
                repulsion_k: 30.0,
                ..Default::default()
            },
        ),
        (
            "high_beta".into(),
            "4 nodes, 4 edges, β=10 — near-greedy deterministic placement".into(),
            Graph {
                nodes: vec![
                    Node {
                        id: NodeId(0),
                        radius: 1,
                    },
                    Node {
                        id: NodeId(1),
                        radius: 1,
                    },
                    Node {
                        id: NodeId(2),
                        radius: 1,
                    },
                    Node {
                        id: NodeId(3),
                        radius: 1,
                    },
                ],
                edges: vec![
                    Edge {
                        id: EdgeId(0),
                        src: NodeId(0),
                        dst: NodeId(1),
                        weight: 1.0,
                    },
                    Edge {
                        id: EdgeId(1),
                        src: NodeId(1),
                        dst: NodeId(2),
                        weight: 1.0,
                    },
                    Edge {
                        id: EdgeId(2),
                        src: NodeId(2),
                        dst: NodeId(3),
                        weight: 1.0,
                    },
                    Edge {
                        id: EdgeId(3),
                        src: NodeId(3),
                        dst: NodeId(0),
                        weight: 1.0,
                    },
                ],
            },
            Space {
                width: 30,
                height: 30,
                obstacles: vec![],
            },
            OgunConfig {
                beta: 10.0,
                seed: 42,
                repulsion_k: 50.0,
                ..Default::default()
            },
        ),
        (
            "low_beta".into(),
            "4 nodes, 4 edges, β=0.1 — near-random exploratory placement".into(),
            Graph {
                nodes: vec![
                    Node {
                        id: NodeId(0),
                        radius: 1,
                    },
                    Node {
                        id: NodeId(1),
                        radius: 1,
                    },
                    Node {
                        id: NodeId(2),
                        radius: 1,
                    },
                    Node {
                        id: NodeId(3),
                        radius: 1,
                    },
                ],
                edges: vec![
                    Edge {
                        id: EdgeId(0),
                        src: NodeId(0),
                        dst: NodeId(1),
                        weight: 1.0,
                    },
                    Edge {
                        id: EdgeId(1),
                        src: NodeId(1),
                        dst: NodeId(2),
                        weight: 1.0,
                    },
                    Edge {
                        id: EdgeId(2),
                        src: NodeId(2),
                        dst: NodeId(3),
                        weight: 1.0,
                    },
                    Edge {
                        id: EdgeId(3),
                        src: NodeId(3),
                        dst: NodeId(0),
                        weight: 1.0,
                    },
                ],
            },
            Space {
                width: 30,
                height: 30,
                obstacles: vec![],
            },
            OgunConfig {
                beta: 0.1,
                seed: 42,
                repulsion_k: 50.0,
                ..Default::default()
            },
        ),
    ]
}

#[test]
#[ignore] // Run manually: cargo test --test generate_fixtures -- --ignored
fn generate_fixtures() {
    let dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures");
    fs::create_dir_all(&dir).unwrap();

    for (name, description, graph, space, config) in scenarios() {
        let layout = generate(&graph, &space, &config);

        let fixture = Fixture {
            name: name.clone(),
            description,
            input: FixtureInput {
                graph,
                space,
                config,
            },
            expected: layout,
        };

        let json = serde_json::to_string_pretty(&fixture).unwrap();
        let path = dir.join(format!("{name}.json"));
        fs::write(&path, &json).unwrap();
        println!("wrote {path:?} ({} bytes)", json.len());
    }
}
