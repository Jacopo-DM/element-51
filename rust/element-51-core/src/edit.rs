//! The rules that decide whether an edit is allowed.
//!
//! Python runs the same rules in `src/element_51/edits.py`;
//! `tests/fixtures/link_cases.json` is read by both test suites.

use std::fmt;

use crate::{Edit, Graph, Link, NodeType, PlacedNode, Port, PortRef, can_link};

/// Why an edit was refused.
#[derive(Clone, Debug, PartialEq)]
pub enum EditError {
    UnknownNodeType(String),
    IdInUse(String),
    UnknownNode(String),
    UnknownPort {
        node: String,
        port: String,
        output: bool,
    },
    TypeMismatch {
        source: crate::PortType,
        target: crate::PortType,
    },
    InputTaken(PortRef),
    Cycle,
    UnknownLink,
    BadValue {
        value: serde_json::Value,
        kind: crate::PortType,
        node: String,
        port: String,
    },
}

impl EditError {
    /// The category, spelled as in the wire format and the shared fixture.
    pub fn kind(&self) -> &'static str {
        match self {
            EditError::UnknownNodeType(_) => "unknown_node_type",
            EditError::IdInUse(_) => "id_in_use",
            EditError::UnknownNode(_) => "unknown_node",
            EditError::UnknownPort { .. } => "unknown_port",
            EditError::TypeMismatch { .. } => "type_mismatch",
            EditError::InputTaken(_) => "input_taken",
            EditError::Cycle => "cycle",
            EditError::UnknownLink => "unknown_link",
            EditError::BadValue { .. } => "bad_value",
        }
    }
}

impl fmt::Display for EditError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            EditError::UnknownNodeType(name) => write!(formatter, "unknown node type '{name}'"),
            EditError::IdInUse(id) => write!(formatter, "node id {id} is already in use"),
            EditError::UnknownNode(id) => write!(formatter, "unknown node {id}"),
            EditError::UnknownPort { node, port, output } => {
                let side = if *output { "output" } else { "input" };
                write!(formatter, "node {node} has no {side} port '{port}'")
            }
            EditError::TypeMismatch { source, target } => {
                write!(formatter, "cannot link {source} to {target}")
            }
            EditError::InputTaken(target) => {
                write!(
                    formatter,
                    "{}.{} already has a link",
                    target.node, target.port
                )
            }
            EditError::Cycle => write!(formatter, "that link would make a cycle"),
            EditError::UnknownLink => write!(formatter, "there is no such link"),
            EditError::BadValue {
                value,
                kind,
                node,
                port,
            } => write!(formatter, "{value} is not a {kind} for {node}.{port}"),
        }
    }
}

impl std::error::Error for EditError {}

/// The graph after `edit`, or the reason the rules refuse it.
pub fn apply_edit(graph: &Graph, node_types: &[NodeType], edit: &Edit) -> Result<Graph, EditError> {
    match edit {
        Edit::AddNode {
            id,
            node_type,
            x,
            y,
        } => {
            if !node_types.iter().any(|known| known.name == *node_type) {
                return Err(EditError::UnknownNodeType(node_type.clone()));
            }
            if graph.nodes.iter().any(|node| node.id == *id) {
                return Err(EditError::IdInUse(id.clone()));
            }
            let mut next = graph.clone();
            next.nodes.push(PlacedNode {
                id: id.clone(),
                node_type: node_type.clone(),
                x: *x,
                y: *y,
                values: Default::default(),
            });
            Ok(next)
        }
        Edit::AddLink { source, target } => {
            let from = port(graph, node_types, source, true)?;
            let to = port(graph, node_types, target, false)?;
            if !can_link(from.kind, to.kind) {
                return Err(EditError::TypeMismatch {
                    source: from.kind,
                    target: to.kind,
                });
            }
            if graph.links.iter().any(|link| link.target == *target) {
                return Err(EditError::InputTaken(target.clone()));
            }
            if reaches(graph, &target.node, &source.node) {
                return Err(EditError::Cycle);
            }
            let mut next = graph.clone();
            next.links.push(Link {
                source: source.clone(),
                target: target.clone(),
            });
            Ok(next)
        }
        Edit::MoveNode { id, x, y } => {
            let mut next = graph.clone();
            let node = next
                .nodes
                .iter_mut()
                .find(|node| node.id == *id)
                .ok_or_else(|| EditError::UnknownNode(id.clone()))?;
            node.x = *x;
            node.y = *y;
            Ok(next)
        }
        Edit::DeleteNode { id } => {
            if !graph.nodes.iter().any(|node| node.id == *id) {
                return Err(EditError::UnknownNode(id.clone()));
            }
            let mut next = graph.clone();
            next.nodes.retain(|node| node.id != *id);
            next.links
                .retain(|link| link.source.node != *id && link.target.node != *id);
            Ok(next)
        }
        Edit::SetValue {
            node,
            port: name,
            value,
        } => {
            let reference = PortRef {
                node: node.clone(),
                port: name.clone(),
            };
            let input = port(graph, node_types, &reference, false)?;
            if let Some(value) = value
                && !value_fits(value, input.kind)
            {
                return Err(EditError::BadValue {
                    value: value.clone(),
                    kind: input.kind,
                    node: node.clone(),
                    port: name.clone(),
                });
            }
            let mut next = graph.clone();
            if let Some(placed) = next.nodes.iter_mut().find(|placed| placed.id == *node) {
                match value {
                    Some(value) => placed.values.insert(name.clone(), value.clone()),
                    None => placed.values.remove(name),
                };
            }
            Ok(next)
        }
        Edit::DeleteLink { source, target } => {
            let position = graph
                .links
                .iter()
                .position(|link| link.source == *source && link.target == *target)
                .ok_or(EditError::UnknownLink)?;
            let mut next = graph.clone();
            next.links.remove(position);
            Ok(next)
        }
    }
}

/// Whether a port of type `kind` can carry `value`. Stricter than JSON: a bool
/// is not a number, an int port wants a whole number, and a float port takes
/// either.
pub fn value_fits(value: &serde_json::Value, kind: crate::PortType) -> bool {
    use crate::PortType;
    match value {
        serde_json::Value::Bool(_) => kind == PortType::Bool,
        serde_json::Value::Number(number) => match kind {
            PortType::Float => true,
            PortType::Int => number.is_i64() || number.is_u64(),
            PortType::Str | PortType::Bool => false,
        },
        serde_json::Value::String(_) => kind == PortType::Str,
        _ => false,
    }
}

/// The port `reference` names, as an output or as an input.
fn port<'a>(
    graph: &Graph,
    node_types: &'a [NodeType],
    reference: &PortRef,
    output: bool,
) -> Result<&'a Port, EditError> {
    let node = graph
        .nodes
        .iter()
        .find(|node| node.id == reference.node)
        .ok_or_else(|| EditError::UnknownNode(reference.node.clone()))?;
    let node_type = node_types
        .iter()
        .find(|known| known.name == node.node_type)
        .ok_or_else(|| EditError::UnknownNodeType(node.node_type.clone()))?;
    let ports = if output {
        &node_type.outputs
    } else {
        &node_type.inputs
    };
    ports
        .iter()
        .find(|candidate| candidate.name == reference.port)
        .ok_or_else(|| EditError::UnknownPort {
            node: reference.node.clone(),
            port: reference.port.clone(),
            output,
        })
}

/// Whether following links forward from node `start` reaches `goal`.
fn reaches(graph: &Graph, start: &str, goal: &str) -> bool {
    let mut seen: Vec<&str> = Vec::new();
    let mut stack = vec![start];
    while let Some(current) = stack.pop() {
        if current == goal {
            return true;
        }
        if seen.contains(&current) {
            continue;
        }
        seen.push(current);
        stack.extend(
            graph
                .links
                .iter()
                .filter(|link| link.source.node == current)
                .map(|link| link.target.node.as_str()),
        );
    }
    false
}
