//! Browser layer: connects to Python and draws what it sends.

use element_51_core::layout::{HEADER_HEIGHT, ROW_HEIGHT, node_rect};
use element_51_core::{Message, Snapshot};
use wasm_bindgen::prelude::*;
use web_sys::{
    CanvasRenderingContext2d, CloseEvent, Document, HtmlCanvasElement, MessageEvent, WebSocket,
};

const BACKGROUND: &str = "#1e1e1e";
const BODY: &str = "#3a3a3a";
const HEADER: &str = "#555555";
const TEXT: &str = "#d0d0d0";

/// Connect to `/ws`, and draw each snapshot Python sends.
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

    let on_message = {
        let document = document.clone();
        Closure::<dyn FnMut(MessageEvent)>::new(move |event: MessageEvent| {
            if let Err(error) = on_message(&canvas, &context, &event) {
                show_error(&document, &error);
            }
        })
    };
    socket.set_onmessage(Some(on_message.as_ref().unchecked_ref()));
    on_message.forget();

    let on_close = Closure::<dyn FnMut(CloseEvent)>::new(move |_event: CloseEvent| {
        show_error(
            &document,
            "The connection to Python closed. Reload the page.",
        );
    });
    socket.set_onclose(Some(on_close.as_ref().unchecked_ref()));
    on_close.forget();
    Ok(())
}

fn on_message(
    canvas: &HtmlCanvasElement,
    context: &CanvasRenderingContext2d,
    event: &MessageEvent,
) -> Result<(), String> {
    let text = event
        .data()
        .as_string()
        .ok_or("Python sent a message that is not text")?;
    let message = Message::from_json(&text).map_err(|error| error.to_string())?;
    let Message::Snapshot(snapshot) = message;
    draw(canvas, context, &snapshot).map_err(|error| format!("{error:?}"))
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

fn draw(
    canvas: &HtmlCanvasElement,
    context: &CanvasRenderingContext2d,
    snapshot: &Snapshot,
) -> Result<(), JsValue> {
    context.set_fill_style_str(BACKGROUND);
    context.fill_rect(
        0.0,
        0.0,
        f64::from(canvas.width()),
        f64::from(canvas.height()),
    );
    context.set_font("13px sans-serif");
    for node in &snapshot.graph.nodes {
        let Some(node_type) = snapshot.node_type(&node.node_type) else {
            continue;
        };
        let rect = node_rect(node, node_type);
        context.set_fill_style_str(BODY);
        context.fill_rect(rect.x, rect.y, rect.w, rect.h);
        context.set_fill_style_str(HEADER);
        context.fill_rect(rect.x, rect.y, rect.w, HEADER_HEIGHT);
        context.set_fill_style_str(TEXT);
        context.set_text_align("left");
        context.fill_text(
            &format!("{}  {}", node.id, node_type.name),
            rect.x + 8.0,
            rect.y + 18.0,
        )?;
        for (row, port) in node_type.inputs.iter().enumerate() {
            let y = rect.y + HEADER_HEIGHT + ROW_HEIGHT * (row as f64 + 0.5) + 4.0;
            context.fill_text(&port.name, rect.x + 8.0, y)?;
        }
        context.set_text_align("right");
        for (row, port) in node_type.outputs.iter().enumerate() {
            let y = rect.y + HEADER_HEIGHT + ROW_HEIGHT * (row as f64 + 0.5) + 4.0;
            context.fill_text(&port.name, rect.x + rect.w - 8.0, y)?;
        }
    }
    Ok(())
}
