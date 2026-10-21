//! A card: its box, title, ports, value boxes, picture or chart area and job row.

use crate::draw::columns;
use crate::draw::fit_text;
use crate::draw_chart::chart;
use crate::pictures::Pictures;
use element_51_core::display::{chart_rect, fit_picture, picture_rect};
use element_51_core::fields::{linked_text, share_room};
use element_51_core::job::{
    JobButton, JobLayout, can_start, can_stop, job_layout, progress_fraction, progress_text,
    state_text,
};
use element_51_core::layout::{Rect, icon_rect, input_anchor, node_rect, output_row};
use element_51_core::theme::Theme;
use element_51_core::{Graph, JobUpdate, NodeType, PlacedNode, RunResult, Status};
use wasm_bindgen::JsValue;
use web_sys::CanvasRenderingContext2d;

/// One node card: body, title bar, rows and value boxes. Ports and wires are
/// drawn afterwards, over it.
pub(crate) fn card(
    context: &CanvasRenderingContext2d,
    theme: &Theme,
    graph: &Graph,
    (node, node_type): (&PlacedNode, &NodeType),
    (run, pictures, job): (Option<&RunResult>, &Pictures, Option<&JobUpdate>),
    selected: bool,
    focused_button: Option<JobButton>,
) -> Result<(), JsValue> {
    let (colours, metrics) = (&theme.colours, &theme.metrics);
    let rect = node_rect(node, node_type, metrics);
    let outcome = run.and_then(|result| result.outcome_for(&node.id));
    let status = outcome.map(|outcome| outcome.status);
    let (header, title, label) = match status {
        Some(Status::Error) => (&colours.header_error, &colours.title, &colours.label),
        // A job that has not started has nothing to show yet; that is not a skip.
        Some(Status::Skipped) if !node_type.job => (
            &colours.header_skipped,
            &colours.text_dim,
            &colours.text_dim,
        ),
        _ => (&colours.header, &colours.title, &colours.label),
    };

    context.set_fill_style_str(&colours.card);
    card_path(context, &rect, metrics.card_radius);
    context.fill();
    // The title bar takes the card's rounded top corners by clipping to it.
    context.set_fill_style_str(header);
    context.save();
    card_path(context, &rect, metrics.card_radius);
    context.clip();
    context.fill_rect(rect.x, rect.y, rect.w, metrics.header_height);
    context.restore();
    context.set_stroke_style_str(&colours.card_border);
    context.set_line_width(metrics.border_width);
    card_path(context, &rect, metrics.card_radius);
    context.stroke();
    if selected {
        context.set_stroke_style_str(&colours.selected);
        context.set_line_width(metrics.selection_width);
        card_path(context, &rect, metrics.card_radius);
        context.stroke();
    }

    // Title bar: the id on the left, the type name and the icon on the right.
    let middle = rect.y + metrics.header_height / 2.0;
    context.set_font(&theme.fonts.title);
    context.set_fill_style_str(title);
    context.set_text_align("left");
    context.fill_text(
        &node.id,
        rect.x + metrics.title_pad,
        middle + metrics.text_baseline,
    )?;
    let icon = icon_rect(&rect, metrics);
    context.set_text_align("right");
    context.fill_text(
        &node_type.name,
        icon.x - metrics.title_pad,
        middle + metrics.text_baseline,
    )?;
    context.set_fill_style_str(&colours.icon);
    for bar in 0..3 {
        let top = icon.y + f64::from(bar) * (metrics.icon_bar + metrics.icon_gap);
        context.fill_rect(icon.x, top, icon.w, metrics.icon_bar);
    }

    // Rows: inputs first, then outputs. Each has a right-aligned name and a
    // value box; the page's own input boxes go over the unlinked inputs.
    context.set_font(&theme.fonts.card);
    let columns = columns(context, node_type, &rect, metrics)?;
    let box_width = columns.box_right - columns.box_left;
    let rows = node_type
        .inputs
        .iter()
        .enumerate()
        .map(|(row, port)| (row, port, true))
        .chain(
            node_type
                .outputs
                .iter()
                .enumerate()
                .map(|(index, port)| (output_row(node_type, index), port, false)),
        );
    for (row, port, is_input) in rows {
        let (_, centre) = input_anchor(&rect, row, metrics);
        let baseline = centre + metrics.text_baseline;
        context.set_font(&theme.fonts.card);
        context.set_fill_style_str(label);
        context.set_text_align("right");
        context.fill_text(&port.name, columns.label_right, baseline)?;
        context.set_fill_style_str(&colours.field);
        round_rect_path(
            context,
            &Rect {
                x: columns.box_left,
                y: centre - metrics.field_height / 2.0,
                w: box_width,
                h: metrics.field_height,
            },
            metrics.box_radius,
        );
        context.fill();
        context.set_text_align("left");
        context.set_font(&theme.fonts.value);
        let room = box_width - 2.0 * metrics.box_pad;
        let text_x = columns.box_left + metrics.box_pad;
        if is_input {
            if let Some(source) = graph
                .links
                .iter()
                .find(|link| link.target.node == node.id && link.target.port == port.name)
            {
                // The value first, then the source dimmer after it; the source is cut
                // first when the box is narrow.
                let parts = linked_text(
                    &source.source,
                    run.and_then(|result| result.outcome_for(&source.source.node)),
                );
                let mut at = text_x;
                if let Some(value) = &parts.value {
                    let (value_room, _) = share_room(
                        context.measure_text(value)?.width(),
                        context.measure_text(&parts.source)?.width(),
                        room,
                        metrics.box_pad,
                    );
                    let shown = fit_text(context, value, value_room)?;
                    context.set_fill_style_str(&colours.field_text);
                    context.fill_text(&shown, at, baseline)?;
                    at += context.measure_text(&shown)?.width() + metrics.box_pad;
                }
                let left = room - (at - text_x);
                context.set_fill_style_str(&colours.placeholder);
                context.fill_text(&fit_text(context, &parts.source, left)?, at, baseline)?;
            }
        } else {
            let value = outcome
                .filter(|_| status == Some(Status::Ok))
                .and_then(|outcome| outcome.output_text(&port.name));
            match value {
                Some(value) => {
                    context.set_fill_style_str(&colours.field_text);
                    context.fill_text(&fit_text(context, &value, room)?, text_x, baseline)?;
                }
                None => {
                    context.set_fill_style_str(&colours.placeholder);
                    context.fill_text("output", text_x, baseline)?;
                }
            }
        }
    }
    if let Some(layout) = job_layout(&rect, node_type, metrics) {
        job_row(context, theme, &layout, (job, focused_button))?;
    }
    if let Some(area) = picture_rect(&rect, node_type, metrics) {
        context.set_fill_style_str(&colours.field);
        round_rect_path(context, &area, metrics.box_radius);
        context.fill();
        let shown = pictures.of(&node.id).filter(|_| status == Some(Status::Ok));
        if let Some(picture) = shown {
            let at = fit_picture(&area, picture.width(), picture.height());
            // Crisp pixels: a 28 by 28 picture is meant to look like 28 by 28.
            context.set_image_smoothing_enabled(false);
            let drawn = context
                .draw_image_with_html_canvas_element_and_dw_and_dh(picture, at.x, at.y, at.w, at.h);
            context.set_image_smoothing_enabled(true);
            drawn?;
        }
    }
    if let Some(area) = chart_rect(&rect, node_type, metrics) {
        context.set_fill_style_str(&colours.field);
        round_rect_path(context, &area, metrics.box_radius);
        context.fill();
        if let Some(outcome) = outcome.filter(|_| status == Some(Status::Ok)) {
            chart(context, theme, graph, (node, node_type), (&area, outcome))?;
        }
    }
    Ok(())
}

/// Trace the outline of a node card with rounded corners.
/// The job row of a card: the Start and Stop buttons and the state word, then the
/// bar and `step 3 of 5`. Stop is drawn only while the job runs; Start is dim
/// while it cannot be used.
pub(crate) fn job_row(
    context: &CanvasRenderingContext2d,
    theme: &Theme,
    layout: &JobLayout,
    (job, focused_button): (Option<&JobUpdate>, Option<JobButton>),
) -> Result<(), JsValue> {
    let (colours, metrics) = (&theme.colours, &theme.metrics);
    let button = |rect: &Rect, which: JobButton, usable: bool| -> Result<(), JsValue> {
        context.set_fill_style_str(&colours.field);
        round_rect_path(context, rect, metrics.box_radius);
        context.fill();
        context.set_stroke_style_str(&colours.field_border);
        context.set_line_width(1.0);
        round_rect_path(context, rect, metrics.box_radius);
        context.stroke();
        context.set_fill_style_str(if usable {
            &colours.field_text
        } else {
            &colours.placeholder
        });
        context.set_text_align("center");
        context.fill_text(
            which.label(),
            rect.x + rect.w / 2.0,
            rect.y + rect.h / 2.0 + metrics.text_baseline,
        )
    };
    context.set_font(&theme.fonts.card);
    button(&layout.start, JobButton::Start, can_start(job))?;
    // Stop is there while the job runs, and while the keyboard is on it.
    if can_stop(job) || focused_button == Some(JobButton::Stop) {
        button(&layout.stop, JobButton::Stop, can_stop(job))?;
    }
    context.set_fill_style_str(&colours.label);
    context.set_text_align("right");
    context.fill_text(state_text(job), layout.state_right, layout.state_baseline)?;
    if let Some(fraction) = progress_fraction(job) {
        context.set_fill_style_str(&colours.field);
        round_rect_path(context, &layout.bar, metrics.box_radius);
        context.fill();
        context.set_fill_style_str(&colours.field_text);
        round_rect_path(
            context,
            &Rect {
                w: layout.bar.w * fraction,
                ..layout.bar
            },
            metrics.box_radius,
        );
        context.fill();
    }
    if let Some(text) = progress_text(job) {
        context.set_fill_style_str(&colours.label);
        context.set_text_align("right");
        context.fill_text(&text, layout.step_right, layout.step_baseline)?;
    }
    Ok(())
}

pub(crate) fn card_path(context: &CanvasRenderingContext2d, rect: &Rect, radius: f64) {
    round_rect_path(context, rect, radius);
}

/// Trace a rectangle with rounded corners.
pub(crate) fn round_rect_path(context: &CanvasRenderingContext2d, rect: &Rect, radius: f64) {
    let (left, top) = (rect.x, rect.y);
    let (right, bottom) = (rect.x + rect.w, rect.y + rect.h);
    context.begin_path();
    context.move_to(left + radius, top);
    let _ = context.arc_to(right, top, right, bottom, radius);
    let _ = context.arc_to(right, bottom, left, bottom, radius);
    let _ = context.arc_to(left, bottom, left, top, radius);
    let _ = context.arc_to(left, top, right, top, radius);
    context.close_path();
}
