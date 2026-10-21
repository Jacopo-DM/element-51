//! The hidden tree's keyboard: arrows, selecting, links, editing, moving, deleting, zoom and pan.

use crate::a11y_ui::announce;
use crate::a11y_ui::ensure_ports;
use crate::a11y_ui::expand_for;
use crate::a11y_ui::focus;
use crate::a11y_ui::focus_of;
use crate::a11y_ui::node_element_id;
use crate::a11y_ui::pan_to_node;
use crate::draw::{Focus, Selected};
use crate::input::{arrow_direction, pan_view, zoom_key, zoom_view};
use crate::menu_keys::{open_add_menu_from_keyboard, open_card_menu_from_keyboard};
use crate::{Shared, redraw, send_job_request, submit};
use element_51_core::access::{Direction, link_edit, moved_to, neighbour};
use element_51_core::announce::{
    cannot_delete_text, editing_text, job_button_refusal, link_cancelled_text, link_started_text,
    linked_input_text, picture_input_text, selected_text,
};
use element_51_core::job::JobButton;
use element_51_core::layout::Side;
use element_51_core::rows::find_input;
use element_51_core::{Edit, PortRef, PortType, Replica};
use wasm_bindgen::prelude::*;
use web_sys::{Element, HtmlInputElement, KeyboardEvent};

/// Arrow keys, Home, End, PageUp and PageDown inside the tree; with Shift an
/// arrow key moves the node, with Alt it pans the view, and `+`, `-` and `0`
/// zoom.
pub(crate) fn on_tree_key(shared: &Shared, event: &KeyboardEvent) {
    if event.ctrl_key() || event.meta_key() {
        return;
    }
    let key = event.key();
    if !event.alt_key() {
        if key.eq_ignore_ascii_case("n") {
            event.prevent_default();
            event.stop_propagation();
            open_add_menu_from_keyboard(shared);
            return;
        }
        if key == "ContextMenu" || (key == "F10" && event.shift_key()) {
            event.prevent_default();
            event.stop_propagation();
            let node = event
                .target()
                .and_then(|target| target.dyn_into::<Element>().ok())
                .and_then(|item| item.get_attribute("data-node"));
            if let Some(node) = node {
                open_card_menu_from_keyboard(shared, &node);
            }
            return;
        }
    }
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
                Focus::Job { node, button } => press_job(shared, node, *button),
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
        (Focus::Port { .. } | Focus::Job { .. }, "ArrowDown") => item.next_element_sibling(),
        (Focus::Port { .. } | Focus::Job { .. }, "ArrowUp") => item.previous_element_sibling(),
        (Focus::Port { .. } | Focus::Job { .. }, "ArrowLeft") => {
            item.parent_element().and_then(|g| g.parent_element())
        }
        _ => return,
    };
    event.prevent_default();
    event.stop_propagation();
    if let Some(next) = next {
        expand_for(&next);
        focus(&next);
    }
}

/// Shift+arrow on a node moves it one grid step, Alt+Shift+arrow four; each
/// press is one edit and so one undo step. A port item does nothing.
pub(crate) fn move_focused(shared: &Shared, event: &KeyboardEvent, direction: Direction) {
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

/// Enter on a job button: ask Python to start or stop the job, or say why the
/// button does nothing now. The job's own updates announce the result.
pub(crate) fn press_job(shared: &Shared, node: &str, button: JobButton) {
    let state = shared.borrow();
    match job_button_refusal(button, state.jobs.get(node)) {
        Some(why) => announce(&state.document, &format!("Not available: {why}")),
        None => send_job_request(&state, node, button),
    }
}

/// Enter on a port: an output starts a link, an input finishes one.
pub(crate) fn press_port(shared: &Shared, end: PortRef, side: Side) {
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
                None => edit_value(shared, &end),
            }
        }
    }
}

/// Enter on an input port with no link started: move the focus into its value
/// box, panned into view first, or say that a linked input has none.
/// The type of the input `end` names, if its node and port exist.
pub(crate) fn input_kind(replica: &Replica, end: &PortRef) -> Option<PortType> {
    let node = replica
        .graph()
        .nodes
        .iter()
        .find(|node| node.id == end.node)?;
    let node_type = replica
        .node_types()
        .iter()
        .find(|known| known.name == node.node_type)?;
    find_input(node_type, &end.port).map(|port| port.kind)
}

pub(crate) fn edit_value(shared: &Shared, end: &PortRef) {
    let document = shared.borrow().document.clone();
    let field = document
        .get_element_by_id(&format!("field-{}-{}", end.node, end.port))
        .and_then(|field| field.dyn_into::<HtmlInputElement>().ok());
    let Some(field) = field else {
        let is_picture = shared
            .borrow()
            .replica
            .as_ref()
            .and_then(|replica| input_kind(replica, end))
            == Some(PortType::Image);
        announce(
            &document,
            &if is_picture {
                picture_input_text()
            } else {
                linked_input_text()
            },
        );
        return;
    };
    pan_to_node(&mut shared.borrow_mut(), &end.node);
    redraw(shared);
    // Keep what the box held, so Escape can put it back.
    let original = if field.type_() == "checkbox" {
        field.checked().to_string()
    } else {
        field.value()
    };
    let _ = field.set_attribute("data-original", &original);
    focus(&field);
    announce(&document, &editing_text(&end.node, &end.port));
}

/// Escape with a link started: drop it.
pub(crate) fn cancel_link(shared: &Shared) {
    let document = {
        let mut state = shared.borrow_mut();
        state.pending_link = None;
        state.document.clone()
    };
    announce(&document, &link_cancelled_text());
    redraw(shared);
}

/// Enter or Space on a node: select it, as a click does.
pub(crate) fn select_node(shared: &Shared, id: &str) {
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
pub(crate) fn delete_focused(shared: &Shared, item: &Element, focus_now: &Focus) {
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
        }
        | Focus::Job { .. } => announce(&document, &cannot_delete_text(Side::Output)),
    }
}

/// The tree item of the first port of the node item `node`, if it has ports.
pub(crate) fn first_port(shared: &Shared, node: &Element) -> Option<Element> {
    ensure_ports(shared, node)?.first_element_child()
}

/// The tree item of the node nearest to `from` in `direction`.
pub(crate) fn spatial(shared: &Shared, from: &str, direction: Direction) -> Option<Element> {
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
