//! Drawing a chart or a picture as a figure for a paper, on its own canvas.
//! The layout and the numbers come from `element-51-core::paper`.

use element_51_core::paper::{
    FONT_FAMILY, FRAME_POINTS, GRID, GRID_POINTS, INK, LABEL_POINTS, LINE_POINTS, MARKER_POINTS,
    Marker, PAGE, PAPER_DPI, PAPER_HEIGHT, PAPER_WIDTH, PaperChart, PaperLine, TICK_POINTS, points,
};
use element_51_core::png::with_dpi;
use std::f64::consts::{FRAC_PI_2, TAU};
use wasm_bindgen::JsValue;
use web_sys::{CanvasRenderingContext2d, HtmlCanvasElement};

/// A tick's length inside the frame, in points, and the gaps round the text.
const TICK_LENGTH: f64 = 2.5;
const TEXT_GAP: f64 = 4.0;
const LEGEND_PAD: f64 = 5.0;
const SWATCH: f64 = 18.0;
const DASH: [f64; 2] = [3.0, 2.0];

fn font(size: f64) -> String {
    format!("{}px {FONT_FAMILY}", points(size).round())
}

fn dash(context: &CanvasRenderingContext2d, pattern: &[f64]) -> Result<(), JsValue> {
    let list: Vec<String> = pattern
        .iter()
        .map(|part| points(*part).to_string())
        .collect();
    context.set_line_dash(&crate::json_parse(&format!("[{}]", list.join(", "))))
}

/// Fill the canvas with the page colour.
pub(crate) fn page(context: &CanvasRenderingContext2d) {
    context.set_fill_style_str(PAGE);
    context.fill_rect(0.0, 0.0, f64::from(PAPER_WIDTH), f64::from(PAPER_HEIGHT));
}

/// Draw `chart`: grid, frame, ticks and their numbers, the two axis labels, the
/// lines with their markers, and a legend when there is more than one line.
pub(crate) fn draw_chart(
    context: &CanvasRenderingContext2d,
    chart: &PaperChart,
) -> Result<(), JsValue> {
    let plot = chart.plot;
    context.set_line_cap("butt");
    context.set_stroke_style_str(GRID);
    context.set_line_width(points(GRID_POINTS));
    for tick in &chart.y_ticks {
        line(context, (plot.x, tick.at), (plot.x + plot.w, tick.at));
    }
    for tick in &chart.x_ticks {
        line(context, (tick.at, plot.y), (tick.at, plot.y + plot.h));
    }
    context.set_stroke_style_str(INK);
    context.set_line_width(points(FRAME_POINTS));
    context.stroke_rect(plot.x, plot.y, plot.w, plot.h);
    let inside = points(TICK_LENGTH);
    for tick in &chart.y_ticks {
        line(context, (plot.x, tick.at), (plot.x + inside, tick.at));
        line(
            context,
            (plot.x + plot.w, tick.at),
            (plot.x + plot.w - inside, tick.at),
        );
    }
    for tick in &chart.x_ticks {
        line(
            context,
            (tick.at, plot.y + plot.h),
            (tick.at, plot.y + plot.h - inside),
        );
        line(context, (tick.at, plot.y), (tick.at, plot.y + inside));
    }
    context.set_fill_style_str(INK);
    context.set_font(&font(TICK_POINTS));
    context.set_text_align("right");
    context.set_text_baseline("middle");
    for tick in &chart.y_ticks {
        context.fill_text(&tick.label, plot.x - points(TEXT_GAP), tick.at)?;
    }
    context.set_text_align("center");
    context.set_text_baseline("top");
    for tick in &chart.x_ticks {
        context.fill_text(&tick.label, tick.at, plot.y + plot.h + points(TEXT_GAP))?;
    }
    context.set_font(&font(LABEL_POINTS));
    context.set_text_baseline("alphabetic");
    context.fill_text(
        &chart.x_label,
        plot.x + plot.w / 2.0,
        f64::from(PAPER_HEIGHT) - points(8.0),
    )?;
    if let Some(label) = &chart.y_label {
        context.save();
        context.translate(points(14.0), plot.y + plot.h / 2.0)?;
        context.rotate(-FRAC_PI_2)?;
        context.fill_text(label, 0.0, 0.0)?;
        context.restore();
    }
    context.save();
    context.begin_path();
    context.rect(plot.x, plot.y, plot.w, plot.h);
    context.clip();
    for drawn in &chart.lines {
        draw_line(context, drawn)?;
    }
    context.restore();
    if chart.lines.len() > 1 {
        legend(context, chart)?;
    }
    Ok(())
}

fn line(context: &CanvasRenderingContext2d, from: (f64, f64), to: (f64, f64)) {
    context.begin_path();
    context.move_to(from.0, from.1);
    context.line_to(to.0, to.1);
    context.stroke();
}

fn draw_line(context: &CanvasRenderingContext2d, drawn: &PaperLine) -> Result<(), JsValue> {
    context.set_stroke_style_str(drawn.colour);
    context.set_line_width(points(LINE_POINTS));
    context.set_line_join("round");
    if drawn.dashed {
        dash(context, &DASH)?;
    }
    for run in &drawn.runs {
        context.begin_path();
        for (step, (x, y)) in run.iter().enumerate() {
            if step == 0 {
                context.move_to(*x, *y);
            } else {
                context.line_to(*x, *y);
            }
        }
        if run.len() == 1 {
            context.line_to(run[0].0 + 0.1, run[0].1);
        }
        context.stroke();
    }
    dash(context, &[])?;
    context.set_fill_style_str(drawn.colour);
    for (x, y) in &drawn.marks {
        marker(context, drawn.marker, *x, *y)?;
    }
    Ok(())
}

/// One marker centred on `(x, y)`, in the colour already set: circles, squares,
/// diamonds and triangles are filled, crosses and pluses are strokes.
fn marker(context: &CanvasRenderingContext2d, kind: Marker, x: f64, y: f64) -> Result<(), JsValue> {
    let r = points(MARKER_POINTS);
    context.begin_path();
    match kind {
        Marker::Circle => {
            context.arc(x, y, r, 0.0, TAU)?;
            context.fill();
        }
        Marker::Square => context.fill_rect(x - r, y - r, 2.0 * r, 2.0 * r),
        Marker::Diamond => {
            context.move_to(x, y - r * 1.3);
            context.line_to(x + r * 1.3, y);
            context.line_to(x, y + r * 1.3);
            context.line_to(x - r * 1.3, y);
            context.close_path();
            context.fill();
        }
        Marker::Triangle => {
            context.move_to(x, y - r * 1.3);
            context.line_to(x + r * 1.2, y + r);
            context.line_to(x - r * 1.2, y + r);
            context.close_path();
            context.fill();
        }
        Marker::Cross => {
            line(context, (x - r, y - r), (x + r, y + r));
            line(context, (x - r, y + r), (x + r, y - r));
        }
        Marker::Plus => {
            line(context, (x - r * 1.2, y), (x + r * 1.2, y));
            line(context, (x, y - r * 1.2), (x, y + r * 1.2));
        }
    }
    Ok(())
}

/// A boxed legend in the top right corner of the plot: a short sample of each
/// line with its marker, and its name.
fn legend(context: &CanvasRenderingContext2d, chart: &PaperChart) -> Result<(), JsValue> {
    context.set_font(&font(TICK_POINTS));
    context.set_text_align("left");
    context.set_text_baseline("middle");
    let pad = points(LEGEND_PAD);
    let swatch = points(SWATCH);
    let row = points(TICK_POINTS) * 1.5;
    let mut widest: f64 = 0.0;
    for drawn in &chart.lines {
        widest = widest.max(context.measure_text(&drawn.name)?.width());
    }
    let width = pad * 3.0 + swatch + widest;
    let height = pad * 2.0 + row * chart.lines.len() as f64;
    let left = chart.plot.x + chart.plot.w - width - pad;
    let top = chart.plot.y + pad;
    context.set_fill_style_str(PAGE);
    context.fill_rect(left, top, width, height);
    context.set_stroke_style_str(INK);
    context.set_line_width(points(FRAME_POINTS));
    context.stroke_rect(left, top, width, height);
    for (index, drawn) in chart.lines.iter().enumerate() {
        let middle = top + pad + row * (index as f64 + 0.5);
        context.set_stroke_style_str(drawn.colour);
        context.set_fill_style_str(drawn.colour);
        context.set_line_width(points(LINE_POINTS));
        if drawn.dashed {
            dash(context, &DASH)?;
        }
        line(context, (left + pad, middle), (left + pad + swatch, middle));
        dash(context, &[])?;
        marker(context, drawn.marker, left + pad + swatch / 2.0, middle)?;
        context.set_fill_style_str(INK);
        context.fill_text(&drawn.name, left + pad * 2.0 + swatch, middle)?;
    }
    Ok(())
}

/// A picture enlarged by a whole number, with crisp pixels and a thin black
/// frame, onto the canvas.
pub(crate) fn draw_picture(
    context: &CanvasRenderingContext2d,
    picture: &HtmlCanvasElement,
    size: (u32, u32),
) -> Result<(), JsValue> {
    let (width, height) = (f64::from(size.0), f64::from(size.1));
    context.set_image_smoothing_enabled(false);
    context.draw_image_with_html_canvas_element_and_dw_and_dh(picture, 0.0, 0.0, width, height)?;
    context.set_stroke_style_str(INK);
    context.set_line_width(points(FRAME_POINTS));
    context.stroke_rect(0.0, 0.0, width, height);
    Ok(())
}

/// The canvas as a PNG data URL that says it has `PAPER_DPI` dots per inch.
pub(crate) fn to_paper_url(canvas: &HtmlCanvasElement) -> Result<String, String> {
    let failed = || "The browser could not make the picture.".to_string();
    let url = canvas.to_data_url().map_err(|_| failed())?;
    let (head, base64) = url.split_once("base64,").ok_or_else(failed)?;
    let window = web_sys::window().ok_or_else(failed)?;
    let binary = window.atob(base64).map_err(|_| failed())?;
    let bytes: Vec<u8> = binary.chars().map(|c| c as u8).collect();
    let marked = with_dpi(&bytes, PAPER_DPI).ok_or_else(failed)?;
    let text: String = marked.iter().map(|byte| char::from(*byte)).collect();
    let encoded = window.btoa(&text).map_err(|_| failed())?;
    Ok(format!("{head}base64,{encoded}"))
}
