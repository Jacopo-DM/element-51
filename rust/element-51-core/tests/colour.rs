//! Series colours: the shared edit cases, undo and the status line.

use element_51_core::access::describe_ports;
use element_51_core::announce::edit_text;
use element_51_core::display::line_slot;
use element_51_core::history::History;
use element_51_core::layout::{
    RESET_SIDE, Rect, SWATCH_ROOM, SWATCH_SIDE, node_rect, swatch_boxes,
};
use element_51_core::theme::Theme;
use element_51_core::{Edit, Graph, Link, NodeType, PortRef, apply_edit, is_colour};
use serde::Deserialize;

const CASES: &str = include_str!("../../../tests/fixtures/colour_cases.json");

#[derive(Deserialize)]
struct Case {
    name: String,
    edit: Edit,
    result: String,
    graph: Option<Graph>,
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

#[test]
fn colour_edits_give_the_graph_or_refusal_python_gives() {
    let fixture = fixture();
    assert!(fixture.cases.len() >= 11);
    for case in &fixture.cases {
        let outcome = apply_edit(&fixture.graph, &fixture.node_types, &case.edit);
        match (case.result.as_str(), outcome) {
            ("ok", Ok(graph)) => assert_eq!(Some(graph), case.graph, "{}", case.name),
            ("ok", Err(error)) => panic!("{}: expected ok, got {error}", case.name),
            (expected, Ok(_)) => panic!("{}: expected {expected}, got ok", case.name),
            (expected, Err(error)) => assert_eq!(error.kind(), expected, "{}", case.name),
        }
    }
}

#[test]
fn a_colour_is_six_lower_case_hex_digits_after_a_hash() {
    for good in ["#000000", "#ff8800", "#0a9fbc"] {
        assert!(is_colour(good), "{good}");
    }
    for bad in [
        "", "#", "ff8800", "#ff880", "#ff88000", "#FF8800", "#ff88zz", "red",
    ] {
        assert!(!is_colour(bad), "{bad}");
    }
}

#[test]
fn a_colour_can_be_undone_and_redone() {
    let fixture = fixture();
    let set = Edit::SetColour {
        node: "n2".to_string(),
        port: "series_1".to_string(),
        colour: Some("#ff8800".to_string()),
    };
    let after = apply_edit(&fixture.graph, &fixture.node_types, &set).unwrap();
    let mut history = History::default();
    history.record(&fixture.graph, &set);
    let undo = history.undo().unwrap();
    let mut graph = after.clone();
    for edit in &undo {
        graph = apply_edit(&graph, &fixture.node_types, edit).unwrap();
    }
    assert_eq!(graph, fixture.graph);
    let redo = history.redo().unwrap();
    for edit in &redo {
        graph = apply_edit(&graph, &fixture.node_types, edit).unwrap();
    }
    assert_eq!(graph, after);
}

#[test]
fn deleting_a_node_and_undoing_brings_its_colours_back() {
    let fixture = fixture();
    let delete = Edit::DeleteNode {
        id: "n2".to_string(),
    };
    let mut history = History::default();
    history.record(&fixture.graph, &delete);
    let gone = apply_edit(&fixture.graph, &fixture.node_types, &delete).unwrap();
    let mut graph = gone;
    for edit in &history.undo().unwrap() {
        graph = apply_edit(&graph, &fixture.node_types, edit).unwrap();
    }
    let restored = graph.nodes.iter().find(|node| node.id == "n2").unwrap();
    assert_eq!(restored.colours.get("series_1").unwrap(), "#112233");
}

#[test]
fn the_status_line_says_which_series_changed_colour() {
    let set = Edit::SetColour {
        node: "n3".to_string(),
        port: "series_2".to_string(),
        colour: Some("#00aa55".to_string()),
    };
    assert_eq!(
        edit_text(&set),
        "Set the colour of series_2 of n3 to #00aa55"
    );
    let clear = Edit::SetColour {
        node: "n3".to_string(),
        port: "series_2".to_string(),
        colour: None,
    };
    assert_eq!(
        edit_text(&clear),
        "The colour of series_2 of n3 follows the theme"
    );
}

fn plot_graph(links: &[(&str, &str)]) -> (Graph, NodeType) {
    let fixture = fixture();
    let plot = fixture
        .node_types
        .iter()
        .find(|node_type| node_type.name == "Plot")
        .unwrap()
        .clone();
    let mut graph = fixture.graph;
    graph.links = links
        .iter()
        .map(|(source, target)| Link {
            source: PortRef {
                node: "n1".to_string(),
                port: (*source).to_string(),
            },
            target: PortRef {
                node: "n2".to_string(),
                port: (*target).to_string(),
            },
        })
        .collect();
    (graph, plot)
}

#[test]
fn a_line_slot_counts_the_linked_series_before_it() {
    let (graph, mut plot) = plot_graph(&[("out", "series_1"), ("out", "series_3")]);
    let extra = |name: &str| element_51_core::Port {
        name: name.to_string(),
        kind: element_51_core::PortType::Series,
    };
    plot.inputs.push(extra("series_2"));
    plot.inputs.push(extra("series_3"));
    let node = graph.nodes.iter().find(|node| node.id == "n2").unwrap();
    assert_eq!(line_slot(&graph, node, &plot, "series_1"), 0);
    assert_eq!(line_slot(&graph, node, &plot, "series_2"), 1);
    assert_eq!(line_slot(&graph, node, &plot, "series_3"), 1);
}

#[test]
fn the_swatch_sits_in_the_label_column_and_its_reset_button_beside_it() {
    let metrics = Theme::default().metrics;
    let (graph, plot) = plot_graph(&[]);
    let node = graph.nodes.iter().find(|node| node.id == "n2").unwrap();
    let card = node_rect(node, &plot, &metrics);
    let (swatch, reset) = swatch_boxes(&card, 0, &metrics);
    assert_eq!((swatch.w, swatch.h), (SWATCH_SIDE, SWATCH_SIDE));
    assert_eq!((reset.w, reset.h), (RESET_SIDE, RESET_SIDE));
    assert!(swatch.x > card.x + metrics.port_inset + metrics.port_size / 2.0);
    assert!(reset.x >= swatch.x + swatch.w);
    let names_start = card.x + metrics.port_inset + metrics.port_size / 2.0 + metrics.port_gap;
    assert!(reset.x + reset.w <= names_start + SWATCH_ROOM);
    let middle = |rect: &Rect| rect.y + rect.h / 2.0;
    assert_eq!(middle(&swatch), middle(&reset));
    let (below, _) = swatch_boxes(&card, 1, &metrics);
    assert_eq!(below.y - swatch.y, metrics.row_height);
    assert!(!swatch.overlaps(&reset));
}

#[test]
fn a_chosen_colour_is_part_of_the_ports_description() {
    let (mut graph, plot) = plot_graph(&[("out", "series_1")]);
    let node = graph.nodes.iter_mut().find(|node| node.id == "n2").unwrap();
    node.colours
        .insert("series_1".to_string(), "#ff8800".to_string());
    let labels: Vec<String> = describe_ports(&graph, &[plot], "n2")
        .into_iter()
        .map(|item| item.label)
        .collect();
    assert_eq!(
        labels[0],
        "Input series_1, series, linked from n1 out, colour #ff8800"
    );
}
