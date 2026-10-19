//! Size and position of a node box.

use element_51_core::layout::{
    HEADER_HEIGHT, NODE_WIDTH, PORT_RADIUS, ROW_HEIGHT, Side, anchor_of, input_anchor, node_rect,
    output_anchor, port_at,
};
use element_51_core::{Graph, NodeType, PlacedNode, Port, PortRef, PortType};

fn port(name: &str) -> Port {
    Port {
        name: name.to_string(),
        kind: PortType::Float,
    }
}

fn node_at(x: f64, y: f64) -> PlacedNode {
    PlacedNode {
        id: "n1".to_string(),
        node_type: "t".to_string(),
        x,
        y,
    }
}

fn node_type(inputs: &[&str], outputs: &[&str]) -> NodeType {
    NodeType {
        name: "t".to_string(),
        inputs: inputs.iter().map(|n| port(n)).collect(),
        outputs: outputs.iter().map(|n| port(n)).collect(),
    }
}

#[test]
fn box_starts_at_the_node_position() {
    let rect = node_rect(&node_at(40.0, 60.0), &node_type(&["a"], &[]));
    assert_eq!((rect.x, rect.y, rect.w), (40.0, 60.0, NODE_WIDTH));
}

#[test]
fn height_is_the_header_plus_one_row_per_port_on_the_busier_side() {
    let two_in_one_out = node_rect(&node_at(0.0, 0.0), &node_type(&["a", "b"], &["out"]));
    assert_eq!(two_in_one_out.h, HEADER_HEIGHT + 2.0 * ROW_HEIGHT);
    let one_out_only = node_rect(&node_at(0.0, 0.0), &node_type(&[], &["out"]));
    assert_eq!(one_out_only.h, HEADER_HEIGHT + ROW_HEIGHT);
}

#[test]
fn a_node_without_ports_is_just_a_header() {
    let rect = node_rect(&node_at(0.0, 0.0), &node_type(&[], &[]));
    assert_eq!(rect.h, HEADER_HEIGHT);
}

fn graph_of(node: PlacedNode) -> Graph {
    Graph {
        nodes: vec![node],
        links: vec![],
    }
}

#[test]
fn anchors_sit_on_the_box_edges_at_the_row_centre() {
    let rect = node_rect(&node_at(40.0, 60.0), &node_type(&["a", "b"], &["out"]));
    let row_y = 60.0 + HEADER_HEIGHT + ROW_HEIGHT * 1.5;
    assert_eq!(input_anchor(&rect, 1), (40.0, row_y));
    assert_eq!(output_anchor(&rect, 1), (40.0 + NODE_WIDTH, row_y));
}

#[test]
fn a_press_on_an_input_anchor_hits_that_port() {
    let types = [node_type(&["a", "b"], &["out"])];
    let graph = graph_of(node_at(40.0, 60.0));
    let rect = node_rect(&graph.nodes[0], &types[0]);
    let (x, y) = input_anchor(&rect, 1);
    let hit = port_at(&graph, &types, x + 3.0, y - 3.0).unwrap();
    assert_eq!(
        (hit.node.as_str(), hit.port.as_str(), hit.side),
        ("n1", "b", Side::Input)
    );
}

#[test]
fn a_press_on_an_output_anchor_hits_that_port() {
    let types = [node_type(&["a"], &["out"])];
    let graph = graph_of(node_at(0.0, 0.0));
    let rect = node_rect(&graph.nodes[0], &types[0]);
    let (x, y) = output_anchor(&rect, 0);
    let hit = port_at(&graph, &types, x, y).unwrap();
    assert_eq!((hit.port.as_str(), hit.side), ("out", Side::Output));
}

#[test]
fn a_press_outside_the_radius_or_off_every_node_hits_nothing() {
    let types = [node_type(&["a"], &["out"])];
    let graph = graph_of(node_at(0.0, 0.0));
    let rect = node_rect(&graph.nodes[0], &types[0]);
    let (x, y) = input_anchor(&rect, 0);
    assert!(port_at(&graph, &types, x + PORT_RADIUS + 1.0, y).is_none());
    assert!(port_at(&graph, &types, 900.0, 900.0).is_none());
    assert!(
        port_at(
            &Graph {
                nodes: vec![],
                links: vec![]
            },
            &types,
            x,
            y
        )
        .is_none()
    );
}

#[test]
fn a_node_of_an_unknown_type_has_no_ports_to_hit() {
    let graph = graph_of(node_at(0.0, 0.0));
    assert!(port_at(&graph, &[], 0.0, HEADER_HEIGHT + 11.0).is_none());
}

#[test]
fn anchor_of_finds_a_named_port_on_either_side() {
    let types = [node_type(&["a", "b"], &["out"])];
    let graph = graph_of(node_at(10.0, 20.0));
    let rect = node_rect(&graph.nodes[0], &types[0]);
    let named = |port: &str| PortRef {
        node: "n1".to_string(),
        port: port.to_string(),
    };
    assert_eq!(
        anchor_of(&graph, &types, &named("b"), Side::Input),
        Some(input_anchor(&rect, 1))
    );
    assert_eq!(
        anchor_of(&graph, &types, &named("out"), Side::Output),
        Some(output_anchor(&rect, 0))
    );
    assert_eq!(anchor_of(&graph, &types, &named("out"), Side::Input), None);
    assert_eq!(
        anchor_of(&graph, &types, &named("nope"), Side::Output),
        None
    );
}
