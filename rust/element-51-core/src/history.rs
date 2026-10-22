//! Undo and redo: a stack of steps, each knowing how to go back.
//!
//! A step is one edit the user made and the edits that undo it. Undoing sends
//! those edits to Python like any other edit, so Python stays the one true copy
//! and other open pages see the undo too.

use crate::{Edit, Graph, Link, PortRef};

#[derive(Clone, Debug)]
struct Step {
    forward: Edit,
    backward: Vec<Edit>,
}

/// What the user can undo, and what they can redo.
#[derive(Clone, Debug, Default)]
pub struct History {
    undo: Vec<Step>,
    redo: Vec<Step>,
}

impl History {
    /// Remember that the user made `edit` on `before`. Clears what could be
    /// redone, because the new edit starts a different future.
    pub fn record(&mut self, before: &Graph, edit: &Edit) {
        self.undo.push(Step {
            forward: edit.clone(),
            backward: inverse(before, edit),
        });
        self.redo.clear();
    }

    /// Whether there is something to undo.
    pub fn can_undo(&self) -> bool {
        !self.undo.is_empty()
    }

    /// Whether there is something to redo.
    pub fn can_redo(&self) -> bool {
        !self.redo.is_empty()
    }

    /// Take the newest step off the undo stack, and return the edits that undo
    /// it, in the order to apply them. The step moves to the redo stack.
    pub fn undo(&mut self) -> Option<Vec<Edit>> {
        let step = self.undo.pop()?;
        let backward = step.backward.clone();
        self.redo.push(step);
        Some(backward)
    }

    /// Take the newest step off the redo stack, and return the edit that
    /// makes it again. The step moves back to the undo stack.
    pub fn redo(&mut self) -> Option<Vec<Edit>> {
        let step = self.redo.pop()?;
        let forward = vec![step.forward.clone()];
        self.undo.push(step);
        Some(forward)
    }
}

/// The edits that put `before` back after `edit` has been applied to it, in the
/// order to apply them. `edit` must be valid on `before`.
///
/// A deleted node comes back at the end of the node list, so it is drawn on
/// top of the others; everything else is exactly as it was.
fn inverse(before: &Graph, edit: &Edit) -> Vec<Edit> {
    match edit {
        Edit::AddNode { id, .. } => vec![Edit::DeleteNode { id: id.clone() }],
        Edit::AddLink { source, target } => vec![Edit::DeleteLink {
            source: source.clone(),
            target: target.clone(),
        }],
        Edit::DeleteLink { source, target } => vec![Edit::AddLink {
            source: source.clone(),
            target: target.clone(),
        }],
        Edit::MoveNode { id, .. } => before
            .nodes
            .iter()
            .find(|node| node.id == *id)
            .map(|node| Edit::MoveNode {
                id: id.clone(),
                x: node.x,
                y: node.y,
            })
            .into_iter()
            .collect(),
        Edit::ResizeNode { id, .. } => before
            .nodes
            .iter()
            .find(|node| node.id == *id)
            .map(|node| Edit::ResizeNode {
                id: id.clone(),
                width: node.width,
                display_height: node.display_height,
            })
            .into_iter()
            .collect(),
        Edit::SetValue { node, port, .. } => {
            let old = before
                .nodes
                .iter()
                .find(|placed| placed.id == *node)
                .and_then(|placed| placed.values.get(port))
                .cloned();
            vec![Edit::SetValue {
                node: node.clone(),
                port: port.clone(),
                value: old,
            }]
        }
        Edit::SetColour { node, port, .. } => {
            let old = before
                .nodes
                .iter()
                .find(|placed| placed.id == *node)
                .and_then(|placed| placed.colours.get(port))
                .cloned();
            vec![Edit::SetColour {
                node: node.clone(),
                port: port.clone(),
                colour: old,
            }]
        }
        Edit::DeleteNode { id } => restore_node(before, id),
    }
}

/// The edits that bring back node `id` with its position, its values and every
/// link that touched it.
fn restore_node(before: &Graph, id: &str) -> Vec<Edit> {
    let Some(node) = before.nodes.iter().find(|node| node.id == id) else {
        return Vec::new();
    };
    let mut edits = vec![Edit::AddNode {
        id: node.id.clone(),
        node_type: node.node_type.clone(),
        x: node.x,
        y: node.y,
    }];
    if node.width.is_some() || node.display_height.is_some() {
        edits.push(Edit::ResizeNode {
            id: node.id.clone(),
            width: node.width,
            display_height: node.display_height,
        });
    }
    edits.extend(node.values.iter().map(|(port, value)| Edit::SetValue {
        node: id.to_string(),
        port: port.clone(),
        value: Some(value.clone()),
    }));
    edits.extend(node.colours.iter().map(|(port, colour)| Edit::SetColour {
        node: id.to_string(),
        port: port.clone(),
        colour: Some(colour.clone()),
    }));
    edits.extend(
        before
            .links
            .iter()
            .filter(|link| link.source.node == id || link.target.node == id)
            .map(|Link { source, target }| Edit::AddLink {
                source: PortRef {
                    node: source.node.clone(),
                    port: source.port.clone(),
                },
                target: PortRef {
                    node: target.node.clone(),
                    port: target.port.clone(),
                },
            }),
    );
    edits
}
