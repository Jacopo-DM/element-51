//! Where a node box sits and how big it is.

use crate::theme::Metrics;
use crate::{Graph, Link, NodeType, PlacedNode, PortRef};

/// A rectangle in canvas coordinates.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Rect {
    pub x: f64,
    pub y: f64,
    pub w: f64,
    pub h: f64,
}

/// The box for `node`: a header plus one row per port on the busier side.
pub fn node_rect(node: &PlacedNode, node_type: &NodeType, metrics: &Metrics) -> Rect {
    let rows = node_type.inputs.len().max(node_type.outputs.len());
    Rect {
        x: node.x,
        y: node.y,
        w: metrics.node_width,
        h: metrics.header_height + metrics.row_height * rows as f64,
    }
}

/// Which side of a node a port is on.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Side {
    Input,
    Output,
}

/// A port found under a point.
#[derive(Clone, Debug, PartialEq)]
pub struct PortHit {
    pub node: String,
    pub port: String,
    pub side: Side,
}

/// Where input row `row` meets the left edge of `rect`.
pub fn input_anchor(rect: &Rect, row: usize, metrics: &Metrics) -> (f64, f64) {
    (rect.x, row_centre(rect, row, metrics))
}

/// Where output row `row` meets the right edge of `rect`.
pub fn output_anchor(rect: &Rect, row: usize, metrics: &Metrics) -> (f64, f64) {
    (rect.x + rect.w, row_centre(rect, row, metrics))
}

fn row_centre(rect: &Rect, row: usize, metrics: &Metrics) -> f64 {
    rect.y + metrics.header_height + metrics.row_height * (row as f64 + 0.5)
}

/// The port whose anchor is within the metrics' hit radius of `(x, y)`, if any.
///
/// A node whose type is not in `node_types` has no ports.
pub fn port_at(
    graph: &Graph,
    node_types: &[NodeType],
    x: f64,
    y: f64,
    metrics: &Metrics,
) -> Option<PortHit> {
    for node in &graph.nodes {
        let Some(node_type) = node_types.iter().find(|known| known.name == node.node_type) else {
            continue;
        };
        let rect = node_rect(node, node_type, metrics);
        let sides = [
            (Side::Input, &node_type.inputs),
            (Side::Output, &node_type.outputs),
        ];
        for (side, ports) in sides {
            for (row, port) in ports.iter().enumerate() {
                let (anchor_x, anchor_y) = match side {
                    Side::Input => input_anchor(&rect, row, metrics),
                    Side::Output => output_anchor(&rect, row, metrics),
                };
                if (x - anchor_x).hypot(y - anchor_y) <= metrics.port_hit_radius {
                    return Some(PortHit {
                        node: node.id.clone(),
                        port: port.name.clone(),
                        side,
                    });
                }
            }
        }
    }
    None
}

/// Where the port `reference` names sits on its node, as an output (`Side::Output`)
/// or as an input. None if the node, its type or the port is missing.
pub fn anchor_of(
    graph: &Graph,
    node_types: &[NodeType],
    reference: &PortRef,
    side: Side,
    metrics: &Metrics,
) -> Option<(f64, f64)> {
    let node = graph.nodes.iter().find(|node| node.id == reference.node)?;
    let node_type = node_types
        .iter()
        .find(|known| known.name == node.node_type)?;
    let rect = node_rect(node, node_type, metrics);
    match side {
        Side::Input => {
            let row = node_type
                .inputs
                .iter()
                .position(|port| port.name == reference.port)?;
            Some(input_anchor(&rect, row, metrics))
        }
        Side::Output => {
            let row = node_type
                .outputs
                .iter()
                .position(|port| port.name == reference.port)?;
            Some(output_anchor(&rect, row, metrics))
        }
    }
}

/// The topmost node whose box holds `(x, y)`, if any. Later nodes are drawn
/// over earlier ones. A node whose type is not in `node_types` cannot be picked.
pub fn node_at(
    graph: &Graph,
    node_types: &[NodeType],
    x: f64,
    y: f64,
    metrics: &Metrics,
) -> Option<String> {
    graph
        .nodes
        .iter()
        .rev()
        .find(|node| {
            let Some(node_type) = node_types.iter().find(|known| known.name == node.node_type)
            else {
                return false;
            };
            let rect = node_rect(node, node_type, metrics);
            x >= rect.x && x <= rect.x + rect.w && y >= rect.y && y <= rect.y + rect.h
        })
        .map(|node| node.id.clone())
}

/// The four control points of the curve a link is drawn along: it leaves
/// `from` and arrives at `to` horizontally.
pub fn wire_controls(from: (f64, f64), to: (f64, f64), metrics: &Metrics) -> [(f64, f64); 4] {
    // Half the horizontal gap, so a wire across a wide gap is a smooth S. When
    // the ends are nearly in one column the wire needs a sideways swing that
    // grows with the height between them, or it would run along the card's edge.
    let across = (to.0 - from.0).abs() / 2.0;
    let swing = ((to.1 - from.1).abs() / 3.0).min(metrics.wire_max_swing);
    let reach = across.max(swing).max(metrics.wire_min_reach);
    [from, (from.0 + reach, from.1), (to.0 - reach, to.1), to]
}

/// How many straight pieces stand in for a curve when testing a click.
const WIRE_STEPS: usize = 32;

fn wire_point(controls: &[(f64, f64); 4], t: f64) -> (f64, f64) {
    let u = 1.0 - t;
    let weights = [u * u * u, 3.0 * u * u * t, 3.0 * u * t * t, t * t * t];
    let mut point = (0.0, 0.0);
    for (weight, control) in weights.iter().zip(controls) {
        point.0 += weight * control.0;
        point.1 += weight * control.1;
    }
    point
}

/// How far `(x, y)` is from the segment `a` to `b`.
fn distance_to_segment(x: f64, y: f64, a: (f64, f64), b: (f64, f64)) -> f64 {
    let (dx, dy) = (b.0 - a.0, b.1 - a.1);
    let length_squared = dx * dx + dy * dy;
    let t = if length_squared == 0.0 {
        0.0
    } else {
        (((x - a.0) * dx + (y - a.1) * dy) / length_squared).clamp(0.0, 1.0)
    };
    (x - (a.0 + t * dx)).hypot(y - (a.1 + t * dy))
}

/// The link whose curve passes within `tolerance` of `(x, y)`, if any. When
/// several do, the one added last.
pub fn link_at(
    graph: &Graph,
    node_types: &[NodeType],
    x: f64,
    y: f64,
    tolerance: f64,
    metrics: &Metrics,
) -> Option<Link> {
    graph
        .links
        .iter()
        .rev()
        .find(|link| {
            let from = anchor_of(graph, node_types, &link.source, Side::Output, metrics);
            let to = anchor_of(graph, node_types, &link.target, Side::Input, metrics);
            let (Some(from), Some(to)) = (from, to) else {
                return false;
            };
            let controls = wire_controls(from, to, metrics);
            let mut previous = from;
            (1..=WIRE_STEPS).any(|step| {
                let next = wire_point(&controls, step as f64 / WIRE_STEPS as f64);
                let near = distance_to_segment(x, y, previous, next) <= tolerance;
                previous = next;
                near
            })
        })
        .cloned()
}
