//! Keyboard and screen-reader access to the canvas: a hidden tree that mirrors
//! the graph. Its items take the keyboard focus, the canvas draws a ring on the
//! focused node or port, and the view pans to keep it in sight.

use element_51_core::access::{
    Direction, NodeItem, ZOOM_STEP, cannot_delete_text, describe_ports, link_cancelled_text,
    link_edit, link_started_text, list_nodes, moved_to, neighbour, no_link_started_text, pan_delta,
    pan_text, pan_to_show, selected_text, zoom_text,
};
use element_51_core::layout::{PortHit, Side, anchor_of, node_rect};
use element_51_core::{Edit, PortRef};
use wasm_bindgen::prelude::*;
use web_sys::{Document, Element, FocusEvent, HtmlElement, KeyboardEvent};

use crate::State;
use crate::draw::{Drag, Focus, Selected};
use crate::{Shared, redraw, submit};

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
fn ensure_ports(shared: &Shared, node: &Element) -> Option<Element> {
    if let Ok(Some(group)) = node.query_selector("ul") {
        return Some(group);
    }
    let id = node.get_attribute("data-node")?;
    let (document, ports) = {
        let state = shared.borrow();
        let replica = state.replica.as_ref()?;
        (
            state.document.clone(),
            describe_ports(replica.graph(), replica.node_types(), &id),
        )
    };
    if ports.is_empty() {
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
        (
            state.document.clone(),
            list_nodes(replica.graph(), replica.node_types()),
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
    if let Some(Focus::Port { node, .. }) = &saved
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
        {
            let mut state = shared.borrow_mut();
            state.focus = None;
            state.pending_link = None;
        }
        redraw(shared);
    }
}

/// Arrow keys, Home, End, PageUp and PageDown inside the tree; with Shift an
/// arrow key moves the node, with Alt it pans the view, and `+`, `-` and `0`
/// zoom.
pub(crate) fn on_tree_key(shared: &Shared, event: &KeyboardEvent) {
    if event.ctrl_key() || event.meta_key() {
        return;
    }
    let key = event.key();
    if let Some(direction) = arrow_direction(&key)
        && (event.alt_key() || event.shift_key())
    {
        event.prevent_default();
        event.stop_propagation();
        if event.shift_key() {
            move_focused(shared, event, direction);
        } else {
            pan_view(shared, direction);
        }
        return;
    }
    if !event.alt_key()
        && let Some(factor) = zoom_key(shared, &key)
    {
        event.prevent_default();
        event.stop_propagation();
        zoom_view(shared, factor);
        return;
    }
    if event.alt_key() || event.shift_key() {
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
    match event.key().as_str() {
        "Enter" | " " => {
            event.prevent_default();
            event.stop_propagation();
            match &focus_now {
                Focus::Node(id) => select_node(shared, id),
                Focus::Port { node, port, side } => {
                    let end = PortRef {
                        node: node.clone(),
                        port: port.clone(),
                    };
                    press_port(shared, end, *side);
                }
            }
            return;
        }
        "Escape" if shared.borrow().pending_link.is_some() => {
            event.stop_propagation();
            cancel_link(shared);
            return;
        }
        "Delete" | "Backspace" => {
            event.prevent_default();
            event.stop_propagation();
            delete_focused(shared, &item, &focus_now);
            return;
        }
        _ => {}
    }
    let next = match (&focus_now, event.key().as_str()) {
        (Focus::Node(id), "ArrowRight") => {
            first_port(shared, &item).or_else(|| spatial(shared, id, Direction::Right))
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

fn arrow_direction(key: &str) -> Option<Direction> {
    match key {
        "ArrowLeft" => Some(Direction::Left),
        "ArrowRight" => Some(Direction::Right),
        "ArrowUp" => Some(Direction::Up),
        "ArrowDown" => Some(Direction::Down),
        _ => None,
    }
}

/// The zoom factor a key asks for, if it is a zoom key: `+` or `=` in, `-` or
/// `_` out, `0` back to 100 percent.
fn zoom_key(shared: &Shared, key: &str) -> Option<f64> {
    match key {
        "+" | "=" => Some(ZOOM_STEP),
        "-" | "_" => Some(1.0 / ZOOM_STEP),
        "0" => Some(1.0 / shared.borrow().view.zoom()),
        _ => None,
    }
}

/// Zoom about the middle of the window and say the new zoom.
fn zoom_view(shared: &Shared, factor: f64) {
    let (document, zoom) = {
        let mut state = shared.borrow_mut();
        let centre = (
            f64::from(state.canvas.client_width()) / 2.0,
            f64::from(state.canvas.client_height()) / 2.0,
        );
        state.view.zoom_at(centre.0, centre.1, factor);
        (state.document.clone(), state.view.zoom())
    };
    announce(&document, &zoom_text(zoom));
    redraw(shared);
}

/// Pan the view toward `direction` and say so.
fn pan_view(shared: &Shared, direction: Direction) {
    let document = {
        let mut state = shared.borrow_mut();
        let (dx, dy) = pan_delta(direction);
        state.view.pan_by(dx, dy);
        state.document.clone()
    };
    announce(&document, &pan_text(direction));
    redraw(shared);
}

/// Shift+arrow on a node moves it one grid step, Alt+Shift+arrow four; each
/// press is one edit and so one undo step. A port item does nothing.
fn move_focused(shared: &Shared, event: &KeyboardEvent, direction: Direction) {
    let Some(Focus::Node(id)) = event
        .target()
        .and_then(|target| target.dyn_into::<Element>().ok())
        .and_then(|item| focus_of(&item))
    else {
        return;
    };
    let edit = {
        let state = shared.borrow();
        state.replica.as_ref().and_then(|replica| {
            let node = replica.graph().nodes.iter().find(|node| node.id == id)?;
            let (x, y) = moved_to(
                node.x,
                node.y,
                direction,
                event.alt_key(),
                &state.theme.metrics,
            );
            Some(Edit::MoveNode {
                id: id.clone(),
                x,
                y,
            })
        })
    };
    if let Some(edit) = edit {
        submit(shared, edit);
    }
}

/// Enter on a port: an output starts a link, an input finishes one.
fn press_port(shared: &Shared, end: PortRef, side: Side) {
    let document = shared.borrow().document.clone();
    match side {
        Side::Output => {
            shared.borrow_mut().pending_link = Some(end.clone());
            announce(&document, &link_started_text(&end));
            redraw(shared);
        }
        Side::Input => {
            let source = shared.borrow_mut().pending_link.take();
            match source {
                Some(source) => submit(shared, link_edit(&source, &end)),
                None => announce(&document, &no_link_started_text()),
            }
        }
    }
}

/// Escape with a link started: drop it.
fn cancel_link(shared: &Shared) {
    let document = {
        let mut state = shared.borrow_mut();
        state.pending_link = None;
        state.document.clone()
    };
    announce(&document, &link_cancelled_text());
    redraw(shared);
}

/// The wire to draw for a link started from the keyboard: from its output port
/// to the input port that has the focus, or to the output port itself while the
/// focus is elsewhere. The input port that has the focus is also returned, so
/// the canvas colours it green or red as it does under the mouse.
pub(crate) fn pending_preview(state: &State) -> Option<(Drag, Option<PortHit>)> {
    let source = state.pending_link.as_ref()?;
    let replica = state.replica.as_ref()?;
    let (graph, types, metrics) = (replica.graph(), replica.node_types(), &state.theme.metrics);
    let from = anchor_of(graph, types, source, Side::Output, metrics)?;
    let target = match &state.focus {
        Some(Focus::Port {
            node,
            port,
            side: Side::Input,
        }) => Some(PortRef {
            node: node.clone(),
            port: port.clone(),
        }),
        _ => None,
    };
    let end = target
        .as_ref()
        .and_then(|target| anchor_of(graph, types, target, Side::Input, metrics))
        .unwrap_or(from);
    let hit = target.map(|target| PortHit {
        node: target.node,
        port: target.port,
        side: Side::Input,
    });
    let drag = Drag {
        source: source.clone(),
        x: end.0,
        y: end.1,
    };
    Some((drag, hit))
}

/// Enter or Space on a node: select it, as a click does.
fn select_node(shared: &Shared, id: &str) {
    let document = {
        let mut state = shared.borrow_mut();
        state.selected = Some(Selected::Node(id.to_string()));
        state.document.clone()
    };
    announce(&document, &selected_text(id));
    redraw(shared);
}

/// Delete on a tree item: a node goes, an input port loses its link, anything
/// else says why nothing happened. The focus moves to a neighbour so it is not
/// lost with the item.
fn delete_focused(shared: &Shared, item: &Element, focus_now: &Focus) {
    let document = shared.borrow().document.clone();
    match focus_now {
        Focus::Node(id) => {
            let neighbour_id = item
                .next_element_sibling()
                .or_else(|| item.previous_element_sibling())
                .and_then(|other| other.get_attribute("data-node"));
            {
                let mut state = shared.borrow_mut();
                if state.selected == Some(Selected::Node(id.clone())) {
                    state.selected = None;
                }
            }
            submit(shared, Edit::DeleteNode { id: id.clone() });
            let next = neighbour_id
                .and_then(|other| document.get_element_by_id(&node_element_id(&other)))
                .or_else(|| document.get_element_by_id("run"));
            if let Some(next) = next {
                focus(&next);
            }
        }
        Focus::Port {
            node,
            port,
            side: Side::Input,
        } => {
            let link = {
                let state = shared.borrow();
                state.replica.as_ref().and_then(|replica| {
                    replica
                        .graph()
                        .links
                        .iter()
                        .find(|link| link.target.node == *node && link.target.port == *port)
                        .cloned()
                })
            };
            match link {
                Some(link) => submit(
                    shared,
                    Edit::DeleteLink {
                        source: link.source,
                        target: link.target,
                    },
                ),
                None => announce(&document, &cannot_delete_text(Side::Input)),
            }
        }
        Focus::Port {
            side: Side::Output, ..
        } => announce(&document, &cannot_delete_text(Side::Output)),
    }
}

/// The tree item of the first port of the node item `node`, if it has ports.
fn first_port(shared: &Shared, node: &Element) -> Option<Element> {
    ensure_ports(shared, node)?.first_element_child()
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
