//! The colour swatch and its reset button beside each series input of a chart
//! card, as page elements over the canvas.

use element_51_core::contrast::{is_low_contrast, ratio};
use element_51_core::display::{line_slot, line_style};
use element_51_core::layout::{Rect, node_rect, swatch_boxes};
use element_51_core::rows::type_of;
use element_51_core::{CardDisplay, Edit, PlacedNode, PortType};
use wasm_bindgen::JsCast;
use web_sys::{Document, Element, HtmlInputElement};

use crate::fields_ui::card_rects;
use crate::input::dragged_graph;
use crate::{Shared, listen, redraw, submit};

/// Make the swatches over the canvas match the graph: one per series input of
/// a chart card, in the colour its line has, and a reset button beside it
/// while the user's own colour is set.
pub(crate) fn sync_swatches(shared: &Shared) {
    let state = shared.borrow();
    let (Some(container), Some(replica)) = (
        state.document.get_element_by_id("swatches"),
        state.replica.as_ref(),
    ) else {
        return;
    };
    let preview = dragged_graph(&state);
    let graph = preview.as_ref().unwrap_or_else(|| replica.graph());
    let (theme, metrics) = (&state.theme, &state.theme.metrics);
    let zoom = state.view.zoom();
    let focused = state.document.active_element().map(|element| element.id());
    let cards = card_rects(graph, replica.node_types(), metrics);
    let mut wanted: Vec<String> = Vec::new();
    for (index, node) in graph.nodes.iter().enumerate() {
        let Some(node_type) = type_of(graph, replica.node_types(), node) else {
            continue;
        };
        if node_type.display != CardDisplay::Chart {
            continue;
        }
        let rect = node_rect(node, &node_type, metrics);
        let above: Vec<Rect> = cards[index + 1..].iter().flatten().copied().collect();
        for (row, port) in node_type.inputs.iter().enumerate() {
            if port.kind != PortType::Series {
                continue;
            }
            let (swatch, reset) = swatch_boxes(&rect, row, metrics);
            let chosen = node.colours.get(&port.name);
            let (kind, _) = line_style(line_slot(graph, node, &node_type, &port.name));
            let shown = chosen
                .cloned()
                .unwrap_or_else(|| theme.port_style(kind, true).colour.clone());
            let swatch_id = format!("swatch-{}-{}", node.id, port.name);
            let reset_id = format!("reset-{}-{}", node.id, port.name);
            wanted.push(swatch_id.clone());
            wanted.push(reset_id.clone());
            let covered = if swatch.is_covered_by(&above) {
                "; visibility: hidden"
            } else {
                ""
            };
            if let Some(field) = swatch_for(
                shared,
                &state.document,
                &container,
                &swatch_id,
                node,
                &port.name,
            ) {
                let (left, top) = state.view.to_screen(swatch.x, swatch.y);
                let _ = field.set_attribute(
                    "style",
                    &format!(
                        "left: {left}px; top: {top}px; width: {}px; height: {}px{covered}",
                        swatch.w * zoom,
                        swatch.h * zoom,
                    ),
                );
                if focused.as_deref() != Some(swatch_id.as_str()) {
                    field.set_value(&shown);
                }
                let hard = is_low_contrast(&shown, &theme.colours.field);
                if hard {
                    let _ = field.set_attribute("data-low", "");
                } else {
                    let _ = field.remove_attribute("data-low");
                }
                let title = match (hard, ratio(&shown, &theme.colours.field)) {
                    (true, Some(ratio)) => format!("Hard to see on this chart ({ratio:.1} to 1)"),
                    _ => format!("Line colour of {}", port.name),
                };
                field.set_title(&title);
            }
            if let Some(button) = reset_for(
                shared,
                &state.document,
                &container,
                &reset_id,
                node,
                &port.name,
            ) {
                let (left, top) = state.view.to_screen(reset.x, reset.y);
                let _ = button.set_attribute(
                    "style",
                    &format!(
                        "left: {left}px; top: {top}px; width: {}px; height: {}px{covered}",
                        reset.w * zoom,
                        reset.h * zoom,
                    ),
                );
                if chosen.is_some() {
                    let _ = button.remove_attribute("hidden");
                } else {
                    let _ = button.set_attribute("hidden", "");
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

/// The colour box with `id`, made (and wired up) if it does not exist yet.
fn swatch_for(
    shared: &Shared,
    document: &Document,
    container: &Element,
    id: &str,
    node: &PlacedNode,
    port: &str,
) -> Option<HtmlInputElement> {
    if let Some(existing) = document.get_element_by_id(id) {
        return existing.dyn_into().ok();
    }
    let field: HtmlInputElement = document.create_element("input").ok()?.dyn_into().ok()?;
    field.set_id(id);
    field.set_type("color");
    let _ = field.set_attribute("aria-label", &format!("Colour of {port} on {}", node.id));
    let _ = field.set_attribute("data-node", &node.id);
    let _ = field.set_attribute("data-port", port);
    let shared = shared.clone();
    let source = field.clone();
    listen(&field, "change", move |_event: web_sys::Event| {
        on_swatch_change(&shared, &source)
    })
    .ok()?;
    container.append_child(&field).ok()?;
    Some(field)
}

/// The reset button with `id`, made (and wired up) if it does not exist yet.
fn reset_for(
    shared: &Shared,
    document: &Document,
    container: &Element,
    id: &str,
    node: &PlacedNode,
    port: &str,
) -> Option<Element> {
    if let Some(existing) = document.get_element_by_id(id) {
        return Some(existing);
    }
    let button = document.create_element("button").ok()?;
    button.set_id(id);
    let _ = button.set_attribute("type", "button");
    let _ = button.set_attribute(
        "aria-label",
        &format!("Use the theme colour for {port} on {}", node.id),
    );
    let _ = button.set_attribute("title", "Use the theme colour");
    button.set_text_content(Some("\u{d7}"));
    let (id, port) = (node.id.clone(), port.to_string());
    let shared = shared.clone();
    listen(&button, "click", move |_event: web_sys::Event| {
        submit(
            &shared,
            Edit::SetColour {
                node: id.clone(),
                port: port.clone(),
                colour: None,
            },
        );
    })
    .ok()?;
    container.append_child(&button).ok()?;
    Some(button)
}

/// The user closed the colour dialog: send the colour, if it is a new one.
fn on_swatch_change(shared: &Shared, field: &HtmlInputElement) {
    let edit = {
        let state = shared.borrow();
        let (Some(node), Some(port), Some(replica)) = (
            field.get_attribute("data-node"),
            field.get_attribute("data-port"),
            state.replica.as_ref(),
        ) else {
            return;
        };
        let colour = field.value();
        let current = replica
            .graph()
            .nodes
            .iter()
            .find(|placed| placed.id == node)
            .and_then(|placed| placed.colours.get(&port));
        (current != Some(&colour)).then_some(Edit::SetColour {
            node,
            port,
            colour: Some(colour),
        })
    };
    match edit {
        Some(edit) => submit(shared, edit),
        None => redraw(shared),
    }
}
