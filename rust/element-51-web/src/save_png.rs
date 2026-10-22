//! Save as PNG: the picture or chart on a card, as a file the browser
//! downloads. Python is not involved.

use element_51_core::menu::png_file_name;
use element_51_core::paper::{PAPER_HEIGHT, PAPER_WIDTH, paper_chart, picture_scale};
use element_51_core::rows::type_of;
use element_51_core::{CardDisplay, Status};
use wasm_bindgen::JsCast;
use web_sys::{CanvasRenderingContext2d, Document, HtmlCanvasElement, HtmlElement};

use crate::a11y_ui::announce;
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
    let nothing = || format!("{id} has nothing to save yet: press Run first.");
    let replica = state.replica.as_ref().ok_or("Not connected to Python.")?;
    let graph = replica.graph();
    let node = graph
        .nodes
        .iter()
        .find(|node| node.id == id)
        .ok_or_else(|| format!("{id} is not in the graph."))?;
    let node_type = type_of(graph, replica.node_types(), node)
        .ok_or_else(|| format!("{id} has an unknown node type."))?;
    let outcome = state
        .run
        .as_ref()
        .and_then(|result| result.outcome_for(id))
        .filter(|outcome| outcome.status == Status::Ok)
        .ok_or_else(nothing)?;
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

/// Hand the browser a link to download, and click it.
fn download(document: &Document, name: &str, url: &str) -> Result<(), String> {
    let failed = || "The browser could not start the download.".to_string();
    let link = document.create_element("a").map_err(|_| failed())?;
    link.set_attribute("href", url).map_err(|_| failed())?;
    link.set_attribute("download", name).map_err(|_| failed())?;
    let body = document.body().ok_or_else(failed)?;
    body.append_child(&link).map_err(|_| failed())?;
    if let Ok(link) = link.clone().dyn_into::<HtmlElement>() {
        link.click();
    }
    link.remove();
    Ok(())
}
