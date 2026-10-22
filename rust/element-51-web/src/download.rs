//! What Save as PNG and Save as CSV share: finding the node's last result and
//! handing the browser a file to download.

use element_51_core::rows::type_of;
use element_51_core::{Graph, NodeOutcome, NodeType, PlacedNode, Status};
use std::borrow::Cow;
use wasm_bindgen::JsCast;
use web_sys::{Document, HtmlElement};

use crate::State;

/// A node, its type and what its last run gave it.
pub(crate) struct Shown<'a> {
    pub graph: &'a Graph,
    pub node: &'a PlacedNode,
    pub node_type: Cow<'a, NodeType>,
    pub outcome: &'a NodeOutcome,
}

/// Node `id` with its type and its result, or the sentence that says why it has
/// nothing to save: not connected, not in the graph, an unknown type, or no
/// good result yet.
pub(crate) fn shown_node<'a>(state: &'a State, id: &str) -> Result<Shown<'a>, String> {
    let replica = state.replica.as_ref().ok_or("Not connected to Python.")?;
    let graph = replica.graph();
    let node = graph
        .nodes
        .iter()
        .find(|node| node.id == id)
        .ok_or_else(|| format!("{id} is not in the graph."))?;
    let node_type = type_of(graph, replica.node_types(), node)
        .ok_or_else(|| format!("{id} has an unknown node type."))?;
    let outcome = state
        .run
        .as_ref()
        .and_then(|result| result.outcome_for(id))
        .filter(|outcome| outcome.status == Status::Ok)
        .ok_or_else(|| nothing_to_save(id))?;
    Ok(Shown {
        graph,
        node,
        node_type,
        outcome,
    })
}

/// The sentence for a node that has no good result to save.
pub(crate) fn nothing_to_save(id: &str) -> String {
    format!("{id} has nothing to save yet: press Run first.")
}

/// Hand the browser a link to download, and click it.
pub(crate) fn download(document: &Document, name: &str, url: &str) -> Result<(), String> {
    let failed = || "The browser could not start the download.".to_string();
    let link = document.create_element("a").map_err(|_| failed())?;
    link.set_attribute("href", url).map_err(|_| failed())?;
    link.set_attribute("download", name).map_err(|_| failed())?;
    let body = document.body().ok_or_else(failed)?;
    body.append_child(&link).map_err(|_| failed())?;
    if let Ok(link) = link.clone().dyn_into::<HtmlElement>() {
        link.click();
    }
    link.remove();
    Ok(())
}
