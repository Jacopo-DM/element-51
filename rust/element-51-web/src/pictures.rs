//! The pictures a run produced, ready to draw.
//!
//! Each is turned into an off-screen canvas once, when the result arrives, so a
//! frame only has to copy it onto the card.

use std::collections::HashMap;

use element_51_core::{Image, RunResult};
use wasm_bindgen::{Clamped, JsCast};
use web_sys::{CanvasRenderingContext2d, Document, HtmlCanvasElement, ImageData};

/// One off-screen canvas per node that showed a picture, by node id. A card
/// shows the first picture of its node, in port name order.
#[derive(Default)]
pub struct Pictures {
    by_node: HashMap<String, HtmlCanvasElement>,
}

impl Pictures {
    /// Draw every node's first picture onto its own canvas. A picture whose
    /// pixels do not fill it is left out, and the card shows an empty area.
    pub fn from_result(document: &Document, result: &RunResult) -> Pictures {
        let by_node = result
            .outcomes
            .iter()
            .filter_map(|outcome| {
                let image = outcome.shown.values().next()?;
                Some((outcome.node.clone(), canvas_of(document, image)?))
            })
            .collect();
        Pictures { by_node }
    }

    /// The picture node `id` showed, if any.
    pub fn of(&self, id: &str) -> Option<&HtmlCanvasElement> {
        self.by_node.get(id)
    }
}

fn canvas_of(document: &Document, image: &Image) -> Option<HtmlCanvasElement> {
    let rgba = image.rgba()?;
    let canvas: HtmlCanvasElement = document.create_element("canvas").ok()?.dyn_into().ok()?;
    canvas.set_width(image.width);
    canvas.set_height(image.height);
    let context: CanvasRenderingContext2d = canvas.get_context("2d").ok()??.dyn_into().ok()?;
    let data =
        ImageData::new_with_u8_clamped_array_and_sh(Clamped(&rgba), image.width, image.height)
            .ok()?;
    context.put_image_data(&data, 0.0, 0.0).ok()?;
    Some(canvas)
}
