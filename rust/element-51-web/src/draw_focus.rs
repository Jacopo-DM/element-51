//! The ring that shows where the keyboard focus is on the canvas.

use crate::draw::Focus;
use crate::draw_card::{card_path, round_rect_path};
use element_51_core::job::{JobButton, job_layout};
use element_51_core::layout::{anchor_of, node_rect};
use element_51_core::rows::type_of;
use element_51_core::theme::Theme;
use element_51_core::view::View;
use element_51_core::{Graph, NodeType, PortRef};
use std::f64::consts::TAU;
use wasm_bindgen::JsValue;
use web_sys::CanvasRenderingContext2d;

/// The ring on the node or port that has the keyboard focus. It is two-tone,
/// the canvas colour under the theme's focus colour, so it shows on a card as
/// well as on the canvas. Its widths are in screen pixels, whatever the zoom.
pub(crate) fn focus_ring(
    context: &CanvasRenderingContext2d,
    (graph, types): (&Graph, &[NodeType]),
    theme: &Theme,
    view: &View,
    focus: &Focus,
) -> Result<(), JsValue> {
    let (colours, metrics) = (&theme.colours, &theme.metrics);
    let pixel = 1.0 / view.zoom();
    let node_id = match focus {
        Focus::Node(id) | Focus::Port { node: id, .. } | Focus::Job { node: id, .. } => id,
    };
    let Some(node) = graph.nodes.iter().find(|node| &node.id == node_id) else {
        return Ok(());
    };
    let Some(node_type) = type_of(graph, types, node) else {
        return Ok(());
    };
    let rect = node_rect(node, &node_type, metrics).grown(FOCUS_GAP * pixel);
    let radius = metrics.card_radius + FOCUS_GAP * pixel;
    for (colour, width) in [
        (&colours.canvas, (FOCUS_WIDTH + 2.0 * FOCUS_HALO) * pixel),
        (&colours.focus, FOCUS_WIDTH * pixel),
    ] {
        context.set_stroke_style_str(colour);
        context.set_line_width(width);
        card_path(context, &rect, radius);
        context.stroke();
    }
    if let Focus::Job { button, .. } = focus
        && let Some(layout) = job_layout(&node_rect(node, &node_type, metrics), &node_type, metrics)
    {
        let button_rect = match button {
            JobButton::Start => layout.start,
            JobButton::Stop => layout.stop,
        }
        .grown(FOCUS_GAP * pixel);
        for (colour, width) in [
            (&colours.canvas, (FOCUS_WIDTH + 2.0 * FOCUS_HALO) * pixel),
            (&colours.focus, FOCUS_WIDTH * pixel),
        ] {
            context.set_stroke_style_str(colour);
            context.set_line_width(width);
            round_rect_path(context, &button_rect, metrics.box_radius);
            context.stroke();
        }
    }
    if let Focus::Port { port, side, .. } = focus {
        let target = PortRef {
            node: node.id.clone(),
            port: port.clone(),
        };
        if let Some((x, y)) = anchor_of(graph, types, &target, *side, metrics) {
            let reach = metrics.port_size / 2.0 + FOCUS_GAP * pixel;
            for (colour, width) in [
                (&colours.canvas, (FOCUS_WIDTH + 2.0 * FOCUS_HALO) * pixel),
                (&colours.focus, FOCUS_WIDTH * pixel),
            ] {
                context.set_stroke_style_str(colour);
                context.set_line_width(width);
                context.begin_path();
                context.arc(x, y, reach, 0.0, TAU)?;
                context.stroke();
            }
        }
    }
    Ok(())
}

pub(crate) const FOCUS_WIDTH: f64 = 2.0;

pub(crate) const FOCUS_HALO: f64 = 1.0;

pub(crate) const FOCUS_GAP: f64 = 3.0;
