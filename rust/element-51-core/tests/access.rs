//! What a screen reader is told about the graph, and how the keyboard moves
//! around it.

use element_51_core::access::{
    Direction, NodeItem, ZOOM_STEP, describe_graph, describe_ports, link_edit, list_nodes,
    moved_to, neighbour, pan_delta, pan_to_show, with_displays,
};
use element_51_core::announce::{
    cannot_delete_text, edit_text, editing_text, link_cancelled_text, link_started_text,
    linked_input_text, pan_text, picture_input_text, refusal_text, run_text, selected_text,
    undo_text, zoom_text,
};
use element_51_core::layout::{Rect, Side};
use element_51_core::theme::{Metrics, Theme};
use element_51_core::view::{MAX_ZOOM, MIN_ZOOM, View};
use element_51_core::{
    Edit, Graph, Image, Link, NodeOutcome, NodeType, PlacedNode, Port, PortRef, PortType,
    RunResult, Series, Shown, Status, apply_edit,
};
use serde::Deserialize;

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
        job: false,
        name: name.to_string(),
        inputs,
        outputs,
        defaults: Default::default(),
        category: String::new(),
        display: Default::default(),
        variadic: Default::default(),
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

#[test]
fn the_cheap_list_has_the_labels_and_port_counts_but_no_ports() {
    let (graph, types) = (graph(), types());
    let cheap = list_nodes(&graph, &types);
    let full = describe_graph(&graph, &types);
    assert_eq!(cheap.len(), full.len());
    for (cheap, full) in cheap.iter().zip(&full) {
        assert_eq!((&cheap.id, &cheap.label), (&full.id, &full.label));
        assert!(cheap.ports.is_empty());
        assert_eq!(cheap.port_count, full.ports.len());
    }
}

#[test]
fn ports_are_described_for_one_node_by_id() {
    let (graph, types) = (graph(), types());
    let full = describe_graph(&graph, &types);
    let n3 = full.iter().find(|item| item.id == "n3").unwrap();
    assert_eq!(describe_ports(&graph, &types, "n3"), n3.ports);
}

#[test]
fn a_missing_node_or_type_has_no_ports() {
    let (graph, types) = (graph(), types());
    assert!(describe_ports(&graph, &types, "nope").is_empty());
    let lone = Graph {
        nodes: vec![node("n1", "gone", 0.0, 0.0)],
        links: vec![],
    };
    assert!(describe_ports(&lone, &types, "n1").is_empty());
}

fn end(node: &str, port: &str) -> PortRef {
    PortRef {
        node: node.to_string(),
        port: port.to_string(),
    }
}

#[test]
fn each_kind_of_edit_has_a_sentence() {
    let add = Edit::AddNode {
        id: "n4".into(),
        node_type: "add".into(),
        x: 0.0,
        y: 0.0,
    };
    assert_eq!(edit_text(&add), "Added node n4, add");
    let linked = Edit::AddLink {
        source: end("n1", "out"),
        target: end("n3", "a"),
    };
    assert_eq!(edit_text(&linked), "Linked n1 out to n3 a");
    let moved = Edit::MoveNode {
        id: "n2".into(),
        x: 5.0,
        y: 6.0,
    };
    assert_eq!(edit_text(&moved), "Moved n2");
    assert_eq!(
        edit_text(&Edit::DeleteNode { id: "n2".into() }),
        "Deleted node n2"
    );
    let unlinked = Edit::DeleteLink {
        source: end("n1", "out"),
        target: end("n3", "a"),
    };
    assert_eq!(edit_text(&unlinked), "Deleted the link from n1 out to n3 a");
}

#[test]
fn a_value_edit_says_what_was_set_or_cleared() {
    let set = |value| Edit::SetValue {
        node: "n1".into(),
        port: "a".into(),
        value,
    };
    assert_eq!(
        edit_text(&set(Some(serde_json::json!(2.5)))),
        "Set a of n1 to 2.5"
    );
    assert_eq!(
        edit_text(&set(Some(serde_json::json!("hello")))),
        "Set a of n1 to hello"
    );
    assert_eq!(edit_text(&set(None)), "Cleared the value of a of n1");
}

#[test]
fn undo_and_redo_say_what_they_did_or_that_they_could_not() {
    assert_eq!(undo_text(true, true), "Undid the last change");
    assert_eq!(undo_text(false, true), "Redid the last change");
    assert_eq!(undo_text(true, false), "Nothing to undo");
    assert_eq!(undo_text(false, false), "Nothing to redo");
}

#[test]
fn a_refusal_and_a_selection_have_their_sentences() {
    assert_eq!(
        refusal_text("float cannot feed str"),
        "Refused: float cannot feed str"
    );
    assert_eq!(selected_text("n3"), "Selected n3");
}

#[test]
fn delete_on_a_port_that_has_nothing_to_delete_explains_why() {
    assert_eq!(
        cannot_delete_text(Side::Output),
        "Delete works on a node, or on an input port with a link"
    );
    assert_eq!(cannot_delete_text(Side::Input), "That input has no link");
}

const LINK_CASES: &str = include_str!("../../../tests/fixtures/link_cases.json");

#[derive(Deserialize)]
struct LinkCase {
    name: String,
    source: PortRef,
    target: PortRef,
    result: String,
}

#[derive(Deserialize)]
struct LinkCases {
    node_types: Vec<NodeType>,
    graph: Graph,
    cases: Vec<LinkCase>,
}

#[test]
fn a_link_made_from_the_keyboard_gets_the_verdict_the_mouse_gets() {
    let fixture: LinkCases = serde_json::from_str(LINK_CASES).unwrap();
    for case in &fixture.cases {
        let edit = link_edit(&case.source, &case.target);
        assert_eq!(
            edit,
            Edit::AddLink {
                source: case.source.clone(),
                target: case.target.clone()
            },
            "{}",
            case.name
        );
        let verdict = match apply_edit(&fixture.graph, &fixture.node_types, &edit) {
            Ok(_) => "ok".to_string(),
            Err(error) => error.kind().to_string(),
        };
        assert_eq!(verdict, case.result, "{}", case.name);
    }
}

#[test]
fn the_link_sentences_say_what_to_do_next() {
    assert_eq!(
        link_started_text(&end("n1", "out")),
        "Link from n1 out started. Move to an input port and press Enter, or Escape to cancel"
    );
    assert_eq!(link_cancelled_text(), "Link cancelled");
}

#[test]
fn the_value_box_sentences_say_what_to_do() {
    assert_eq!(
        editing_text("n3", "a"),
        "Type a value for a of n3, then press Enter. Escape leaves it unchanged"
    );
    assert_eq!(
        linked_input_text(),
        "That input is linked, so it has no value box. Press Delete to remove the link"
    );
}

#[test]
fn a_node_moves_one_grid_step_or_four() {
    let metrics = m();
    let step = metrics.grid_step;
    assert_eq!(
        moved_to(100.0, 100.0, Direction::Right, false, &metrics),
        (100.0 + step, 100.0)
    );
    assert_eq!(
        moved_to(100.0, 100.0, Direction::Left, false, &metrics),
        (100.0 - step, 100.0)
    );
    assert_eq!(
        moved_to(100.0, 100.0, Direction::Down, false, &metrics),
        (100.0, 100.0 + step)
    );
    assert_eq!(
        moved_to(100.0, 100.0, Direction::Up, false, &metrics),
        (100.0, 100.0 - step)
    );
    assert_eq!(
        moved_to(100.0, 100.0, Direction::Up, true, &metrics),
        (100.0, 100.0 - 4.0 * step)
    );
    assert_eq!(
        moved_to(0.0, 0.0, Direction::Right, true, &metrics),
        (4.0 * step, 0.0)
    );
}

#[test]
fn the_view_pans_toward_the_key_so_the_graph_moves_the_other_way() {
    assert_eq!(pan_delta(Direction::Right), (-48.0, 0.0));
    assert_eq!(pan_delta(Direction::Left), (48.0, 0.0));
    assert_eq!(pan_delta(Direction::Down), (0.0, -48.0));
    assert_eq!(pan_delta(Direction::Up), (0.0, 48.0));
}

#[test]
fn repeated_zoom_steps_stop_at_the_limits() {
    let mut view = View::default();
    for _ in 0..30 {
        view.zoom_at(400.0, 300.0, ZOOM_STEP);
    }
    assert_eq!(view.zoom(), MAX_ZOOM);
    for _ in 0..60 {
        view.zoom_at(400.0, 300.0, 1.0 / ZOOM_STEP);
    }
    assert_eq!(view.zoom(), MIN_ZOOM);
}

#[test]
fn the_zoom_and_pan_sentences_give_whole_percentages_and_directions() {
    assert_eq!(zoom_text(1.0), "Zoom 100 percent");
    assert_eq!(zoom_text(1.25), "Zoom 125 percent");
    assert_eq!(zoom_text(0.25), "Zoom 25 percent");
    assert_eq!(zoom_text(1.5625), "Zoom 156 percent");
    assert_eq!(pan_text(Direction::Left), "Panned left");
    assert_eq!(pan_text(Direction::Right), "Panned right");
    assert_eq!(pan_text(Direction::Up), "Panned up");
    assert_eq!(pan_text(Direction::Down), "Panned down");
}

fn outcome(node: &str, status: Status, error: Option<&str>) -> NodeOutcome {
    NodeOutcome {
        node: node.to_string(),
        status,
        outputs: Default::default(),
        shown: Default::default(),
        error: error.map(str::to_string),
    }
}

fn result(outcomes: Vec<NodeOutcome>) -> RunResult {
    RunResult {
        outcomes,
        partial: false,
    }
}

#[test]
fn a_clean_run_says_how_many_nodes_were_ok() {
    let all_ok = result(vec![
        outcome("n1", Status::Ok, None),
        outcome("n2", Status::Ok, None),
    ]);
    assert_eq!(run_text(&all_ok), "Run finished: 2 ok");
}

#[test]
fn a_run_with_failures_counts_each_kind_and_names_the_first_error() {
    let mixed = result(vec![
        outcome("n1", Status::Ok, None),
        outcome("n2", Status::Ok, None),
        outcome(
            "n3",
            Status::Error,
            Some("ZeroDivisionError: float division by zero"),
        ),
        outcome("n4", Status::Skipped, None),
        outcome("n5", Status::Error, Some("input 'x' has no value")),
    ]);
    assert_eq!(
        run_text(&mixed),
        "Run finished: 2 ok, 2 errors, 1 skipped. First error: n3: ZeroDivisionError: float division by zero"
    );
}

#[test]
fn one_error_is_not_plural_and_an_error_without_a_message_says_failed() {
    let one = result(vec![outcome("n1", Status::Error, None)]);
    assert_eq!(
        run_text(&one),
        "Run finished: 1 error. First error: n1: failed"
    );
}

#[test]
fn a_run_of_an_empty_graph_says_so() {
    assert_eq!(run_text(&result(vec![])), "Run finished: no nodes");
}

fn picture_outcome(node: &str, sizes: &[(&str, u32, u32)]) -> NodeOutcome {
    NodeOutcome {
        shown: sizes
            .iter()
            .map(|(port, width, height)| {
                (
                    port.to_string(),
                    Shown::Image(Image {
                        width: *width,
                        height: *height,
                        channels: 1,
                        pixels: vec![0; (width * height) as usize],
                    }),
                )
            })
            .collect(),
        ..outcome(node, Status::Ok, None)
    }
}

fn label_of<'a>(items: &'a [NodeItem], id: &str) -> &'a str {
    &items.iter().find(|item| item.id == id).unwrap().label
}

#[test]
fn a_node_that_shows_a_picture_says_its_size() {
    let items = list_nodes(&graph(), &types());
    let shown = with_displays(
        items.clone(),
        &graph(),
        &result(vec![picture_outcome("n1", &[("out", 28, 28)])]),
    );
    assert_eq!(
        label_of(&shown, "n1"),
        format!("{} Shows a 28 by 28 picture.", label_of(&items, "n1"))
    );
    assert_eq!(label_of(&shown, "n2"), label_of(&items, "n2"));
}

#[test]
fn a_node_with_two_pictures_names_each_size() {
    let items = list_nodes(&graph(), &types());
    let two = picture_outcome("n1", &[("a", 2, 3), ("b", 4, 5)]);
    let shown = with_displays(items.clone(), &graph(), &result(vec![two]));
    assert_eq!(
        label_of(&shown, "n1"),
        format!(
            "{} Shows a 2 by 3 picture and a 4 by 5 picture.",
            label_of(&items, "n1")
        )
    );
}

#[test]
fn a_list_is_unchanged_when_nothing_was_shown() {
    let items = list_nodes(&graph(), &types());
    let plain = result(vec![outcome("n1", Status::Ok, None)]);
    assert_eq!(with_displays(items.clone(), &graph(), &plain), items);
}

#[test]
fn a_picture_input_says_it_can_only_be_linked() {
    let text = picture_input_text();
    assert!(
        text.contains("picture") && text.contains("linked"),
        "{text}"
    );
    assert_ne!(text, linked_input_text());
}

fn plot_world() -> (Graph, Vec<NodeType>) {
    let mut node_types = types();
    node_types.push(NodeType {
        job: false,
        name: "curves".to_string(),
        inputs: vec![],
        outputs: vec![
            port("loss", PortType::Series),
            port("accuracy", PortType::Series),
        ],
        defaults: Default::default(),
        category: String::new(),
        display: Default::default(),
        variadic: Default::default(),
    });
    node_types.push(NodeType {
        job: false,
        name: "Plot".to_string(),
        inputs: vec![
            port("series_1", PortType::Series),
            port("series_2", PortType::Series),
        ],
        outputs: vec![],
        defaults: Default::default(),
        category: String::new(),
        display: element_51_core::CardDisplay::Chart,
        variadic: Default::default(),
    });
    let graph = Graph {
        nodes: vec![
            node("n1", "curves", 0.0, 0.0),
            node("n2", "Plot", 300.0, 0.0),
        ],
        links: vec![
            link(("n1", "loss"), ("n2", "series_1")),
            link(("n1", "accuracy"), ("n2", "series_2")),
        ],
    };
    (graph, node_types)
}

fn plot_outcome(lines: &[(&str, &[Option<f64>])]) -> NodeOutcome {
    NodeOutcome {
        shown: lines
            .iter()
            .map(|(port, values)| {
                (
                    port.to_string(),
                    Shown::Series(Series {
                        values: values.to_vec(),
                    }),
                )
            })
            .collect(),
        ..outcome("n2", Status::Ok, None)
    }
}

#[test]
fn a_chart_says_how_many_lines_it_has_and_where_each_ends() {
    let (graph, node_types) = plot_world();
    let items = list_nodes(&graph, &node_types);
    let one = plot_outcome(&[("series_1", &[Some(0.9), Some(0.31)])]);
    let shown = with_displays(items.clone(), &graph, &result(vec![one]));
    assert_eq!(
        label_of(&shown, "n2"),
        format!(
            "{} Shows a plot of 1 line, loss last 0.31.",
            label_of(&items, "n2")
        )
    );
    let two = plot_outcome(&[
        ("series_1", &[Some(0.31)]),
        ("series_2", &[Some(0.5), Some(0.92)]),
    ]);
    let shown = with_displays(items.clone(), &graph, &result(vec![two]));
    assert_eq!(
        label_of(&shown, "n2"),
        format!(
            "{} Shows a plot of 2 lines, loss last 0.31 and accuracy last 0.92.",
            label_of(&items, "n2")
        )
    );
}

#[test]
fn a_line_with_no_numbers_says_so_and_an_empty_chart_says_nothing() {
    let (graph, node_types) = plot_world();
    let items = list_nodes(&graph, &node_types);
    let gaps = plot_outcome(&[("series_1", &[None, None])]);
    let shown = with_displays(items.clone(), &graph, &result(vec![gaps]));
    assert_eq!(
        label_of(&shown, "n2"),
        format!(
            "{} Shows a plot of 1 line, loss has no value yet.",
            label_of(&items, "n2")
        )
    );
    let empty = plot_outcome(&[]);
    let shown = with_displays(items.clone(), &graph, &result(vec![empty]));
    assert_eq!(label_of(&shown, "n2"), label_of(&items, "n2"));
}

#[test]
fn a_node_that_made_a_series_does_not_say_it_plots() {
    // `curves` has series outputs, which the page receives for the box text. It
    // draws nothing, so its label stays as it is; only the Plot that they feed
    // says "a plot".
    let (graph, node_types) = plot_world();
    let items = list_nodes(&graph, &node_types);
    let made = NodeOutcome {
        shown: [("loss", &[Some(0.9)][..]), ("accuracy", &[Some(0.5)][..])]
            .into_iter()
            .map(|(port, values)| {
                (
                    port.to_string(),
                    Shown::Series(Series {
                        values: values.to_vec(),
                    }),
                )
            })
            .collect(),
        ..outcome("n1", Status::Ok, None)
    };
    let shown = with_displays(items.clone(), &graph, &result(vec![made]));
    assert_eq!(label_of(&shown, "n1"), label_of(&items, "n1"));
}

fn growing_plot_world(rows: &[(&str, &str, &str)]) -> (Graph, Vec<NodeType>) {
    let (_, mut node_types) = plot_world();
    let plot = node_types.iter_mut().find(|t| t.name == "Plot").unwrap();
    plot.variadic = Some(element_51_core::Variadic {
        prefix: "series".to_string(),
        kind: PortType::Series,
    });
    let mut nodes = vec![node("n9", "Plot", 600.0, 0.0)];
    let mut links = Vec::new();
    for (id, port, row) in rows {
        nodes.push(node(id, "curves", 0.0, 0.0));
        links.push(link((id, port), ("n9", row)));
    }
    (Graph { nodes, links }, node_types)
}

#[test]
fn a_growing_plot_lists_its_rows_in_the_tree() {
    let (graph, node_types) = growing_plot_world(&[("n1", "loss", "series_3")]);
    let ports = describe_ports(&graph, &node_types, "n9");
    let labels: Vec<&str> = ports.iter().map(|p| p.label.as_str()).collect();
    assert_eq!(
        labels,
        [
            "Input series_3, series, linked from n1 loss",
            "Input series_4, series, not linked"
        ]
    );
    let items = list_nodes(&graph, &node_types);
    assert_eq!(label_of(&items, "n9"), "n9, Plot. 2 inputs, 0 outputs.");
    assert_eq!(items.iter().find(|i| i.id == "n9").unwrap().port_count, 2);
}

#[test]
fn two_lines_with_one_name_are_told_apart_by_node_in_the_label() {
    let (graph, node_types) =
        growing_plot_world(&[("n1", "loss", "series_1"), ("n2", "loss", "series_2")]);
    let items = list_nodes(&graph, &node_types);
    let both = plot_outcome_for(
        "n9",
        &[("series_1", &[Some(0.5)]), ("series_2", &[Some(0.25)])],
    );
    let shown = with_displays(items.clone(), &graph, &result(vec![both]));
    assert_eq!(
        label_of(&shown, "n9"),
        format!(
            "{} Shows a plot of 2 lines, n1 loss last 0.5 and n2 loss last 0.25.",
            label_of(&items, "n9")
        )
    );
}

#[test]
fn row_ten_follows_row_nine_in_the_label() {
    let rows: Vec<(String, String)> = (1..=10)
        .map(|k| (format!("n{k}"), format!("series_{k}")))
        .collect();
    let wired: Vec<(&str, &str, &str)> = rows
        .iter()
        .enumerate()
        .map(|(i, (id, row))| {
            (
                id.as_str(),
                if i % 2 == 0 { "loss" } else { "accuracy" },
                row.as_str(),
            )
        })
        .collect();
    let (graph, node_types) = growing_plot_world(&wired);
    let items = list_nodes(&graph, &node_types);
    let values: Vec<(String, Vec<Option<f64>>)> = (1..=10)
        .map(|k| (format!("series_{k}"), vec![Some(f64::from(k))]))
        .collect();
    let lines: Vec<(&str, &[Option<f64>])> = values
        .iter()
        .map(|(p, v)| (p.as_str(), v.as_slice()))
        .collect();
    let shown = with_displays(items, &graph, &result(vec![plot_outcome_for("n9", &lines)]));
    let label = label_of(&shown, "n9");
    let ten = label.find("n10 accuracy last 10").unwrap_or(usize::MAX);
    let nine = label.find("n9 loss last 9").unwrap_or(usize::MAX);
    assert!(nine < ten, "{label}");
}

fn plot_outcome_for(node: &str, lines: &[(&str, &[Option<f64>])]) -> NodeOutcome {
    NodeOutcome {
        node: node.to_string(),
        ..plot_outcome(lines)
    }
}
