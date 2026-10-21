//! Links between ports: their colour, and the wires drawn under and over the cards.

use crate::draw::Selected;
use element_51_core::layout::{Rect, Side, anchor_of, wire_bounds, wire_controls};
use element_51_core::theme::{Metrics, Theme};
use element_51_core::{Graph, NodeType, PortRef, PortType};
use wasm_bindgen::JsValue;
use web_sys::CanvasRenderingContext2d;

/// Every link of the graph as a wire, in the colour of the port it leaves, or the
/// selected colour and width for the selected one.
pub(crate) fn draw_links(
    context: &CanvasRenderingContext2d,
    theme: &Theme,
    (graph, types): (&Graph, &[NodeType]),
    selected: Option<&Selected>,
    visible: &Rect,
) -> Result<(), JsValue> {
    let (colours, metrics) = (&theme.colours, &theme.metrics);
    context.set_line_cap("round");
    context.set_line_join("round");
    for link in &graph.links {
        let from = anchor_of(graph, types, &link.source, Side::Output, metrics);
        let to = anchor_of(graph, types, &link.target, Side::Input, metrics);
        if let (Some(from), Some(to)) = (from, to) {
            // A wire that cannot reach the view draws nothing in it; skipping it
            // keeps a frame's cost to what is on screen. See DESIGN.md, speed-up-frame.
            if !visible.overlaps(&wire_bounds(from, to, metrics)) {
                continue;
            }
            let chosen = selected == Some(&Selected::Link(link.clone()));
            let colour = wire_colour(graph, types, &link.source, theme);
            context.set_stroke_style_str(if chosen {
                &colours.selected_wire
            } else {
                colour
            });
            context.set_line_width(if chosen {
                metrics.wire_width_selected
            } else {
                metrics.wire_width
            });
            wire(context, from, to, metrics);
        }
    }
    Ok(())
}

/// A wire is drawn in the colour of the port it leaves, as in Antimony.
pub(crate) fn wire_colour<'a>(
    graph: &Graph,
    types: &[NodeType],
    source: &PortRef,
    theme: &'a Theme,
) -> &'a str {
    let style = graph
        .nodes
        .iter()
        .find(|node| node.id == source.node)
        .and_then(|node| types.iter().find(|known| known.name == node.node_type))
        .and_then(|node_type| {
            node_type
                .outputs
                .iter()
                .find(|port| port.name == source.port)
        })
        .map_or(PortType::Str, |port| port.kind);
    &theme.port_style(style, true).colour
}

/// A smooth curve that leaves `from` and arrives at `to` horizontally.
pub(crate) fn wire(
    context: &CanvasRenderingContext2d,
    from: (f64, f64),
    to: (f64, f64),
    metrics: &Metrics,
) {
    let [start, first, second, end] = wire_controls(from, to, metrics);
    context.begin_path();
    context.move_to(start.0, start.1);
    context.bezier_curve_to(first.0, first.1, second.0, second.1, end.0, end.1);
    context.stroke();
}
