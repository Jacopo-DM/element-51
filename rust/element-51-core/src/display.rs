//! What a card shows below its ports: where the picture or the chart goes, how a
//! picture fits, and the chart's scale, ticks and lines.

use crate::job::job_height;
use crate::layout::Rect;
use crate::theme::Metrics;
use crate::{CardDisplay, NodeType, PortType, Series, Shown};

/// The height of a chart area as a fraction of the card's width.
const CHART_HEIGHT_RATIO: f64 = 0.85;
/// Room for the y axis labels, as a multiple of the row height.
const LABEL_ROOMS: f64 = 1.8;
/// How many ticks the x axis aims for.
const X_TICK_TARGET: usize = 4;
/// How many ticks the y axis aims for: a plot is wider than it is tall, and a
/// scale with only one or two labels reads as unfinished.
const Y_TICK_TARGET: usize = 5;
/// The height of the legend row above the plot, as a multiple of the row height.
const LEGEND_ROOMS: f64 = 1.0;
/// How far the y range reaches past the data, as a fraction of its span.
const RANGE_PAD: f64 = 0.05;

/// The side of the square picture area at the default card width: the width
/// less the inset on each side.
pub fn picture_side(metrics: &Metrics) -> f64 {
    metrics.node_width - 2.0 * metrics.title_pad
}

/// The height of the picture or chart area at the default size, 0 for a type
/// that shows nothing.
pub fn default_area_height(node_type: &NodeType, metrics: &Metrics) -> f64 {
    match node_type.display {
        CardDisplay::None => 0.0,
        CardDisplay::Image => picture_side(metrics),
        CardDisplay::Chart => chart_height(metrics),
    }
}

/// How much taller a card of this type is because of what it shows, at the
/// default size: the picture or chart area and a margin under it. It depends on
/// the type only, so a card does not change size when results arrive.
pub fn display_height(node_type: &NodeType, metrics: &Metrics) -> f64 {
    match node_type.display {
        CardDisplay::None => 0.0,
        _ => default_area_height(node_type, metrics) + metrics.title_pad,
    }
}

/// The height of the chart area at the default size: a fraction of the card's
/// default width.
pub fn chart_height(metrics: &Metrics) -> f64 {
    CHART_HEIGHT_RATIO * metrics.node_width
}

/// The picture or chart area of the card at `card`, below the port rows: the
/// card's width less the inset on each side, and what is left of its height
/// above the margin under it. None if the node type shows nothing.
fn display_area(card: &Rect, node_type: &NodeType, metrics: &Metrics) -> Option<Rect> {
    if node_type.display == CardDisplay::None {
        return None;
    }
    let rows = node_type.inputs.len() + node_type.outputs.len();
    let top = card.y
        + metrics.header_height
        + metrics.row_height * rows as f64
        + job_height(node_type, metrics);
    Some(Rect {
        x: card.x + metrics.title_pad,
        y: top,
        w: card.w - 2.0 * metrics.title_pad,
        h: card.y + card.h - metrics.title_pad - top,
    })
}

/// The picture area of the card at `card`, below the port rows. None if the
/// node type shows no picture.
pub fn picture_rect(card: &Rect, node_type: &NodeType, metrics: &Metrics) -> Option<Rect> {
    if node_type.display != CardDisplay::Image {
        return None;
    }
    display_area(card, node_type, metrics)
}

/// The largest rectangle of a `width` by `height` picture's shape that fits in
/// `area`, centred in it.
pub fn fit_picture(area: &Rect, width: u32, height: u32) -> Rect {
    let scale = (area.w / f64::from(width)).min(area.h / f64::from(height));
    let (w, h) = (f64::from(width) * scale, f64::from(height) * scale);
    Rect {
        x: area.x + (area.w - w) / 2.0,
        y: area.y + (area.h - h) / 2.0,
        w,
        h,
    }
}

/// The chart area of the card at `card`, below the port rows. None if the node
/// type does not show a chart.
pub fn chart_rect(card: &Rect, node_type: &NodeType, metrics: &Metrics) -> Option<Rect> {
    if node_type.display != CardDisplay::Chart {
        return None;
    }
    display_area(card, node_type, metrics)
}

/// The side of the resize handle in the bottom right corner of a card, in graph
/// units.
pub const HANDLE_SIDE: f64 = 14.0;

/// The resize handle of the card at `card`: a square inside its bottom right
/// corner. None if the node type shows nothing to enlarge.
pub fn resize_handle(card: &Rect, node_type: &NodeType) -> Option<Rect> {
    if node_type.display == CardDisplay::None {
        return None;
    }
    Some(Rect {
        x: card.x + card.w - HANDLE_SIDE,
        y: card.y + card.h - HANDLE_SIDE,
        w: HANDLE_SIDE,
        h: HANDLE_SIDE,
    })
}

/// The width and the picture or chart height a card gets when its bottom right
/// corner is dragged by `(dx, dy)` from a card of `start` width and area
/// height. Neither goes below the default size or above `MAX_CARD_SIZE`.
pub fn dragged_size(
    start: (f64, f64),
    (dx, dy): (f64, f64),
    node_type: &NodeType,
    metrics: &Metrics,
) -> (f64, f64) {
    let width = (start.0 + dx).clamp(metrics.node_width, crate::MAX_CARD_SIZE);
    let height = (start.1 + dy).clamp(
        default_area_height(node_type, metrics),
        crate::MAX_CARD_SIZE,
    );
    (width, height)
}

/// How a chart line looks: the port type whose colour it takes, and whether it
/// is dashed. Lines take the colours in turn and dash from the sixth, so colour
/// is never the only way to tell them apart.
pub const CHART_LINE_STYLES: [PortType; 5] = [
    PortType::Float,
    PortType::Str,
    PortType::Bool,
    PortType::Image,
    PortType::Series,
];

/// The style of line number `index`, counting from 0.
pub fn line_style(index: usize) -> (PortType, bool) {
    (
        CHART_LINE_STYLES[index % CHART_LINE_STYLES.len()],
        index >= CHART_LINE_STYLES.len(),
    )
}

/// The name of each chart line from its source `(node, port)`: the output port's
/// name, or `n3 loss` with the node id when another line has the same name.
pub fn line_names(sources: &[(&str, &str)]) -> Vec<String> {
    sources
        .iter()
        .map(|(node, port)| {
            let shared = sources.iter().filter(|(_, other)| other == port).count() > 1;
            if shared {
                format!("{node} {port}")
            } else {
                (*port).to_string()
            }
        })
        .collect()
}

/// The last value of a series that is a number, if there is one.
pub fn last_value(series: &Series) -> Option<f64> {
    series
        .values
        .iter()
        .rev()
        .flatten()
        .copied()
        .find(|value| value.is_finite())
}

/// One line about a picture or a series, for the box of the port that has it:
/// `28 x 28 grey`, `0.05` (a series shows its last number). A series with no
/// number in it (all gaps) says how many gaps. The box is about a dozen characters wide, so the part that
/// matters comes first and the line is short.
pub fn shown_summary(shown: &Shown) -> String {
    match shown {
        Shown::Image(image) => {
            let kind = if image.channels == 1 {
                "grey"
            } else {
                "colour"
            };
            format!("{} x {} {kind}", image.width, image.height)
        }
        Shown::Series(series) => match last_value(series) {
            Some(last) => short_number(last),
            None => {
                let count = series.values.len();
                format!("{count} {}", if count == 1 { "gap" } else { "gaps" })
            }
        },
    }
}

/// The hover text for the box of a picture or a series, which says what the box's
/// short text is a summary of: `last of 282 points`, `grey picture, 28 x 28`.
pub fn shown_tooltip(shown: &Shown) -> String {
    match shown {
        Shown::Image(image) => {
            let kind = if image.channels == 1 {
                "grey"
            } else {
                "colour"
            };
            format!("{kind} picture, {} x {}", image.width, image.height)
        }
        Shown::Series(series) => {
            let count = series.values.len();
            let points = format!("{count} {}", if count == 1 { "point" } else { "points" });
            match last_value(series) {
                Some(_) => format!("last of {points}"),
                None => format!("{points}, none of them a number"),
            }
        }
    }
}

/// A number in about three significant digits, for ticks and for the last
/// value: `0.314`, `12.3`, `1235`, `1.2e-5`.
pub fn short_number(value: f64) -> String {
    if value.is_nan() {
        return "nan".to_string();
    }
    if value.is_infinite() {
        return if value > 0.0 { "inf" } else { "-inf" }.to_string();
    }
    if value == 0.0 {
        return "0".to_string();
    }
    let size = value.abs();
    if size < 1e-4 {
        return format!("{value:.1e}");
    }
    let decimals = if size >= 1000.0 {
        0
    } else {
        (2 - size.log10().floor() as i32).clamp(0, 6) as usize
    };
    let text = format!("{value:.decimals$}");
    if text.contains('.') {
        text.trim_end_matches('0').trim_end_matches('.').to_string()
    } else {
        text
    }
}

/// Round numbers from `low` to `high`, about `target` of them: a step of 1, 2
/// or 5 times a power of ten. A range that is empty or backwards gives just
/// `low`.
pub fn nice_ticks(low: f64, high: f64, target: usize) -> Vec<f64> {
    if low.partial_cmp(&high) != Some(std::cmp::Ordering::Less) {
        return vec![low];
    }
    let raw = (high - low) / target.max(1) as f64;
    let magnitude = 10f64.powf(raw.log10().floor());
    let scaled = raw / magnitude;
    let step = magnitude
        * if scaled <= 1.0 {
            1.0
        } else if scaled <= 2.0 {
            2.0
        } else if scaled <= 5.0 {
            5.0
        } else {
            10.0
        };
    let first = (low / step).ceil() as i64;
    let last = (high / step + 1e-9).floor() as i64;
    (first..=last)
        .map(|k| (k as f64 * step * 1e12).round() / 1e12)
        .collect()
}

/// The points of a line as `(index, value)`, in runs: a gap (no number) starts a
/// new run. A series longer than twice `buckets` is cut into `buckets` equal
/// parts and each keeps its lowest and highest point, so a spike survives and
/// the line has about two points per bucket. A part with no numbers breaks the
/// line.
pub fn thin(values: &[Option<f64>], buckets: usize) -> Vec<Vec<(f64, f64)>> {
    let buckets = buckets.max(1);
    let number = |value: &Option<f64>| value.filter(|number| number.is_finite());
    let mut runs: Vec<Vec<(f64, f64)>> = Vec::new();
    let mut current: Vec<(f64, f64)> = Vec::new();
    let mut end_run = |current: &mut Vec<(f64, f64)>| {
        if !current.is_empty() {
            runs.push(std::mem::take(current));
        }
    };
    if values.len() <= 2 * buckets {
        for (index, value) in values.iter().enumerate() {
            match number(value) {
                Some(y) => current.push((index as f64, y)),
                None => end_run(&mut current),
            }
        }
    } else {
        let size = values.len().div_ceil(buckets);
        for (part, chunk) in values.chunks(size).enumerate() {
            let mut low: Option<(usize, f64)> = None;
            let mut high: Option<(usize, f64)> = None;
            for (offset, value) in chunk.iter().enumerate() {
                if let Some(y) = number(value) {
                    let at = part * size + offset;
                    if low.is_none_or(|(_, best)| y < best) {
                        low = Some((at, y));
                    }
                    if high.is_none_or(|(_, best)| y > best) {
                        high = Some((at, y));
                    }
                }
            }
            let (Some(low), Some(high)) = (low, high) else {
                end_run(&mut current);
                continue;
            };
            let (first, second) = if low.0 <= high.0 {
                (low, high)
            } else {
                (high, low)
            };
            current.push((first.0 as f64, first.1));
            if second.0 != first.0 {
                current.push((second.0 as f64, second.1));
            }
        }
    }
    end_run(&mut current);
    runs
}

/// One tick on an axis: where it is on the canvas, and its label.
#[derive(Clone, Debug, PartialEq)]
pub struct Tick {
    pub at: f64,
    pub label: String,
}

/// A chart worked out for drawing: where the plot is inside the chart area,
/// the legend row, the ticks of both axes, and each line as runs of canvas
/// points.
#[derive(Clone, Debug, PartialEq)]
pub struct Chart {
    pub plot: Rect,
    /// The row above the plot where the legend goes.
    pub legend: Rect,
    pub x_ticks: Vec<Tick>,
    pub y_ticks: Vec<Tick>,
    /// One entry per series, in the order given; each is a list of runs.
    pub lines: Vec<Vec<Vec<(f64, f64)>>>,
}

/// Lay out `lines` in the chart area `area`: a plot with room on the left for
/// the y labels and below for the x labels, one scale for every line, ticks at
/// round numbers, and each line thinned to the plot's width. None if no series
/// has a number in it.
pub fn chart_layout(area: &Rect, metrics: &Metrics, lines: &[&Series]) -> Option<Chart> {
    let left = metrics.row_height * LABEL_ROOMS;
    let (below, pad) = (metrics.row_height, metrics.box_pad);
    let legend = Rect {
        x: area.x + pad,
        y: area.y + pad,
        w: area.w - 2.0 * pad,
        h: metrics.row_height * LEGEND_ROOMS,
    };
    let top = legend.y + legend.h;
    let plot = Rect {
        x: area.x + left,
        y: top,
        w: area.w - left - pad,
        h: area.y + area.h - below - top,
    };
    let numbers: Vec<f64> = lines
        .iter()
        .flat_map(|series| series.values.iter().flatten().copied())
        .filter(|number| number.is_finite())
        .collect();
    let lowest = numbers.iter().copied().reduce(f64::min)?;
    let highest = numbers.iter().copied().reduce(f64::max)?;
    let (low, high) = if highest > lowest {
        let reach = (highest - lowest) * RANGE_PAD;
        (lowest - reach, highest + reach)
    } else {
        (lowest - 0.5, highest + 0.5)
    };
    let longest = lines.iter().map(|series| series.values.len()).max()?;
    let x_max = (longest.max(2) - 1) as f64;
    let to_x = |index: f64| plot.x + index / x_max * plot.w;
    let to_y = |value: f64| plot.y + plot.h - (value - low) / (high - low) * plot.h;
    let x_ticks = nice_ticks(0.0, x_max, X_TICK_TARGET)
        .into_iter()
        .map(|tick| Tick {
            at: to_x(tick),
            label: short_number(tick),
        })
        .collect();
    let y_ticks = nice_ticks(low, high, Y_TICK_TARGET)
        .into_iter()
        .map(|tick| Tick {
            at: to_y(tick),
            label: short_number(tick),
        })
        .collect();
    let buckets = plot.w.max(1.0) as usize;
    let lines = lines
        .iter()
        .map(|series| {
            thin(&series.values, buckets)
                .into_iter()
                .map(|run| {
                    run.into_iter()
                        .map(|(index, value)| (to_x(index), to_y(value)))
                        .collect()
                })
                .collect()
        })
        .collect();
    Some(Chart {
        plot,
        legend,
        x_ticks,
        y_ticks,
        lines,
    })
}

/// The legend row split into one equal slot per line, left to right. Empty for
/// no lines.
pub fn legend_slots(legend: &Rect, count: usize) -> Vec<Rect> {
    let width = legend.w / count.max(1) as f64;
    (0..count)
        .map(|index| Rect {
            x: legend.x + index as f64 * width,
            y: legend.y,
            w: width,
            h: legend.h,
        })
        .collect()
}
