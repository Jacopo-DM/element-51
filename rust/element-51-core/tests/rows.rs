//! Inputs that grow as they are linked: the rows a node has in a graph, and how
//! they change the card.

use std::borrow::Cow;

use element_51_core::display::line_names;
use element_51_core::layout::{Side, anchor_of, node_at, node_rect, port_at};
use element_51_core::rows::{effective_inputs, find_input, resolve_type, type_of, variadic_index};
use element_51_core::theme::Theme;
use element_51_core::{
    CardDisplay, Graph, Link, NodeType, PlacedNode, Port, PortRef, PortType, Variadic,
};

fn series_port(name: &str) -> Port {
    Port {
        name: name.to_string(),
        kind: PortType::Series,
    }
}

fn plot() -> NodeType {
    NodeType {
        job: false,
        name: "Plot".to_string(),
        inputs: vec![series_port("series_1")],
        outputs: vec![],
        defaults: Default::default(),
        category: String::new(),
        display: CardDisplay::Chart,
        variadic: Some(Variadic {
            prefix: "series".to_string(),
            kind: PortType::Series,
        }),
    }
}

fn curve() -> NodeType {
    NodeType {
        job: false,
        name: "curve".to_string(),
        inputs: vec![],
        outputs: vec![series_port("out")],
        defaults: Default::default(),
        category: String::new(),
        display: CardDisplay::None,
        variadic: None,
    }
}

fn placed(id: &str, kind: &str, x: f64, y: f64) -> PlacedNode {
    PlacedNode {
        id: id.to_string(),
        node_type: kind.to_string(),
        x,
        y,
        values: Default::default(),
        width: None,
        display_height: None,
    }
}

fn link(from: &str, to: &str, row: &str) -> Link {
    Link {
        source: PortRef {
            node: from.to_string(),
            port: "out".to_string(),
        },
        target: PortRef {
            node: to.to_string(),
            port: row.to_string(),
        },
    }
}

/// Curves n1, n2, ... each linked to the row named in `rows`, and a Plot after them.
fn graph_with(rows: &[&str]) -> Graph {
    let mut nodes: Vec<PlacedNode> = (0..rows.len())
        .map(|index| {
            placed(
                &format!("n{}", index + 1),
                "curve",
                0.0,
                100.0 * index as f64,
            )
        })
        .collect();
    let plot_id = format!("n{}", rows.len() + 1);
    nodes.push(placed(&plot_id, "Plot", 400.0, 0.0));
    let links = rows
        .iter()
        .enumerate()
        .map(|(index, row)| link(&format!("n{}", index + 1), &plot_id, row))
        .collect();
    Graph { nodes, links }
}

fn names(ports: &[Port]) -> Vec<&str> {
    ports.iter().map(|port| port.name.as_str()).collect()
}

fn rows(graph: &Graph) -> Vec<String> {
    let plot_id = graph.nodes.last().unwrap().id.clone();
    effective_inputs(&plot(), graph, &plot_id)
        .into_iter()
        .map(|port| port.name)
        .collect()
}

fn variadic() -> Variadic {
    plot().variadic.unwrap()
}

#[test]
fn only_a_prefix_and_a_positive_whole_number_is_a_row() {
    for (name, expected) in [
        ("series_1", Some(1)),
        ("series_12", Some(12)),
        ("series_0", None),
        ("series_01", None),
        ("series_-1", None),
        ("series_x", None),
        ("series_", None),
        ("series", None),
        ("other_1", None),
        ("xseries_1", None),
        ("series_1x", None),
    ] {
        assert_eq!(variadic_index(&variadic(), name), expected, "{name}");
    }
}

#[test]
fn a_row_that_is_not_listed_still_has_the_variadic_kind() {
    assert_eq!(
        find_input(&plot(), "series_1"),
        Some(series_port("series_1"))
    );
    assert_eq!(
        find_input(&plot(), "series_7"),
        Some(series_port("series_7"))
    );
    assert_eq!(find_input(&plot(), "series_0"), None);
    assert_eq!(find_input(&curve(), "series_1"), None);
}

#[test]
fn a_type_that_does_not_grow_keeps_its_inputs_and_costs_nothing() {
    let graph = graph_with(&[]);
    assert!(matches!(
        resolve_type(&curve(), &graph, "n1"),
        Cow::Borrowed(_)
    ));
    assert_eq!(effective_inputs(&curve(), &graph, "n1"), curve().inputs);
}

#[test]
fn a_plot_with_nothing_linked_has_one_empty_row() {
    assert_eq!(rows(&graph_with(&[])), ["series_1"]);
}

#[test]
fn each_link_adds_a_row_and_one_empty_row_follows() {
    assert_eq!(rows(&graph_with(&["series_1"])), ["series_1", "series_2"]);
    assert_eq!(
        rows(&graph_with(&["series_1", "series_2"])),
        ["series_1", "series_2", "series_3"]
    );
}

#[test]
fn rows_follow_the_row_numbers_and_the_empty_row_is_one_past_the_highest() {
    assert_eq!(
        rows(&graph_with(&["series_3", "series_1"])),
        ["series_1", "series_3", "series_4"]
    );
    assert_eq!(rows(&graph_with(&["series_5"])), ["series_5", "series_6"]);
}

#[test]
fn removing_a_middle_link_removes_its_row_and_keeps_the_other_names() {
    let mut graph = graph_with(&["series_1", "series_2", "series_3"]);
    graph.links.remove(1);
    assert_eq!(rows(&graph), ["series_1", "series_3", "series_4"]);
}

#[test]
fn deleting_the_nodes_that_fed_a_plot_shrinks_it() {
    let mut graph = graph_with(&["series_1", "series_2"]);
    graph.links.clear();
    assert_eq!(rows(&graph), ["series_1"]);
}

#[test]
fn inputs_that_are_not_rows_stay_first() {
    let mut mixed = plot();
    mixed.inputs.insert(
        0,
        Port {
            name: "gain".to_string(),
            kind: PortType::Float,
        },
    );
    let graph = graph_with(&["series_2"]);
    let plot_id = graph.nodes.last().unwrap().id.clone();
    let ports = effective_inputs(&mixed, &graph, &plot_id);
    assert_eq!(names(&ports), ["gain", "series_2", "series_3"]);
    assert_eq!(ports[0].kind, PortType::Float);
}

#[test]
fn the_resolved_type_has_the_rows_as_its_inputs() {
    let graph = graph_with(&["series_1"]);
    let base = plot();
    let resolved = resolve_type(&base, &graph, "n2");
    assert_eq!(names(&resolved.inputs), ["series_1", "series_2"]);
    assert_eq!(resolved.name, "Plot");
    assert_eq!(resolved.display, CardDisplay::Chart);
    let types = [plot(), curve()];
    let found = type_of(&graph, &types, &graph.nodes[1]).unwrap();
    assert_eq!(found.inputs.len(), 2);
    assert!(type_of(&graph, &types, &placed("n9", "nope", 0.0, 0.0)).is_none());
}

fn m() -> element_51_core::theme::Metrics {
    Theme::default().metrics
}

#[test]
fn a_card_grows_by_a_row_for_each_link() {
    let types = [plot(), curve()];
    let height = |rows: &[&str]| {
        let graph = graph_with(rows);
        let node = graph.nodes.last().unwrap();
        let resolved = type_of(&graph, &types, node).unwrap();
        node_rect(node, &resolved, &m()).h
    };
    let empty = height(&[]);
    assert_eq!(height(&["series_1"]) - empty, m().row_height);
    assert_eq!(
        height(&["series_1", "series_2"]) - empty,
        2.0 * m().row_height
    );
}

#[test]
fn the_empty_row_can_be_hit_and_a_link_lands_on_it() {
    let types = [plot(), curve()];
    let graph = graph_with(&["series_1"]);
    let node = graph.nodes.last().unwrap();
    let resolved = type_of(&graph, &types, node).unwrap();
    let rect = node_rect(node, &resolved, &m());
    let empty_row = element_51_core::layout::input_anchor(&rect, 1, &m());
    let hit = port_at(&graph, &types, empty_row.0, empty_row.1, &m()).unwrap();
    assert_eq!(
        (hit.node.as_str(), hit.port.as_str(), hit.side),
        ("n2", "series_2", Side::Input)
    );
    let reference = PortRef {
        node: "n2".to_string(),
        port: "series_2".to_string(),
    };
    assert_eq!(
        anchor_of(&graph, &types, &reference, Side::Input, &m()),
        Some(empty_row)
    );
    assert_eq!(
        node_at(&graph, &types, rect.x + 5.0, rect.y + rect.h - 2.0, &m()).as_deref(),
        Some("n2")
    );
}

#[test]
fn a_link_ends_on_the_row_it_names_after_the_rows_above_it_change() {
    let types = [plot(), curve()];
    let graph = graph_with(&["series_3", "series_1"]);
    let reference = PortRef {
        node: "n3".to_string(),
        port: "series_3".to_string(),
    };
    let node = graph.nodes.last().unwrap();
    let resolved = type_of(&graph, &types, node).unwrap();
    let rect = node_rect(node, &resolved, &m());
    let second_row = element_51_core::layout::input_anchor(&rect, 1, &m());
    assert_eq!(
        anchor_of(&graph, &types, &reference, Side::Input, &m()),
        Some(second_row)
    );
}

#[test]
fn a_line_is_named_after_its_source_port_and_a_shared_name_gets_the_node() {
    assert_eq!(
        line_names(&[("n1", "loss"), ("n2", "accuracy")]),
        ["loss", "accuracy"]
    );
    assert_eq!(
        line_names(&[("n1", "loss"), ("n2", "loss"), ("n3", "accuracy")]),
        ["n1 loss", "n2 loss", "accuracy"]
    );
    assert_eq!(line_names(&[]), Vec::<String>::new());
}
