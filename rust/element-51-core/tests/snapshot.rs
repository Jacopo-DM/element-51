//! The Python side writes `tests/fixtures/snapshot.json`; this test checks the
//! Rust side reads the same file the same way.

use element_51_core::{Message, PortType};

const FIXTURE: &str = include_str!("../../../tests/fixtures/snapshot.json");

#[test]
fn shared_fixture_parses() {
    let Message::Snapshot(snapshot) = Message::from_json(FIXTURE).unwrap() else {
        panic!("not a snapshot");
    };
    assert_eq!(snapshot.version, 2);
    assert_eq!(snapshot.theme, "antimony-light");
    assert_eq!(snapshot.theme_changes["fonts"]["value"], "14px Menlo");
    let names: Vec<&str> = snapshot
        .node_types
        .iter()
        .map(|t| t.name.as_str())
        .collect();
    assert_eq!(names, ["add", "show", "picture", "Plot", "train"]);
    let jobs: Vec<bool> = snapshot.node_types.iter().map(|t| t.job).collect();
    assert_eq!(jobs, [false, false, false, false, true]);
    let add = &snapshot.node_types[0];
    assert_eq!(add.inputs.len(), 2);
    assert_eq!(add.inputs[0].kind, PortType::Float);
    assert_eq!(add.outputs[0].name, "out");
    assert_eq!(snapshot.graph.nodes.len(), 2);
    assert_eq!(snapshot.graph.nodes[1].node_type, "show");
    let link = &snapshot.graph.links[0];
    assert_eq!(
        (link.source.node.as_str(), link.source.port.as_str()),
        ("n1", "out")
    );
    assert_eq!(
        (link.target.node.as_str(), link.target.port.as_str()),
        ("n2", "value")
    );
}

#[test]
fn fixture_round_trips_through_serde() {
    let message = Message::from_json(FIXTURE).unwrap();
    let again = Message::from_json(&serde_json::to_string(&message).unwrap()).unwrap();
    assert_eq!(message, again);
}

#[test]
fn unknown_op_is_refused() {
    assert!(Message::from_json(r#"{"op": "explode"}"#).is_err());
}

#[test]
fn unknown_field_is_refused() {
    let broken = FIXTURE.replacen("\"version\": 2,", "\"version\": 2, \"extra\": true,", 1);
    assert!(Message::from_json(&broken).is_err());
}

#[test]
fn other_protocol_version_is_refused() {
    let old = FIXTURE.replacen("\"version\": 2", "\"version\": 1", 1);
    let error = Message::from_json(&old).unwrap_err();
    assert!(error.to_string().contains("version 1"), "{error}");
}

#[test]
fn snapshot_finds_a_node_type_by_name() {
    let Message::Snapshot(snapshot) = Message::from_json(FIXTURE).unwrap() else {
        panic!("not a snapshot");
    };
    assert!(snapshot.node_type("show").is_some());
    assert!(snapshot.node_type("nope").is_none());
}
