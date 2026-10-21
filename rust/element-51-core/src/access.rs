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
use crate::{Edit, Graph, NodeType, PlacedNode, PortRef};

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
    /// How many ports the node has.
    pub port_count: usize,
    /// The inputs, then the outputs. Empty from `list_nodes`, which leaves the
    /// ports to `describe_ports`.
    pub ports: Vec<PortItem>,
}

/// The hidden list: one item per node in reading order (top to bottom, then
/// left to right, then by id), each with its ports.
pub fn describe_graph(graph: &Graph, node_types: &[NodeType]) -> Vec<NodeItem> {
    list_nodes(graph, node_types)
        .into_iter()
        .map(|item| NodeItem {
            ports: describe_ports(graph, node_types, &item.id),
            ..item
        })
        .collect()
}

/// The same list without the ports, which is cheap enough to rebuild after
/// every edit of a big graph; `describe_ports` gives a node's ports when the
/// keyboard enters it.
pub fn list_nodes(graph: &Graph, node_types: &[NodeType]) -> Vec<NodeItem> {
    let mut nodes: Vec<&PlacedNode> = graph.nodes.iter().collect();
    nodes.sort_by(|a, b| {
        a.y.total_cmp(&b.y)
            .then(a.x.total_cmp(&b.x))
            .then_with(|| a.id.cmp(&b.id))
    });
    nodes
        .into_iter()
        .map(|node| describe_node(node_types, node))
        .collect()
}

fn describe_node(node_types: &[NodeType], node: &PlacedNode) -> NodeItem {
    let Some(node_type) = node_types.iter().find(|known| known.name == node.node_type) else {
        return NodeItem {
            id: node.id.clone(),
            label: format!("{}, unknown type {}.", node.id, node.node_type),
            port_count: 0,
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
    NodeItem {
        id: node.id.clone(),
        label,
        port_count: node_type.inputs.len() + node_type.outputs.len(),
        ports: Vec::new(),
    }
}

/// The ports of node `id`: the inputs, then the outputs, each with its links.
/// A node that is not in the graph or has an unknown type has none.
pub fn describe_ports(graph: &Graph, node_types: &[NodeType], id: &str) -> Vec<PortItem> {
    let Some(node) = graph.nodes.iter().find(|node| node.id == id) else {
        return Vec::new();
    };
    let Some(node_type) = node_types.iter().find(|known| known.name == node.node_type) else {
        return Vec::new();
    };
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
    inputs.chain(outputs).collect()
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

/// The sentence for the status line after `edit` was applied.
pub fn edit_text(edit: &Edit) -> String {
    match edit {
        Edit::AddNode { id, node_type, .. } => format!("Added node {id}, {node_type}"),
        Edit::AddLink { source, target } => {
            format!("Linked {} to {}", end_name(source), end_name(target))
        }
        Edit::MoveNode { id, .. } => format!("Moved {id}"),
        Edit::DeleteNode { id } => format!("Deleted node {id}"),
        Edit::DeleteLink { source, target } => format!(
            "Deleted the link from {} to {}",
            end_name(source),
            end_name(target)
        ),
        Edit::SetValue {
            node,
            port,
            value: Some(value),
        } => {
            let shown = value
                .as_str()
                .map_or_else(|| value.to_string(), str::to_string);
            format!("Set {port} of {node} to {shown}")
        }
        Edit::SetValue {
            node,
            port,
            value: None,
        } => format!("Cleared the value of {port} of {node}"),
    }
}

/// The sentence after undo (`undo` true) or redo, whether or not there was a
/// step to take.
pub fn undo_text(undo: bool, changed: bool) -> String {
    match (undo, changed) {
        (true, true) => "Undid the last change",
        (false, true) => "Redid the last change",
        (true, false) => "Nothing to undo",
        (false, false) => "Nothing to redo",
    }
    .to_string()
}

/// The sentence when Python or the rules refused something.
pub fn refusal_text(reason: &str) -> String {
    format!("Refused: {reason}")
}

/// The sentence when node `id` was selected from the keyboard.
pub fn selected_text(id: &str) -> String {
    format!("Selected {id}")
}

/// Why Delete did nothing on a port of this side.
pub fn cannot_delete_text(side: Side) -> String {
    match side {
        Side::Output => "Delete works on a node, or on an input port with a link",
        Side::Input => "That input has no link",
    }
    .to_string()
}

/// The edit that links output `source` to input `target`. The mouse and the
/// keyboard both build it here, so both meet the same rules and refusals.
pub fn link_edit(source: &PortRef, target: &PortRef) -> Edit {
    Edit::AddLink {
        source: source.clone(),
        target: target.clone(),
    }
}

/// The sentence when a keyboard link was started from `source`.
pub fn link_started_text(source: &PortRef) -> String {
    format!(
        "Link from {} started. Move to an input port and press Enter, or Escape to cancel",
        end_name(source)
    )
}

/// The sentence when a started link was cancelled.
pub fn link_cancelled_text() -> String {
    "Link cancelled".to_string()
}

/// The sentence when Enter on an input port finds no link started.
pub fn no_link_started_text() -> String {
    "No link started. Press Enter on an output port first".to_string()
}

/// How far the arrow keys pan the view, in screen pixels.
const PAN_STEP: f64 = 48.0;
/// How much one zoom key press scales the view. A wheel notch is about 1.22.
pub const ZOOM_STEP: f64 = 1.25;
/// How many grid steps a big move (Alt held) takes.
const BIG_MOVE_STEPS: f64 = 4.0;

/// Where a node at `(x, y)` goes when moved from the keyboard: one grid step
/// in `direction`, or four with `big`.
pub fn moved_to(x: f64, y: f64, direction: Direction, big: bool, metrics: &Metrics) -> (f64, f64) {
    let step = metrics.grid_step * if big { BIG_MOVE_STEPS } else { 1.0 };
    match direction {
        Direction::Left => (x - step, y),
        Direction::Right => (x + step, y),
        Direction::Up => (x, y - step),
        Direction::Down => (x, y + step),
    }
}

/// How much to pan, in screen pixels, for an Alt+arrow key. The view moves
/// toward the key, so the graph moves the other way.
pub fn pan_delta(direction: Direction) -> (f64, f64) {
    match direction {
        Direction::Left => (PAN_STEP, 0.0),
        Direction::Right => (-PAN_STEP, 0.0),
        Direction::Up => (0.0, PAN_STEP),
        Direction::Down => (0.0, -PAN_STEP),
    }
}

/// The sentence after the view panned.
pub fn pan_text(direction: Direction) -> String {
    let word = match direction {
        Direction::Left => "left",
        Direction::Right => "right",
        Direction::Up => "up",
        Direction::Down => "down",
    };
    format!("Panned {word}")
}

/// The sentence after the zoom changed to `zoom` (1.0 is 100 percent).
pub fn zoom_text(zoom: f64) -> String {
    format!("Zoom {} percent", (zoom * 100.0).round())
}
