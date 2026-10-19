//! The browser's copy of the graph: optimistic edits, acks and rollback.

use element_51_core::{Edit, Graph, NodeType, PortRef, Replica, Snapshot};
use serde::Deserialize;

const CASES: &str = include_str!("../../../tests/fixtures/link_cases.json");

#[derive(Deserialize)]
struct Fixture {
    node_types: Vec<NodeType>,
}

fn replica() -> Replica {
    let fixture: Fixture = serde_json::from_str(CASES).unwrap();
    Replica::new(Snapshot {
        version: Snapshot::VERSION,
        node_types: fixture.node_types,
        graph: Graph {
            nodes: vec![],
            links: vec![],
        },
    })
}

fn add_node(id: &str) -> Edit {
    Edit::AddNode {
        id: id.into(),
        node_type: "add".into(),
        x: 0.0,
        y: 0.0,
    }
}

fn link(from: &str, to: &str) -> Edit {
    Edit::AddLink {
        source: PortRef {
            node: from.into(),
            port: "out".into(),
        },
        target: PortRef {
            node: to.into(),
            port: "a".into(),
        },
    }
}

fn node_ids(replica: &Replica) -> Vec<&str> {
    replica
        .graph()
        .nodes
        .iter()
        .map(|node| node.id.as_str())
        .collect()
}

#[test]
fn an_edit_applies_at_once_and_gets_a_request_number() {
    let mut replica = replica();
    assert_eq!(replica.submit(add_node("n1")).unwrap().id, 1);
    assert_eq!(replica.submit(add_node("n2")).unwrap().id, 2);
    assert_eq!(node_ids(&replica), ["n1", "n2"]);
    assert_eq!(replica.pending_len(), 2);
}

#[test]
fn a_refused_edit_is_not_sent_and_uses_no_number() {
    let mut replica = replica();
    let error = replica.submit(link("n1", "n2")).unwrap_err();
    assert_eq!(error.kind(), "unknown_node");
    assert_eq!(replica.pending_len(), 0);
    assert_eq!(replica.submit(add_node("n1")).unwrap().id, 1);
}

#[test]
fn an_ack_keeps_the_edit_and_drops_it_from_pending() {
    let mut replica = replica();
    replica.submit(add_node("n1")).unwrap();
    assert!(replica.ack(1));
    assert_eq!(replica.pending_len(), 0);
    assert_eq!(node_ids(&replica), ["n1"]);
    assert!(!replica.ack(99), "an unknown id is ignored");
}

#[test]
fn a_reject_rolls_back_the_edit_and_those_that_depended_on_it() {
    let mut replica = replica();
    replica.submit(add_node("n1")).unwrap();
    replica.submit(add_node("n2")).unwrap();
    replica.submit(link("n1", "n2")).unwrap();
    assert!(replica.reject(1));
    assert_eq!(node_ids(&replica), ["n2"]);
    assert!(replica.graph().links.is_empty());
    assert_eq!(replica.pending_len(), 1);
    assert!(!replica.reject(3), "the dependent edit was already dropped");
    assert!(replica.ack(2));
    assert_eq!(node_ids(&replica), ["n2"]);
}

#[test]
fn a_reject_keeps_what_was_already_acknowledged() {
    let mut replica = replica();
    replica.submit(add_node("n1")).unwrap();
    assert!(replica.ack(1));
    replica.submit(add_node("n2")).unwrap();
    replica.submit(link("n1", "n2")).unwrap();
    assert!(replica.reject(2));
    assert_eq!(node_ids(&replica), ["n1"]);
    assert!(replica.graph().links.is_empty());
}

#[test]
fn a_new_snapshot_replaces_the_confirmed_graph() {
    let mut replica = replica();
    replica.reset(snapshot_with(&["n7"]));
    assert_eq!(node_ids(&replica), ["n7"]);
    assert_eq!(replica.pending_len(), 0);
}

fn snapshot_with(ids: &[&str]) -> Snapshot {
    let fixture: Fixture = serde_json::from_str(CASES).unwrap();
    let nodes = ids
        .iter()
        .map(|id| element_51_core::PlacedNode {
            id: (*id).into(),
            node_type: "add".into(),
            x: 5.0,
            y: 6.0,
            values: Default::default(),
        })
        .collect();
    Snapshot {
        version: Snapshot::VERSION,
        node_types: fixture.node_types,
        graph: Graph {
            nodes,
            links: vec![],
        },
    }
}

#[test]
fn a_new_snapshot_keeps_the_pending_edits_that_still_apply() {
    let mut replica = replica();
    replica.submit(add_node("n2")).unwrap();
    replica.reset(snapshot_with(&["n1"]));
    assert_eq!(node_ids(&replica), ["n1", "n2"]);
    assert_eq!(replica.pending_len(), 1);
    assert!(replica.ack(1));
    assert_eq!(node_ids(&replica), ["n1", "n2"]);
}

#[test]
fn a_new_snapshot_drops_a_pending_edit_that_no_longer_applies() {
    let mut replica = replica();
    replica.submit(add_node("n1")).unwrap();
    replica.reset(snapshot_with(&["n1"]));
    assert_eq!(node_ids(&replica), ["n1"]);
    assert_eq!(replica.pending_len(), 0);
}

#[test]
fn the_next_node_id_follows_the_local_graph() {
    let mut replica = replica();
    assert_eq!(replica.next_node_id(), "n1");
    replica.submit(add_node("n1")).unwrap();
    assert_eq!(replica.next_node_id(), "n2");
    assert!(replica.reject(1));
    assert_eq!(replica.next_node_id(), "n1");
}
