//! Drawing: a frame from the replica and the state of the pointer.
//!
//! Every colour, size and font comes from the `Theme` in the frame.

use crate::draw_card::card;
use crate::draw_focus::focus_ring;
use crate::draw_wires::{draw_links, wire, wire_colour};
use crate::pictures::Pictures;
use element_51_core::job::JobButton;
use element_51_core::layout::{
    PortHit, Rect, Side, anchor_of, input_anchor, node_rect, output_anchor, output_row,
};
use element_51_core::rows::{find_input, type_of};
use element_51_core::theme::{Metrics, PortShape, PortStyle, Theme};
use element_51_core::view::View;
use element_51_core::{
    Edit, Graph, JobUpdate, Link, NodeType, PlacedNode, PortRef, PortType, RunResult, apply_edit,
};
use std::collections::BTreeMap;
use std::f64::consts::TAU;
use wasm_bindgen::JsValue;
use web_sys::{CanvasRenderingContext2d, HtmlCanvasElement};

/// The centre and style of each port marker of one card.
type Markers<'a> = Vec<(f64, f64, &'a PortStyle)>;

/// The grid is left out when its dots would be closer than this on screen.
const MIN_GRID_PIXELS: f64 = 8.0;
/// How far a diamond marker reaches from its centre, as a fraction of the port
/// size: 0.7 gives it about the area of a box of that size.
const DIAMOND_REACH: f64 = 0.7;

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

/// What has the keyboard focus, as the hidden tree reports it.
#[derive(Clone, Debug, PartialEq)]
pub enum Focus {
    Node(String),
    Port {
        node: String,
        port: String,
        side: Side,
    },
    /// The Start or Stop button of a job node.
    Job {
        node: String,
        button: JobButton,
    },
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
    pub focus: Option<&'a Focus>,
    pub run: Option<&'a RunResult>,
    pub pictures: &'a Pictures,
    /// How each job that has been started stands, by node id.
    pub jobs: &'a BTreeMap<String, JobUpdate>,
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
        focus,
        run,
        pictures,
        jobs,
    } = *frame;
    let (colours, metrics) = (&theme.colours, &theme.metrics);
    context.set_transform(1.0, 0.0, 0.0, 1.0, 0.0, 0.0)?;
    context.set_fill_style_str(&colours.canvas);
    let (pixel_width, pixel_height) = (f64::from(canvas.width()), f64::from(canvas.height()));
    context.fill_rect(0.0, 0.0, pixel_width, pixel_height);
    // The canvas has one pixel per device pixel, which on a sharp screen is
    // more than one per CSS pixel; the view and the pointer work in CSS pixels.
    let ratio = pixel_width / f64::from(canvas.client_width().max(1));
    let (width, height) = (pixel_width / ratio, pixel_height / ratio);
    context.set_transform(
        view.zoom() * ratio,
        0.0,
        0.0,
        view.zoom() * ratio,
        view.pan_x() * ratio,
        view.pan_y() * ratio,
    )?;
    grid(context, (width, height), view, theme)?;
    let Some(graph) = graph else {
        context.set_transform(1.0, 0.0, 0.0, 1.0, 0.0, 0.0)?;
        return Ok(());
    };
    // Only the cards in view are drawn; the outline and the ports reach a little
    // outside a card's box. See DESIGN.md, speed-up task 3.
    let (left, top) = view.to_graph(0.0, 0.0);
    let (right, bottom) = view.to_graph(width, height);
    let visible = Rect {
        x: left,
        y: top,
        w: right - left,
        h: bottom - top,
    };
    let margin = metrics.selection_width + metrics.border_width;
    let on_screen = |node: &PlacedNode, node_type: &NodeType| {
        visible.overlaps(&node_rect(node, node_type, metrics).grown(margin))
    };

    // A wire is drawn solid under the cards, so it shows on the canvas, and again
    // over the cards at the theme's opacity, so a card's boxes show through it.
    draw_links(context, theme, (graph, types), selected, &visible)?;
    let mut cards: Vec<Rect> = Vec::new();
    for node in &graph.nodes {
        let Some(node_type) = type_of(graph, types, node) else {
            continue;
        };
        if !on_screen(node, &node_type) {
            continue;
        }
        let is_selected = selected == Some(&Selected::Node(node.id.clone()));
        card(
            context,
            theme,
            graph,
            (node, &node_type),
            (run, pictures, jobs.get(&node.id)),
            is_selected,
            match focus {
                Some(Focus::Job { node: id, button }) if *id == node.id => Some(*button),
                _ => None,
            },
        )?;
        cards.push(node_rect(node, &node_type, metrics));
    }
    let opacity = metrics.wire_over_card.clamp(0.0, 1.0);
    if opacity > 0.0 && !cards.is_empty() {
        context.save();
        context.begin_path();
        for rect in &cards {
            context.rect(rect.x, rect.y, rect.w, rect.h);
        }
        context.clip();
        context.set_global_alpha(opacity);
        let drawn = draw_links(context, theme, (graph, types), selected, &visible);
        context.restore();
        drawn?;
    }
    context.set_line_width(metrics.wire_width);
    if let Some(drag) = drag
        && let Some(from) = anchor_of(graph, types, &drag.source, Side::Output, metrics)
    {
        context.set_stroke_style_str(wire_colour(graph, types, &drag.source, theme));
        wire(context, from, (drag.x, drag.y), metrics);
    }

    // Ports go over the wires, so a wire ends at the port's square. A port a
    // later card covers is left out, as the card hides it.
    let size = metrics.port_size;
    let mut placed: Vec<(Rect, Markers)> = Vec::new();
    for node in &graph.nodes {
        let Some(node_type) = type_of(graph, types, node) else {
            continue;
        };
        if !on_screen(node, &node_type) {
            continue;
        }
        let rect = node_rect(node, &node_type, metrics);
        let mut markers: Markers = Vec::new();
        for (row, port) in node_type.inputs.iter().enumerate() {
            let (x, y) = input_anchor(&rect, row, metrics);
            markers.push((x, y, theme.port_style(port.kind, false)));
        }
        for (index, port) in node_type.outputs.iter().enumerate() {
            let (x, y) = output_anchor(&rect, output_row(&node_type, index), metrics);
            markers.push((x, y, theme.port_style(port.kind, true)));
        }
        placed.push((rect, markers));
    }
    for (index, (_, markers)) in placed.iter().enumerate() {
        let above: Vec<Rect> = placed[index + 1..].iter().map(|(rect, _)| *rect).collect();
        for (x, y, style) in markers {
            let reach = Rect {
                x: x - size / 2.0,
                y: y - size / 2.0,
                w: size,
                h: size,
            };
            if !reach.is_covered_by(&above) {
                marker(context, (*x, *y), style, metrics);
            }
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
            let kind = node_port_kind(graph, types, &target).unwrap_or(PortType::Str);
            let shape = theme.port_style(kind, false).shape;
            let style = PortStyle {
                colour: colour.clone(),
                shape,
            };
            marker(context, (x, y), &style, metrics);
        }
    }
    if let Some(focus) = focus {
        focus_ring(context, (graph, types), theme, view, focus)?;
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
    context.begin_path();
    let mut y = (top / step).floor() * step;
    while y <= bottom {
        let mut x = (left / step).floor() * step;
        while x <= right {
            context.move_to(x + metrics.dot_radius, y);
            context.arc(x, y, metrics.dot_radius, 0.0, TAU)?;
            x += step;
        }
        y += step;
    }
    context.fill();
    Ok(())
}

/// The type of the input `target` names, if the node and port exist.
fn node_port_kind(graph: &Graph, types: &[NodeType], target: &PortRef) -> Option<PortType> {
    let node = graph.nodes.iter().find(|node| node.id == target.node)?;
    let node_type = types.iter().find(|known| known.name == node.node_type)?;
    find_input(node_type, &target.port).map(|port| port.kind)
}

/// A port marker centred on `at`, in the style's colour and shape. A circle
/// has the port size as its diameter, a box as its side, and a diamond is as
/// tall as a box's diagonal shrunk to look about as big.
fn marker(
    context: &CanvasRenderingContext2d,
    at: (f64, f64),
    style: &PortStyle,
    metrics: &Metrics,
) {
    let (x, y) = at;
    let size = metrics.port_size;
    context.set_fill_style_str(&style.colour);
    match style.shape {
        PortShape::Box => context.fill_rect(x - size / 2.0, y - size / 2.0, size, size),
        PortShape::Circle => {
            context.begin_path();
            let _ = context.arc(x, y, size / 2.0, 0.0, TAU);
            context.fill();
        }
        PortShape::Diamond => {
            let reach = size * DIAMOND_REACH;
            context.begin_path();
            context.move_to(x, y - reach);
            context.line_to(x + reach, y);
            context.line_to(x, y + reach);
            context.line_to(x - reach, y);
            context.close_path();
            context.fill();
        }
    }
}

/// `text`, shortened with an ellipsis until it is at most `room` pixels wide.
/// Empty if even the ellipsis does not fit.
pub(crate) fn fit_text(
    context: &CanvasRenderingContext2d,
    text: &str,
    room: f64,
) -> Result<String, JsValue> {
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
