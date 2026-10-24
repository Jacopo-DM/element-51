//! The slider control of a slider card, as a page element over the canvas.

use element_51_core::fields::{field_label, parse_input, slider_values};
use element_51_core::layout::{Rect, node_rect};
use element_51_core::rows::type_of;
use element_51_core::{Edit, PlacedNode, PortType};
use wasm_bindgen::JsCast;
use web_sys::{Document, Element, HtmlInputElement};

use crate::draw::field_box;
use crate::edit_ui::submit_edit;
use crate::fields_ui::card_rects;
use crate::input::dragged_graph;
use crate::{Shared, listen};

/// The port a slider card's control sets.
const VALUE_PORT: &str = "value";

/// Make the sliders over the canvas match the graph: one per slider card whose
/// `value` input is not linked, on that input's row, with the card's range.
pub(crate) fn sync_sliders(shared: &Shared) {
    let state = shared.borrow();
    let (Some(container), Some(replica)) = (
        state.document.get_element_by_id("sliders"),
        state.replica.as_ref(),
    ) else {
        return;
    };
    let preview = dragged_graph(&state);
    let graph = preview.as_ref().unwrap_or_else(|| replica.graph());
    let metrics = &state.theme.metrics;
    let focused = state.document.active_element().map(|element| element.id());
    let cards = card_rects(graph, replica.node_types(), metrics);
    let mut wanted: Vec<String> = Vec::new();
    for (index, node) in graph.nodes.iter().enumerate() {
        let Some(node_type) = type_of(graph, replica.node_types(), node) else {
            continue;
        };
        let Some(shown) = slider_values(node, &node_type) else {
            continue;
        };
        let Some(row) = node_type.inputs.iter().position(|p| p.name == VALUE_PORT) else {
            continue;
        };
        if graph.is_linked(&node.id, VALUE_PORT) {
            continue;
        }
        let rect = node_rect(node, &node_type, metrics);
        let Ok((x, y, width, height)) = field_box(&state.context, &node_type, &rect, row, metrics)
        else {
            continue;
        };
        let above: Vec<Rect> = cards[index + 1..].iter().flatten().copied().collect();
        let covered = Rect {
            x,
            y,
            w: width,
            h: height,
        }
        .is_covered_by(&above);
        let id = format!("slider-{}", node.id);
        wanted.push(id.clone());
        let Some(control) = slider_for(shared, &state.document, &container, &id, node) else {
            continue;
        };
        let (left, top) = state.view.to_screen(x, y);
        let zoom = state.view.zoom();
        let _ = control.set_attribute(
            "style",
            &format!(
                "left: {left}px; top: {top}px; width: {}px; height: {}px{}",
                width * zoom,
                height * zoom,
                if covered { "; visibility: hidden" } else { "" },
            ),
        );
        control.set_min(&shown.minimum.to_string());
        control.set_max(&shown.maximum.to_string());
        control.set_step(&shown.step.to_string());
        if focused.as_deref() != Some(id.as_str()) {
            control.set_value(&shown.value.to_string());
        }
    }
    let children = container.children();
    for index in (0..children.length()).rev() {
        if let Some(child) = children.item(index)
            && !wanted.contains(&child.id())
        {
            child.remove();
        }
    }
}

/// The range input with `id`, made (and wired up) if it does not exist yet.
fn slider_for(
    shared: &Shared,
    document: &Document,
    container: &Element,
    id: &str,
    node: &PlacedNode,
) -> Option<HtmlInputElement> {
    if let Some(existing) = document.get_element_by_id(id) {
        return existing.dyn_into().ok();
    }
    let control: HtmlInputElement = document.create_element("input").ok()?.dyn_into().ok()?;
    control.set_id(id);
    control.set_type("range");
    let _ = control.set_attribute("aria-label", &field_label(&node.id, VALUE_PORT));
    let _ = control.set_attribute("data-node", &node.id);
    let shared = shared.clone();
    let source = control.clone();
    listen(&control, "input", move |_event: web_sys::Event| {
        on_slider_input(&shared, &source)
    })
    .ok()?;
    container.append_child(&control).ok()?;
    Some(control)
}

/// The handle moved: send the new value now, so the graph follows the hand.
/// The edit joins the undo history in `on_slider_change`, not here.
fn on_slider_input(shared: &Shared, control: &HtmlInputElement) {
    let Some(node) = control.get_attribute("data-node") else {
        return;
    };
    let Ok(Some(value)) = parse_input(&control.value(), PortType::Float) else {
        return;
    };
    submit_edit(
        shared,
        Edit::SetValue {
            node,
            port: VALUE_PORT.to_string(),
            value: Some(value),
        },
        false,
    );
}
