//! Where a node box sits and how big it is.

use crate::{Node, NodeType};

/// Width of every node box.
pub const NODE_WIDTH: f64 = 160.0;
/// Height of the title bar.
pub const HEADER_HEIGHT: f64 = 28.0;
/// Height of one port row.
pub const ROW_HEIGHT: f64 = 22.0;

/// A rectangle in canvas coordinates.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Rect {
    pub x: f64,
    pub y: f64,
    pub w: f64,
    pub h: f64,
}

/// The box for `node`: a header plus one row per port on the busier side.
pub fn node_rect(node: &Node, node_type: &NodeType) -> Rect {
    let rows = node_type.inputs.len().max(node_type.outputs.len());
    Rect {
        x: node.x,
        y: node.y,
        w: NODE_WIDTH,
        h: HEADER_HEIGHT + ROW_HEIGHT * rows as f64,
    }
}
