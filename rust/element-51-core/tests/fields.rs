//! What a person types into an input box, and what the box shows.

use element_51_core::fields::{
    SliderValues, field_label, format_value, parse_input, slider_values,
};
use element_51_core::{CardDisplay, Graph, Link, NodeType, PlacedNode, Port, PortRef, PortType};
use serde_json::json;

#[test]
fn text_is_read_as_the_port_type_wants() {
    assert_eq!(parse_input("2.5", PortType::Float), Ok(Some(json!(2.5))));
    assert_eq!(parse_input(" 3 ", PortType::Float), Ok(Some(json!(3.0))));
    assert_eq!(parse_input("7", PortType::Int), Ok(Some(json!(7))));
    assert_eq!(parse_input("-7", PortType::Int), Ok(Some(json!(-7))));
    assert_eq!(
        parse_input(" hello ", PortType::Str),
        Ok(Some(json!(" hello ")))
    );
    assert_eq!(parse_input("true", PortType::Bool), Ok(Some(json!(true))));
    assert_eq!(parse_input("false", PortType::Bool), Ok(Some(json!(false))));
}

#[test]
fn an_empty_box_means_no_value_of_its_own() {
    for kind in [
        PortType::Float,
        PortType::Int,
        PortType::Str,
        PortType::Bool,
    ] {
        assert_eq!(parse_input("", kind), Ok(None));
        assert_eq!(parse_input("   ", kind), Ok(None));
    }
}

#[test]
fn text_that_does_not_fit_is_refused_with_a_reason() {
    assert_eq!(
        parse_input("abc", PortType::Float),
        Err("'abc' is not a number".to_string())
    );
    assert_eq!(
        parse_input("inf", PortType::Float),
        Err("'inf' is not a number".to_string())
    );
    assert_eq!(
        parse_input("1.5", PortType::Int),
        Err("'1.5' is not a whole number".to_string())
    );
    assert_eq!(
        parse_input("maybe", PortType::Bool),
        Err("'maybe' is not true or false".to_string())
    );
}

#[test]
fn a_value_is_shown_the_way_it_was_typed() {
    assert_eq!(format_value(&json!(2.5)), "2.5");
    assert_eq!(format_value(&json!(7)), "7");
    assert_eq!(format_value(&json!("hi there")), "hi there");
    assert_eq!(format_value(&json!(true)), "true");
}

fn node(id: &str) -> PlacedNode {
    PlacedNode {
        id: id.to_string(),
        node_type: "add".to_string(),
        x: 0.0,
        y: 0.0,
        values: Default::default(),
        width: None,
        display_height: None,
        colours: Default::default(),
    }
}

#[test]
fn an_input_is_linked_when_a_link_ends_at_it() {
    let graph = Graph {
        nodes: vec![node("n1"), node("n2")],
        links: vec![Link {
            source: PortRef {
                node: "n1".into(),
                port: "out".into(),
            },
            target: PortRef {
                node: "n2".into(),
                port: "a".into(),
            },
        }],
    };
    assert!(graph.is_linked("n2", "a"));
    assert!(!graph.is_linked("n2", "b"));
    assert!(!graph.is_linked("n1", "out"));
}

#[test]
fn an_input_box_is_named_by_its_port_and_its_node() {
    assert_eq!(field_label("n6", "loud"), "loud of node n6");
    assert_eq!(field_label("n12", "text"), "text of node n12");
}

// ---- what a linked input shows ----

use element_51_core::fields::{linked_text, share_room};
use element_51_core::{NodeOutcome, Status};
use std::collections::BTreeMap;

fn source() -> PortRef {
    PortRef {
        node: "n1".to_string(),
        port: "out".to_string(),
    }
}

fn outcome(status: Status, outputs: &[(&str, serde_json::Value)]) -> NodeOutcome {
    NodeOutcome {
        node: "n1".to_string(),
        status,
        outputs: outputs
            .iter()
            .map(|(port, value)| (port.to_string(), value.clone()))
            .collect(),
        shown: BTreeMap::new(),
        error: None,
    }
}

#[test]
fn before_a_run_a_linked_input_shows_only_its_source() {
    let text = linked_text(&source(), None);
    assert_eq!(text.value, None);
    assert_eq!(text.source, "n1.out");
}

#[test]
fn after_a_run_the_value_comes_first_and_the_source_is_bracketed() {
    let done = outcome(Status::Ok, &[("out", serde_json::json!(21.0))]);
    let text = linked_text(&source(), Some(&done));
    assert_eq!(text.value.as_deref(), Some("21.0"));
    assert_eq!(text.source, "[n1.out]");
    let word = outcome(Status::Ok, &[("out", serde_json::json!("hi"))]);
    assert_eq!(
        linked_text(&source(), Some(&word)).value.as_deref(),
        Some("\"hi\"")
    );
}

#[test]
fn a_source_that_failed_or_was_skipped_has_no_value_to_show() {
    for status in [Status::Error, Status::Skipped] {
        let result = outcome(status, &[("out", serde_json::json!(1.0))]);
        let text = linked_text(&source(), Some(&result));
        assert_eq!(text.value, None);
        assert_eq!(text.source, "n1.out");
    }
}

#[test]
fn a_picture_or_series_source_has_no_text_value() {
    let result = outcome(Status::Ok, &[]);
    let text = linked_text(&source(), Some(&result));
    assert_eq!(text.value, None);
    assert_eq!(text.source, "n1.out");
}

#[test]
fn a_port_the_outcome_lacks_has_no_value() {
    let result = outcome(Status::Ok, &[("other", serde_json::json!(1))]);
    assert_eq!(linked_text(&source(), Some(&result)).value, None);
}

#[test]
fn both_texts_that_fit_keep_their_full_width() {
    assert_eq!(share_room(30.0, 40.0, 100.0, 6.0), (30.0, 40.0));
}

#[test]
fn the_source_is_cut_first_when_the_box_is_narrow() {
    let (value, source) = share_room(30.0, 40.0, 60.0, 6.0);
    assert_eq!(value, 30.0);
    assert_eq!(source, 24.0);
}

#[test]
fn a_value_that_fills_the_box_leaves_no_room_for_the_source() {
    assert_eq!(share_room(80.0, 40.0, 60.0, 6.0), (60.0, 0.0));
    assert_eq!(share_room(54.0, 40.0, 60.0, 6.0), (54.0, 0.0));
    assert_eq!(share_room(10.0, 10.0, 0.0, 6.0), (0.0, 0.0));
}

fn slider_type(display: CardDisplay) -> NodeType {
    let float = |name: &str| Port {
        name: name.to_string(),
        kind: PortType::Float,
    };
    NodeType {
        job: false,
        name: "Slider".to_string(),
        inputs: vec![float("value"), float("minimum"), float("maximum")],
        outputs: vec![float("out")],
        defaults: [
            ("value".to_string(), json!(0.0)),
            ("minimum".to_string(), json!(-10.0)),
            ("maximum".to_string(), json!(10.0)),
        ]
        .into_iter()
        .collect(),
        category: "Input nodes".to_string(),
        display,
        variadic: Default::default(),
    }
}

fn slider_node(values: &[(&str, f64)]) -> PlacedNode {
    PlacedNode {
        id: "n1".to_string(),
        node_type: "Slider".to_string(),
        x: 0.0,
        y: 0.0,
        values: values
            .iter()
            .map(|(name, value)| ((*name).to_string(), json!(*value)))
            .collect(),
        width: None,
        display_height: None,
        colours: Default::default(),
    }
}

#[test]
fn a_slider_with_no_values_of_its_own_shows_the_defaults() {
    let shown = slider_values(&slider_node(&[]), &slider_type(CardDisplay::Slider));
    assert_eq!(
        shown,
        Some(SliderValues {
            value: 0.0,
            minimum: -10.0,
            maximum: 10.0,
            step: 0.2
        })
    );
}

#[test]
fn a_slider_shows_its_own_values_and_a_hundredth_of_the_range_as_the_step() {
    let node = slider_node(&[("value", 3.2), ("minimum", 0.0), ("maximum", 5.0)]);
    let shown = slider_values(&node, &slider_type(CardDisplay::Slider)).unwrap();
    assert_eq!((shown.value, shown.minimum, shown.maximum), (3.2, 0.0, 5.0));
    assert!((shown.step - 0.05).abs() < 1e-12);
}

#[test]
fn only_a_slider_card_has_slider_values() {
    let node = slider_node(&[]);
    assert_eq!(slider_values(&node, &slider_type(CardDisplay::None)), None);
}

#[test]
fn an_empty_range_has_no_slider_to_move() {
    let node = slider_node(&[("minimum", 5.0), ("maximum", 5.0)]);
    assert_eq!(
        slider_values(&node, &slider_type(CardDisplay::Slider)),
        None
    );
}

#[test]
fn a_slider_type_without_a_default_and_a_value_has_no_slider() {
    let mut node_type = slider_type(CardDisplay::Slider);
    node_type.defaults.remove("maximum");
    assert_eq!(slider_values(&slider_node(&[]), &node_type), None);
}
