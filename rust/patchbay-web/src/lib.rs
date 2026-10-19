//! Browser layer: draws on the canvas and talks to the page.

use patchbay_core::{PortType, can_link};
use wasm_bindgen::prelude::*;
use web_sys::{CanvasRenderingContext2d, HtmlCanvasElement};

/// Draw a first frame on the canvas with id `canvas`.
///
/// The text comes from `patchbay-core`, so the page shows that the engine ran.
#[wasm_bindgen(start)]
pub fn start() -> Result<(), JsValue> {
    let document = web_sys::window()
        .and_then(|window| window.document())
        .ok_or("no document")?;
    let canvas: HtmlCanvasElement = document
        .get_element_by_id("canvas")
        .ok_or("no canvas element")?
        .dyn_into()?;
    let context: CanvasRenderingContext2d = canvas
        .get_context("2d")?
        .ok_or("no 2d context")?
        .dyn_into()?;
    context.set_fill_style_str("#1e1e1e");
    context.fill_rect(
        0.0,
        0.0,
        f64::from(canvas.width()),
        f64::from(canvas.height()),
    );
    context.set_fill_style_str("#d0d0d0");
    context.set_font("16px sans-serif");
    let verdict = can_link(PortType::Float, PortType::Str);
    context.fill_text(
        &format!("patchbay: float to str link allowed: {verdict}"),
        20.0,
        40.0,
    )?;
    Ok(())
}
