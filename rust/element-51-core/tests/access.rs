//! What a screen reader is told about the graph, and how the keyboard moves
//! around it.

use element_51_core::access::{Direction, describe_graph, neighbour, pan_to_show};
use element_51_core::layout::{Rect, Side};
use element_51_core::theme::{Metrics, Theme};
use element_51_core::view::View;
use element_51_core::{Graph, Link, NodeType, PlacedNode, Port, PortRef, PortType};

fn m() -> Metrics {
    Theme::default().metrics
}

fn port(name: &str, kind: PortType) -> Port {
    Port {
        name: name.to_string(),
        kind,
    }
}

fn types() -> Vec<NodeType> {
    let make = |name: &str, inputs: Vec<Port>, outputs: Vec<Port>| NodeType {
        name: name.to_string(),
        inputs,
        outputs,
        defaults: Default::default(),
        category: String::new(),
    };
    vec![
        make("number", vec![], vec![port("out", PortType::Float)]),
        make(
            "add",
            vec![port("a", PortType::Float), port("b", PortType::Float)],
            vec![port("out", PortType::Float)],
        ),
        make("show", vec![port("value", PortType::Float)], vec![]),
    ]
}

fn node(id: &str, kind: &str, x: f64, y: f64) -> PlacedNode {
    PlacedNode {
        id: id.to_string(),
        node_type: kind.to_string(),
        x,
        y,
        values: Default::default(),
    }
}

fn link(source: (&str, &str), target: (&str, &str)) -> Link {
    let end = |(node, port): (&str, &str)| PortRef {
        node: node.to_string(),
        port: port.to_string(),
    };
    Link {
        source: end(source),
        target: end(target),
    }
}

/// n1 and n2 feed n3; n3 feeds n4 and n5.
fn graph() -> Graph {
    Graph {
        nodes: vec![
            node("n1", "number", 0.0, 0.0),
            node("n2", "number", 0.0, 200.0),
            node("n3", "add", 300.0, 100.0),
            node("n4", "show", 600.0, 0.0),
            node("n5", "show", 600.0, 200.0),
        ],
        links: vec![
            link(("n1", "out"), ("n3", "a")),
            link(("n2", "out"), ("n3", "b")),
            link(("n3", "out"), ("n4", "value")),
            link(("n3", "out"), ("n5", "value")),
        ],
    }
}

#[test]
fn nodes_come_in_reading_order() {
    let items = describe_graph(&graph(), &types());
    let ids: Vec<&str> = items.iter().map(|item| item.id.as_str()).collect();
    assert_eq!(ids, ["n1", "n4", "n3", "n2", "n5"]);
}

#[test]
fn equal_positions_are_ordered_by_id() {
    let graph = Graph {
        nodes: vec![
            node("n2", "number", 10.0, 10.0),
            node("n1", "number", 10.0, 10.0),
        ],
        links: vec![],
    };
    let items = describe_graph(&graph, &types());
    assert_eq!(items[0].id, "n1");
    assert_eq!(items[1].id, "n2");
}

#[test]
fn a_node_is_named_by_its_id_type_and_port_counts() {
    let items = describe_graph(&graph(), &types());
    let label = |id: &str| {
        items
            .iter()
            .find(|item| item.id == id)
            .unwrap()
            .label
            .clone()
    };
    assert_eq!(label("n1"), "n1, number. 0 inputs, 1 output.");
    assert_eq!(label("n3"), "n3, add. 2 inputs, 1 output.");
    assert_eq!(label("n4"), "n4, show. 1 input, 0 outputs.");
}

#[test]
fn a_node_of_an_unknown_type_says_so_and_has_no_ports() {
    let graph = Graph {
        nodes: vec![node("n1", "gone", 0.0, 0.0)],
        links: vec![],
    };
    let items = describe_graph(&graph, &types());
    assert_eq!(items[0].label, "n1, unknown type gone.");
    assert!(items[0].ports.is_empty());
}

#[test]
fn ports_list_inputs_then_outputs_with_their_links() {
    let items = describe_graph(&graph(), &types());
    let n3 = items.iter().find(|item| item.id == "n3").unwrap();
    let labels: Vec<(&str, Side, &str)> = n3
        .ports
        .iter()
        .map(|p| (p.port.as_str(), p.side, p.label.as_str()))
        .collect();
    assert_eq!(
        labels,
        [
            ("a", Side::Input, "Input a, float, linked from n1 out"),
            ("b", Side::Input, "Input b, float, linked from n2 out"),
            (
                "out",
                Side::Output,
                "Output out, float, linked to n4 value and n5 value"
            ),
        ]
    );
}

#[test]
fn an_unlinked_port_says_so() {
    let graph = Graph {
        nodes: vec![node("n1", "show", 0.0, 0.0)],
        links: vec![],
    };
    let items = describe_graph(&graph, &types());
    assert_eq!(items[0].ports[0].label, "Input value, float, not linked");
}

#[test]
fn three_or_more_links_are_joined_with_commas() {
    let graph = Graph {
        nodes: vec![
            node("n1", "number", 0.0, 0.0),
            node("n2", "show", 300.0, 0.0),
            node("n3", "show", 300.0, 100.0),
            node("n4", "show", 300.0, 200.0),
        ],
        links: vec![
            link(("n1", "out"), ("n2", "value")),
            link(("n1", "out"), ("n3", "value")),
            link(("n1", "out"), ("n4", "value")),
        ],
    };
    let items = describe_graph(&graph, &types());
    let out = &items[0].ports[0].label;
    assert_eq!(
        out,
        "Output out, float, linked to n2 value, n3 value and n4 value"
    );
}

#[test]
fn the_list_has_one_item_per_node_and_a_port_per_row() {
    let items = describe_graph(&graph(), &types());
    let ports: usize = items.iter().map(|item| item.ports.len()).sum();
    assert_eq!(items.len(), graph().nodes.len());
    assert_eq!(ports, 1 + 1 + 3 + 1 + 1);
}

#[test]
fn the_nearest_node_in_each_direction_is_found() {
    let (graph, types, metrics) = (graph(), types(), m());
    let go = |from: &str, direction| neighbour(&graph, &types, &metrics, from, direction);
    assert_eq!(go("n3", Direction::Right).as_deref(), Some("n5"));
    assert_eq!(go("n3", Direction::Left).as_deref(), Some("n2"));
    assert_eq!(go("n1", Direction::Down).as_deref(), Some("n2"));
    assert_eq!(go("n2", Direction::Up).as_deref(), Some("n1"));
}

#[test]
fn there_is_no_neighbour_past_the_edge() {
    let (graph, types, metrics) = (graph(), types(), m());
    assert_eq!(
        neighbour(&graph, &types, &metrics, "n1", Direction::Up),
        None
    );
    assert_eq!(
        neighbour(&graph, &types, &metrics, "n4", Direction::Right),
        None
    );
}

#[test]
fn a_node_straight_ahead_beats_one_far_off_to_the_side() {
    let graph = Graph {
        nodes: vec![
            node("n1", "number", 0.0, 0.0),
            node("n2", "number", 300.0, 400.0),
            node("n3", "number", 700.0, 0.0),
        ],
        links: vec![],
    };
    let found = neighbour(&graph, &types(), &m(), "n1", Direction::Right);
    assert_eq!(found.as_deref(), Some("n3"));
}

#[test]
fn equal_candidates_are_broken_by_id() {
    let graph = Graph {
        nodes: vec![
            node("n1", "number", 0.0, 0.0),
            node("n3", "number", 300.0, 0.0),
            node("n2", "number", 300.0, 0.0),
        ],
        links: vec![],
    };
    let found = neighbour(&graph, &types(), &m(), "n1", Direction::Right);
    assert_eq!(found.as_deref(), Some("n2"));
}

#[test]
fn an_unknown_start_or_type_gives_no_neighbour() {
    let (graph, types, metrics) = (graph(), types(), m());
    assert_eq!(
        neighbour(&graph, &types, &metrics, "nope", Direction::Right),
        None
    );
    let lone = Graph {
        nodes: vec![
            node("n1", "gone", 0.0, 0.0),
            node("n2", "number", 300.0, 0.0),
        ],
        links: vec![],
    };
    assert_eq!(
        neighbour(&lone, &types, &metrics, "n1", Direction::Right),
        None
    );
    assert_eq!(
        neighbour(&lone, &types, &metrics, "n2", Direction::Left),
        None
    );
}

fn rect(x: f64, y: f64) -> Rect {
    Rect {
        x,
        y,
        w: 100.0,
        h: 50.0,
    }
}

#[test]
fn a_visible_node_needs_no_pan() {
    let delta = pan_to_show(&View::default(), &rect(200.0, 200.0), (800.0, 600.0), 20.0);
    assert_eq!(delta, (0.0, 0.0));
}

#[test]
fn a_node_off_the_right_and_bottom_is_brought_in_with_a_margin() {
    let delta = pan_to_show(&View::default(), &rect(780.0, 590.0), (800.0, 600.0), 20.0);
    assert_eq!(delta, (-100.0, -60.0));
}

#[test]
fn a_node_off_the_left_and_top_is_brought_in_with_a_margin() {
    let delta = pan_to_show(&View::default(), &rect(-50.0, -30.0), (800.0, 600.0), 20.0);
    assert_eq!(delta, (70.0, 50.0));
}

#[test]
fn the_pan_counts_the_zoom_and_the_current_pan() {
    let mut view = View::default();
    view.zoom_at(0.0, 0.0, 2.0);
    view.pan_by(-100.0, 0.0);
    // On screen the node spans x from 100 to 300 and y from 400 to 500.
    let delta = pan_to_show(&view, &rect(100.0, 200.0), (800.0, 600.0), 20.0);
    assert_eq!(delta, (0.0, 0.0));
    let far = pan_to_show(&view, &rect(500.0, 200.0), (800.0, 600.0), 20.0);
    assert_eq!(far, (-320.0, 0.0));
}

#[test]
fn a_node_bigger_than_the_window_lines_up_with_the_top_left_margin() {
    let big = Rect {
        x: 50.0,
        y: 50.0,
        w: 2000.0,
        h: 2000.0,
    };
    let delta = pan_to_show(&View::default(), &big, (800.0, 600.0), 20.0);
    assert_eq!(delta, (-30.0, -30.0));
}
