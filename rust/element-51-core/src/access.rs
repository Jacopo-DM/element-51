//! What a screen reader is told about the graph, and how the keyboard moves
//! around it.
//!
//! The page keeps a hidden list of the graph built from `describe_graph`; the
//! keyboard moves between its items with `neighbour`, and `pan_to_show` keeps
//! the focused node on screen.

use std::collections::BTreeMap;

use crate::layout::{Rect, Side, node_rect};
use crate::theme::Metrics;
use crate::view::View;
use crate::{Graph, NodeType, PlacedNode, PortRef};

/// How much more a step sideways counts than a step ahead, when the keyboard
/// looks for the nearest node in a direction.
const SIDEWAYS_WEIGHT: f64 = 2.0;

/// A direction of an arrow key.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Direction {
    Up,
    Down,
    Left,
    Right,
}

/// A port as the hidden list shows it.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PortItem {
    pub port: String,
    pub side: Side,
    /// What a screen reader says for the port, for example
    /// `Output out, float, linked to n3 value`.
    pub label: String,
}

/// A node as the hidden list shows it.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct NodeItem {
    pub id: String,
    /// For example `n3, add. 2 inputs, 1 output.`
    pub label: String,
    /// The inputs, then the outputs.
    pub ports: Vec<PortItem>,
}

/// The hidden list: one item per node in reading order (top to bottom, then
/// left to right, then by id), each with its ports.
pub fn describe_graph(graph: &Graph, node_types: &[NodeType]) -> Vec<NodeItem> {
    let mut nodes: Vec<&PlacedNode> = graph.nodes.iter().collect();
    nodes.sort_by(|a, b| {
        a.y.total_cmp(&b.y)
            .then(a.x.total_cmp(&b.x))
            .then_with(|| a.id.cmp(&b.id))
    });
    nodes
        .into_iter()
        .map(|node| describe_node(graph, node_types, node))
        .collect()
}

fn describe_node(graph: &Graph, node_types: &[NodeType], node: &PlacedNode) -> NodeItem {
    let Some(node_type) = node_types.iter().find(|known| known.name == node.node_type) else {
        return NodeItem {
            id: node.id.clone(),
            label: format!("{}, unknown type {}.", node.id, node.node_type),
            ports: Vec::new(),
        };
    };
    let label = format!(
        "{}, {}. {} {}, {} {}.",
        node.id,
        node_type.name,
        node_type.inputs.len(),
        plural(node_type.inputs.len(), "input"),
        node_type.outputs.len(),
        plural(node_type.outputs.len(), "output"),
    );
    let inputs = node_type.inputs.iter().map(|port| {
        let source = graph
            .links
            .iter()
            .find(|link| link.target.node == node.id && link.target.port == port.name)
            .map(|link| &link.source);
        let state = match source {
            Some(end) => format!("linked from {}", end_name(end)),
            None => "not linked".to_string(),
        };
        PortItem {
            port: port.name.clone(),
            side: Side::Input,
            label: format!("Input {}, {}, {state}", port.name, port.kind),
        }
    });
    let outputs = node_type.outputs.iter().map(|port| {
        let targets: Vec<String> = graph
            .links
            .iter()
            .filter(|link| link.source.node == node.id && link.source.port == port.name)
            .map(|link| end_name(&link.target))
            .collect();
        let state = if targets.is_empty() {
            "not linked".to_string()
        } else {
            format!("linked to {}", join_names(&targets))
        };
        PortItem {
            port: port.name.clone(),
            side: Side::Output,
            label: format!("Output {}, {}, {state}", port.name, port.kind),
        }
    });
    NodeItem {
        id: node.id.clone(),
        label,
        ports: inputs.chain(outputs).collect(),
    }
}

fn plural(count: usize, word: &str) -> String {
    if count == 1 {
        word.to_string()
    } else {
        format!("{word}s")
    }
}

fn end_name(end: &PortRef) -> String {
    format!("{} {}", end.node, end.port)
}

/// `a`, `a and b`, `a, b and c`.
fn join_names(names: &[String]) -> String {
    match names {
        [] => String::new(),
        [only] => only.clone(),
        [first @ .., last] => format!("{} and {last}", first.join(", ")),
    }
}

/// The node nearest to `from` in `direction`, judged by the centres of the
/// boxes: a node straight ahead beats one that is nearer but far to the side.
/// Nodes of an unknown type are skipped. Equal candidates go by id.
pub fn neighbour(
    graph: &Graph,
    node_types: &[NodeType],
    metrics: &Metrics,
    from: &str,
    direction: Direction,
) -> Option<String> {
    let centres: BTreeMap<&str, (f64, f64)> = graph
        .nodes
        .iter()
        .filter_map(|node| {
            let node_type = node_types
                .iter()
                .find(|known| known.name == node.node_type)?;
            let rect = node_rect(node, node_type, metrics);
            Some((
                node.id.as_str(),
                (rect.x + rect.w / 2.0, rect.y + rect.h / 2.0),
            ))
        })
        .collect();
    let (from_x, from_y) = *centres.get(from)?;
    centres
        .iter()
        .filter(|(id, _)| **id != from)
        .filter_map(|(id, (x, y))| {
            let (dx, dy) = (x - from_x, y - from_y);
            let (ahead, aside) = match direction {
                Direction::Right => (dx, dy),
                Direction::Left => (-dx, dy),
                Direction::Down => (dy, dx),
                Direction::Up => (-dy, dx),
            };
            (ahead > 0.0).then(|| (ahead + SIDEWAYS_WEIGHT * aside.abs(), *id))
        })
        .min_by(|a, b| a.0.total_cmp(&b.0).then_with(|| a.1.cmp(b.1)))
        .map(|(_, id)| id.to_string())
}

/// How far to pan, in screen pixels, so that `rect` (in graph units) is fully
/// inside a window of `viewport` pixels with `margin` pixels to spare. A box
/// too big for the window lines up with the top left margin.
pub fn pan_to_show(view: &View, rect: &Rect, viewport: (f64, f64), margin: f64) -> (f64, f64) {
    let (left, top) = view.to_screen(rect.x, rect.y);
    let (width, height) = (rect.w * view.zoom(), rect.h * view.zoom());
    (
        shift(left, width, viewport.0, margin),
        shift(top, height, viewport.1, margin),
    )
}

fn shift(start: f64, length: f64, window: f64, margin: f64) -> f64 {
    if length > window - 2.0 * margin || start < margin {
        margin - start
    } else if start + length > window - margin {
        window - margin - (start + length)
    } else {
        0.0
    }
}
