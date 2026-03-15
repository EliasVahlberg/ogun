//! Regression tests: load JSON fixtures and verify generate() produces
//! identical output. Any behavioral change in the algorithm will fail these.
//!
//! To update baselines after intentional changes:
//!   cargo test --test generate_fixtures -- --ignored

use ogun::*;
use serde::Deserialize;

#[derive(Deserialize)]
struct Fixture {
    #[allow(dead_code)]
    name: String,
    input: FixtureInput,
    expected: Layout,
}

#[derive(Deserialize)]
struct FixtureInput {
    graph: Graph,
    space: Space,
    config: OgunConfig,
}

fn load_fixture(name: &str) -> Fixture {
    let path = format!("{}/tests/fixtures/{name}.json", env!("CARGO_MANIFEST_DIR"));
    let json =
        std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("failed to read {path}: {e}"));
    serde_json::from_str(&json).unwrap_or_else(|e| panic!("failed to parse {path}: {e}"))
}

fn check_fixture(name: &str) {
    let fixture = load_fixture(name);
    let actual = generate(
        &fixture.input.graph,
        &fixture.input.space,
        &fixture.input.config,
    );

    assert_eq!(
        actual.positions, fixture.expected.positions,
        "[{name}] positions differ"
    );
    assert_eq!(
        actual.paths, fixture.expected.paths,
        "[{name}] paths differ"
    );
    assert_eq!(
        actual.score, fixture.expected.score,
        "[{name}] score differs: got {:?} expected {:?}",
        actual.score, fixture.expected.score
    );
}

#[test]
fn regression_trivial() {
    check_fixture("trivial");
}

#[test]
fn regression_small_cluster() {
    check_fixture("small_cluster");
}

#[test]
fn regression_with_obstacles() {
    check_fixture("with_obstacles");
}

#[test]
fn regression_linear_chain() {
    check_fixture("linear_chain");
}

#[test]
fn regression_high_beta() {
    check_fixture("high_beta");
}

#[test]
fn regression_low_beta() {
    check_fixture("low_beta");
}
