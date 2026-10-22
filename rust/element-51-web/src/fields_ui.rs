//! The input boxes laid over the canvas, one per unlinked input.

use element_51_core::PortType;
use element_51_core::fields::{field_label, format_value, parse_input};
use element_51_core::layout::{Rect, Side, node_rect, output_row};
use element_51_core::rows::{find_input, type_of};
use element_51_core::theme::Metrics;
use element_51_core::{Edit, Graph, NodeType, PlacedNode, Port, Status};
use wasm_bindgen::JsCast;
use web_sys::{Document, Element, HtmlInputElement};

use crate::a11y_ui::ensure_ports;
use crate::a11y_ui::expand_for;
use crate::a11y_ui::focus;
use crate::a11y_ui::node_element_id;
use crate::a11y_ui::port_element_id;
use crate::draw::field_box;
use crate::input::dragged_graph;
use crate::{Shared, State, listen, redraw, show_error, submit};

/// The box of every node's card, in graph order; None for a node of an unknown
/// type. A page element over the canvas is hidden where a card after its own
/// covers it.
pub(crate) fn card_rects(
    graph: &Graph,
    types: &[NodeType],
    metrics: &Metrics,
) -> Vec<Option<Rect>> {
    graph
        .nodes
        .iter()
        .map(|node| {
            type_of(graph, types, node).map(|node_type| node_rect(node, &node_type, metrics))
        })
        .collect()
}

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
    let cards = card_rects(graph, replica.node_types(), metrics);
    for (index, node) in graph.nodes.iter().enumerate() {
        let Some(node_type) = type_of(graph, replica.node_types(), node) else {
            continue;
        };
        let rect = node_rect(node, &node_type, metrics);
        // The boxes are page elements above the whole canvas, so a card drawn
        // later cannot hide them by painting over them; hide them instead.
        let above: Vec<Rect> = cards[index + 1..].iter().flatten().copied().collect();
        for (row, port) in node_type.inputs.iter().enumerate() {
            if graph.is_linked(&node.id, &port.name)
                || matches!(port.kind, PortType::Image | PortType::Series)
            {
                continue;
            }
            let id = format!("field-{}-{}", node.id, port.name);
            wanted.push(id.clone());
            let Ok((x, y, width, height)) =
                field_box(&state.context, &node_type, &rect, row, metrics)
            else {
                continue;
            };
            let covered = Rect {
                x,
                y,
                w: width,
                h: height,
            }
            .is_covered_by(&above);
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
                     font-size: {}px; padding: 0 {}px; border-radius: {}px{}",
                    state.theme.fonts.field_px * zoom,
                    metrics.box_pad * zoom,
                    metrics.box_radius * zoom,
                    if covered { "; visibility: hidden" } else { "" },
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
    let _ = field.set_attribute("aria-label", &field_label(&node.id, &port.name));
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
            .and_then(|node_type| find_input(node_type, &port))
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

/// Escape in a value box that the keyboard opened: put back what it held.
pub(crate) fn restore_field(field: &HtmlInputElement) {
    if let Some(original) = field.get_attribute("data-original") {
        if field.type_() == "checkbox" {
            field.set_checked(original == "true");
        } else {
            field.set_value(&original);
        }
    }
}

/// A value box that the keyboard opened was left (Enter or Escape): give the
/// focus back to the input port it came from. A box the mouse focused has no
/// such memory, and nothing happens.
pub(crate) fn return_from_field(shared: &Shared, field: &HtmlInputElement) {
    if field.get_attribute("data-original").is_none() {
        return;
    }
    let _ = field.remove_attribute("data-original");
    let (Some(node), Some(port)) = (
        field.get_attribute("data-node"),
        field.get_attribute("data-port"),
    ) else {
        return;
    };
    let document = shared.borrow().document.clone();
    let Some(node_item) = document.get_element_by_id(&node_element_id(&node)) else {
        return;
    };
    let _ = ensure_ports(shared, &node_item);
    if let Some(item) = document.get_element_by_id(&port_element_id(&node, &port, Side::Input)) {
        expand_for(&item);
        focus(&item);
    }
}

/// The hover text for the value box under the graph point `(x, y)`, if that box
/// holds a picture or a series: what its short text is a summary of. A linked
/// input also says which output it is linked to.
pub(crate) fn tip_at(state: &State, x: f64, y: f64) -> Option<String> {
    let replica = state.replica.as_ref()?;
    let run = state.run.as_ref()?;
    let graph = replica.graph();
    let metrics = &state.theme.metrics;
    for node in &graph.nodes {
        let Some(node_type) = type_of(graph, replica.node_types(), node) else {
            continue;
        };
        let rect = node_rect(node, &node_type, metrics);
        if x < rect.x || x > rect.x + rect.w || y < rect.y || y > rect.y + rect.h {
            continue;
        }
        let inputs = node_type.inputs.iter().enumerate().map(|(row, port)| {
            let tip = graph
                .links
                .iter()
                .find(|link| link.target.node == node.id && link.target.port == port.name)
                .and_then(|link| {
                    let outcome = run.outcome_for(&link.source.node)?;
                    let tip = (outcome.status == Status::Ok)
                        .then(|| outcome.output_tooltip(&link.source.port))??;
                    Some(format!(
                        "{tip}, from {}.{}",
                        link.source.node, link.source.port
                    ))
                });
            (row, tip)
        });
        let outputs = node_type.outputs.iter().enumerate().map(|(index, port)| {
            let tip = run
                .outcome_for(&node.id)
                .filter(|outcome| outcome.status == Status::Ok)
                .and_then(|outcome| outcome.output_tooltip(&port.name));
            (output_row(&node_type, index), tip)
        });
        state.context.set_font(&state.theme.fonts.card);
        for (row, tip) in inputs.chain(outputs) {
            let Some(tip) = tip else {
                continue;
            };
            let Ok((left, top, width, height)) =
                field_box(&state.context, &node_type, &rect, row, metrics)
            else {
                continue;
            };
            if x >= left && x <= left + width && y >= top && y <= top + height {
                return Some(tip);
            }
        }
    }
    None
}
