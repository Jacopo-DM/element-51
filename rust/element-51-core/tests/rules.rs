//! The edit rules, run on the same cases `tests/test_edits.py` runs.

use element_51_core::{Edit, Graph, NodeType, PortRef, apply_edit};
use serde::Deserialize;

const CASES: &str = include_str!("../../../tests/fixtures/link_cases.json");

#[derive(Deserialize)]
struct Case {
    name: String,
    source: PortRef,
    target: PortRef,
    result: String,
}

#[derive(Deserialize)]
struct LinkCases {
    node_types: Vec<NodeType>,
    graph: Graph,
    cases: Vec<Case>,
}

fn fixture() -> LinkCases {
    serde_json::from_str(CASES).unwrap()
}

#[test]
fn rust_gives_the_verdict_python_gives_on_every_case() {
    let fixture = fixture();
    assert!(fixture.cases.len() >= 10);
    for case in &fixture.cases {
        let edit = Edit::AddLink {
            source: case.source.clone(),
            target: case.target.clone(),
        };
        let outcome = apply_edit(&fixture.graph, &fixture.node_types, &edit);
        match (case.result.as_str(), outcome) {
            ("ok", Ok(graph)) => {
                assert_eq!(
                    graph.links.len(),
                    fixture.graph.links.len() + 1,
                    "{}",
                    case.name
                );
            }
            ("ok", Err(error)) => panic!("{}: expected ok, got {error}", case.name),
            (expected, Ok(_)) => panic!("{}: expected {expected}, got ok", case.name),
            (expected, Err(error)) => assert_eq!(error.kind(), expected, "{}", case.name),
        }
    }
}

#[test]
fn add_node_places_a_node() {
    let fixture = fixture();
    let edit = Edit::AddNode {
        id: "n5".into(),
        node_type: "add".into(),
        x: 1.0,
        y: 2.0,
    };
    let graph = apply_edit(&fixture.graph, &fixture.node_types, &edit).unwrap();
    assert_eq!(graph.nodes.last().unwrap().id, "n5");
    assert_eq!(graph.links, fixture.graph.links);
}

#[test]
fn add_node_of_an_unknown_type_is_refused() {
    let fixture = fixture();
    let edit = Edit::AddNode {
        id: "n5".into(),
        node_type: "nope".into(),
        x: 0.0,
        y: 0.0,
    };
    let error = apply_edit(&fixture.graph, &fixture.node_types, &edit).unwrap_err();
    assert_eq!(error.kind(), "unknown_node_type");
}

#[test]
fn add_node_with_a_used_id_is_refused() {
    let fixture = fixture();
    let edit = Edit::AddNode {
        id: "n1".into(),
        node_type: "add".into(),
        x: 0.0,
        y: 0.0,
    };
    let error = apply_edit(&fixture.graph, &fixture.node_types, &edit).unwrap_err();
    assert_eq!(error.kind(), "id_in_use");
}

#[test]
fn next_id_is_one_above_the_highest_in_use() {
    let fixture = fixture();
    assert_eq!(fixture.graph.next_id(), "n5");
    let empty = Graph {
        nodes: vec![],
        links: vec![],
    };
    assert_eq!(empty.next_id(), "n1");
}
