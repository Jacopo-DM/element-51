//! Drawing: a frame from the replica and the state of the pointer.
//!
//! Every colour, size and font comes from the `Theme` in the frame.

use element_51_core::layout::{
    PortHit, Rect, Side, anchor_of, input_anchor, node_rect, output_anchor, output_row,
    wire_controls,
};
use element_51_core::theme::{Colours, Metrics, Theme};
use element_51_core::view::View;
use element_51_core::{
    Edit, Graph, Link, NodeType, PlacedNode, PortRef, PortType, RunResult, Status, apply_edit,
};
use std::f64::consts::TAU;
use wasm_bindgen::JsValue;
use web_sys::{CanvasRenderingContext2d, HtmlCanvasElement};

/// The grid is left out when its dots would be closer than this on screen.
const MIN_GRID_PIXELS: f64 = 8.0;

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

/// Where a card's columns sit: port names are right-aligned against
/// `label_right`, and the value boxes run from `box_left` to `box_right`.
#[derive(Clone, Copy, Debug)]
pub struct Columns {
    pub label_right: f64,
    pub box_left: f64,
    pub box_right: f64,
}

/// The columns of the card for `node_type`. The context's font must already be
/// the theme's card font.
pub fn columns(
    context: &CanvasRenderingContext2d,
    node_type: &NodeType,
    rect: &Rect,
    metrics: &Metrics,
) -> Result<Columns, JsValue> {
    let mut widest: f64 = 0.0;
    for port in node_type.inputs.iter().chain(&node_type.outputs) {
        widest = widest.max(context.measure_text(&port.name)?.width());
    }
    let edge = metrics.port_inset + metrics.port_size / 2.0 + metrics.port_gap;
    let label_right = rect.x + edge + widest;
    Ok(Columns {
        label_right,
        box_left: label_right + metrics.box_gap,
        box_right: rect.x + rect.w - edge,
    })
}

/// Where the value box of row `row` sits, as `(x, y, width, height)` in graph
/// units. The context's font must already be the theme's card font.
pub fn field_box(
    context: &CanvasRenderingContext2d,
    node_type: &NodeType,
    rect: &Rect,
    row: usize,
    metrics: &Metrics,
) -> Result<(f64, f64, f64, f64), JsValue> {
    let columns = columns(context, node_type, rect, metrics)?;
    let (_, centre) = input_anchor(rect, row, metrics);
    Ok((
        columns.box_left,
        centre - metrics.field_height / 2.0,
        columns.box_right - columns.box_left,
        metrics.field_height,
    ))
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
    let (colours, metrics) = (&theme.colours, &theme.metrics);
    context.set_transform(1.0, 0.0, 0.0, 1.0, 0.0, 0.0)?;
    context.set_fill_style_str(&colours.canvas);
    let (width, height) = (f64::from(canvas.width()), f64::from(canvas.height()));
    context.fill_rect(0.0, 0.0, width, height);
    context.set_transform(
        view.zoom(),
        0.0,
        0.0,
        view.zoom(),
        view.pan_x(),
        view.pan_y(),
    )?;
    grid(context, (width, height), view, theme)?;
    let Some(graph) = graph else {
        context.set_transform(1.0, 0.0, 0.0, 1.0, 0.0, 0.0)?;
        return Ok(());
    };

    for node in &graph.nodes {
        let Some(node_type) = types.iter().find(|known| known.name == node.node_type) else {
            continue;
        };
        let is_selected = selected == Some(&Selected::Node(node.id.clone()));
        card(context, theme, graph, (node, node_type), run, is_selected)?;
    }

    context.set_line_cap("round");
    context.set_line_join("round");
    for link in &graph.links {
        let from = anchor_of(graph, types, &link.source, Side::Output, metrics);
        let to = anchor_of(graph, types, &link.target, Side::Input, metrics);
        if let (Some(from), Some(to)) = (from, to) {
            let chosen = selected == Some(&Selected::Link(link.clone()));
            let colour = wire_colour(graph, types, &link.source, colours);
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
    context.set_line_width(metrics.wire_width);
    if let Some(drag) = drag
        && let Some(from) = anchor_of(graph, types, &drag.source, Side::Output, metrics)
    {
        context.set_stroke_style_str(wire_colour(graph, types, &drag.source, colours));
        wire(context, from, (drag.x, drag.y), metrics);
    }

    // Ports go over the wires, so a wire ends at the port's square.
    for node in &graph.nodes {
        let Some(node_type) = types.iter().find(|known| known.name == node.node_type) else {
            continue;
        };
        let rect = node_rect(node, node_type, metrics);
        for (row, port) in node_type.inputs.iter().enumerate() {
            let (x, y) = input_anchor(&rect, row, metrics);
            square(context, x, y, port_colour(port.kind, colours), metrics);
        }
        for (index, port) in node_type.outputs.iter().enumerate() {
            let (x, y) = output_anchor(&rect, output_row(node_type, index), metrics);
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

/// The dot grid over the visible part of the canvas. The transform is the
/// view's, so the grid pans and zooms with the graph.
fn grid(
    context: &CanvasRenderingContext2d,
    size: (f64, f64),
    view: &View,
    theme: &Theme,
) -> Result<(), JsValue> {
    let metrics = &theme.metrics;
    let step = metrics.grid_step;
    if step * view.zoom() < MIN_GRID_PIXELS {
        return Ok(());
    }
    let (left, top) = view.to_graph(0.0, 0.0);
    let (right, bottom) = view.to_graph(size.0, size.1);
    context.set_fill_style_str(&theme.colours.dot);
    let mut y = (top / step).floor() * step;
    while y <= bottom {
        let mut x = (left / step).floor() * step;
        while x <= right {
            context.begin_path();
            context.arc(x, y, metrics.dot_radius, 0.0, TAU)?;
            context.fill();
            x += step;
        }
        y += step;
    }
    Ok(())
}

/// One node card: body, title bar, rows and value boxes. Ports and wires are
/// drawn afterwards, over it.
fn card(
    context: &CanvasRenderingContext2d,
    theme: &Theme,
    graph: &Graph,
    (node, node_type): (&PlacedNode, &NodeType),
    run: Option<&RunResult>,
    selected: bool,
) -> Result<(), JsValue> {
    let (colours, metrics) = (&theme.colours, &theme.metrics);
    let rect = node_rect(node, node_type, metrics);
    let outcome = run.and_then(|result| result.outcome_for(&node.id));
    let status = outcome.map(|outcome| outcome.status);
    let (header, title, label) = match status {
        Some(Status::Error) => (&colours.header_error, &colours.title, &colours.label),
        Some(Status::Skipped) => (
            &colours.header_skipped,
            &colours.text_dim,
            &colours.text_dim,
        ),
        _ => (&colours.header, &colours.title, &colours.label),
    };

    context.set_fill_style_str(&colours.card);
    card_path(context, &rect, metrics.card_radius);
    context.fill();
    // The title bar takes the card's rounded top corners by clipping to it.
    context.set_fill_style_str(header);
    context.save();
    card_path(context, &rect, metrics.card_radius);
    context.clip();
    context.fill_rect(rect.x, rect.y, rect.w, metrics.header_height);
    context.restore();
    context.set_stroke_style_str(&colours.card_border);
    context.set_line_width(metrics.border_width);
    card_path(context, &rect, metrics.card_radius);
    context.stroke();
    if selected {
        context.set_stroke_style_str(&colours.selected);
        context.set_line_width(metrics.selection_width);
        card_path(context, &rect, metrics.card_radius);
        context.stroke();
    }

    // Title bar: the id on the left, the type name and the icon on the right.
    let middle = rect.y + metrics.header_height / 2.0;
    context.set_font(&theme.fonts.title);
    context.set_fill_style_str(title);
    context.set_text_align("left");
    context.fill_text(
        &node.id,
        rect.x + metrics.title_pad,
        middle + metrics.text_baseline,
    )?;
    let icon_left = rect.x + rect.w - metrics.title_pad - metrics.icon_width;
    context.set_text_align("right");
    context.fill_text(
        &node_type.name,
        icon_left - metrics.title_pad,
        middle + metrics.text_baseline,
    )?;
    context.set_fill_style_str(&colours.icon);
    let bars = 3.0;
    let stack = bars * metrics.icon_bar + (bars - 1.0) * metrics.icon_gap;
    for bar in 0..3 {
        let top = middle - stack / 2.0 + f64::from(bar) * (metrics.icon_bar + metrics.icon_gap);
        context.fill_rect(icon_left, top, metrics.icon_width, metrics.icon_bar);
    }

    // Rows: inputs first, then outputs. Each has a right-aligned name and a
    // value box; the page's own input boxes go over the unlinked inputs.
    context.set_font(&theme.fonts.card);
    let columns = columns(context, node_type, &rect, metrics)?;
    let box_width = columns.box_right - columns.box_left;
    let rows = node_type
        .inputs
        .iter()
        .enumerate()
        .map(|(row, port)| (row, port, true))
        .chain(
            node_type
                .outputs
                .iter()
                .enumerate()
                .map(|(index, port)| (output_row(node_type, index), port, false)),
        );
    for (row, port, is_input) in rows {
        let (_, centre) = input_anchor(&rect, row, metrics);
        let baseline = centre + metrics.text_baseline;
        context.set_fill_style_str(label);
        context.set_text_align("right");
        context.fill_text(&port.name, columns.label_right, baseline)?;
        context.set_fill_style_str(&colours.field);
        round_rect_path(
            context,
            &Rect {
                x: columns.box_left,
                y: centre - metrics.field_height / 2.0,
                w: box_width,
                h: metrics.field_height,
            },
            metrics.box_radius,
        );
        context.fill();
        context.set_text_align("left");
        let room = box_width - 2.0 * metrics.box_pad;
        let text_x = columns.box_left + metrics.box_pad;
        if is_input {
            if let Some(source) = graph
                .links
                .iter()
                .find(|link| link.target.node == node.id && link.target.port == port.name)
            {
                context.set_fill_style_str(&colours.placeholder);
                let from = format!("{}.{}", source.source.node, source.source.port);
                context.fill_text(&fit_text(context, &from, room)?, text_x, baseline)?;
            }
        } else {
            let value = outcome
                .filter(|_| status == Some(Status::Ok))
                .and_then(|outcome| outcome.output_text(&port.name));
            match value {
                Some(value) => {
                    context.set_fill_style_str(&colours.field_text);
                    context.fill_text(&fit_text(context, &value, room)?, text_x, baseline)?;
                }
                None => {
                    context.set_fill_style_str(&colours.placeholder);
                    context.fill_text("output", text_x, baseline)?;
                }
            }
        }
    }
    Ok(())
}

/// Trace the outline of a node card with rounded corners.
fn card_path(context: &CanvasRenderingContext2d, rect: &Rect, radius: f64) {
    round_rect_path(context, rect, radius);
}

/// Trace a rectangle with rounded corners.
fn round_rect_path(context: &CanvasRenderingContext2d, rect: &Rect, radius: f64) {
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
