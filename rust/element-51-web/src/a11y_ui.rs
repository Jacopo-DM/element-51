//! Keyboard and screen-reader access to the canvas: a hidden tree that mirrors
//! the graph. Its items take the keyboard focus, the canvas draws a ring on the
//! focused node or port, and the view pans to keep it in sight.

use element_51_core::access::{
    NodeItem, describe_ports, list_nodes, pan_to_show, with_displays, with_jobs,
};
use element_51_core::announce::{job_button_label, job_button_refusal};
use element_51_core::job::JobButton;
use element_51_core::layout::{Side, node_rect};
use element_51_core::rows::type_of;
use wasm_bindgen::prelude::*;
use web_sys::{Document, Element, FocusEvent, HtmlElement};

use crate::State;
use crate::draw::Focus;
use crate::{Shared, redraw};

/// The least gap, in pixels, between a focused node and the window edge. It is
/// big enough to clear the Run button and the menu button, so the focus is not
/// hidden behind them.
const FOCUS_MARGIN: f64 = 72.0;

/// Say `text` to a screen reader through the hidden status line. A repeated
/// sentence is cleared first, so it is read again.
pub(crate) fn announce(document: &Document, text: &str) {
    let Some(status) = document.get_element_by_id("status") else {
        return;
    };
    if status.text_content().as_deref() == Some(text) {
        status.set_text_content(None);
    }
    status.set_text_content(Some(text));
}

/// The id of the tree item for node `id`.
pub(crate) fn node_element_id(id: &str) -> String {
    format!("tree-{id}")
}

/// The id of the tree item for a port.
pub(crate) fn port_element_id(node: &str, port: &str, side: Side) -> String {
    format!("tree-{node}-{}-{port}", side_name(side))
}

/// The id of the tree item for a job button.
fn job_element_id(node: &str, button: JobButton) -> String {
    format!("tree-{node}-job-{}", button_name(button))
}

fn button_name(button: JobButton) -> &'static str {
    match button {
        JobButton::Start => "start",
        JobButton::Stop => "stop",
    }
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

/// The tree item of a node. Its ports are added by `ensure_ports` when the
/// keyboard first enters it, so rebuilding a big graph's tree stays cheap.
fn node_element(document: &Document, item: &NodeItem) -> Option<Element> {
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
    if item.port_count > 0 {
        li.set_attribute("aria-expanded", "false").ok()?;
    }
    Some(li)
}

/// The list of port items under the node item `node`, built the first time it
/// is wanted. It starts hidden; `expand_for` shows it.
pub(crate) fn ensure_ports(shared: &Shared, node: &Element) -> Option<Element> {
    if let Ok(Some(group)) = node.query_selector("ul") {
        return Some(group);
    }
    let id = node.get_attribute("data-node")?;
    let (document, ports, job) = {
        let state = shared.borrow();
        let replica = state.replica.as_ref()?;
        let is_job = replica
            .graph()
            .nodes
            .iter()
            .find(|n| n.id == id)
            .and_then(|n| type_of(replica.graph(), replica.node_types(), n))
            .is_some_and(|node_type| node_type.job);
        (
            state.document.clone(),
            describe_ports(replica.graph(), replica.node_types(), &id),
            is_job.then(|| state.jobs.get(&id).cloned()),
        )
    };
    if ports.is_empty() && job.is_none() {
        return None;
    }
    let group = element(&document, "ul", &[("role", "group"), ("hidden", "")])?;
    for port in &ports {
        let entry = element(
            &document,
            "li",
            &[
                ("role", "treeitem"),
                ("id", &port_element_id(&id, &port.port, port.side)),
                ("data-node", &id),
                ("data-port", &port.port),
                ("data-side", side_name(port.side)),
                ("aria-label", &port.label),
                ("tabindex", "-1"),
            ],
        )?;
        group.append_child(&entry).ok()?;
    }
    // A job node's buttons come after its ports. A button that does not work
    // now is `aria-disabled`, and its label says why.
    if let Some(update) = &job {
        for button in [JobButton::Start, JobButton::Stop] {
            let refused = job_button_refusal(button, update.as_ref()).is_some();
            let entry = element(
                &document,
                "li",
                &[
                    ("role", "treeitem"),
                    ("id", &job_element_id(&id, button)),
                    ("data-node", &id),
                    ("data-job", button_name(button)),
                    ("aria-label", &job_button_label(button, update.as_ref())),
                    ("tabindex", "-1"),
                ],
            )?;
            if refused {
                entry.set_attribute("aria-disabled", "true").ok()?;
            }
            group.append_child(&entry).ok()?;
        }
    }
    node.append_child(&group).ok()?;
    Some(group)
}

/// Rebuild the hidden tree if the replica changed since it was built. The
/// tree is hidden outright while the graph is empty, because a tree with no
/// items is not valid.
pub(crate) fn sync_tree(shared: &Shared) {
    let (document, items) = {
        let state = shared.borrow();
        let Some(replica) = state.replica.as_ref() else {
            return;
        };
        if !state.tree_stale {
            return;
        }
        let items = list_nodes(replica.graph(), replica.node_types());
        let items = match state.run.as_ref() {
            Some(result) => with_displays(items, replica.graph(), result),
            None => items,
        };
        (
            state.document.clone(),
            with_jobs(items, replica.graph(), replica.node_types(), &state.jobs),
        )
    };
    let Some(tree) = document.get_element_by_id("graph-tree") else {
        return;
    };
    // Rebuilding drops the keyboard focus with the old items, and removing the
    // focused item reports the focus as lost: remember both, and mark the tree
    // as built first so that report does not rebuild it again.
    let saved = {
        let mut state = shared.borrow_mut();
        state.tree_stale = false;
        state.focus.clone()
    };
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
    // A focused port needs its node's port list, which the rebuild dropped.
    shared.borrow_mut().focus = saved.clone();
    if let Some(Focus::Port { node, .. } | Focus::Job { node, .. }) = &saved
        && let Some(item) = document.get_element_by_id(&node_element_id(node))
    {
        ensure_ports(shared, &item);
    }
    let wanted = saved.as_ref().map(focus_element_id);
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
        Focus::Job { node, button } => job_element_id(node, *button),
    }
}

pub(crate) fn focus(item: &Element) {
    if let Ok(html) = item.clone().dyn_into::<HtmlElement>() {
        let _ = html.focus();
    }
}

/// Open the port list of the node that holds `item`, so a port can take focus.
pub(crate) fn expand_for(item: &Element) {
    if (item.has_attribute("data-port") || item.has_attribute("data-job"))
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
pub(crate) fn focus_of(item: &Element) -> Option<Focus> {
    let node = item.get_attribute("data-node")?;
    if let Some(button) = item.get_attribute("data-job") {
        let button = if button == "stop" {
            JobButton::Stop
        } else {
            JobButton::Start
        };
        return Some(Focus::Job { node, button });
    }
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
            Focus::Node(id) | Focus::Port { node: id, .. } | Focus::Job { node: id, .. } => {
                id.clone()
            }
        };
        pan_to_node(&mut state, &node_id);
        state.focus = Some(focus);
    }
    redraw(shared);
}

/// Pan the view, if it has to, so node `id` is fully in the window.
pub(crate) fn pan_to_node(state: &mut State, id: &str) {
    let size = (
        f64::from(state.canvas.client_width()),
        f64::from(state.canvas.client_height()),
    );
    let rect = state.replica.as_ref().and_then(|replica| {
        let node = replica.graph().nodes.iter().find(|n| n.id == id)?;
        let node_type = type_of(replica.graph(), replica.node_types(), node)?;
        Some(node_rect(node, &node_type, &state.theme.metrics))
    });
    if let Some(rect) = rect {
        let (dx, dy) = pan_to_show(&state.view, &rect, size, FOCUS_MARGIN);
        state.view.pan_by(dx, dy);
    }
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
        {
            let mut state = shared.borrow_mut();
            state.focus = None;
            state.pending_link = None;
        }
        redraw(shared);
    }
}
