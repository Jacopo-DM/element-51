//! Save as PNG: the picture or chart on a card, as a file the browser
//! downloads. Python is not involved.

use element_51_core::CardDisplay;
use element_51_core::menu::png_file_name;
use element_51_core::paper::{PAPER_HEIGHT, PAPER_WIDTH, paper_chart, picture_scale};
use wasm_bindgen::JsCast;
use web_sys::{CanvasRenderingContext2d, Document, HtmlCanvasElement};

use crate::a11y_ui::announce;
use crate::download::{Shown, download, nothing_to_save, shown_node};
use crate::draw_chart::plotted;
use crate::paper_ui::{draw_chart, draw_picture, page, to_paper_url};
use crate::{Shared, State, show_error};

/// Save what node `id` shows. A node with nothing to save yet (no run, or a
/// run that failed for it) gets a banner that says so, with Dismiss.
pub(crate) fn save_png(shared: &Shared, id: &str) {
    let made = render(&shared.borrow(), id);
    let document = shared.borrow().document.clone();
    match made.and_then(|(name, url)| download(&document, &name, &url).map(|()| name)) {
        Ok(name) => announce(&document, &format!("Saving {name}")),
        Err(reason) => show_error(&document, &reason),
    }
}

/// The file name and the PNG, as a data URL, for node `id`'s picture or chart.
fn render(state: &State, id: &str) -> Result<(String, String), String> {
    let nothing = || nothing_to_save(id);
    let Shown {
        graph,
        node,
        node_type,
        outcome,
    } = shown_node(state, id)?;
    let failed = || "The browser could not make the picture.".to_string();
    match node_type.display {
        CardDisplay::None => Err(format!("{id} shows no picture or chart.")),
        CardDisplay::Image => {
            // The card shows the first picture in port name order; so does the file.
            let port = outcome
                .shown
                .iter()
                .find(|(_, shown)| shown.as_image().is_some())
                .map(|(port, _)| port)
                .ok_or_else(nothing)?;
            let picture = state.pictures.of(id).ok_or_else(nothing)?;
            let (_, width, height) = picture_scale(picture.width(), picture.height());
            let canvas = paper_canvas(&state.document, width, height).ok_or_else(failed)?;
            let context = context_of(&canvas).ok_or_else(failed)?;
            draw_picture(&context, picture, (width, height)).map_err(|_| failed())?;
            Ok((png_file_name(id, port), to_paper_url(&canvas)?))
        }
        CardDisplay::Chart => {
            let lines = plotted(graph, (node, &node_type), outcome);
            let series: Vec<(&str, &element_51_core::Series)> = lines
                .iter()
                .map(|line| (line.name.as_str(), line.series))
                .collect();
            let chart = paper_chart(&series).ok_or("The chart has no numbers to save.")?;
            let canvas =
                paper_canvas(&state.document, PAPER_WIDTH, PAPER_HEIGHT).ok_or_else(failed)?;
            let context = context_of(&canvas).ok_or_else(failed)?;
            page(&context);
            draw_chart(&context, &chart).map_err(|_| failed())?;
            Ok((png_file_name(id, "chart"), to_paper_url(&canvas)?))
        }
    }
}

fn paper_canvas(document: &Document, width: u32, height: u32) -> Option<HtmlCanvasElement> {
    let canvas: HtmlCanvasElement = document.create_element("canvas").ok()?.dyn_into().ok()?;
    canvas.set_width(width);
    canvas.set_height(height);
    Some(canvas)
}

fn context_of(canvas: &HtmlCanvasElement) -> Option<CanvasRenderingContext2d> {
    canvas
        .get_context("2d")
        .ok()
        .flatten()
        .and_then(|context| context.dyn_into().ok())
}
