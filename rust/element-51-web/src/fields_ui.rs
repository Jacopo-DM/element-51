//! The input boxes laid over the canvas, one per unlinked input.

use element_51_core::PortType;
use element_51_core::fields::{format_value, parse_input};
use element_51_core::layout::node_rect;
use element_51_core::{Edit, PlacedNode, Port};
use wasm_bindgen::JsCast;
use web_sys::{Document, Element, HtmlInputElement};

use crate::draw::field_box;
use crate::input::dragged_graph;
use crate::{Shared, listen, redraw, show_error, submit};

/// Make the input boxes over the canvas match the graph: one per unlinked
/// input, placed on its node's row under the current pan and zoom.
pub(crate) fn sync_fields(shared: &Shared) {
    let state = shared.borrow();
    let (Some(container), Some(replica)) = (
        state.document.get_element_by_id("fields"),
        state.replica.as_ref(),
    ) else {
        return;
    };
    let preview = dragged_graph(&state);
    let graph = preview.as_ref().unwrap_or_else(|| replica.graph());
    state.context.set_font(&state.theme.fonts.card);
    let metrics = &state.theme.metrics;
    let zoom = state.view.zoom();
    let focused = state.document.active_element().map(|element| element.id());
    let mut wanted: Vec<String> = Vec::new();
    for node in &graph.nodes {
        let Some(node_type) = replica
            .node_types()
            .iter()
            .find(|known| known.name == node.node_type)
        else {
            continue;
        };
        let rect = node_rect(node, node_type, metrics);
        for (row, port) in node_type.inputs.iter().enumerate() {
            if graph.is_linked(&node.id, &port.name) {
                continue;
            }
            let id = format!("field-{}-{}", node.id, port.name);
            wanted.push(id.clone());
            let Ok((x, y, width, height)) =
                field_box(&state.context, node_type, &rect, row, metrics)
            else {
                continue;
            };
            let (left, top) = state.view.to_screen(x, y);
            let Some(field) = field_for(shared, &state.document, &container, &id, node, port)
            else {
                continue;
            };
            // A bool is a small checkbox at the start of its box; any other
            // kind of input fills the whole box.
            let (left, width, height) = if port.kind == PortType::Bool {
                let side = height * 0.8;
                (left + metrics.box_pad * zoom, side * zoom, side * zoom)
            } else {
                (left, width * zoom, height * zoom)
            };
            let top = if port.kind == PortType::Bool {
                top + (metrics.field_height * 0.1) * zoom
            } else {
                top
            };
            let _ = field.set_attribute(
                "style",
                &format!(
                    "left: {left}px; top: {top}px; width: {width}px; height: {height}px; \
                     font-size: {}px; padding: 0 {}px; border-radius: {}px",
                    state.theme.fonts.field_px * zoom,
                    metrics.box_pad * zoom,
                    metrics.box_radius * zoom,
                ),
            );
            let own = node.values.get(&port.name);
            let default = node_type.defaults.get(&port.name);
            if port.kind == PortType::Bool {
                let on = own.or(default).and_then(|value| value.as_bool()) == Some(true);
                field.set_checked(on);
            } else {
                field.set_placeholder(&default.map_or("value".to_string(), format_value));
                if focused.as_deref() != Some(id.as_str()) {
                    field.set_value(&own.map(format_value).unwrap_or_default());
                }
            }
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

/// The input box with `id`, made (and wired up) if it does not exist yet.
fn field_for(
    shared: &Shared,
    document: &Document,
    container: &Element,
    id: &str,
    node: &PlacedNode,
    port: &Port,
) -> Option<HtmlInputElement> {
    if let Some(existing) = document.get_element_by_id(id) {
        return existing.dyn_into().ok();
    }
    let field: HtmlInputElement = document.create_element("input").ok()?.dyn_into().ok()?;
    field.set_id(id);
    let kind = if port.kind == PortType::Bool {
        "checkbox"
    } else {
        "text"
    };
    field.set_type(kind);
    let _ = field.set_attribute("data-node", &node.id);
    let _ = field.set_attribute("data-port", &port.name);
    let _ = field.set_attribute("spellcheck", "false");
    let shared = shared.clone();
    let source = field.clone();
    listen(&field, "change", move |_event: web_sys::Event| {
        on_field_change(&shared, &source)
    })
    .ok()?;
    container.append_child(&field).ok()?;
    Some(field)
}

/// The user finished typing in a box: read it, and send the new value.
fn on_field_change(shared: &Shared, field: &HtmlInputElement) {
    let edit = {
        let state = shared.borrow();
        let (Some(node), Some(port), Some(replica)) = (
            field.get_attribute("data-node"),
            field.get_attribute("data-port"),
            state.replica.as_ref(),
        ) else {
            return;
        };
        let placed = replica
            .graph()
            .nodes
            .iter()
            .find(|placed| placed.id == node);
        let kind = placed
            .and_then(|placed| {
                replica
                    .node_types()
                    .iter()
                    .find(|known| known.name == placed.node_type)
            })
            .and_then(|node_type| node_type.inputs.iter().find(|input| input.name == port))
            .map(|input| input.kind);
        let (Some(placed), Some(kind)) = (placed, kind) else {
            return;
        };
        let text = if kind == PortType::Bool {
            field.checked().to_string()
        } else {
            field.value()
        };
        match parse_input(&text, kind) {
            Ok(value) if value.as_ref() == placed.values.get(&port) => None,
            Ok(value) => Some(Edit::SetValue { node, port, value }),
            Err(reason) => {
                show_error(&state.document, &reason);
                None
            }
        }
    };
    match edit {
        Some(edit) => submit(shared, edit),
        // Nothing to send: put the box back to what the graph holds.
        None => redraw(shared),
    }
}
