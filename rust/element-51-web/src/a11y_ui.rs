//! Keyboard and screen-reader access to the canvas: a hidden tree that mirrors
//! the graph. Its items take the keyboard focus, the canvas draws a ring on the
//! focused node or port, and the view pans to keep it in sight.

use element_51_core::access::{Direction, NodeItem, describe_graph, neighbour, pan_to_show};
use element_51_core::layout::{Side, node_rect};
use wasm_bindgen::prelude::*;
use web_sys::{Document, Element, FocusEvent, HtmlElement, KeyboardEvent};

use crate::draw::Focus;
use crate::{Shared, redraw};

/// The least gap, in pixels, between a focused node and the window edge. It is
/// big enough to clear the Run button and the menu button, so the focus is not
/// hidden behind them.
const FOCUS_MARGIN: f64 = 72.0;

/// The id of the tree item for node `id`.
fn node_element_id(id: &str) -> String {
    format!("tree-{id}")
}

/// The id of the tree item for a port.
fn port_element_id(node: &str, port: &str, side: Side) -> String {
    format!("tree-{node}-{}-{port}", side_name(side))
}

fn side_name(side: Side) -> &'static str {
    match side {
        Side::Input => "input",
        Side::Output => "output",
    }
}

fn element(document: &Document, tag: &str, attributes: &[(&str, &str)]) -> Option<Element> {
    let element = document.create_element(tag).ok()?;
    for (name, value) in attributes {
        element.set_attribute(name, value).ok()?;
    }
    Some(element)
}

fn node_element(document: &Document, item: &NodeItem) -> Option<Element> {
    let expandable = if item.ports.is_empty() {
        Vec::new()
    } else {
        vec![("aria-expanded", "false")]
    };
    let li = element(
        document,
        "li",
        &[
            ("role", "treeitem"),
            ("id", &node_element_id(&item.id)),
            ("data-node", &item.id),
            ("aria-label", &item.label),
            ("tabindex", "-1"),
        ],
    )?;
    for (name, value) in expandable {
        li.set_attribute(name, value).ok()?;
    }
    if !item.ports.is_empty() {
        let group = element(document, "ul", &[("role", "group"), ("hidden", "")])?;
        for port in &item.ports {
            let entry = element(
                document,
                "li",
                &[
                    ("role", "treeitem"),
                    ("id", &port_element_id(&item.id, &port.port, port.side)),
                    ("data-node", &item.id),
                    ("data-port", &port.port),
                    ("data-side", side_name(port.side)),
                    ("aria-label", &port.label),
                    ("tabindex", "-1"),
                ],
            )?;
            group.append_child(&entry).ok()?;
        }
        li.append_child(&group).ok()?;
    }
    Some(li)
}

/// Rebuild the hidden tree if the graph is not the one it was built from. The
/// tree is hidden outright while the graph is empty, because a tree with no
/// items is not valid.
pub(crate) fn sync_tree(shared: &Shared) {
    let (document, items) = {
        let state = shared.borrow();
        let Some(replica) = state.replica.as_ref() else {
            return;
        };
        if state.tree_graph.as_ref() == Some(replica.graph()) {
            return;
        }
        (
            state.document.clone(),
            describe_graph(replica.graph(), replica.node_types()),
        )
    };
    let Some(tree) = document.get_element_by_id("graph-tree") else {
        return;
    };
    // Rebuilding drops the keyboard focus with the old items; remember it.
    let refocus = document
        .active_element()
        .filter(|active| tree.contains(Some(active)))
        .map(|active| active.id());
    tree.set_text_content(None);
    for item in &items {
        if let Some(entry) = node_element(&document, item) {
            let _ = tree.append_child(&entry);
        }
    }
    if items.is_empty() {
        let _ = tree.set_attribute("hidden", "");
    } else {
        let _ = tree.remove_attribute("hidden");
    }
    let wanted = {
        let mut state = shared.borrow_mut();
        state.tree_graph = state
            .replica
            .as_ref()
            .map(|replica| replica.graph().clone());
        state.focus.as_ref().map(focus_element_id)
    };
    // One tab stop: the focused item if it survived, else the first node.
    let stop = wanted
        .as_deref()
        .and_then(|id| document.get_element_by_id(id))
        .or_else(|| tree.first_element_child());
    if let Some(stop) = stop {
        let _ = stop.set_attribute("tabindex", "0");
    }
    if let Some(id) = refocus.filter(|id| !id.is_empty())
        && let Some(item) = document.get_element_by_id(&id)
    {
        expand_for(&item);
        focus(&item);
    }
}

fn focus_element_id(focus: &Focus) -> String {
    match focus {
        Focus::Node(id) => node_element_id(id),
        Focus::Port { node, port, side } => port_element_id(node, port, *side),
    }
}

fn focus(item: &Element) {
    if let Ok(html) = item.clone().dyn_into::<HtmlElement>() {
        let _ = html.focus();
    }
}

/// Open the port list of the node that holds `item`, so a port can take focus.
fn expand_for(item: &Element) {
    if item.has_attribute("data-port")
        && let Some(group) = item.parent_element()
    {
        let _ = group.remove_attribute("hidden");
        if let Some(node) = group.parent_element() {
            let _ = node.set_attribute("aria-expanded", "true");
        }
    }
}

/// Close every open port list except the one that holds `keep`.
fn collapse_others(tree: &Element, keep: &Element) {
    let Ok(open) = tree.query_selector_all("[aria-expanded=\"true\"]") else {
        return;
    };
    for index in 0..open.length() {
        let Some(node) = open.item(index).and_then(|n| n.dyn_into::<Element>().ok()) else {
            continue;
        };
        if keep.id() == node.id() || node.contains(Some(keep)) {
            continue;
        }
        let _ = node.set_attribute("aria-expanded", "false");
        if let Ok(Some(group)) = node.query_selector("ul") {
            let _ = group.set_attribute("hidden", "");
        }
    }
}

/// What the tree item `item` stands for.
fn focus_of(item: &Element) -> Option<Focus> {
    let node = item.get_attribute("data-node")?;
    match item.get_attribute("data-port") {
        None => Some(Focus::Node(node)),
        Some(port) => {
            let side = match item.get_attribute("data-side")?.as_str() {
                "input" => Side::Input,
                _ => Side::Output,
            };
            Some(Focus::Port { node, port, side })
        }
    }
}

/// A tree item took the keyboard focus: make it the one tab stop, draw the ring
/// on the canvas and pan so the node is in view.
pub(crate) fn on_tree_focus_in(shared: &Shared, event: &FocusEvent) {
    let Some(item) = event
        .target()
        .and_then(|target| target.dyn_into::<Element>().ok())
    else {
        return;
    };
    let Some(focus) = focus_of(&item) else {
        return;
    };
    let document = shared.borrow().document.clone();
    if let Some(tree) = document.get_element_by_id("graph-tree") {
        if let Ok(Some(previous)) = tree.query_selector("[tabindex=\"0\"]") {
            let _ = previous.set_attribute("tabindex", "-1");
        }
        let _ = item.set_attribute("tabindex", "0");
        collapse_others(&tree, &item);
    }
    {
        let mut state = shared.borrow_mut();
        let node_id = match &focus {
            Focus::Node(id) | Focus::Port { node: id, .. } => id.clone(),
        };
        let size = (
            f64::from(state.canvas.client_width()),
            f64::from(state.canvas.client_height()),
        );
        let rect = state.replica.as_ref().and_then(|replica| {
            let node = replica.graph().nodes.iter().find(|n| n.id == node_id)?;
            let node_type = replica
                .node_types()
                .iter()
                .find(|known| known.name == node.node_type)?;
            Some(node_rect(node, node_type, &state.theme.metrics))
        });
        if let Some(rect) = rect {
            let (dx, dy) = pan_to_show(&state.view, &rect, size, FOCUS_MARGIN);
            state.view.pan_by(dx, dy);
        }
        state.focus = Some(focus);
    }
    redraw(shared);
}

/// The focus left the tree: the canvas ring goes away.
pub(crate) fn on_tree_focus_out(shared: &Shared, event: &FocusEvent) {
    let document = shared.borrow().document.clone();
    let inside = event
        .related_target()
        .and_then(|target| target.dyn_into::<Element>().ok())
        .zip(document.get_element_by_id("graph-tree"))
        .is_some_and(|(next, tree)| tree.contains(Some(&next)));
    if !inside {
        shared.borrow_mut().focus = None;
        redraw(shared);
    }
}

/// Arrow keys, Home, End, PageUp and PageDown inside the tree.
pub(crate) fn on_tree_key(shared: &Shared, event: &KeyboardEvent) {
    if event.alt_key() || event.ctrl_key() || event.meta_key() || event.shift_key() {
        return;
    }
    let Some(item) = event
        .target()
        .and_then(|target| target.dyn_into::<Element>().ok())
    else {
        return;
    };
    let Some(focus_now) = focus_of(&item) else {
        return;
    };
    let next = match (&focus_now, event.key().as_str()) {
        (Focus::Node(id), "ArrowRight") => {
            first_port(&item).or_else(|| spatial(shared, id, Direction::Right))
        }
        (Focus::Node(id), "ArrowLeft") => spatial(shared, id, Direction::Left),
        (Focus::Node(id), "ArrowUp") => spatial(shared, id, Direction::Up),
        (Focus::Node(id), "ArrowDown") => spatial(shared, id, Direction::Down),
        (Focus::Node(_), "PageDown") => item.next_element_sibling(),
        (Focus::Node(_), "PageUp") => item.previous_element_sibling(),
        (Focus::Node(_), "Home") => item.parent_element().and_then(|t| t.first_element_child()),
        (Focus::Node(_), "End") => item.parent_element().and_then(|t| t.last_element_child()),
        (Focus::Port { .. }, "ArrowDown") => item.next_element_sibling(),
        (Focus::Port { .. }, "ArrowUp") => item.previous_element_sibling(),
        (Focus::Port { .. }, "ArrowLeft") => item.parent_element().and_then(|g| g.parent_element()),
        _ => return,
    };
    event.prevent_default();
    event.stop_propagation();
    if let Some(next) = next {
        expand_for(&next);
        focus(&next);
    }
}

/// The tree item of the first port of the node item `node`, if it has ports.
fn first_port(node: &Element) -> Option<Element> {
    let group = node.query_selector("ul").ok().flatten()?;
    group.first_element_child()
}

/// The tree item of the node nearest to `from` in `direction`.
fn spatial(shared: &Shared, from: &str, direction: Direction) -> Option<Element> {
    let (document, id) = {
        let state = shared.borrow();
        let replica = state.replica.as_ref()?;
        let id = neighbour(
            replica.graph(),
            replica.node_types(),
            &state.theme.metrics,
            from,
            direction,
        )?;
        (state.document.clone(), id)
    };
    document.get_element_by_id(&node_element_id(&id))
}
