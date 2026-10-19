//! Pan and zoom maths: what is on screen and where a click lands in the graph.

use element_51_core::view::{MAX_ZOOM, MIN_ZOOM, View};

fn close(a: (f64, f64), b: (f64, f64)) -> bool {
    (a.0 - b.0).abs() < 1e-9 && (a.1 - b.1).abs() < 1e-9
}

#[test]
fn the_default_view_maps_screen_to_graph_unchanged() {
    let view = View::default();
    assert_eq!(view.to_graph(12.0, 34.0), (12.0, 34.0));
    assert_eq!(view.to_screen(12.0, 34.0), (12.0, 34.0));
}

#[test]
fn panning_moves_the_graph_with_the_pointer() {
    let mut view = View::default();
    view.pan_by(30.0, -10.0);
    assert_eq!(view.to_screen(0.0, 0.0), (30.0, -10.0));
    assert_eq!(view.to_graph(30.0, -10.0), (0.0, 0.0));
}

#[test]
fn to_graph_undoes_to_screen_at_any_pan_and_zoom() {
    let mut view = View::default();
    view.pan_by(-40.0, 25.0);
    view.zoom_at(300.0, 200.0, 1.7);
    let screen = view.to_screen(123.0, -45.0);
    assert!(close(view.to_graph(screen.0, screen.1), (123.0, -45.0)));
}

#[test]
fn zooming_keeps_the_point_under_the_pointer_still() {
    let mut view = View::default();
    view.pan_by(50.0, 60.0);
    let under = view.to_graph(300.0, 200.0);
    view.zoom_at(300.0, 200.0, 2.0);
    assert!(close(view.to_graph(300.0, 200.0), under));
    assert_eq!(view.zoom(), 2.0);
}

#[test]
fn zoom_stops_at_the_limits() {
    let mut view = View::default();
    view.zoom_at(0.0, 0.0, 1000.0);
    assert_eq!(view.zoom(), MAX_ZOOM);
    view.zoom_at(0.0, 0.0, 1e-9);
    assert_eq!(view.zoom(), MIN_ZOOM);
}

#[test]
fn a_zoom_that_changes_nothing_leaves_the_pan_alone() {
    let mut view = View::default();
    view.zoom_at(0.0, 0.0, 1000.0);
    let before = (view.pan_x(), view.pan_y());
    view.zoom_at(77.0, 88.0, 2.0);
    assert_eq!((view.pan_x(), view.pan_y()), before);
}
