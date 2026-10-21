//! Save as PNG: the picture or chart on a card, as a file the browser
//! downloads. Python is not involved.

use element_51_core::display::chart_rect;
use element_51_core::layout::node_rect;
use element_51_core::menu::png_file_name;
use element_51_core::rows::type_of;
use element_51_core::{CardDisplay, Status};
use wasm_bindgen::JsCast;
use web_sys::{CanvasRenderingContext2d, Document, HtmlCanvasElement, HtmlElement};

use crate::a11y_ui::announce;
use crate::draw_chart::chart;
use crate::{Shared, State, show_error};

/// How many pixels a chart gets for each pixel of the card, so the file is
/// sharp when it is zoomed.
const CHART_SCALE: f64 = 4.0;

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
    let failed = |_| "The browser could not make the picture.".to_string();
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
            let canvas = state.pictures.of(id).ok_or_else(nothing)?;
            let url = canvas.to_data_url().map_err(failed)?;
            Ok((png_file_name(id, port), url))
        }
        CardDisplay::Chart => {
            let rect = node_rect(node, &node_type, &state.theme.metrics);
            let area = chart_rect(&rect, &node_type, &state.theme.metrics)
                .ok_or("This card has no chart.")?;
            let canvas = new_canvas(
                &state.document,
                (area.w * CHART_SCALE).round() as u32,
                (area.h * CHART_SCALE).round() as u32,
            )
            .ok_or("The browser could not make the picture.")?;
            let context: CanvasRenderingContext2d = canvas
                .get_context("2d")
                .ok()
                .flatten()
                .and_then(|context| context.dyn_into().ok())
                .ok_or("The browser could not make the picture.")?;
            context.scale(CHART_SCALE, CHART_SCALE).map_err(failed)?;
            context.translate(-area.x, -area.y).map_err(failed)?;
            context.set_fill_style_str(&state.theme.colours.field);
            context.fill_rect(area.x, area.y, area.w, area.h);
            chart(
                &context,
                &state.theme,
                graph,
                (node, &node_type),
                (&area, outcome),
            )
            .map_err(failed)?;
            let url = canvas.to_data_url().map_err(failed)?;
            Ok((png_file_name(id, "chart"), url))
        }
    }
}

fn new_canvas(document: &Document, width: u32, height: u32) -> Option<HtmlCanvasElement> {
    let canvas: HtmlCanvasElement = document.create_element("canvas").ok()?.dyn_into().ok()?;
    canvas.set_width(width);
    canvas.set_height(height);
    Some(canvas)
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
