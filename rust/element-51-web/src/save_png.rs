//! Save as PNG: Python draws the file. The page sends the lines of a chart or
//! the picture on a card, and downloads the PNG that comes back.

use element_51_core::display::line_style;
use element_51_core::menu::png_file_name;
use element_51_core::{
    CardDisplay, Message, PngChartRequest, PngLine, PngPictureRequest, PngReply,
};

use crate::a11y_ui::announce;
use crate::download::{Shown, download, nothing_to_save, shown_node};
use crate::draw_chart::{line_colour, plotted};
use crate::{Shared, State, show_error};

/// Ask Python for the file of what node `id` shows. A node with nothing to save
/// yet (no run, or a run that failed for it) gets a banner that says so, with
/// Dismiss; so does a connection that is gone.
pub(crate) fn save_png(shared: &Shared, id: &str) {
    let asked = request(&shared.borrow(), id);
    let document = shared.borrow().document.clone();
    match asked {
        Ok(name) => announce(&document, &format!("Making {name}")),
        Err(reason) => show_error(&document, &reason),
    }
}

/// Python's answer: download the file, or say why there is none.
pub(crate) fn on_png_reply(shared: &Shared, reply: &PngReply) {
    let document = shared.borrow().document.clone();
    match (&reply.data, &reply.reason) {
        (Some(data), _) => {
            let url = format!("data:image/png;base64,{data}");
            match download(&document, &reply.name, &url) {
                Ok(()) => announce(&document, &format!("Saving {}", reply.name)),
                Err(reason) => show_error(&document, &reason),
            }
        }
        (None, Some(reason)) => show_error(
            &document,
            &format!("Python could not make {}: {reason}", reply.name),
        ),
        (None, None) => show_error(&document, "Python sent an empty answer to the PNG request."),
    }
}

/// Send the request for node `id` and return the file name it asks for.
fn request(state: &State, id: &str) -> Result<String, String> {
    let shown = shown_node(state, id)?;
    let (name, message) = match shown.node_type.display {
        CardDisplay::None => return Err(format!("{id} shows no picture or chart.")),
        CardDisplay::Image => picture_request(id, &shown)?,
        CardDisplay::Chart => chart_request(state, id, &shown),
    };
    state
        .socket
        .send_with_str(&message.to_json())
        .map_err(|_| "Not connected to Python.".to_string())?;
    Ok(name)
}

/// The request for the first picture of the node, in port name order; so does
/// the card.
fn picture_request(id: &str, shown: &Shown) -> Result<(String, Message), String> {
    let (port, picture) = shown
        .outcome
        .shown
        .iter()
        .find_map(|(port, shown)| shown.as_image().map(|image| (port, image)))
        .ok_or_else(|| nothing_to_save(id))?;
    let name = png_file_name(id, port);
    let request = PngPictureRequest {
        node: id.to_string(),
        name: name.clone(),
        picture: picture.clone(),
    };
    Ok((name, Message::PngPicture(request)))
}

/// The request for the chart of the node: each line with the colour the card
/// draws it in, and whether the user chose it.
fn chart_request(state: &State, id: &str, shown: &Shown) -> (String, Message) {
    let lines = plotted(shown.graph, (shown.node, &shown.node_type), shown.outcome)
        .into_iter()
        .enumerate()
        .map(|(index, line)| {
            let (kind, _) = line_style(index);
            PngLine {
                colour: line_colour(&state.theme, kind, line.colour.as_deref()),
                chosen: line.colour.is_some(),
                name: line.name,
                values: line.series.values.clone(),
            }
        })
        .collect();
    let name = png_file_name(id, "chart");
    let request = PngChartRequest {
        node: id.to_string(),
        name: name.clone(),
        lines,
    };
    (name, Message::PngChart(request))
}
