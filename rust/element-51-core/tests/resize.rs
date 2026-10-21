//! A card the user resized: its box, its picture or chart area, the handle in
//! its corner, how a drag changes it, and the way back through undo.

use element_51_core::announce::edit_text;
use element_51_core::display::{
    HANDLE_SIDE, chart_rect, default_area_height, dragged_size, picture_rect, resize_handle,
};
use element_51_core::history::History;
use element_51_core::layout::node_rect;
use element_51_core::theme::Theme;
use element_51_core::{Edit, Graph, MAX_CARD_SIZE, NodeType, PlacedNode, apply_edit};
use serde_json::json;

fn node_type(display: &str) -> NodeType {
    serde_json::from_value(json!({
        "name": "t",
        "inputs": [{"name": "a", "kind": "float"}],
        "outputs": [{"name": "out", "kind": "float"}],
        "display": display,
    }))
    .unwrap()
}

fn placed(width: Option<f64>, display_height: Option<f64>) -> PlacedNode {
    PlacedNode {
        id: "n1".to_string(),
        node_type: "t".to_string(),
        x: 10.0,
        y: 20.0,
        values: Default::default(),
        width,
        display_height,
    }
}

#[test]
fn a_card_without_a_size_is_the_default_size() {
    let metrics = Theme::default().metrics;
    let card = node_rect(&placed(None, None), &node_type("image"), &metrics);
    assert!((card.w - metrics.node_width).abs() < 1e-9);
    let area = picture_rect(&card, &node_type("image"), &metrics).unwrap();
    assert!(
        (area.w - area.h).abs() < 1e-9,
        "the default picture is square"
    );
}

#[test]
fn a_bigger_card_has_a_bigger_picture_area() {
    let metrics = Theme::default().metrics;
    let kind = node_type("image");
    let default = node_rect(&placed(None, None), &kind, &metrics);
    let wanted_w = metrics.node_width + 100.0;
    let wanted_h = default_area_height(&kind, &metrics) + 60.0;
    let card = node_rect(&placed(Some(wanted_w), Some(wanted_h)), &kind, &metrics);
    assert!((card.w - wanted_w).abs() < 1e-9);
    assert!((card.h - default.h - 60.0).abs() < 1e-9);
    let area = picture_rect(&card, &kind, &metrics).unwrap();
    assert!((area.w - (wanted_w - 2.0 * metrics.title_pad)).abs() < 1e-9);
    assert!((area.h - wanted_h).abs() < 1e-9);
}

#[test]
fn the_chart_area_follows_the_card_in_the_same_way() {
    let metrics = Theme::default().metrics;
    let kind = node_type("chart");
    let default = node_rect(&placed(None, None), &kind, &metrics);
    let default_area = chart_rect(&default, &kind, &metrics).unwrap();
    let wanted = default_area_height(&kind, &metrics) + 40.0;
    let card = node_rect(&placed(None, Some(wanted)), &kind, &metrics);
    let area = chart_rect(&card, &kind, &metrics).unwrap();
    assert!((area.w - default_area.w).abs() < 1e-9);
    assert!((area.h - wanted).abs() < 1e-9);
}

#[test]
fn a_size_below_the_default_is_drawn_at_the_default() {
    let metrics = Theme::default().metrics;
    let kind = node_type("image");
    let default = node_rect(&placed(None, None), &kind, &metrics);
    let card = node_rect(&placed(Some(10.0), Some(5.0)), &kind, &metrics);
    assert_eq!(card, default);
}

#[test]
fn a_card_that_shows_nothing_ignores_a_height_but_takes_a_width() {
    let metrics = Theme::default().metrics;
    let kind = node_type("none");
    let default = node_rect(&placed(None, None), &kind, &metrics);
    let card = node_rect(
        &placed(Some(metrics.node_width + 50.0), Some(500.0)),
        &kind,
        &metrics,
    );
    assert!((card.h - default.h).abs() < 1e-9);
    assert!((card.w - metrics.node_width - 50.0).abs() < 1e-9);
    assert_eq!(resize_handle(&card, &kind), None);
}

#[test]
fn the_handle_sits_inside_the_bottom_right_corner() {
    let metrics = Theme::default().metrics;
    let kind = node_type("chart");
    let card = node_rect(&placed(None, None), &kind, &metrics);
    let handle = resize_handle(&card, &kind).unwrap();
    assert!((handle.x + handle.w - (card.x + card.w)).abs() < 1e-9);
    assert!((handle.y + handle.h - (card.y + card.h)).abs() < 1e-9);
    assert!((handle.w - HANDLE_SIDE).abs() < 1e-9);
}

#[test]
fn a_drag_grows_the_card_and_never_goes_below_the_default_or_above_the_maximum() {
    let metrics = Theme::default().metrics;
    let kind = node_type("image");
    let start = (metrics.node_width, default_area_height(&kind, &metrics));
    let (w, h) = dragged_size(start, (50.0, 30.0), &kind, &metrics);
    assert!((w - start.0 - 50.0).abs() < 1e-9 && (h - start.1 - 30.0).abs() < 1e-9);
    let smaller = dragged_size(start, (-500.0, -500.0), &kind, &metrics);
    assert_eq!(smaller, start);
    let huge = dragged_size(start, (1e9, 1e9), &kind, &metrics);
    assert_eq!(huge, (MAX_CARD_SIZE, MAX_CARD_SIZE));
}

#[test]
fn undo_puts_the_old_size_back_and_redo_the_new_one() {
    let kind = node_type("image");
    let before = Graph {
        nodes: vec![placed(Some(300.0), None)],
        links: vec![],
    };
    let edit = Edit::ResizeNode {
        id: "n1".to_string(),
        width: Some(400.0),
        display_height: Some(250.0),
    };
    let after = apply_edit(&before, std::slice::from_ref(&kind), &edit).unwrap();
    assert_eq!(after.nodes[0].width, Some(400.0));
    assert_eq!(after.nodes[0].display_height, Some(250.0));
    let mut history = History::default();
    history.record(&before, &edit);
    let mut graph = after;
    for undo in history.undo().unwrap() {
        graph = apply_edit(&graph, std::slice::from_ref(&kind), &undo).unwrap();
    }
    assert_eq!(graph, before);
    for redo in history.redo().unwrap() {
        graph = apply_edit(&graph, std::slice::from_ref(&kind), &redo).unwrap();
    }
    assert_eq!(graph.nodes[0].width, Some(400.0));
}

#[test]
fn undoing_a_delete_brings_back_the_size_of_the_card() {
    let kind = node_type("chart");
    let before = Graph {
        nodes: vec![placed(Some(333.0), Some(222.0))],
        links: vec![],
    };
    let edit = Edit::DeleteNode {
        id: "n1".to_string(),
    };
    let after = apply_edit(&before, std::slice::from_ref(&kind), &edit).unwrap();
    let mut history = History::default();
    history.record(&before, &edit);
    let mut graph = after;
    for undo in history.undo().unwrap() {
        graph = apply_edit(&graph, std::slice::from_ref(&kind), &undo).unwrap();
    }
    assert_eq!(graph.nodes[0].width, Some(333.0));
    assert_eq!(graph.nodes[0].display_height, Some(222.0));
}

#[test]
fn the_status_line_says_which_card_was_resized() {
    let edit = Edit::ResizeNode {
        id: "n3".to_string(),
        width: Some(300.0),
        display_height: None,
    };
    assert_eq!(edit_text(&edit), "Resized n3");
}
