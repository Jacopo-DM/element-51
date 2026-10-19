//! Drawing: a frame from the replica and the state of the pointer.

use element_51_core::layout::{
    HEADER_HEIGHT, PortHit, Side, anchor_of, input_anchor, node_rect, output_anchor, wire_controls,
};
use element_51_core::view::View;
use element_51_core::{
    Edit, Graph, Link, NodeType, PortRef, PortType, RunResult, Status, apply_edit,
};
use wasm_bindgen::JsValue;
use web_sys::{CanvasRenderingContext2d, HtmlCanvasElement};

const BACKGROUND: &str = "#1e1e1e";
const BODY: &str = "#3a3a3a";
const HEADER: &str = "#555555";
const HEADER_ERROR: &str = "#8a3a3a";
const HEADER_SKIPPED: &str = "#3f3f3f";
const TEXT_DIM: &str = "#808080";
const VALUE: &str = "#a6e3a6";
const TEXT: &str = "#d0d0d0";
const WIRE: &str = "#8fa04f";
const SELECTED: &str = "#e8e8e8";
const ALLOWED: &str = "#6fcf6f";
const REFUSED: &str = "#e06060";
const PORT_SIZE: f64 = 8.0;
/// How round the corners of a node card are, in graph units.
const CARD_RADIUS: f64 = 8.0;
/// How round the corners of a port marker are, in graph units.
const PORT_RADIUS: f64 = 3.0;
const VALUE_GAP: f64 = 8.0;
/// The font every label on a node is set in.
pub const FONT: &str = "13px sans-serif";
/// Width of an input box on a node, in graph units.
pub const FIELD_WIDTH: f64 = 70.0;
/// Height of an input box on a node, in graph units.
pub const FIELD_HEIGHT: f64 = 18.0;
/// Space between an input's label and its box.
const FIELD_GAP: f64 = 6.0;

/// A link being dragged from an output port. `x` and `y` are graph
/// coordinates.
#[derive(Clone, Debug)]
pub struct Drag {
    pub source: PortRef,
    pub x: f64,
    pub y: f64,
}

/// What the user has selected.
#[derive(Clone, Debug, PartialEq)]
pub enum Selected {
    Node(String),
    Link(Link),
}

/// Everything one frame shows. With no graph yet, only the background.
pub struct Frame<'a> {
    pub graph: Option<&'a Graph>,
    pub types: &'a [NodeType],
    pub view: &'a View,
    pub selected: Option<&'a Selected>,
    pub drag: Option<&'a Drag>,
    pub hover: Option<&'a PortHit>,
    pub run: Option<&'a RunResult>,
}

fn port_colour(kind: PortType) -> &'static str {
    match kind {
        PortType::Float | PortType::Int => "#f5b461",
        PortType::Str => "#8fbf6f",
        PortType::Bool => "#6fa8dc",
    }
}

/// Draw `frame`.
pub fn draw(
    canvas: &HtmlCanvasElement,
    context: &CanvasRenderingContext2d,
    frame: &Frame,
) -> Result<(), JsValue> {
    let Frame {
        graph,
        types,
        view,
        selected,
        drag,
        hover,
        run,
    } = *frame;
    context.set_transform(1.0, 0.0, 0.0, 1.0, 0.0, 0.0)?;
    context.set_fill_style_str(BACKGROUND);
    context.fill_rect(
        0.0,
        0.0,
        f64::from(canvas.width()),
        f64::from(canvas.height()),
    );
    let Some(graph) = graph else {
        return Ok(());
    };
    context.set_transform(
        view.zoom(),
        0.0,
        0.0,
        view.zoom(),
        view.pan_x(),
        view.pan_y(),
    )?;

    context.set_line_cap("round");
    context.set_line_join("round");
    context.set_line_width(2.0);
    for link in &graph.links {
        let from = anchor_of(graph, types, &link.source, Side::Output);
        let to = anchor_of(graph, types, &link.target, Side::Input);
        if let (Some(from), Some(to)) = (from, to) {
            let chosen = selected == Some(&Selected::Link(link.clone()));
            context.set_stroke_style_str(if chosen { SELECTED } else { WIRE });
            context.set_line_width(if chosen { 3.0 } else { 2.0 });
            wire(context, from, to);
        }
    }
    context.set_stroke_style_str(WIRE);
    context.set_line_width(2.0);
    if let Some(drag) = drag
        && let Some(from) = anchor_of(graph, types, &drag.source, Side::Output)
    {
        wire(context, from, (drag.x, drag.y));
    }

    context.set_font(FONT);
    for node in &graph.nodes {
        let Some(node_type) = types.iter().find(|known| known.name == node.node_type) else {
            continue;
        };
        let rect = node_rect(node, node_type);
        context.set_fill_style_str(BODY);
        card_path(context, &rect);
        context.fill();
        let outcome = run.and_then(|result| result.outcome_for(&node.id));
        let status = outcome.map(|outcome| outcome.status);
        let (header, text) = match status {
            Some(Status::Error) => (HEADER_ERROR, TEXT),
            Some(Status::Skipped) => (HEADER_SKIPPED, TEXT_DIM),
            _ => (HEADER, TEXT),
        };
        context.set_fill_style_str(header);
        // The header takes the card's rounded top corners by clipping to it.
        context.save();
        card_path(context, &rect);
        context.clip();
        context.fill_rect(rect.x, rect.y, rect.w, HEADER_HEIGHT);
        context.restore();
        if selected == Some(&Selected::Node(node.id.clone())) {
            context.set_stroke_style_str(SELECTED);
            context.set_line_width(2.0);
            card_path(context, &rect);
            context.stroke();
        }
        context.set_fill_style_str(text);
        context.set_text_align("left");
        context.fill_text(
            &format!("{}  {}", node.id, node_type.name),
            rect.x + 8.0,
            rect.y + 18.0,
        )?;
        for (row, port) in node_type.inputs.iter().enumerate() {
            let (x, y) = input_anchor(&rect, row);
            context.set_fill_style_str(text);
            context.fill_text(&port.name, x + 10.0, y + 4.0)?;
            square(context, x, y, port_colour(port.kind));
        }
        context.set_text_align("right");
        for (row, port) in node_type.outputs.iter().enumerate() {
            let (x, y) = output_anchor(&rect, row);
            context.set_fill_style_str(text);
            context.fill_text(&port.name, x - 10.0, y + 4.0)?;
            if status == Some(Status::Ok)
                && let Some(value) = outcome.and_then(|outcome| outcome.output_text(&port.name))
            {
                // The value sits just left of the port's name, in whatever
                // room the input label on the same row leaves free.
                let name_width = context.measure_text(&port.name)?.width();
                let right = x - 10.0 - name_width - VALUE_GAP;
                let input_width = match node_type.inputs.get(row) {
                    Some(input) => {
                        let label = context.measure_text(&input.name)?.width() + 10.0;
                        if graph.is_linked(&node.id, &input.name) {
                            label
                        } else {
                            label + FIELD_GAP + FIELD_WIDTH
                        }
                    }
                    None => 0.0,
                };
                let room = right - (rect.x + input_width + 2.0 * VALUE_GAP);
                let shown = fit_text(context, &value, room)?;
                context.set_fill_style_str(VALUE);
                context.fill_text(&shown, right, y + 4.0)?;
            }
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
    context.set_transform(1.0, 0.0, 0.0, 1.0, 0.0, 0.0)?;
    Ok(())
}

/// Where the input box for input row `row` sits: just right of its label.
/// Returns `(x, y, width, height)` in graph units. The font must already be
/// `FONT`.
pub fn field_box(
    context: &CanvasRenderingContext2d,
    rect: &element_51_core::layout::Rect,
    row: usize,
    label: &str,
) -> Result<(f64, f64, f64, f64), JsValue> {
    let (_, centre) = input_anchor(rect, row);
    let label_width = context.measure_text(label)?.width();
    Ok((
        rect.x + 10.0 + label_width + FIELD_GAP,
        centre - FIELD_HEIGHT / 2.0,
        FIELD_WIDTH,
        FIELD_HEIGHT,
    ))
}

/// Trace the outline of a node card with rounded corners.
fn card_path(context: &CanvasRenderingContext2d, rect: &element_51_core::layout::Rect) {
    let (left, top) = (rect.x, rect.y);
    let (right, bottom) = (rect.x + rect.w, rect.y + rect.h);
    context.begin_path();
    context.move_to(left + CARD_RADIUS, top);
    let _ = context.arc_to(right, top, right, bottom, CARD_RADIUS);
    let _ = context.arc_to(right, bottom, left, bottom, CARD_RADIUS);
    let _ = context.arc_to(left, bottom, left, top, CARD_RADIUS);
    let _ = context.arc_to(left, top, right, top, CARD_RADIUS);
    context.close_path();
}

fn square(context: &CanvasRenderingContext2d, x: f64, y: f64, colour: &str) {
    let half = PORT_SIZE / 2.0;
    let (left, top, right, bottom) = (x - half, y - half, x + half, y + half);
    context.set_fill_style_str(colour);
    context.begin_path();
    context.move_to(left + PORT_RADIUS, top);
    let _ = context.arc_to(right, top, right, bottom, PORT_RADIUS);
    let _ = context.arc_to(right, bottom, left, bottom, PORT_RADIUS);
    let _ = context.arc_to(left, bottom, left, top, PORT_RADIUS);
    let _ = context.arc_to(left, top, right, top, PORT_RADIUS);
    context.close_path();
    context.fill();
}

/// A smooth curve that leaves `from` and arrives at `to` horizontally.
fn wire(context: &CanvasRenderingContext2d, from: (f64, f64), to: (f64, f64)) {
    let [start, first, second, end] = wire_controls(from, to);
    context.begin_path();
    context.move_to(start.0, start.1);
    context.bezier_curve_to(first.0, first.1, second.0, second.1, end.0, end.1);
    context.stroke();
}

/// `text`, shortened with an ellipsis until it is at most `room` pixels wide.
/// Empty if even the ellipsis does not fit.
fn fit_text(context: &CanvasRenderingContext2d, text: &str, room: f64) -> Result<String, JsValue> {
    if context.measure_text(text)?.width() <= room {
        return Ok(text.to_string());
    }
    let mut chars: Vec<char> = text.chars().collect();
    while !chars.is_empty() {
        chars.pop();
        let candidate = format!("{}\u{2026}", chars.iter().collect::<String>());
        if context.measure_text(&candidate)?.width() <= room {
            return Ok(candidate);
        }
    }
    Ok(String::new())
}
