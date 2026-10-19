//! Drawing: a frame from the replica and the state of the pointer.

use element_51_core::layout::{
    PortHit, Rect, Side, anchor_of, input_anchor, node_rect, output_anchor, wire_controls,
};
use element_51_core::theme::{Colours, Metrics, Theme};
use element_51_core::view::View;
use element_51_core::{
    Edit, Graph, Link, NodeType, PortRef, PortType, RunResult, Status, apply_edit,
};
use wasm_bindgen::JsValue;
use web_sys::{CanvasRenderingContext2d, HtmlCanvasElement};

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
    pub theme: &'a Theme,
    pub graph: Option<&'a Graph>,
    pub types: &'a [NodeType],
    pub view: &'a View,
    pub selected: Option<&'a Selected>,
    pub drag: Option<&'a Drag>,
    pub hover: Option<&'a PortHit>,
    pub run: Option<&'a RunResult>,
}

/// A wire is drawn in the colour of the port it leaves, as in Antimony.
fn wire_colour<'a>(
    graph: &Graph,
    types: &[NodeType],
    source: &PortRef,
    colours: &'a Colours,
) -> &'a str {
    graph
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
        .map_or(port_colour(PortType::Str, colours), |port| {
            port_colour(port.kind, colours)
        })
}

fn port_colour(kind: PortType, colours: &Colours) -> &str {
    match kind {
        PortType::Float | PortType::Int => &colours.port_number,
        PortType::Str => &colours.port_text,
        PortType::Bool => &colours.port_bool,
    }
}

/// Draw `frame`.
pub fn draw(
    canvas: &HtmlCanvasElement,
    context: &CanvasRenderingContext2d,
    frame: &Frame,
) -> Result<(), JsValue> {
    let Frame {
        theme,
        graph,
        types,
        view,
        selected,
        drag,
        hover,
        run,
    } = *frame;
    context.set_transform(1.0, 0.0, 0.0, 1.0, 0.0, 0.0)?;
    let (colours, metrics) = (&theme.colours, &theme.metrics);
    context.set_fill_style_str(&colours.canvas);
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
    context.set_line_width(metrics.wire_width);
    for link in &graph.links {
        let from = anchor_of(graph, types, &link.source, Side::Output, metrics);
        let to = anchor_of(graph, types, &link.target, Side::Input, metrics);
        if let (Some(from), Some(to)) = (from, to) {
            let chosen = selected == Some(&Selected::Link(link.clone()));
            let colour = wire_colour(graph, types, &link.source, colours);
            context.set_stroke_style_str(if chosen { &colours.selected } else { colour });
            context.set_line_width(if chosen {
                metrics.wire_width_selected
            } else {
                metrics.wire_width
            });
            wire(context, from, to, metrics);
        }
    }
    context.set_line_width(metrics.wire_width);
    if let Some(drag) = drag
        && let Some(from) = anchor_of(graph, types, &drag.source, Side::Output, metrics)
    {
        context.set_stroke_style_str(wire_colour(graph, types, &drag.source, colours));
        wire(context, from, (drag.x, drag.y), metrics);
    }

    context.set_font(&theme.fonts.card);
    for node in &graph.nodes {
        let Some(node_type) = types.iter().find(|known| known.name == node.node_type) else {
            continue;
        };
        let rect = node_rect(node, node_type, metrics);
        context.set_fill_style_str(&colours.card);
        card_path(context, &rect, metrics.card_radius);
        context.fill();
        let outcome = run.and_then(|result| result.outcome_for(&node.id));
        let status = outcome.map(|outcome| outcome.status);
        let (header, text) = match status {
            Some(Status::Error) => (&colours.header_error, &colours.text),
            Some(Status::Skipped) => (&colours.header_skipped, &colours.text_dim),
            _ => (&colours.header, &colours.text),
        };
        context.set_fill_style_str(header);
        // The header takes the card's rounded top corners by clipping to it.
        context.save();
        card_path(context, &rect, metrics.card_radius);
        context.clip();
        context.fill_rect(rect.x, rect.y, rect.w, metrics.header_height);
        context.restore();
        if selected == Some(&Selected::Node(node.id.clone())) {
            context.set_stroke_style_str(&colours.selected);
            context.set_line_width(metrics.selection_width);
            card_path(context, &rect, metrics.card_radius);
            context.stroke();
        }
        context.set_fill_style_str(text);
        context.set_text_align("left");
        context.fill_text(
            &format!("{}  {}", node.id, node_type.name),
            rect.x + metrics.title_pad,
            rect.y + metrics.title_baseline,
        )?;
        for (row, port) in node_type.inputs.iter().enumerate() {
            let (x, y) = input_anchor(&rect, row, metrics);
            context.set_fill_style_str(text);
            context.fill_text(&port.name, x + metrics.label_pad, y + metrics.text_baseline)?;
            square(context, x, y, port_colour(port.kind, colours), metrics);
        }
        context.set_text_align("right");
        for (row, port) in node_type.outputs.iter().enumerate() {
            let (x, y) = output_anchor(&rect, row, metrics);
            context.set_fill_style_str(text);
            context.fill_text(&port.name, x - metrics.label_pad, y + metrics.text_baseline)?;
            if status == Some(Status::Ok)
                && let Some(value) = outcome.and_then(|outcome| outcome.output_text(&port.name))
            {
                // The value sits just left of the port's name, in whatever
                // room the input label on the same row leaves free.
                let name_width = context.measure_text(&port.name)?.width();
                let right = x - metrics.label_pad - name_width - metrics.value_gap;
                let input_width = match node_type.inputs.get(row) {
                    Some(input) => {
                        let label = context.measure_text(&input.name)?.width() + metrics.label_pad;
                        if graph.is_linked(&node.id, &input.name) {
                            label
                        } else {
                            label + metrics.field_gap + metrics.field_width
                        }
                    }
                    None => 0.0,
                };
                let room = right - (rect.x + input_width + 2.0 * metrics.value_gap);
                let shown = fit_text(context, &value, room)?;
                context.set_fill_style_str(&colours.value);
                context.fill_text(&shown, right, y + 4.0)?;
            }
            square(context, x, y, port_colour(port.kind, colours), metrics);
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
        if let Some((x, y)) = anchor_of(graph, types, &target, Side::Input, metrics) {
            let colour = if allowed {
                &colours.allowed
            } else {
                &colours.refused
            };
            square(context, x, y, colour, metrics);
        }
    }
    context.set_transform(1.0, 0.0, 0.0, 1.0, 0.0, 0.0)?;
    Ok(())
}

/// Where the input box for input row `row` sits: just right of its label.
/// Returns `(x, y, width, height)` in graph units. The context's font must
/// already be the theme's card font.
pub fn field_box(
    context: &CanvasRenderingContext2d,
    rect: &Rect,
    row: usize,
    label: &str,
    metrics: &Metrics,
) -> Result<(f64, f64, f64, f64), JsValue> {
    let (_, centre) = input_anchor(rect, row, metrics);
    let label_width = context.measure_text(label)?.width();
    Ok((
        rect.x + metrics.label_pad + label_width + metrics.field_gap,
        centre - metrics.field_height / 2.0,
        metrics.field_width,
        metrics.field_height,
    ))
}

/// Trace the outline of a node card with rounded corners.
fn card_path(context: &CanvasRenderingContext2d, rect: &Rect, radius: f64) {
    let (left, top) = (rect.x, rect.y);
    let (right, bottom) = (rect.x + rect.w, rect.y + rect.h);
    context.begin_path();
    context.move_to(left + radius, top);
    let _ = context.arc_to(right, top, right, bottom, radius);
    let _ = context.arc_to(right, bottom, left, bottom, radius);
    let _ = context.arc_to(left, bottom, left, top, radius);
    let _ = context.arc_to(left, top, right, top, radius);
    context.close_path();
}

fn square(context: &CanvasRenderingContext2d, x: f64, y: f64, colour: &str, metrics: &Metrics) {
    let size = metrics.port_size;
    context.set_fill_style_str(colour);
    context.fill_rect(x - size / 2.0, y - size / 2.0, size, size);
}

/// A smooth curve that leaves `from` and arrives at `to` horizontally.
fn wire(context: &CanvasRenderingContext2d, from: (f64, f64), to: (f64, f64), metrics: &Metrics) {
    let [start, first, second, end] = wire_controls(from, to, metrics);
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
