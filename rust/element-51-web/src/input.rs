//! Pointer and key handling on the canvas: pan, zoom, move, link, delete.

use element_51_core::layout::{PortHit, Side, icon_at, link_at, node_at, port_at};
use element_51_core::{Edit, Graph, PortRef, apply_edit};
use wasm_bindgen::JsCast;
use web_sys::{HtmlInputElement, KeyboardEvent, MouseEvent, WheelEvent};

use crate::draw::{Drag, Selected};
use crate::menu_ui::{hide_menu, open_card_menu};
use crate::{Shared, State, redraw, step_history, submit};

/// What the pointer is doing between a press and a release.
pub(crate) enum Gesture {
    None,
    /// Dragging a new link out of an output port.
    Link(Drag),
    /// Dragging a node. Nothing is sent until the button is released, so one
    /// drag is one edit.
    Move(Moving),
    /// Dragging the empty canvas; holds the last pointer position on screen.
    Pan(f64, f64),
}

pub(crate) struct Moving {
    id: String,
    /// Where inside the node the pointer took hold, in graph units.
    grab: (f64, f64),
    start: (f64, f64),
    now: (f64, f64),
}

/// How close, in screen pixels, a press must be to a link to select it.
const LINK_PICK_PIXELS: f64 = 6.0;
/// How strongly the wheel zooms: one wheel pixel scales by `exp(0.002)`, so a
/// mouse wheel notch (100 pixels) zooms by about 22%.
const WHEEL_ZOOM_RATE: f64 = 0.002;
/// The same for a trackpad pinch, which browsers report as a wheel with Ctrl
/// held and much smaller steps.
const PINCH_ZOOM_RATE: f64 = 0.01;

/// The graph with a node being dragged shown at the pointer, if one is. The
/// replica is not touched; the move is sent once, when the button is released.
pub(crate) fn dragged_graph(state: &State) -> Option<Graph> {
    let (Gesture::Move(moving), Some(replica)) = (&state.gesture, state.replica.as_ref()) else {
        return None;
    };
    let edit = Edit::MoveNode {
        id: moving.id.clone(),
        x: moving.now.0,
        y: moving.now.1,
    };
    apply_edit(replica.graph(), replica.node_types(), &edit).ok()
}

/// The pointer position on the canvas, in screen pixels.
fn screen_point(event: &MouseEvent) -> (f64, f64) {
    (f64::from(event.offset_x()), f64::from(event.offset_y()))
}

fn port_under(state: &State, x: f64, y: f64) -> Option<PortHit> {
    let replica = state.replica.as_ref()?;
    port_at(
        replica.graph(),
        replica.node_types(),
        x,
        y,
        &state.theme.metrics,
    )
}

pub(crate) fn on_mouse_down(shared: &Shared, event: &MouseEvent) {
    {
        let mut state = shared.borrow_mut();
        hide_menu(&state);
        if event.button() != 0 {
            return;
        }
        let screen = screen_point(event);
        let (x, y) = state.view.to_graph(screen.0, screen.1);
        let icon = state
            .replica
            .as_ref()
            .and_then(|r| icon_at(r.graph(), r.node_types(), x, y, &state.theme.metrics));
        if let Some(id) = icon {
            state.selected = Some(Selected::Node(id.clone()));
            open_card_menu(&mut state, id, event);
        } else if let Some(hit) = port_under(&state, x, y).filter(|hit| hit.side == Side::Output) {
            state.gesture = Gesture::Link(Drag {
                source: PortRef {
                    node: hit.node,
                    port: hit.port,
                },
                x,
                y,
            });
        } else if let Some(id) = state
            .replica
            .as_ref()
            .and_then(|r| node_at(r.graph(), r.node_types(), x, y, &state.theme.metrics))
        {
            let node = state
                .replica
                .as_ref()
                .and_then(|r| r.graph().nodes.iter().find(|node| node.id == id))
                .map(|node| (node.x, node.y));
            if let Some(start) = node {
                state.gesture = Gesture::Move(Moving {
                    id: id.clone(),
                    grab: (x - start.0, y - start.1),
                    start,
                    now: start,
                });
            }
            state.selected = Some(Selected::Node(id));
        } else {
            let tolerance = LINK_PICK_PIXELS / state.view.zoom();
            let link = state.replica.as_ref().and_then(|r| {
                link_at(
                    r.graph(),
                    r.node_types(),
                    x,
                    y,
                    tolerance,
                    &state.theme.metrics,
                )
            });
            match link {
                Some(link) => state.selected = Some(Selected::Link(link)),
                None => {
                    state.selected = None;
                    state.gesture = Gesture::Pan(screen.0, screen.1);
                }
            }
        }
    }
    redraw(shared);
}

pub(crate) fn on_mouse_move(shared: &Shared, event: &MouseEvent) {
    {
        let mut state = shared.borrow_mut();
        let screen = screen_point(event);
        let (x, y) = state.view.to_graph(screen.0, screen.1);
        state.hover = port_under(&state, x, y);
        match &mut state.gesture {
            Gesture::None => return,
            Gesture::Link(drag) => {
                drag.x = x;
                drag.y = y;
            }
            Gesture::Move(moving) => moving.now = (x - moving.grab.0, y - moving.grab.1),
            Gesture::Pan(last_x, last_y) => {
                let (dx, dy) = (screen.0 - *last_x, screen.1 - *last_y);
                *last_x = screen.0;
                *last_y = screen.1;
                state.view.pan_by(dx, dy);
            }
        }
    }
    redraw(shared);
}

pub(crate) fn on_mouse_up(shared: &Shared, event: &MouseEvent) {
    let edit = {
        let mut state = shared.borrow_mut();
        let screen = screen_point(event);
        let (x, y) = state.view.to_graph(screen.0, screen.1);
        match std::mem::replace(&mut state.gesture, Gesture::None) {
            Gesture::Link(drag) => port_under(&state, x, y)
                .filter(|hit| hit.side == Side::Input)
                .map(|hit| Edit::AddLink {
                    source: drag.source,
                    target: PortRef {
                        node: hit.node,
                        port: hit.port,
                    },
                }),
            Gesture::Move(moving) if moving.now != moving.start => Some(Edit::MoveNode {
                id: moving.id,
                x: moving.now.0,
                y: moving.now.1,
            }),
            Gesture::Move(_) | Gesture::Pan(..) | Gesture::None => None,
        }
    };
    match edit {
        Some(edit) => submit(shared, edit),
        None => redraw(shared),
    }
}

/// Zoom about the pointer. The wheel's own scrolling is not wanted.
pub(crate) fn on_wheel(shared: &Shared, event: &WheelEvent) {
    event.prevent_default();
    {
        let mut state = shared.borrow_mut();
        let rate = if event.ctrl_key() {
            PINCH_ZOOM_RATE
        } else {
            WHEEL_ZOOM_RATE
        };
        let factor = (-event.delta_y() * rate).exp();
        state.view.zoom_at(
            f64::from(event.offset_x()),
            f64::from(event.offset_y()),
            factor,
        );
    }
    redraw(shared);
}

/// Delete or Backspace removes what is selected; Escape lets go of it. Keys
/// typed into an input box belong to the box; Enter there ends the edit.
pub(crate) fn on_key_down(shared: &Shared, event: &KeyboardEvent) {
    if let Some(field) = event
        .target()
        .and_then(|target| target.dyn_into::<HtmlInputElement>().ok())
    {
        if event.key() == "Enter" {
            let _ = field.blur();
        }
        return;
    }
    if event.ctrl_key() || event.meta_key() {
        let key = event.key().to_lowercase();
        if key == "z" || key == "y" {
            event.prevent_default();
            step_history(shared, key == "z" && !event.shift_key());
        }
        return;
    }
    let edit = {
        let mut state = shared.borrow_mut();
        match event.key().as_str() {
            "Delete" | "Backspace" => match state.selected.take() {
                Some(Selected::Node(id)) => Some(Edit::DeleteNode { id }),
                Some(Selected::Link(link)) => Some(Edit::DeleteLink {
                    source: link.source,
                    target: link.target,
                }),
                None => None,
            },
            "Escape" => {
                state.selected = None;
                hide_menu(&state);
                None
            }
            _ => return,
        }
    };
    match edit {
        Some(edit) => {
            event.prevent_default();
            submit(shared, edit);
        }
        None => redraw(shared),
    }
}
