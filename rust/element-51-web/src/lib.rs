//! Browser layer: connects to Python, draws the graph, and sends edits.
//!
//! This file holds the shared `State`, start-up, the websocket and the edit
//! flow. The canvas gestures are in `input`, the menus in `menu_ui` and the
//! value boxes in `fields_ui`.

mod a11y_ui;
mod autorun_ui;
mod canvas_ui;
mod download;
mod draw;
mod draw_card;
mod draw_chart;
mod draw_focus;
mod draw_wires;
mod edit_ui;
mod fields_ui;
mod input;
mod menu_keys;
mod menu_ui;
mod messages;
mod paper_ui;
mod pictures;
mod run_ui;
mod save_csv;
mod save_png;
mod source_ui;
mod swatch_ui;
mod theme_ui;
mod tree_keys;

use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use element_51_core::address::websocket_url;
use element_51_core::history::History;
use element_51_core::layout::PortHit;
use element_51_core::theme::Theme;
use element_51_core::view::View;
use element_51_core::{JobUpdate, PortRef, Replica, RunResult};
use wasm_bindgen::convert::FromWasmAbi;
use wasm_bindgen::prelude::*;
use web_sys::{
    CanvasRenderingContext2d, CloseEvent, Document, EventTarget, HtmlCanvasElement, KeyboardEvent,
    MessageEvent, MouseEvent, WebSocket, WheelEvent,
};

use crate::autorun_ui::on_toggle;
use crate::canvas_ui::{apply_theme, draw_canvas, fit_canvas, mark_ready};
use crate::edit_ui::{step_history, submit};
use crate::messages::on_message;
use crate::run_ui::{on_run, send_job_request, set_running};
use a11y_ui::{on_tree_focus_in, on_tree_focus_out, sync_tree};
use draw::{Focus, Selected};
use fields_ui::sync_fields;
use input::{Gesture, on_key_down, on_mouse_down, on_mouse_move, on_mouse_up, on_wheel};
use menu_keys::{on_card_menu_key, on_menu_key, on_menu_search_key};
use menu_ui::{on_card_menu_click, on_context_menu, on_menu_click, on_menu_hover, on_menu_search};
use pictures::Pictures;
use source_ui::{close_source, on_press_outside, on_source_key};
use swatch_ui::sync_swatches;
use theme_ui::{
    on_app_menu_button, on_palette_key, on_palette_submit, on_select_theme, on_theme_click,
    on_try_palette,
};
use tree_keys::on_tree_key;

/// Everything the event handlers share.
pub(crate) struct State {
    document: Document,
    canvas: HtmlCanvasElement,
    context: CanvasRenderingContext2d,
    socket: WebSocket,
    replica: Option<Replica>,
    theme: Theme,
    view: View,
    history: History,
    selected: Option<Selected>,
    /// The hidden tree's item that has the keyboard focus, if any.
    focus: Option<Focus>,
    /// The output port a keyboard link was started from, until it is finished
    /// or cancelled.
    pending_link: Option<PortRef>,
    /// Whether the replica changed since the hidden tree was built.
    tree_stale: bool,
    gesture: Gesture,
    hover: Option<PortHit>,
    /// The last run's outcome. Cleared when the graph changes.
    run: Option<RunResult>,
    /// The pictures of the last run, drawn once.
    pictures: Pictures,
    /// How each job that has been started stands, by node id.
    jobs: BTreeMap<String, JobUpdate>,
    /// Where the right-click menu was opened: new nodes are placed here.
    menu_at: (f64, f64),
    /// The node whose card menu is open, if any.
    card_menu_node: Option<String>,
    /// The theme name and changes the current theme was built from.
    theme_key: String,
    /// The `Run automatically` rule: the box, the waiting change, the run going.
    auto: element_51_core::autorun::AutoRun,
    /// Whether the error banner shows a failed run, so that a passing run clears it.
    run_failed: bool,
}

pub(crate) type Shared = Rc<RefCell<State>>;

#[wasm_bindgen::prelude::wasm_bindgen]
extern "C" {
    /// `JSON.parse`, to build the options object for the canvas context.
    #[wasm_bindgen(js_namespace = JSON, js_name = parse)]
    pub(crate) fn json_parse(text: &str) -> JsValue;
}

/// Connect to `/ws`, draw the graph Python sends, and send the user's edits.
#[wasm_bindgen(start)]
pub fn start() -> Result<(), JsValue> {
    let window = web_sys::window().ok_or("no window")?;
    let document = window.document().ok_or("no document")?;
    let canvas: HtmlCanvasElement = document
        .get_element_by_id("canvas")
        .ok_or("no canvas element")?
        .dyn_into()?;
    // An opaque canvas lets the browser use sharper text smoothing.
    let context: CanvasRenderingContext2d = canvas
        .get_context_with_context_options("2d", &json_parse("{\"alpha\": false}"))?
        .ok_or("no 2d context")?
        .dyn_into()?;
    let location = window.location();
    let socket = WebSocket::new(&websocket_url(
        &location.protocol()?,
        &location.host()?,
        &location.pathname()?,
    ))?;

    let theme = Theme::default();
    apply_theme(&document, &theme);
    let state: Shared = Rc::new(RefCell::new(State {
        document: document.clone(),
        canvas: canvas.clone(),
        context,
        socket: socket.clone(),
        replica: None,
        view: View::default(),
        theme,
        history: History::default(),
        selected: None,
        focus: None,
        pending_link: None,
        tree_stale: true,
        gesture: Gesture::None,
        hover: None,
        run: None,
        pictures: Pictures::default(),
        jobs: BTreeMap::new(),
        menu_at: (0.0, 0.0),
        card_menu_node: None,
        theme_key: String::new(),
        auto: element_51_core::autorun::AutoRun::new(),
        run_failed: false,
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
        set_running(&state.document, false);
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
    let shared = state.clone();
    listen(&canvas, "mouseleave", move |_event: MouseEvent| {
        shared.borrow_mut().gesture = Gesture::None;
        redraw(&shared);
    })?;
    let shared = state.clone();
    listen(&canvas, "wheel", move |event: WheelEvent| {
        on_wheel(&shared, &event)
    })?;
    let shared = state.clone();
    listen(&window, "keydown", move |event: KeyboardEvent| {
        on_key_down(&shared, &event)
    })?;
    if let Some(tree) = document.get_element_by_id("graph-tree") {
        let shared = state.clone();
        listen(&tree, "keydown", move |event: KeyboardEvent| {
            on_tree_key(&shared, &event)
        })?;
        let shared = state.clone();
        listen(&tree, "focusin", move |event: web_sys::FocusEvent| {
            on_tree_focus_in(&shared, &event)
        })?;
        let shared = state.clone();
        listen(&tree, "focusout", move |event: web_sys::FocusEvent| {
            on_tree_focus_out(&shared, &event)
        })?;
    }
    if let Some(button) = document.get_element_by_id("run") {
        let shared = state.clone();
        listen(&button, "click", move |_event: MouseEvent| on_run(&shared))?;
    }
    if let Some(auto) = document.get_element_by_id("auto-run") {
        let shared = state.clone();
        listen(&auto, "change", move |_event: web_sys::Event| {
            on_toggle(&shared)
        })?;
    }
    if let Some(menu) = document.get_element_by_id("menu") {
        let shared = state.clone();
        listen(&menu, "click", move |event: MouseEvent| {
            on_menu_click(&shared, &event)
        })?;
        let shared = state.clone();
        listen(&menu, "keydown", move |event: KeyboardEvent| {
            on_menu_key(&shared, &event)
        })?;
    }
    if let Some(menu) = document.get_element_by_id("menu") {
        let shared = state.clone();
        listen(&menu, "mouseover", move |event: MouseEvent| {
            on_menu_hover(&shared, &event)
        })?;
    }
    if let Some(card_menu) = document.get_element_by_id("card-menu") {
        let shared = state.clone();
        listen(&card_menu, "click", move |event: MouseEvent| {
            on_card_menu_click(&shared, &event)
        })?;
        let shared = state.clone();
        listen(&card_menu, "keydown", move |event: KeyboardEvent| {
            on_card_menu_key(&shared, &event)
        })?;
    }
    {
        let shared = state.clone();
        listen(&document, "mousedown", move |event: MouseEvent| {
            on_press_outside(&shared, &event)
        })?;
    }
    if let Some(close) = document.get_element_by_id("source-close") {
        let shared = state.clone();
        listen(&close, "click", move |_event: MouseEvent| {
            close_source(&shared)
        })?;
    }
    if let Some(panel) = document.get_element_by_id("source-panel") {
        let shared = state.clone();
        listen(&panel, "keydown", move |event: KeyboardEvent| {
            on_source_key(&shared, &event)
        })?;
    }
    if let Some(search) = document.get_element_by_id("menu-search") {
        let shared = state.clone();
        listen(&search, "input", move |_event: web_sys::Event| {
            on_menu_search(&shared)
        })?;
        let shared = state.clone();
        listen(&search, "keydown", move |event: KeyboardEvent| {
            on_menu_search_key(&shared, &event)
        })?;
    }
    if let Some(button) = document.get_element_by_id("app-menu-button") {
        let shared = state.clone();
        listen(&button, "click", move |_event: MouseEvent| {
            on_app_menu_button(&shared)
        })?;
    }
    if let Some(item) = document.get_element_by_id("select-theme") {
        let shared = state.clone();
        listen(&item, "click", move |_event: MouseEvent| {
            on_select_theme(&shared)
        })?;
    }
    if let Some(list) = document.get_element_by_id("theme-menu") {
        let shared = state.clone();
        listen(&list, "click", move |event: MouseEvent| {
            on_theme_click(&shared, &event)
        })?;
    }
    if let Some(item) = document.get_element_by_id("try-palette") {
        let shared = state.clone();
        listen(&item, "click", move |_event: MouseEvent| {
            on_try_palette(&shared)
        })?;
    }
    if let Some(form) = document.get_element_by_id("palette-box") {
        let shared = state.clone();
        listen(&form, "submit", move |event: web_sys::Event| {
            on_palette_submit(&shared, &event)
        })?;
    }
    if let Some(input) = document.get_element_by_id("palette-input") {
        let shared = state.clone();
        listen(&input, "keydown", move |event: KeyboardEvent| {
            on_palette_key(&shared, &event)
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

pub(crate) fn redraw(shared: &Shared) {
    sync_tree(shared);
    draw_canvas(shared);
    sync_fields(shared);
    sync_swatches(shared);
}

/// Hide the error banner.
pub(crate) fn hide_error(document: &Document) {
    if let Some(banner) = document.get_element_by_id("error") {
        let _ = banner.set_attribute("hidden", "");
    }
}

/// Put `message` in the error banner. It stays until the user presses Dismiss.
pub(crate) fn show_error(document: &Document, message: &str) {
    mark_ready(document);
    let Some(banner) = document.get_element_by_id("error") else {
        return;
    };
    let Ok(Some(text)) = banner.query_selector("span") else {
        return;
    };
    text.set_text_content(Some(message));
    let _ = banner.remove_attribute("hidden");
}
