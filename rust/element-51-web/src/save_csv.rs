//! Save as CSV: the numbers of a chart, as a file the browser downloads.
//! Python is not involved.

use element_51_core::csv::{chart_csv, csv_data_url};
use element_51_core::menu::csv_file_name;
use element_51_core::{CardDisplay, Series};

use crate::a11y_ui::announce;
use crate::download::{download, nothing_to_save, shown_node};
use crate::draw_chart::plotted;
use crate::{Shared, show_error};

/// Save the lines of the chart on node `id`. A node with nothing to save yet
/// (no run, or a run that failed for it) gets a banner that says so, with
/// Dismiss.
pub(crate) fn save_csv(shared: &Shared, id: &str) {
    let made = render(&shared.borrow(), id);
    let document = shared.borrow().document.clone();
    match made.and_then(|(name, url)| download(&document, &name, &url).map(|()| name)) {
        Ok(name) => announce(&document, &format!("Saving {name}")),
        Err(reason) => show_error(&document, &reason),
    }
}

/// The file name and the CSV, as a data URL, for the chart of node `id`.
fn render(state: &crate::State, id: &str) -> Result<(String, String), String> {
    let shown = shown_node(state, id)?;
    if shown.node_type.display != CardDisplay::Chart {
        return Err(format!("{id} shows no chart."));
    }
    let lines = plotted(shown.graph, (shown.node, &shown.node_type), shown.outcome);
    let named: Vec<(&str, &Series)> = lines
        .iter()
        .map(|line| (line.name.as_str(), line.series))
        .collect();
    let text = chart_csv(&named).ok_or_else(|| nothing_to_save(id))?;
    Ok((csv_file_name(id), csv_data_url(&text)))
}
