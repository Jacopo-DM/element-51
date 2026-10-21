//! Inputs that grow as they are linked.
//!
//! A node type with a `Variadic` (the Plot node) has rows named
//! `<prefix>_1`, `<prefix>_2`, and so on. In a graph it shows one row for each
//! row that something is linked to, in order of row number, and one empty row
//! after them, named one past the highest row in use, where the next link can
//! land. Everything that places or hit-tests a card works on the type as
//! `resolve_type` gives it for that node, so the card's height, its port
//! anchors and its hidden tree all follow the links.

use std::borrow::Cow;

use crate::{Graph, NodeType, PlacedNode, Port, Variadic};

/// The row number in port name `name`: the prefix, an underscore and a positive
/// whole number with no sign and no leading zero. None for any other name.
pub fn variadic_index(variadic: &Variadic, name: &str) -> Option<u64> {
    let (head, tail) = name.rsplit_once('_')?;
    let digits = !tail.is_empty() && tail.bytes().all(|byte| byte.is_ascii_digit());
    if head != variadic.prefix || !digits || tail.starts_with('0') {
        return None;
    }
    tail.parse().ok()
}

/// The input `name` of a node type, with the rows that grow included: a listed
/// input, or for a variadic type any row, which has the variadic kind.
pub fn find_input(node_type: &NodeType, name: &str) -> Option<Port> {
    if let Some(port) = node_type.inputs.iter().find(|port| port.name == name) {
        return Some(port.clone());
    }
    let variadic = node_type.variadic.as_ref()?;
    variadic_index(variadic, name).map(|_| Port {
        name: name.to_string(),
        kind: variadic.kind,
    })
}

/// The inputs node `node_id` has in `graph`: the listed inputs that are not
/// rows, then each linked row in order of row number, then one empty row.
pub fn effective_inputs(node_type: &NodeType, graph: &Graph, node_id: &str) -> Vec<Port> {
    let Some(variadic) = node_type.variadic.as_ref() else {
        return node_type.inputs.clone();
    };
    let mut ports: Vec<Port> = node_type
        .inputs
        .iter()
        .filter(|port| variadic_index(variadic, &port.name).is_none())
        .cloned()
        .collect();
    let mut rows: Vec<u64> = graph
        .links
        .iter()
        .filter(|link| link.target.node == node_id)
        .filter_map(|link| variadic_index(variadic, &link.target.port))
        .collect();
    rows.sort_unstable();
    rows.dedup();
    let next = rows.last().map_or(1, |last| last + 1);
    ports.extend(
        rows.into_iter()
            .chain(std::iter::once(next))
            .map(|row| Port {
                name: format!("{}_{row}", variadic.prefix),
                kind: variadic.kind,
            }),
    );
    ports
}

/// The node type as node `node_id` has it in `graph`: the same type with the
/// rows that grew as its inputs. A type that does not grow is returned as it
/// is, without copying.
pub fn resolve_type<'a>(
    node_type: &'a NodeType,
    graph: &Graph,
    node_id: &str,
) -> Cow<'a, NodeType> {
    if node_type.variadic.is_none() {
        return Cow::Borrowed(node_type);
    }
    Cow::Owned(NodeType {
        inputs: effective_inputs(node_type, graph, node_id),
        ..node_type.clone()
    })
}

/// The resolved type of `node`, or None if its type is not in `node_types`.
pub fn type_of<'a>(
    graph: &Graph,
    node_types: &'a [NodeType],
    node: &PlacedNode,
) -> Option<Cow<'a, NodeType>> {
    let known = node_types
        .iter()
        .find(|known| known.name == node.node_type)?;
    Some(resolve_type(known, graph, &node.id))
}
