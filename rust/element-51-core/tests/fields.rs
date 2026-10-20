//! What a person types into an input box, and what the box shows.

use element_51_core::fields::{field_label, format_value, parse_input};
use element_51_core::{Graph, Link, PlacedNode, PortRef, PortType};
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
