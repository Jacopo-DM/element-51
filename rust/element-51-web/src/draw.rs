//! Drawing: a frame from the replica and the state of the pointer.

use element_51_core::layout::{
    HEADER_HEIGHT, PortHit, Side, anchor_of, input_anchor, node_rect, output_anchor,
};
use element_51_core::{Edit, PortRef, PortType, Replica, apply_edit};
use wasm_bindgen::JsValue;
use web_sys::{CanvasRenderingContext2d, HtmlCanvasElement};

const BACKGROUND: &str = "#1e1e1e";
const BODY: &str = "#3a3a3a";
const HEADER: &str = "#555555";
const TEXT: &str = "#d0d0d0";
const WIRE: &str = "#8fa04f";
const ALLOWED: &str = "#6fcf6f";
const REFUSED: &str = "#e06060";
const PORT_SIZE: f64 = 8.0;

/// A link being dragged from an output port.
#[derive(Clone, Debug)]
pub struct Drag {
    pub source: PortRef,
    pub x: f64,
    pub y: f64,
}

fn port_colour(kind: PortType) -> &'static str {
    match kind {
        PortType::Float | PortType::Int => "#f5b461",
        PortType::Str => "#8fbf6f",
        PortType::Bool => "#6fa8dc",
    }
}

/// Draw everything. With no replica yet, only the background.
pub fn draw(
    canvas: &HtmlCanvasElement,
    context: &CanvasRenderingContext2d,
    replica: Option<&Replica>,
    drag: Option<&Drag>,
    hover: Option<&PortHit>,
) -> Result<(), JsValue> {
    context.set_fill_style_str(BACKGROUND);
    context.fill_rect(
        0.0,
        0.0,
        f64::from(canvas.width()),
        f64::from(canvas.height()),
    );
    let Some(replica) = replica else {
        return Ok(());
    };
    let graph = replica.graph();
    let types = replica.node_types();

    context.set_stroke_style_str(WIRE);
    context.set_line_width(2.0);
    for link in &graph.links {
        let from = anchor_of(graph, types, &link.source, Side::Output);
        let to = anchor_of(graph, types, &link.target, Side::Input);
        if let (Some(from), Some(to)) = (from, to) {
            wire(context, from, to);
        }
    }
    if let Some(drag) = drag
        && let Some(from) = anchor_of(graph, types, &drag.source, Side::Output)
    {
        wire(context, from, (drag.x, drag.y));
    }

    context.set_font("13px sans-serif");
    for node in &graph.nodes {
        let Some(node_type) = types.iter().find(|known| known.name == node.node_type) else {
            continue;
        };
        let rect = node_rect(node, node_type);
        context.set_fill_style_str(BODY);
        context.fill_rect(rect.x, rect.y, rect.w, rect.h);
        context.set_fill_style_str(HEADER);
        context.fill_rect(rect.x, rect.y, rect.w, HEADER_HEIGHT);
        context.set_fill_style_str(TEXT);
        context.set_text_align("left");
        context.fill_text(
            &format!("{}  {}", node.id, node_type.name),
            rect.x + 8.0,
            rect.y + 18.0,
        )?;
        for (row, port) in node_type.inputs.iter().enumerate() {
            let (x, y) = input_anchor(&rect, row);
            context.set_fill_style_str(TEXT);
            context.fill_text(&port.name, x + 10.0, y + 4.0)?;
            square(context, x, y, port_colour(port.kind));
        }
        context.set_text_align("right");
        for (row, port) in node_type.outputs.iter().enumerate() {
            let (x, y) = output_anchor(&rect, row);
            context.set_fill_style_str(TEXT);
            context.fill_text(&port.name, x - 10.0, y + 4.0)?;
            square(context, x, y, port_colour(port.kind));
        }
    }
    if let (Some(drag), Some(hit)) = (drag, hover)
        && hit.side == Side::Input
    {
        let target = PortRef {
            node: hit.node.clone(),
            port: hit.port.clone(),
        };
        let edit = Edit::AddLink {
            source: drag.source.clone(),
            target: target.clone(),
        };
        let allowed = apply_edit(graph, types, &edit).is_ok();
        if let Some((x, y)) = anchor_of(graph, types, &target, Side::Input) {
            square(context, x, y, if allowed { ALLOWED } else { REFUSED });
        }
    }
    Ok(())
}

fn square(context: &CanvasRenderingContext2d, x: f64, y: f64, colour: &str) {
    context.set_fill_style_str(colour);
    context.fill_rect(
        x - PORT_SIZE / 2.0,
        y - PORT_SIZE / 2.0,
        PORT_SIZE,
        PORT_SIZE,
    );
}

/// A smooth curve that leaves `from` and arrives at `to` horizontally.
fn wire(context: &CanvasRenderingContext2d, from: (f64, f64), to: (f64, f64)) {
    let reach = ((to.0 - from.0).abs() / 2.0).max(40.0);
    context.begin_path();
    context.move_to(from.0, from.1);
    context.bezier_curve_to(from.0 + reach, from.1, to.0 - reach, to.1, to.0, to.1);
    context.stroke();
}
