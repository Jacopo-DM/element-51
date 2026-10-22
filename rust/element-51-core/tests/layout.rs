//! Size and position of a node box.

use element_51_core::layout::{
    Rect, Side, anchor_of, icon_at, icon_rect, input_anchor, link_at, node_at as pick_node,
    node_rect, output_anchor, output_row, port_at, wire_bounds, wire_controls,
};
use element_51_core::theme::{Metrics, Theme};
use element_51_core::{Graph, Link, NodeType, PlacedNode, Port, PortRef, PortType};

fn m() -> Metrics {
    Theme::default().metrics
}

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
        values: Default::default(),
        width: None,
        display_height: None,
        colours: Default::default(),
    }
}

fn node_type(inputs: &[&str], outputs: &[&str]) -> NodeType {
    NodeType {
        job: false,
        name: "t".to_string(),
        inputs: inputs.iter().map(|n| port(n)).collect(),
        outputs: outputs.iter().map(|n| port(n)).collect(),
        defaults: Default::default(),
        category: String::new(),
        display: Default::default(),
        variadic: Default::default(),
    }
}

#[test]
fn box_starts_at_the_node_position() {
    let rect = node_rect(&node_at(40.0, 60.0), &node_type(&["a"], &[]), &m());
    assert_eq!((rect.x, rect.y, rect.w), (40.0, 60.0, m().node_width));
}

#[test]
fn height_is_the_header_plus_one_row_per_input_and_one_per_output() {
    let two_in_one_out = node_rect(&node_at(0.0, 0.0), &node_type(&["a", "b"], &["out"]), &m());
    assert_eq!(two_in_one_out.h, m().header_height + 3.0 * m().row_height);
    let one_out_only = node_rect(&node_at(0.0, 0.0), &node_type(&[], &["out"]), &m());
    assert_eq!(one_out_only.h, m().header_height + m().row_height);
}

#[test]
fn a_node_without_ports_is_just_a_header() {
    let rect = node_rect(&node_at(0.0, 0.0), &node_type(&[], &[]), &m());
    assert_eq!(rect.h, m().header_height);
}

fn graph_of(node: PlacedNode) -> Graph {
    Graph {
        nodes: vec![node],
        links: vec![],
    }
}

#[test]
fn rows_stack_inputs_first_and_outputs_after() {
    let two_in_two_out = node_type(&["a", "b"], &["p", "q"]);
    assert_eq!(output_row(&two_in_two_out, 0), 2);
    assert_eq!(output_row(&two_in_two_out, 1), 3);
}

#[test]
fn anchors_sit_inside_the_card_at_the_row_centre() {
    let types = node_type(&["a", "b"], &["out"]);
    let rect = node_rect(&node_at(40.0, 60.0), &types, &m());
    let row_y = |row: f64| 60.0 + m().header_height + m().row_height * (row + 0.5);
    assert_eq!(
        input_anchor(&rect, 1, &m()),
        (40.0 + m().port_inset, row_y(1.0))
    );
    assert_eq!(
        output_anchor(&rect, output_row(&types, 0), &m()),
        (40.0 + m().node_width - m().port_inset, row_y(2.0))
    );
}

#[test]
fn a_press_on_an_input_anchor_hits_that_port() {
    let types = [node_type(&["a", "b"], &["out"])];
    let graph = graph_of(node_at(40.0, 60.0));
    let rect = node_rect(&graph.nodes[0], &types[0], &m());
    let (x, y) = input_anchor(&rect, 1, &m());
    let hit = port_at(&graph, &types, x + 3.0, y - 3.0, &m()).unwrap();
    assert_eq!(
        (hit.node.as_str(), hit.port.as_str(), hit.side),
        ("n1", "b", Side::Input)
    );
}

#[test]
fn a_press_on_an_output_anchor_hits_that_port() {
    let types = [node_type(&["a"], &["out"])];
    let graph = graph_of(node_at(0.0, 0.0));
    let rect = node_rect(&graph.nodes[0], &types[0], &m());
    let (x, y) = output_anchor(&rect, output_row(&types[0], 0), &m());
    let hit = port_at(&graph, &types, x, y, &m()).unwrap();
    assert_eq!((hit.port.as_str(), hit.side), ("out", Side::Output));
}

#[test]
fn a_press_outside_the_radius_or_off_every_node_hits_nothing() {
    let types = [node_type(&["a"], &["out"])];
    let graph = graph_of(node_at(0.0, 0.0));
    let rect = node_rect(&graph.nodes[0], &types[0], &m());
    let (x, y) = input_anchor(&rect, 0, &m());
    assert!(port_at(&graph, &types, x + m().port_hit_radius + 1.0, y, &m()).is_none());
    assert!(port_at(&graph, &types, 900.0, 900.0, &m()).is_none());
    assert!(
        port_at(
            &Graph {
                nodes: vec![],
                links: vec![]
            },
            &types,
            x,
            y,
            &m()
        )
        .is_none()
    );
}

#[test]
fn a_node_of_an_unknown_type_has_no_ports_to_hit() {
    let graph = graph_of(node_at(0.0, 0.0));
    assert!(port_at(&graph, &[], 0.0, m().header_height + 11.0, &m()).is_none());
}

#[test]
fn anchor_of_finds_a_named_port_on_either_side() {
    let types = [node_type(&["a", "b"], &["out"])];
    let graph = graph_of(node_at(10.0, 20.0));
    let rect = node_rect(&graph.nodes[0], &types[0], &m());
    let named = |port: &str| PortRef {
        node: "n1".to_string(),
        port: port.to_string(),
    };
    assert_eq!(
        anchor_of(&graph, &types, &named("b"), Side::Input, &m()),
        Some(input_anchor(&rect, 1, &m()))
    );
    assert_eq!(
        anchor_of(&graph, &types, &named("out"), Side::Output, &m()),
        Some(output_anchor(&rect, output_row(&types[0], 0), &m()))
    );
    assert_eq!(
        anchor_of(&graph, &types, &named("out"), Side::Input, &m()),
        None
    );
    assert_eq!(
        anchor_of(&graph, &types, &named("nope"), Side::Output, &m()),
        None
    );
}

#[test]
fn node_at_finds_the_node_under_a_point_and_the_topmost_when_they_overlap() {
    let types = [node_type(&["a"], &["out"])];
    let graph = Graph {
        nodes: vec![
            PlacedNode {
                id: "n1".into(),
                node_type: "t".into(),
                x: 0.0,
                y: 0.0,
                values: Default::default(),
                width: None,
                display_height: None,
                colours: Default::default(),
            },
            PlacedNode {
                id: "n2".into(),
                node_type: "t".into(),
                x: 100.0,
                y: 10.0,
                values: Default::default(),
                width: None,
                display_height: None,
                colours: Default::default(),
            },
        ],
        links: vec![],
    };
    assert_eq!(
        pick_node(&graph, &types, 10.0, 10.0, &m()).as_deref(),
        Some("n1")
    );
    assert_eq!(
        pick_node(&graph, &types, 120.0, 20.0, &m()).as_deref(),
        Some("n2")
    );
    assert_eq!(
        pick_node(&graph, &types, 130.0, 20.0, &m()).as_deref(),
        Some("n2")
    );
    assert_eq!(pick_node(&graph, &types, 400.0, 400.0, &m()), None);
}

#[test]
fn a_node_of_an_unknown_type_cannot_be_picked() {
    let graph = graph_of(node_at(0.0, 0.0));
    assert_eq!(pick_node(&graph, &[], 5.0, 5.0, &m()), None);
}

#[test]
fn link_at_finds_a_link_near_its_curve_and_nothing_far_from_it() {
    let types = [node_type(&["a"], &["out"])];
    let graph = Graph {
        nodes: vec![
            PlacedNode {
                id: "n1".into(),
                node_type: "t".into(),
                x: 0.0,
                y: 0.0,
                values: Default::default(),
                width: None,
                display_height: None,
                colours: Default::default(),
            },
            PlacedNode {
                id: "n2".into(),
                node_type: "t".into(),
                x: 400.0,
                y: 0.0,
                values: Default::default(),
                width: None,
                display_height: None,
                colours: Default::default(),
            },
        ],
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
    // The curve is symmetric, so its middle is halfway between its two anchors.
    let source = PortRef {
        node: "n1".into(),
        port: "out".into(),
    };
    let target = PortRef {
        node: "n2".into(),
        port: "a".into(),
    };
    let from = anchor_of(&graph, &types, &source, Side::Output, &m()).unwrap();
    let to = anchor_of(&graph, &types, &target, Side::Input, &m()).unwrap();
    let (mid_x, mid_y) = ((from.0 + to.0) / 2.0, (from.1 + to.1) / 2.0);
    let hit = link_at(&graph, &types, mid_x, mid_y + 2.0, 6.0, &m()).unwrap();
    assert_eq!(hit.target.node, "n2");
    assert!(link_at(&graph, &types, mid_x, mid_y + 60.0, 6.0, &m()).is_none());
    assert!(link_at(&graph, &types, 0.0, mid_y, 6.0, &m()).is_none());
}

#[test]
fn a_wire_to_a_port_straight_above_swings_out_far_enough_to_clear_the_card() {
    // Output and input in one column, 300 units apart: the curve must leave
    // sideways by at least 60 units, or it runs along the card's edge.
    let [from, first, second, to] = wire_controls((0.0, 300.0), (0.0, 0.0), &m());
    assert_eq!((from, to), ((0.0, 300.0), (0.0, 0.0)));
    assert!(first.0 >= 60.0, "{first:?}");
    assert!(second.0 <= -60.0, "{second:?}");
}

#[test]
fn a_wire_across_a_wide_gap_keeps_half_the_gap_as_its_reach() {
    let [_, first, second, _] = wire_controls((0.0, 0.0), (300.0, 10.0), &m());
    assert_eq!(first, (150.0, 0.0));
    assert_eq!(second, (150.0, 10.0));
}

#[test]
fn a_short_flat_wire_still_leaves_and_arrives_straight() {
    let [_, first, second, _] = wire_controls((0.0, 0.0), (10.0, 0.0), &m());
    assert_eq!(first, (20.0, 0.0));
    assert_eq!(second, (-10.0, 0.0));
}

#[test]
fn layout_follows_the_metrics_it_is_given() {
    let mut wide = m();
    wide.node_width = 300.0;
    wide.header_height = 40.0;
    wide.row_height = 30.0;
    wide.port_inset = 5.0;
    let types = node_type(&["a"], &["out"]);
    let rect = node_rect(&node_at(0.0, 0.0), &types, &wide);
    // A header and two rows: one input, one output.
    assert_eq!((rect.w, rect.h), (300.0, 100.0));
    assert_eq!(input_anchor(&rect, 0, &wide), (5.0, 55.0));
    assert_eq!(
        output_anchor(&rect, output_row(&types, 0), &wide),
        (295.0, 85.0)
    );
}

#[test]
fn the_icon_sits_at_the_right_end_of_the_header_and_is_centred_in_it() {
    let rect = node_rect(&node_at(40.0, 60.0), &node_type(&["a"], &[]), &m());
    let icon = icon_rect(&rect, &m());
    assert_eq!(
        icon.x,
        40.0 + m().node_width - m().title_pad - m().icon_width
    );
    assert_eq!(icon.w, m().icon_width);
    let bars = 3.0;
    assert_eq!(icon.h, bars * m().icon_bar + (bars - 1.0) * m().icon_gap);
    assert_eq!(icon.y + icon.h / 2.0, 60.0 + m().header_height / 2.0);
}

#[test]
fn a_press_on_the_icon_finds_its_node_and_a_press_elsewhere_does_not() {
    let types = [node_type(&["a"], &["out"])];
    let graph = graph_of(node_at(40.0, 60.0));
    let rect = node_rect(&graph.nodes[0], &types[0], &m());
    let icon = icon_rect(&rect, &m());
    let (cx, cy) = (icon.x + icon.w / 2.0, icon.y + icon.h / 2.0);
    assert_eq!(icon_at(&graph, &types, cx, cy, &m()).as_deref(), Some("n1"));
    // The hit area is the whole header height, a little wider than the bars.
    assert!(icon_at(&graph, &types, cx, rect.y + 1.0, &m()).is_some());
    // The title text, the body of the card and empty canvas are not the icon.
    assert!(icon_at(&graph, &types, rect.x + 10.0, cy, &m()).is_none());
    assert!(icon_at(&graph, &types, cx, rect.y + m().header_height + 5.0, &m()).is_none());
    assert!(icon_at(&graph, &types, 900.0, 900.0, &m()).is_none());
    assert!(icon_at(&graph, &[], cx, cy, &m()).is_none());
}

#[test]
fn an_icon_under_another_card_cannot_be_pressed() {
    let types = [node_type(&["a"], &["out"])];
    let mut under = node_at(0.0, 0.0);
    under.id = "under".into();
    let mut over = node_at(60.0, 0.0);
    over.id = "over".into();
    let graph = Graph {
        nodes: vec![under.clone(), over.clone()],
        links: vec![],
    };
    let icon_under = icon_rect(&node_rect(&under, &types[0], &m()), &m());
    let icon_over = icon_rect(&node_rect(&over, &types[0], &m()), &m());
    let y = icon_under.y + icon_under.h / 2.0;
    // The lower card's icon lies inside the upper card's header.
    let x = icon_under.x + icon_under.w / 2.0;
    assert!(icon_at(&graph, &types, x, y, &m()).is_none());
    let x = icon_over.x + icon_over.w / 2.0;
    assert_eq!(icon_at(&graph, &types, x, y, &m()).as_deref(), Some("over"));
}

fn rect(x: f64, y: f64, w: f64, h: f64) -> Rect {
    Rect { x, y, w, h }
}

#[test]
fn overlapping_rectangles_overlap_whichever_way_round() {
    let view = rect(0.0, 0.0, 100.0, 100.0);
    for other in [
        rect(10.0, 10.0, 20.0, 20.0),
        rect(-10.0, -10.0, 20.0, 20.0),
        rect(90.0, 90.0, 50.0, 50.0),
        rect(-50.0, -50.0, 300.0, 300.0),
    ] {
        assert!(view.overlaps(&other), "{other:?}");
        assert!(other.overlaps(&view), "{other:?}");
    }
}

#[test]
fn rectangles_apart_on_either_axis_do_not_overlap() {
    let view = rect(0.0, 0.0, 100.0, 100.0);
    for other in [
        rect(150.0, 10.0, 20.0, 20.0),
        rect(-50.0, 10.0, 20.0, 20.0),
        rect(10.0, 150.0, 20.0, 20.0),
        rect(10.0, -50.0, 20.0, 20.0),
    ] {
        assert!(!view.overlaps(&other), "{other:?}");
        assert!(!other.overlaps(&view), "{other:?}");
    }
}

#[test]
fn rectangles_that_only_touch_do_not_overlap() {
    let view = rect(0.0, 0.0, 100.0, 100.0);
    assert!(!view.overlaps(&rect(100.0, 0.0, 10.0, 10.0)));
    assert!(!view.overlaps(&rect(0.0, 100.0, 10.0, 10.0)));
}

#[test]
fn a_grown_rectangle_reaches_a_neighbour_the_plain_one_misses() {
    let card = rect(100.0, 0.0, 50.0, 50.0);
    let view = rect(0.0, 0.0, 98.0, 100.0);
    assert!(!view.overlaps(&card));
    assert!(view.overlaps(&card.grown(5.0)));
    assert_eq!(
        card.grown(5.0),
        rect(95.0, -5.0, 60.0, 60.0),
        "grown on every side"
    );
}

fn view_box() -> Rect {
    Rect {
        x: 0.0,
        y: 0.0,
        w: 800.0,
        h: 600.0,
    }
}

#[test]
fn a_wire_box_holds_the_whole_curve_and_its_line() {
    // The curve stays inside the hull of its control points, so a box around the
    // four points, and half the thickest line, holds every drawn pixel.
    let metrics = m();
    for (from, to) in [
        ((0.0, 0.0), (300.0, 10.0)),
        ((300.0, 400.0), (0.0, 0.0)),
        ((100.0, 100.0), (100.0, 500.0)),
        ((50.0, 50.0), (50.0, 50.0)),
    ] {
        let bounds = wire_bounds(from, to, &metrics);
        let half = metrics.wire_width.max(metrics.wire_width_selected) / 2.0;
        for point in wire_controls(from, to, &metrics) {
            assert!(point.0 >= bounds.x + half - 1e-9, "{from:?} {to:?}");
            assert!(
                point.0 <= bounds.x + bounds.w - half + 1e-9,
                "{from:?} {to:?}"
            );
            assert!(point.1 >= bounds.y + half - 1e-9, "{from:?} {to:?}");
            assert!(
                point.1 <= bounds.y + bounds.h - half + 1e-9,
                "{from:?} {to:?}"
            );
        }
    }
}

#[test]
fn a_wire_inside_the_view_overlaps_it() {
    let bounds = wire_bounds((100.0, 100.0), (400.0, 300.0), &m());
    assert!(view_box().overlaps(&bounds));
}

#[test]
fn a_wire_crossing_the_view_with_both_ends_outside_overlaps_it() {
    let bounds = wire_bounds((-500.0, 300.0), (1500.0, 320.0), &m());
    assert!(view_box().overlaps(&bounds));
}

#[test]
fn a_wire_wholly_outside_the_view_does_not_overlap_it() {
    let metrics = m();
    for (from, to) in [
        ((-900.0, 100.0), (-300.0, 300.0)),
        ((1000.0, 100.0), (1600.0, 300.0)),
        ((100.0, -900.0), (400.0, -300.0)),
        ((100.0, 900.0), (400.0, 1500.0)),
    ] {
        assert!(
            !view_box().overlaps(&wire_bounds(from, to, &metrics)),
            "{from:?} {to:?}"
        );
    }
}

#[test]
fn a_rectangle_under_a_later_card_is_covered() {
    let area = Rect {
        x: 10.0,
        y: 10.0,
        w: 20.0,
        h: 10.0,
    };
    let beside = Rect {
        x: 30.0,
        y: 10.0,
        w: 50.0,
        h: 50.0,
    };
    let over = Rect {
        x: 25.0,
        y: 0.0,
        w: 50.0,
        h: 50.0,
    };
    assert!(!area.is_covered_by(&[]));
    assert!(!area.is_covered_by(&[beside]));
    assert!(area.is_covered_by(&[beside, over]));
}
