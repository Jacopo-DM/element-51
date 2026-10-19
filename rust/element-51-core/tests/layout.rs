//! Size and position of a node box.

use element_51_core::layout::{HEADER_HEIGHT, NODE_WIDTH, ROW_HEIGHT, node_rect};
use element_51_core::{Node, NodeType, Port, PortType};

fn port(name: &str) -> Port {
    Port {
        name: name.to_string(),
        kind: PortType::Float,
    }
}

fn node_at(x: f64, y: f64) -> Node {
    Node {
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
