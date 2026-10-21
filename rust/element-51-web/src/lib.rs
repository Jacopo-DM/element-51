//! Browser layer: connects to Python, draws the graph, and sends edits.
//!
//! This file holds the shared `State`, start-up, the websocket and the edit
//! flow. The canvas gestures are in `input`, the menus in `menu_ui` and the
//! value boxes in `fields_ui`.

mod a11y_ui;
mod draw;
mod draw_card;
mod draw_chart;
mod draw_focus;
mod draw_wires;
mod fields_ui;
mod input;
mod menu_keys;
mod menu_ui;
mod pictures;
mod save_png;
mod source_ui;
mod theme_ui;
mod tree_keys;

use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use element_51_core::address::websocket_url;
use element_51_core::history::History;
use element_51_core::job::JobButton;
use element_51_core::layout::PortHit;
use element_51_core::theme::Theme;
use element_51_core::view::View;
use element_51_core::{
    Edit, EditRequest, JobStatus, JobUpdate, Message, PortRef, Replica, RunRequest, RunResult,
    StartRequest, StopRequest,
};
use wasm_bindgen::convert::FromWasmAbi;
use wasm_bindgen::prelude::*;
use web_sys::{
    CanvasRenderingContext2d, CloseEvent, Document, EventTarget, HtmlCanvasElement, KeyboardEvent,
    MessageEvent, MouseEvent, WebSocket, WheelEvent,
};

use a11y_ui::{announce, on_tree_focus_in, on_tree_focus_out, sync_tree};
use draw::{Focus, Frame, Selected, draw};
use element_51_core::access::{edit_text, job_announcement, refusal_text, run_text, undo_text};
use fields_ui::sync_fields;
use input::{
    Gesture, dragged_graph, on_key_down, on_mouse_down, on_mouse_move, on_mouse_up, on_wheel,
    pending_preview,
};
use menu_keys::{on_card_menu_key, on_menu_key, on_menu_search_key};
use menu_ui::{
    fill_menu, on_card_menu_click, on_context_menu, on_menu_click, on_menu_hover, on_menu_search,
};
use pictures::Pictures;
use source_ui::{close_source, on_source_key, show_source};
use theme_ui::{
    on_app_menu_button, on_palette_key, on_palette_submit, on_select_theme, on_theme_click,
    on_try_palette,
};
use tree_keys::on_tree_key;

/// Everything the event handlers share.
struct State {
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
}

type Shared = Rc<RefCell<State>>;

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

/// Hand the theme's colours and font to the page's own parts (input boxes, the
/// menu, the error banner) as CSS custom properties on the root element.
fn apply_theme(document: &Document, theme: &Theme) {
    let Some(root) = document.document_element() else {
        return;
    };
    let style: String = theme
        .css_variables()
        .iter()
        .map(|(name, value)| format!("{name}: {value}; "))
        .collect();
    let _ = root.set_attribute("style", &style);
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
    // One canvas pixel per device pixel, or the browser stretches it and blurs it.
    let ratio = window.device_pixel_ratio();
    let state = state.borrow();
    state.canvas.set_width((width * ratio).round() as u32);
    state.canvas.set_height((height * ratio).round() as u32);
}

fn redraw(shared: &Shared) {
    sync_tree(shared);
    draw_canvas(shared);
    sync_fields(shared);
}

fn draw_canvas(state: &Shared) {
    let state = state.borrow();
    let preview = dragged_graph(&state);
    let keyboard_link = pending_preview(&state);
    let frame = Frame {
        theme: &state.theme,
        graph: preview
            .as_ref()
            .or_else(|| state.replica.as_ref().map(|replica| replica.graph())),
        types: state
            .replica
            .as_ref()
            .map_or(&[], |replica| replica.node_types()),
        view: &state.view,
        selected: state.selected.as_ref(),
        drag: match (&state.gesture, &keyboard_link) {
            (Gesture::Link(drag), _) => Some(drag),
            (_, Some((drag, _))) => Some(drag),
            _ => None,
        },
        hover: match &keyboard_link {
            Some((_, hit)) => hit.as_ref(),
            None => state.hover.as_ref(),
        },
        focus: state.focus.as_ref(),
        run: state.run.as_ref(),
        pictures: &state.pictures,
        jobs: &state.jobs,
    };
    let result = draw(&state.canvas, &state.context, &frame);
    if let Err(error) = result {
        show_error(&state.document, &format!("Drawing failed: {error:?}"));
    }
}

/// Show the page. It stays hidden until the first snapshot has set the theme, so
/// a light-theme page never shows the default dark one first.
fn mark_ready(document: &Document) {
    if let Some(root) = document.document_element() {
        root.set_class_name("ready");
    }
}

/// Put `message` in the error banner. It stays until the user presses Dismiss.
fn show_error(document: &Document, message: &str) {
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

fn send(state: &State, request: EditRequest) {
    let text = Message::Edit(request).to_json();
    if state.socket.send_with_str(&text).is_err() {
        show_error(
            &state.document,
            "Could not send the edit: not connected to Python.",
        );
    }
}

/// Apply an edit the user made, send it, and remember it so it can be undone.
fn submit(shared: &Shared, edit: Edit) {
    submit_edit(shared, edit, true);
}

/// Apply `edit` locally and send it. A refused edit shows its reason. With
/// `record` the edit joins the undo history; undo and redo pass false.
fn submit_edit(shared: &Shared, edit: Edit, record: bool) {
    {
        let mut state = shared.borrow_mut();
        let Some(replica) = state.replica.as_mut() else {
            return;
        };
        // Moving a node changes no result, so the shown values stay.
        let keeps_results = matches!(edit, Edit::MoveNode { .. });
        let before = replica.graph().clone();
        match replica.submit(edit.clone()) {
            Ok(request) => {
                if record {
                    state.history.record(&before, &edit);
                    announce(&state.document, &edit_text(&edit));
                }
                send(&state, request);
                state.tree_stale = true;
                if !keeps_results {
                    clear_run(&mut state);
                }
            }
            Err(error) => {
                show_error(&state.document, &error.to_string());
                announce(&state.document, &refusal_text(&error.to_string()));
            }
        }
    }
    redraw(shared);
}

/// Undo the newest step, or redo the newest undone one. The edits go to Python
/// like any others; the first one that no longer applies shows its reason and
/// ends the step.
fn step_history(shared: &Shared, undo: bool) {
    let edits = {
        let mut state = shared.borrow_mut();
        state.selected = None;
        if undo {
            state.history.undo()
        } else {
            state.history.redo()
        }
    };
    let changed = edits.as_ref().is_some_and(|edits| !edits.is_empty());
    let mut interrupted = false;
    for edit in edits.unwrap_or_default() {
        let before = shared.borrow().replica.as_ref().map(|r| r.graph().clone());
        submit_edit(shared, edit, false);
        let after = shared.borrow().replica.as_ref().map(|r| r.graph().clone());
        if before == after {
            interrupted = true;
            break;
        }
    }
    // A refused step has already been announced with its reason.
    if !interrupted {
        announce(&shared.borrow().document, &undo_text(undo, changed));
    }
    redraw(shared);
}

/// Forget the last run, then show what the jobs have reported: a job's results
/// are not the result of a run, so an edit must not lose them.
fn clear_run(state: &mut State) {
    state.run = None;
    show_jobs(state);
}

/// Put every job's reported outputs into the run result, and draw its pictures.
fn show_jobs(state: &mut State) {
    if state.jobs.is_empty() {
        return;
    }
    let mut run = state.run.take().unwrap_or_default();
    for update in state.jobs.values() {
        run.apply_job(update);
    }
    state.pictures = Pictures::from_result(&state.document, &run);
    state.run = Some(run);
}

/// A job changed: keep it, show what it reported, and tell the person when its
/// state changes. A failed job also shows its error in the banner.
fn on_job_update(state: &mut State, update: JobUpdate) {
    let before = state.jobs.get(&update.node).map(|job| job.state);
    let changed = before != Some(update.state);
    if changed {
        announce(&state.document, &job_announcement(&update));
    }
    if update.state == JobStatus::Failed && changed {
        let why = update.error.as_deref().unwrap_or("failed");
        show_error(&state.document, &format!("{}: {why}", update.node));
    }
    // The update joins the last result; it must not replace it, or the run after
    // another job, or a plain run, would be lost with every report.
    let mut run = state.run.take().unwrap_or_default();
    run.apply_job(&update);
    state.pictures = Pictures::from_result(&state.document, &run);
    state.run = Some(run);
    state.jobs.insert(update.node.clone(), update);
    if changed {
        state.tree_stale = true;
    }
}

/// Ask Python to start or stop the job of node `id`.
fn send_job_request(state: &State, id: &str, button: JobButton) {
    let node = id.to_string();
    let message = match button {
        JobButton::Start => Message::Start(StartRequest { node }),
        JobButton::Stop => Message::Stop(StopRequest { node }),
    };
    if state.socket.send_with_str(&message.to_json()).is_err() {
        show_error(
            &state.document,
            "Could not send the request: not connected to Python.",
        );
    }
}

fn on_message(shared: &Shared, event: &MessageEvent) {
    let parsed = event
        .data()
        .as_string()
        .ok_or_else(|| "Python sent a message that is not text".to_string())
        .and_then(|text| Message::from_json(&text).map_err(|error| error.to_string()));
    let mut source_reply = None;
    {
        let mut state = shared.borrow_mut();
        match parsed {
            Ok(Message::Snapshot(snapshot)) => {
                state.jobs = snapshot
                    .jobs
                    .iter()
                    .map(|job| (job.node.clone(), job.clone()))
                    .collect();
                clear_run(&mut state);
                state.selected = None;
                state.tree_stale = true;
                let theme_key = format!("{} {}", snapshot.theme, snapshot.theme_changes);
                if theme_key != state.theme_key {
                    match Theme::with_changes(&snapshot.theme, &snapshot.theme_changes) {
                        Ok(theme) => {
                            apply_theme(&state.document, &theme);
                            state.theme = theme;
                            state.theme_key = theme_key;
                        }
                        Err(reason) => show_error(
                            &state.document,
                            &format!("The theme cannot be used: {reason}"),
                        ),
                    }
                }
                mark_ready(&state.document);
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
                    state.tree_stale = true;
                    clear_run(&mut state);
                    show_error(
                        &state.document,
                        &format!("Python refused the edit: {}", reject.reason),
                    );
                    announce(&state.document, &refusal_text(&reject.reason));
                }
            }
            Ok(Message::RunResult(result)) => {
                if !result.partial {
                    set_running(&state.document, false);
                }
                if let Some(summary) = result.error_summary() {
                    show_error(&state.document, &summary);
                }
                // The run after a job follows the job's own announcement; it is only
                // spoken when something in it failed.
                if !result.partial || result.error_summary().is_some() {
                    announce(&state.document, &run_text(&result));
                }
                // A partial result covers the nodes after a job that ended; it
                // joins the last result instead of replacing it.
                let whole = match (result.partial, state.run.take()) {
                    (true, Some(mut run)) => {
                        run.merge(&result);
                        run
                    }
                    _ => result,
                };
                state.pictures = Pictures::from_result(&state.document, &whole);
                state.tree_stale = true;
                state.run = Some(whole);
            }
            Ok(Message::SourceReply(reply)) => source_reply = Some(reply),
            Ok(Message::JobUpdate(update)) => on_job_update(&mut state, update),
            Ok(Message::JobRefused(refusal)) => {
                show_error(&state.document, &refusal.reason);
                announce(&state.document, &refusal_text(&refusal.reason));
            }
            Ok(
                Message::Edit(_)
                | Message::Run(_)
                | Message::Source(_)
                | Message::Start(_)
                | Message::Stop(_),
            ) => {
                show_error(
                    &state.document,
                    "Python sent a message only the browser should send.",
                );
            }
            Err(error) => show_error(&state.document, &error),
        }
    }
    redraw(shared);
    // The panel moves the focus, which runs handlers that borrow the state, so
    // it is shown once the borrow above has ended.
    if let Some(reply) = source_reply {
        show_source(shared, &reply);
    }
}

/// Press Run: send the request and show that a run is in progress until its
/// result comes. A press while a run is in progress does nothing.
fn on_run(shared: &Shared) {
    let state = shared.borrow();
    let busy = state
        .document
        .get_element_by_id("run")
        .is_some_and(|button| button.get_attribute("aria-busy").as_deref() == Some("true"));
    if busy {
        return;
    }
    if state
        .socket
        .send_with_str(&Message::Run(RunRequest {}).to_json())
        .is_err()
    {
        show_error(
            &state.document,
            "Could not send the run request: not connected to Python.",
        );
    } else {
        set_running(&state.document, true);
    }
}

/// Show the Run button as busy, `Running`, or as it was. It stays focusable
/// (`aria-busy`, not `disabled`), so a keyboard user keeps their place.
fn set_running(document: &Document, running: bool) {
    let Some(button) = document.get_element_by_id("run") else {
        return;
    };
    if running {
        let _ = button.set_attribute("aria-busy", "true");
        let _ = button.set_attribute("aria-disabled", "true");
        button.set_text_content(Some("Running"));
    } else {
        let _ = button.remove_attribute("aria-busy");
        let _ = button.remove_attribute("aria-disabled");
        button.set_text_content(Some("Run"));
    }
}
