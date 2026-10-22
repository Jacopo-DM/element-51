//! A card's chart: legend, gridlines, axes and one line per series.

use crate::draw::fit_text;
use element_51_core::display::{chart_layout, legend_slots, line_names, line_style};
use element_51_core::layout::Rect;
use element_51_core::theme::Theme;
use element_51_core::{Graph, NodeOutcome, NodeType, PlacedNode, PortType, Series};
use wasm_bindgen::JsValue;
use web_sys::CanvasRenderingContext2d;

/// The chart of a plot node: a faint grid, the axes and tick labels, one line
/// per linked series, and a legend row above the plot.
pub(crate) fn chart(
    context: &CanvasRenderingContext2d,
    theme: &Theme,
    graph: &Graph,
    (node, node_type): (&PlacedNode, &NodeType),
    (area, outcome): (&Rect, &NodeOutcome),
) -> Result<(), JsValue> {
    let (colours, metrics) = (&theme.colours, &theme.metrics);
    let lines = plotted(graph, (node, node_type), outcome);
    let series: Vec<&Series> = lines.iter().map(|line| line.series).collect();
    let Some(layout) = chart_layout(area, metrics, &series) else {
        return Ok(());
    };
    let plot = layout.plot;
    context.set_font(&theme.fonts.value);
    context.set_line_width(1.0);
    context.set_stroke_style_str(&colours.card_border);
    context.set_fill_style_str(&colours.field_text);
    let room = plot.x - area.x - metrics.box_pad;
    // The grid is faint; the two axes are drawn over it at full strength.
    context.set_global_alpha(GRID_ALPHA);
    context.set_text_align("right");
    for tick in &layout.y_ticks {
        context.begin_path();
        context.move_to(plot.x, tick.at);
        context.line_to(plot.x + plot.w, tick.at);
        context.stroke();
    }
    for tick in &layout.x_ticks {
        context.begin_path();
        context.move_to(tick.at, plot.y);
        context.line_to(tick.at, plot.y + plot.h);
        context.stroke();
    }
    context.set_global_alpha(1.0);
    context.begin_path();
    context.move_to(plot.x, plot.y);
    context.line_to(plot.x, plot.y + plot.h);
    context.line_to(plot.x + plot.w, plot.y + plot.h);
    context.stroke();
    for tick in &layout.y_ticks {
        let label = fit_text(context, &tick.label, room)?;
        context.fill_text(
            &label,
            plot.x - metrics.box_pad / 2.0,
            tick.at + metrics.text_baseline,
        )?;
    }
    context.set_text_align("center");
    for tick in &layout.x_ticks {
        context.fill_text(
            &tick.label,
            tick.at,
            plot.y + plot.h + metrics.row_height * 0.5 + metrics.text_baseline,
        )?;
    }
    context.save();
    context.begin_path();
    context.rect(plot.x, plot.y, plot.w, plot.h);
    context.clip();
    context.set_line_width(metrics.wire_width * LINE_WIDTH_RATIO);
    context.set_line_join("round");
    context.set_line_cap("round");
    for (index, runs) in layout.lines.iter().enumerate() {
        let (kind, dashed) = line_style(index);
        context.set_stroke_style_str(&line_colour(theme, kind, lines[index].colour.as_deref()));
        if dashed {
            context.set_line_dash(&crate::json_parse("[6, 4]"))?;
        }
        for run in runs {
            context.begin_path();
            for (step, (x, y)) in run.iter().enumerate() {
                if step == 0 {
                    context.move_to(*x, *y);
                } else {
                    context.line_to(*x, *y);
                }
            }
            // A run of one point would draw nothing: give it a dot.
            if run.len() == 1 {
                context.line_to(run[0].0 + 0.1, run[0].1);
            }
            context.stroke();
        }
        if dashed {
            context.set_line_dash(&crate::json_parse("[]"))?;
        }
    }
    context.restore();
    // The legend names each line beside a short swatch of its colour and dash,
    // so colour is not the only cue.
    context.set_text_align("left");
    let slots = legend_slots(&layout.legend, lines.len());
    for (index, (line, slot)) in lines.iter().zip(&slots).enumerate() {
        let (kind, dashed) = line_style(index);
        let middle = slot.y + slot.h / 2.0;
        context.set_line_width(metrics.wire_width * LINE_WIDTH_RATIO);
        context.set_stroke_style_str(&line_colour(theme, kind, line.colour.as_deref()));
        if dashed {
            context.set_line_dash(&crate::json_parse("[4, 3]"))?;
        }
        context.begin_path();
        context.move_to(slot.x, middle);
        context.line_to(slot.x + SWATCH_WIDTH, middle);
        context.stroke();
        if dashed {
            context.set_line_dash(&crate::json_parse("[]"))?;
        }
        let room = slot.w - SWATCH_WIDTH - metrics.box_pad;
        let label = fit_text(context, &line.name, room)?;
        context.fill_text(
            &label,
            slot.x + SWATCH_WIDTH + metrics.box_pad / 2.0,
            middle + metrics.text_baseline,
        )?;
    }
    Ok(())
}

/// One line of a plot node's chart.
pub(crate) struct Plotted<'a> {
    /// The legend name: the output port's name, or `n3 loss` when two lines
    /// share one.
    pub name: String,
    pub series: &'a Series,
    /// The colour the user chose for this input, if any.
    pub colour: Option<String>,
}

/// The lines of a plot node: each linked series with its name and its chosen
/// colour, in input order.
pub(crate) fn plotted<'a>(
    graph: &Graph,
    (node, node_type): (&PlacedNode, &NodeType),
    outcome: &'a NodeOutcome,
) -> Vec<Plotted<'a>> {
    let sourced: Vec<((&str, &str), &Series, &str)> = node_type
        .inputs
        .iter()
        .filter_map(|port| {
            let series = outcome.shown.get(&port.name)?.as_series()?;
            let source = graph
                .links
                .iter()
                .find(|link| link.target.node == node.id && link.target.port == port.name)?;
            Some((
                (source.source.node.as_str(), source.source.port.as_str()),
                series,
                port.name.as_str(),
            ))
        })
        .collect();
    let sources: Vec<(&str, &str)> = sourced.iter().map(|(source, _, _)| *source).collect();
    line_names(&sources)
        .into_iter()
        .zip(sourced)
        .map(|(name, (_, series, port))| Plotted {
            name,
            series,
            colour: node.colours.get(port).cloned(),
        })
        .collect()
}

/// The colour line `index` is drawn in: the one the user chose, else the
/// theme's for the line's kind.
fn line_colour(theme: &Theme, kind: PortType, chosen: Option<&str>) -> String {
    chosen.map_or_else(
        || theme.port_style(kind, true).colour.clone(),
        str::to_string,
    )
}

/// A chart line's width as a fraction of a wire's.
pub(crate) const LINE_WIDTH_RATIO: f64 = 0.75;

/// The length of a legend swatch, in canvas pixels.
pub(crate) const SWATCH_WIDTH: f64 = 16.0;

/// The focus ring's width, its dark halo on each side, and its gap from what it
/// rings, all in screen pixels.
/// How strong the chart's grid lines are, from 0 to 1.
pub(crate) const GRID_ALPHA: f64 = 0.4;
