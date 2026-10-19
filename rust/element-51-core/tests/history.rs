//! Undo and redo: every edit has a way back, and the stacks keep their order.

use element_51_core::history::History;
use element_51_core::{Edit, Graph, NodeType, PortRef, apply_edit};
use serde::Deserialize;
use serde_json::json;

const CASES: &str = include_str!("../../../tests/fixtures/edit_cases.json");

#[derive(Deserialize)]
struct Case {
    name: String,
    edit: Edit,
    result: String,
}

#[derive(Deserialize)]
struct Fixture {
    node_types: Vec<NodeType>,
    graph: Graph,
    cases: Vec<Case>,
}

fn fixture() -> Fixture {
    serde_json::from_str(CASES).unwrap()
}

/// The graph with nodes and links in a fixed order, so two graphs that hold the
/// same things compare equal. Undoing a delete puts a node back at the end of
/// the list, which changes nothing a person sees except which node is drawn on
/// top of which.
fn sorted(graph: &Graph) -> Graph {
    let mut graph = graph.clone();
    graph.nodes.sort_by(|a, b| a.id.cmp(&b.id));
    graph.links.sort_by_key(|link| {
        format!(
            "{}.{}>{}.{}",
            link.source.node, link.source.port, link.target.node, link.target.port
        )
    });
    graph
}

fn apply_all(graph: &Graph, types: &[NodeType], edits: &[Edit]) -> Graph {
    let mut graph = graph.clone();
    for edit in edits {
        graph = apply_edit(&graph, types, edit).unwrap_or_else(|error| panic!("{edit:?}: {error}"));
    }
    graph
}

#[test]
fn every_accepted_edit_can_be_undone_and_redone() {
    let fixture = fixture();
    let mut tried = 0;
    for case in fixture.cases.iter().filter(|case| case.result == "ok") {
        let mut history = History::default();
        let before = &fixture.graph;
        let after = apply_edit(before, &fixture.node_types, &case.edit).unwrap();
        history.record(before, &case.edit);
        let backward = history.undo().expect("a step to undo");
        let restored = apply_all(&after, &fixture.node_types, &backward);
        assert_eq!(sorted(&restored), sorted(before), "undo: {}", case.name);
        let forward = history.redo().expect("a step to redo");
        let again = apply_all(&restored, &fixture.node_types, &forward);
        assert_eq!(sorted(&again), sorted(&after), "redo: {}", case.name);
        tried += 1;
    }
    assert!(tried >= 8, "only {tried} cases tried");
}

#[test]
fn adding_a_node_and_a_link_can_be_undone() {
    let fixture = fixture();
    let types = &fixture.node_types;
    let add_node = Edit::AddNode {
        id: "n9".into(),
        node_type: "add".into(),
        x: 5.0,
        y: 6.0,
    };
    let with_node = apply_edit(&fixture.graph, types, &add_node).unwrap();
    let add_link = Edit::AddLink {
        source: PortRef {
            node: "n3".into(),
            port: "out".into(),
        },
        target: PortRef {
            node: "n9".into(),
            port: "a".into(),
        },
    };
    let mut history = History::default();
    history.record(&fixture.graph, &add_node);
    history.record(&with_node, &add_link);
    let with_link = apply_edit(&with_node, types, &add_link).unwrap();
    let step = history.undo().unwrap();
    assert_eq!(
        sorted(&apply_all(&with_link, types, &step)),
        sorted(&with_node)
    );
    let step = history.undo().unwrap();
    assert_eq!(
        sorted(&apply_all(&with_node, types, &step)),
        sorted(&fixture.graph)
    );
}

#[test]
fn deleting_a_node_brings_back_its_links_and_values() {
    let fixture = fixture();
    let types = &fixture.node_types;
    let with_value = Edit::SetValue {
        node: "n2".into(),
        port: "b".into(),
        value: Some(json!(7.5)),
    };
    let valued = apply_edit(&fixture.graph, types, &with_value).unwrap();
    let delete = Edit::DeleteNode { id: "n2".into() };
    let gone = apply_edit(&valued, types, &delete).unwrap();
    assert_eq!(gone.links.len(), 1);
    let mut history = History::default();
    history.record(&valued, &delete);
    let back = apply_all(&gone, types, &history.undo().unwrap());
    assert_eq!(sorted(&back), sorted(&valued));
}

#[test]
fn a_new_edit_clears_what_could_be_redone() {
    let fixture = fixture();
    let mut history = History::default();
    let first = Edit::MoveNode {
        id: "n1".into(),
        x: 1.0,
        y: 1.0,
    };
    let second = Edit::MoveNode {
        id: "n1".into(),
        x: 2.0,
        y: 2.0,
    };
    history.record(&fixture.graph, &first);
    history.undo().unwrap();
    assert!(history.can_redo());
    history.record(&fixture.graph, &second);
    assert!(!history.can_redo());
    assert!(history.redo().is_none());
}

#[test]
fn steps_come_back_newest_first_and_the_stacks_run_dry() {
    let fixture = fixture();
    let mut history = History::default();
    assert!(!history.can_undo());
    assert!(history.undo().is_none());
    let to_one = Edit::MoveNode {
        id: "n1".into(),
        x: 1.0,
        y: 1.0,
    };
    let to_two = Edit::MoveNode {
        id: "n1".into(),
        x: 2.0,
        y: 2.0,
    };
    let at_one = apply_edit(&fixture.graph, &fixture.node_types, &to_one).unwrap();
    history.record(&fixture.graph, &to_one);
    history.record(&at_one, &to_two);
    assert!(history.can_undo());
    // The newest step goes back to where n1 was after the first move.
    let newest = history.undo().unwrap();
    assert_eq!(
        newest,
        vec![Edit::MoveNode {
            id: "n1".into(),
            x: 1.0,
            y: 1.0
        }]
    );
    history.undo().unwrap();
    assert!(history.undo().is_none());
    // Redo replays them oldest first.
    assert_eq!(history.redo().unwrap(), vec![to_one]);
    assert_eq!(history.redo().unwrap(), vec![to_two]);
    assert!(history.redo().is_none());
}
