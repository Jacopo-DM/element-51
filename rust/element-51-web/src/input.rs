//! Pointer and key handling on the canvas: pan, zoom, move, link, delete.

use element_51_core::access::{Direction, ZOOM_STEP, link_edit, pan_delta};
use element_51_core::announce::{pan_text, zoom_text};
use element_51_core::job::job_button_at;
use element_51_core::layout::{PortHit, Side, anchor_of, icon_at, link_at, node_at, port_at};
use element_51_core::{Edit, Graph, PortRef, apply_edit};
use wasm_bindgen::JsCast;
use web_sys::{HtmlInputElement, KeyboardEvent, MouseEvent, WheelEvent};

use crate::a11y_ui::announce;
use crate::draw::Focus;
use crate::draw::{Drag, Selected};
use crate::fields_ui::{restore_field, return_from_field, tip_at};
use crate::menu_ui::{hide_menu, open_card_menu};
use crate::source_ui::{close_source, source_open};
use crate::{Shared, State, redraw, send_job_request, step_history, submit};

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
        let button = state.replica.as_ref().and_then(|r| {
            job_button_at(
                r.graph(),
                r.node_types(),
                &state.jobs,
                (x, y),
                &state.theme.metrics,
            )
        });
        if let Some(id) = icon {
            state.selected = Some(Selected::Node(id.clone()));
            open_card_menu(&mut state, id, event);
        } else if let Some((id, button)) = button {
            state.selected = Some(Selected::Node(id.clone()));
            send_job_request(&state, &id, button);
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
        // The canvas's own hover text names what a picture or series box shows.
        match tip_at(&state, x, y) {
            Some(tip) => state.canvas.set_title(&tip),
            None => {
                if !state.canvas.title().is_empty() {
                    state.canvas.set_title("");
                }
            }
        }
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
                .map(|hit| {
                    let target = PortRef {
                        node: hit.node,
                        port: hit.port,
                    };
                    link_edit(&drag.source, &target)
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
    // The source panel is on top of everything: Escape closes it wherever the
    // focus is.
    if event.key() == "Escape" && source_open(&shared.borrow().document) {
        event.prevent_default();
        close_source(shared);
        return;
    }
    if let Some(field) = event
        .target()
        .and_then(|target| target.dyn_into::<HtmlInputElement>().ok())
    {
        if field.id().starts_with("field-") {
            match event.key().as_str() {
                "Enter" => {
                    let _ = field.blur();
                    return_from_field(shared, &field);
                }
                "Escape" if field.has_attribute("data-original") => {
                    restore_field(&field);
                    let _ = field.blur();
                    return_from_field(shared, &field);
                }
                _ => {}
            }
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

pub(crate) fn arrow_direction(key: &str) -> Option<Direction> {
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
pub(crate) fn zoom_key(shared: &Shared, key: &str) -> Option<f64> {
    match key {
        "+" | "=" => Some(ZOOM_STEP),
        "-" | "_" => Some(1.0 / ZOOM_STEP),
        "0" => Some(1.0 / shared.borrow().view.zoom()),
        _ => None,
    }
}

/// Zoom about the middle of the window and say the new zoom.
pub(crate) fn zoom_view(shared: &Shared, factor: f64) {
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
pub(crate) fn pan_view(shared: &Shared, direction: Direction) {
    let document = {
        let mut state = shared.borrow_mut();
        let (dx, dy) = pan_delta(direction);
        state.view.pan_by(dx, dy);
        state.document.clone()
    };
    announce(&document, &pan_text(direction));
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
