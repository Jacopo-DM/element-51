//! The canvas element: its size, its theme colours and drawing a frame.

use crate::Shared;
use crate::draw::{Frame, draw};
use crate::input::{Gesture, dragged_graph, pending_preview};
use crate::show_error;
use element_51_core::theme::Theme;
use web_sys::Document;

/// Make the canvas bitmap as big as the window, so one pixel is one pixel.
pub(crate) fn fit_canvas(state: &Shared, window: &web_sys::Window) {
    let width = window
        .inner_width()
        .ok()
        .and_then(|v| v.as_f64())
        .unwrap_or(800.0);
    let height = window
        .inner_height()
        .ok()
        .and_then(|v| v.as_f64())
        .unwrap_or(600.0);
    // One canvas pixel per device pixel, or the browser stretches it and blurs it.
    let ratio = window.device_pixel_ratio();
    let state = state.borrow();
    state.canvas.set_width((width * ratio).round() as u32);
    state.canvas.set_height((height * ratio).round() as u32);
}

pub(crate) fn draw_canvas(state: &Shared) {
    let state = state.borrow();
    let preview = dragged_graph(&state);
    let keyboard_link = pending_preview(&state);
    let frame = Frame {
        theme: &state.theme,
        graph: preview
            .as_ref()
            .or_else(|| state.replica.as_ref().map(|replica| replica.graph())),
        types: state
            .replica
            .as_ref()
            .map_or(&[], |replica| replica.node_types()),
        view: &state.view,
        selected: state.selected.as_ref(),
        drag: match (&state.gesture, &keyboard_link) {
            (Gesture::Link(drag), _) => Some(drag),
            (_, Some((drag, _))) => Some(drag),
            _ => None,
        },
        hover: match &keyboard_link {
            Some((_, hit)) => hit.as_ref(),
            None => state.hover.as_ref(),
        },
        focus: state.focus.as_ref(),
        run: state.run.as_ref(),
        pictures: &state.pictures,
        jobs: &state.jobs,
    };
    let result = draw(&state.canvas, &state.context, &frame);
    if let Err(error) = result {
        show_error(&state.document, &format!("Drawing failed: {error:?}"));
    }
}

/// Hand the theme's colours and font to the page's own parts (input boxes, the
/// menu, the error banner) as CSS custom properties on the root element.
pub(crate) fn apply_theme(document: &Document, theme: &Theme) {
    let Some(root) = document.document_element() else {
        return;
    };
    let style: String = theme
        .css_variables()
        .iter()
        .map(|(name, value)| format!("{name}: {value}; "))
        .collect();
    let _ = root.set_attribute("style", &style);
}

/// Show the page. It stays hidden until the first snapshot has set the theme, so
/// a light-theme page never shows the default dark one first.
pub(crate) fn mark_ready(document: &Document) {
    if let Some(root) = document.document_element() {
        root.set_class_name("ready");
    }
}
