//! Browser layer: connects to Python, draws the graph, and sends edits.

mod draw;

use std::cell::RefCell;
use std::rc::Rc;

use element_51_core::layout::{PortHit, Side, port_at};
use element_51_core::{Edit, EditRequest, Message, PortRef, Replica, RunRequest, RunResult};
use wasm_bindgen::convert::FromWasmAbi;
use wasm_bindgen::prelude::*;
use web_sys::{
    CanvasRenderingContext2d, CloseEvent, Document, Element, EventTarget, HtmlCanvasElement,
    MessageEvent, MouseEvent, WebSocket,
};

use draw::{Drag, draw};

/// Everything the event handlers share.
struct State {
    document: Document,
    canvas: HtmlCanvasElement,
    context: CanvasRenderingContext2d,
    socket: WebSocket,
    replica: Option<Replica>,
    drag: Option<Drag>,
    hover: Option<PortHit>,
    /// The last run's outcome. Cleared when the graph changes.
    run: Option<RunResult>,
    /// Where the right-click menu was opened: new nodes are placed here.
    menu_at: (f64, f64),
}

type Shared = Rc<RefCell<State>>;

/// The least gap between the right-click menu and the window edge, in pixels.
const MENU_MARGIN: f64 = 8.0;

/// Connect to `/ws`, draw the graph Python sends, and send the user's edits.
#[wasm_bindgen(start)]
pub fn start() -> Result<(), JsValue> {
    let window = web_sys::window().ok_or("no window")?;
    let document = window.document().ok_or("no document")?;
    let canvas: HtmlCanvasElement = document
        .get_element_by_id("canvas")
        .ok_or("no canvas element")?
        .dyn_into()?;
    let context: CanvasRenderingContext2d = canvas
        .get_context("2d")?
        .ok_or("no 2d context")?
        .dyn_into()?;
    let location = window.location();
    let scheme = if location.protocol()? == "https:" {
        "wss"
    } else {
        "ws"
    };
    let socket = WebSocket::new(&format!("{scheme}://{}/ws", location.host()?))?;

    let state: Shared = Rc::new(RefCell::new(State {
        document: document.clone(),
        canvas: canvas.clone(),
        context,
        socket: socket.clone(),
        replica: None,
        drag: None,
        hover: None,
        run: None,
        menu_at: (0.0, 0.0),
    }));
    fit_canvas(&state, &window);

    let shared = state.clone();
    listen(&socket, "message", move |event: MessageEvent| {
        on_message(&shared, &event)
    })?;
    let shared = state.clone();
    listen(&socket, "close", move |event: CloseEvent| {
        let reason = event.reason();
        let state = shared.borrow();
        if reason.is_empty() {
            show_error(
                &state.document,
                "The connection to Python closed. Reload the page.",
            );
        } else {
            show_error(
                &state.document,
                &format!("Python closed the connection: {reason}"),
            );
        }
    })?;
    let shared = state.clone();
    listen(&canvas, "contextmenu", move |event: MouseEvent| {
        on_context_menu(&shared, &event)
    })?;
    let shared = state.clone();
    listen(&canvas, "mousedown", move |event: MouseEvent| {
        on_mouse_down(&shared, &event)
    })?;
    let shared = state.clone();
    listen(&canvas, "mousemove", move |event: MouseEvent| {
        on_mouse_move(&shared, &event)
    })?;
    let shared = state.clone();
    listen(&canvas, "mouseup", move |event: MouseEvent| {
        on_mouse_up(&shared, &event)
    })?;
    if let Some(button) = document.get_element_by_id("run") {
        let shared = state.clone();
        listen(&button, "click", move |_event: MouseEvent| on_run(&shared))?;
    }
    if let Some(menu) = document.get_element_by_id("menu") {
        let shared = state.clone();
        listen(&menu, "click", move |event: MouseEvent| {
            on_menu_click(&shared, &event)
        })?;
    }
    let shared = state.clone();
    let resize_window = window.clone();
    listen(&window, "resize", move |_event: web_sys::Event| {
        fit_canvas(&shared, &resize_window);
        redraw(&shared);
    })?;
    Ok(())
}

/// Register `handler` for `name` events on `target`, for the page's lifetime.
fn listen<E: FromWasmAbi + 'static>(
    target: &EventTarget,
    name: &str,
    handler: impl FnMut(E) + 'static,
) -> Result<(), JsValue> {
    let closure = Closure::<dyn FnMut(E)>::new(handler);
    target.add_event_listener_with_callback(name, closure.as_ref().unchecked_ref())?;
    closure.forget();
    Ok(())
}

/// Make the canvas bitmap as big as the window, so one pixel is one pixel.
fn fit_canvas(state: &Shared, window: &web_sys::Window) {
    let width = window
        .inner_width()
        .ok()
        .and_then(|v| v.as_f64())
        .unwrap_or(800.0);
    let height = window
        .inner_height()
        .ok()
        .and_then(|v| v.as_f64())
        .unwrap_or(600.0);
    let state = state.borrow();
    state.canvas.set_width(width as u32);
    state.canvas.set_height(height as u32);
}

fn redraw(state: &Shared) {
    let state = state.borrow();
    let result = draw(
        &state.canvas,
        &state.context,
        state.replica.as_ref(),
        state.drag.as_ref(),
        state.hover.as_ref(),
        state.run.as_ref(),
    );
    if let Err(error) = result {
        show_error(&state.document, &format!("Drawing failed: {error:?}"));
    }
}

/// Put `message` in the error banner. It stays until the user presses Dismiss.
fn show_error(document: &Document, message: &str) {
    let Some(banner) = document.get_element_by_id("error") else {
        return;
    };
    let Ok(Some(text)) = banner.query_selector("span") else {
        return;
    };
    text.set_text_content(Some(message));
    let _ = banner.remove_attribute("hidden");
}

fn send(state: &State, request: EditRequest) {
    let text = Message::Edit(request).to_json();
    if state.socket.send_with_str(&text).is_err() {
        show_error(
            &state.document,
            "Could not send the edit: not connected to Python.",
        );
    }
}

/// Apply `edit` locally and send it. A refused edit shows its reason.
fn submit(shared: &Shared, edit: Edit) {
    {
        let mut state = shared.borrow_mut();
        let Some(replica) = state.replica.as_mut() else {
            return;
        };
        match replica.submit(edit) {
            Ok(request) => {
                send(&state, request);
                state.run = None;
            }
            Err(error) => show_error(&state.document, &error.to_string()),
        }
    }
    redraw(shared);
}

fn on_message(shared: &Shared, event: &MessageEvent) {
    let parsed = event
        .data()
        .as_string()
        .ok_or_else(|| "Python sent a message that is not text".to_string())
        .and_then(|text| Message::from_json(&text).map_err(|error| error.to_string()));
    {
        let mut state = shared.borrow_mut();
        match parsed {
            Ok(Message::Snapshot(snapshot)) => {
                state.run = None;
                fill_menu(&state.document, &snapshot);
                match state.replica.as_mut() {
                    Some(replica) => replica.reset(snapshot),
                    None => state.replica = Some(Replica::new(snapshot)),
                }
            }
            Ok(Message::Ack(ack)) => {
                if let Some(replica) = state.replica.as_mut() {
                    replica.ack(ack.id);
                }
            }
            Ok(Message::Reject(reject)) => {
                let known = state
                    .replica
                    .as_mut()
                    .is_some_and(|replica| replica.reject(reject.id));
                if known {
                    state.run = None;
                    show_error(
                        &state.document,
                        &format!("Python refused the edit: {}", reject.reason),
                    );
                }
            }
            Ok(Message::RunResult(result)) => {
                if let Some(summary) = result.error_summary() {
                    show_error(&state.document, &summary);
                }
                state.run = Some(result);
            }
            Ok(Message::Edit(_) | Message::Run(_)) => {
                show_error(
                    &state.document,
                    "Python sent a message only the browser should send.",
                );
            }
            Err(error) => show_error(&state.document, &error),
        }
    }
    redraw(shared);
}

/// One button per node type, each carrying its type name.
fn fill_menu(document: &Document, snapshot: &element_51_core::Snapshot) {
    let Some(menu) = document.get_element_by_id("menu") else {
        return;
    };
    menu.set_text_content(None);
    for node_type in &snapshot.node_types {
        let Ok(button) = document.create_element("button") else {
            continue;
        };
        let _ = button.set_attribute("type", "button");
        let _ = button.set_attribute("data-node-type", &node_type.name);
        button.set_text_content(Some(&node_type.name));
        let _ = menu.append_child(&button);
    }
}

fn on_run(shared: &Shared) {
    let state = shared.borrow();
    if state
        .socket
        .send_with_str(&Message::Run(RunRequest {}).to_json())
        .is_err()
    {
        show_error(
            &state.document,
            "Could not send the run request: not connected to Python.",
        );
    }
}

fn on_context_menu(shared: &Shared, event: &MouseEvent) {
    event.prevent_default();
    let mut state = shared.borrow_mut();
    state.menu_at = (f64::from(event.offset_x()), f64::from(event.offset_y()));
    let Some(menu) = state.document.get_element_by_id("menu") else {
        return;
    };
    let (left, top) = (f64::from(event.client_x()), f64::from(event.client_y()));
    let _ = menu.set_attribute("style", &format!("left: {left}px; top: {top}px"));
    let _ = menu.remove_attribute("hidden");
    // Now that it is laid out, move it back inside the window if it hangs out.
    let Some(window) = web_sys::window() else {
        return;
    };
    let size = |value: Result<JsValue, JsValue>| value.ok().and_then(|v| v.as_f64()).unwrap_or(0.0);
    let (width, height) = (size(window.inner_width()), size(window.inner_height()));
    let rect = menu.get_bounding_client_rect();
    let left = left
        .min(width - rect.width() - MENU_MARGIN)
        .max(MENU_MARGIN);
    let top = top
        .min(height - rect.height() - MENU_MARGIN)
        .max(MENU_MARGIN);
    let _ = menu.set_attribute("style", &format!("left: {left}px; top: {top}px"));
}

fn hide_menu(state: &State) {
    if let Some(menu) = state.document.get_element_by_id("menu") {
        let _ = menu.set_attribute("hidden", "");
    }
}

fn on_menu_click(shared: &Shared, event: &MouseEvent) {
    let edit = {
        let state = shared.borrow();
        let name = event
            .target()
            .and_then(|target| target.dyn_into::<Element>().ok())
            .and_then(|element| element.get_attribute("data-node-type"));
        hide_menu(&state);
        let (Some(name), Some(replica)) = (name, state.replica.as_ref()) else {
            return;
        };
        Edit::AddNode {
            id: replica.next_node_id(),
            node_type: name,
            x: state.menu_at.0,
            y: state.menu_at.1,
        }
    };
    submit(shared, edit);
}

fn on_mouse_down(shared: &Shared, event: &MouseEvent) {
    {
        let mut state = shared.borrow_mut();
        hide_menu(&state);
        if event.button() != 0 {
            return;
        }
        let (x, y) = (f64::from(event.offset_x()), f64::from(event.offset_y()));
        let hit = state
            .replica
            .as_ref()
            .and_then(|r| port_at(r.graph(), r.node_types(), x, y));
        if let Some(hit) = hit.filter(|hit| hit.side == Side::Output) {
            state.drag = Some(Drag {
                source: PortRef {
                    node: hit.node,
                    port: hit.port,
                },
                x,
                y,
            });
        }
    }
    redraw(shared);
}

fn on_mouse_move(shared: &Shared, event: &MouseEvent) {
    {
        let mut state = shared.borrow_mut();
        let (x, y) = (f64::from(event.offset_x()), f64::from(event.offset_y()));
        let hover = state
            .replica
            .as_ref()
            .and_then(|r| port_at(r.graph(), r.node_types(), x, y));
        state.hover = hover;
        let Some(drag) = state.drag.as_mut() else {
            return;
        };
        drag.x = x;
        drag.y = y;
    }
    redraw(shared);
}

fn on_mouse_up(shared: &Shared, event: &MouseEvent) {
    let edit = {
        let mut state = shared.borrow_mut();
        let Some(drag) = state.drag.take() else {
            return;
        };
        let (x, y) = (f64::from(event.offset_x()), f64::from(event.offset_y()));
        let hit = state
            .replica
            .as_ref()
            .and_then(|r| port_at(r.graph(), r.node_types(), x, y));
        hit.filter(|hit| hit.side == Side::Input)
            .map(|hit| Edit::AddLink {
                source: drag.source,
                target: PortRef {
                    node: hit.node,
                    port: hit.port,
                },
            })
    };
    match edit {
        Some(edit) => submit(shared, edit),
        None => redraw(shared),
    }
}
