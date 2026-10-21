//! Drawing: a frame from the replica and the state of the pointer.
//!
//! Every colour, size and font comes from the `Theme` in the frame.

use crate::pictures::Pictures;
use element_51_core::display::{
    chart_layout, chart_rect, fit_picture, last_value, line_names, line_style, picture_rect,
    short_number,
};
use element_51_core::fields::{linked_text, share_room};
use element_51_core::layout::{
    PortHit, Rect, Side, anchor_of, icon_rect, input_anchor, node_rect, output_anchor, output_row,
    wire_bounds, wire_controls,
};
use element_51_core::rows::{find_input, type_of};
use element_51_core::theme::{Metrics, PortShape, PortStyle, Theme};
use element_51_core::view::View;
use element_51_core::{
    Edit, Graph, Link, NodeOutcome, NodeType, PlacedNode, PortRef, PortType, RunResult, Series,
    Status, apply_edit,
};
use std::f64::consts::TAU;
use wasm_bindgen::JsValue;
use web_sys::{CanvasRenderingContext2d, HtmlCanvasElement};

/// The grid is left out when its dots would be closer than this on screen.
const MIN_GRID_PIXELS: f64 = 8.0;
/// The focus ring's width, its dark halo on each side, and its gap from what it
/// rings, all in screen pixels.
const FOCUS_WIDTH: f64 = 2.0;
const FOCUS_HALO: f64 = 1.0;
const FOCUS_GAP: f64 = 3.0;
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
}

/// A wire is drawn in the colour of the port it leaves, as in Antimony.
fn wire_colour<'a>(
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
            (run, pictures),
            is_selected,
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

    // Ports go over the wires, so a wire ends at the port's square.
    for node in &graph.nodes {
        let Some(node_type) = type_of(graph, types, node) else {
            continue;
        };
        if !on_screen(node, &node_type) {
            continue;
        }
        let rect = node_rect(node, &node_type, metrics);
        for (row, port) in node_type.inputs.iter().enumerate() {
            let (x, y) = input_anchor(&rect, row, metrics);
            marker(context, (x, y), theme.port_style(port.kind, false), metrics);
        }
        for (index, port) in node_type.outputs.iter().enumerate() {
            let (x, y) = output_anchor(&rect, output_row(&node_type, index), metrics);
            marker(context, (x, y), theme.port_style(port.kind, true), metrics);
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

/// Every link of the graph as a wire, in the colour of the port it leaves, or the
/// selected colour and width for the selected one.
fn draw_links(
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

/// The ring on the node or port that has the keyboard focus. It is two-tone,
/// the canvas colour under the theme's focus colour, so it shows on a card as
/// well as on the canvas. Its widths are in screen pixels, whatever the zoom.
fn focus_ring(
    context: &CanvasRenderingContext2d,
    (graph, types): (&Graph, &[NodeType]),
    theme: &Theme,
    view: &View,
    focus: &Focus,
) -> Result<(), JsValue> {
    let (colours, metrics) = (&theme.colours, &theme.metrics);
    let pixel = 1.0 / view.zoom();
    let node_id = match focus {
        Focus::Node(id) | Focus::Port { node: id, .. } => id,
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
    (run, pictures): (Option<&RunResult>, &Pictures),
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
    let icon = icon_rect(&rect, metrics);
    context.set_text_align("right");
    context.fill_text(
        &node_type.name,
        icon.x - metrics.title_pad,
        middle + metrics.text_baseline,
    )?;
    context.set_fill_style_str(&colours.icon);
    for bar in 0..3 {
        let top = icon.y + f64::from(bar) * (metrics.icon_bar + metrics.icon_gap);
        context.fill_rect(icon.x, top, icon.w, metrics.icon_bar);
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
        context.set_font(&theme.fonts.card);
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
        context.set_font(&theme.fonts.value);
        let room = box_width - 2.0 * metrics.box_pad;
        let text_x = columns.box_left + metrics.box_pad;
        if is_input {
            if let Some(source) = graph
                .links
                .iter()
                .find(|link| link.target.node == node.id && link.target.port == port.name)
            {
                // The value first, then the source dimmer after it; the source is cut
                // first when the box is narrow.
                let parts = linked_text(
                    &source.source,
                    run.and_then(|result| result.outcome_for(&source.source.node)),
                );
                let mut at = text_x;
                if let Some(value) = &parts.value {
                    let (value_room, _) = share_room(
                        context.measure_text(value)?.width(),
                        context.measure_text(&parts.source)?.width(),
                        room,
                        metrics.box_pad,
                    );
                    let shown = fit_text(context, value, value_room)?;
                    context.set_fill_style_str(&colours.field_text);
                    context.fill_text(&shown, at, baseline)?;
                    at += context.measure_text(&shown)?.width() + metrics.box_pad;
                }
                let left = room - (at - text_x);
                context.set_fill_style_str(&colours.placeholder);
                context.fill_text(&fit_text(context, &parts.source, left)?, at, baseline)?;
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
    if let Some(area) = picture_rect(&rect, node_type, metrics) {
        context.set_fill_style_str(&colours.field);
        round_rect_path(context, &area, metrics.box_radius);
        context.fill();
        let shown = pictures.of(&node.id).filter(|_| status == Some(Status::Ok));
        if let Some(picture) = shown {
            let at = fit_picture(&area, picture.width(), picture.height());
            // Crisp pixels: a 28 by 28 picture is meant to look like 28 by 28.
            context.set_image_smoothing_enabled(false);
            let drawn = context
                .draw_image_with_html_canvas_element_and_dw_and_dh(picture, at.x, at.y, at.w, at.h);
            context.set_image_smoothing_enabled(true);
            drawn?;
        }
    }
    if let Some(area) = chart_rect(&rect, node_type, metrics) {
        context.set_fill_style_str(&colours.field);
        round_rect_path(context, &area, metrics.box_radius);
        context.fill();
        if let Some(outcome) = outcome.filter(|_| status == Some(Status::Ok)) {
            chart(context, theme, graph, (node, node_type), (&area, outcome))?;
        }
    }
    Ok(())
}

/// The chart of a plot node: gridlines and tick labels, then one line per
/// linked series with its source port's name and last value at its end.
pub(crate) fn chart(
    context: &CanvasRenderingContext2d,
    theme: &Theme,
    graph: &Graph,
    (node, node_type): (&PlacedNode, &NodeType),
    (area, outcome): (&Rect, &NodeOutcome),
) -> Result<(), JsValue> {
    let (colours, metrics) = (&theme.colours, &theme.metrics);
    let sourced: Vec<((&str, &str), &Series)> = node_type
        .inputs
        .iter()
        .filter_map(|port| {
            let series = outcome.shown.get(&port.name)?.as_series()?;
            let source = graph
                .links
                .iter()
                .find(|link| link.target.node == node.id && link.target.port == port.name)?;
            Some((
                (source.source.node.as_str(), source.source.port.as_str()),
                series,
            ))
        })
        .collect();
    let sources: Vec<(&str, &str)> = sourced.iter().map(|(source, _)| *source).collect();
    let names = line_names(&sources);
    let lines: Vec<(&str, &Series)> = names
        .iter()
        .map(String::as_str)
        .zip(sourced.iter().map(|(_, series)| *series))
        .collect();
    let series: Vec<&Series> = lines.iter().map(|(_, series)| *series).collect();
    let Some(layout) = chart_layout(area, metrics, &series) else {
        return Ok(());
    };
    let plot = layout.plot;
    context.set_font(&theme.fonts.value);
    context.set_line_width(1.0);
    context.set_stroke_style_str(&colours.card_border);
    context.set_fill_style_str(&colours.field_text);
    let room = plot.x - area.x - metrics.box_pad;
    context.set_text_align("right");
    for tick in &layout.y_ticks {
        context.begin_path();
        context.move_to(plot.x, tick.at);
        context.line_to(plot.x + plot.w, tick.at);
        context.stroke();
        let label = fit_text(context, &tick.label, room)?;
        context.fill_text(
            &label,
            plot.x - metrics.box_pad / 2.0,
            tick.at + metrics.text_baseline,
        )?;
    }
    context.set_text_align("center");
    for tick in &layout.x_ticks {
        context.begin_path();
        context.move_to(tick.at, plot.y);
        context.line_to(tick.at, plot.y + plot.h);
        context.stroke();
        context.fill_text(
            &tick.label,
            tick.at,
            plot.y + plot.h + metrics.row_height * 0.5 + metrics.text_baseline,
        )?;
    }
    context.save();
    context.begin_path();
    context.rect(plot.x, plot.y, plot.w, plot.h);
    context.clip();
    context.set_line_width(metrics.wire_width * 0.6);
    context.set_line_join("round");
    for (index, runs) in layout.lines.iter().enumerate() {
        let (kind, dashed) = line_style(index);
        context.set_stroke_style_str(&theme.port_style(kind, true).colour);
        if dashed {
            context.set_line_dash(&crate::json_parse("[6, 4]"))?;
        }
        for run in runs {
            context.begin_path();
            for (step, (x, y)) in run.iter().enumerate() {
                if step == 0 {
                    context.move_to(*x, *y);
                } else {
                    context.line_to(*x, *y);
                }
            }
            // A run of one point would draw nothing: give it a dot.
            if run.len() == 1 {
                context.line_to(run[0].0 + 0.1, run[0].1);
            }
            context.stroke();
        }
        if dashed {
            context.set_line_dash(&crate::json_parse("[]"))?;
        }
    }
    context.restore();
    // Names sit at the line ends, in the line's colour, so colour is not the only cue.
    for (index, ((name, series), runs)) in lines.iter().zip(&layout.lines).enumerate() {
        let Some(end) = runs.last().and_then(|run| run.last()) else {
            continue;
        };
        let (kind, _) = line_style(index);
        context.set_fill_style_str(&theme.port_style(kind, true).colour);
        let text = match last_value(series) {
            Some(last) => format!("{name} {}", short_number(last)),
            None => (*name).to_string(),
        };
        // The label goes above the end of the line in the lower half of the plot and
        // below it in the upper half, so it stays off the line's own tail.
        let gap = metrics.text_baseline * 3.0;
        let label_y = if end.1 > plot.y + plot.h / 2.0 {
            end.1 - gap
        } else {
            end.1 + gap + metrics.text_baseline
        };
        // Text runs away from the line: right of an end in the left half, left of
        // one in the right half.
        let near_left = end.0 < plot.x + plot.w / 2.0;
        context.set_text_align(if near_left { "left" } else { "right" });
        let side = if near_left { 4.0 } else { -4.0 };
        context.fill_text(&text, end.0.min(plot.x + plot.w) + side, label_y)?;
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
