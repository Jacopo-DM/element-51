//! A chart laid out as a figure for a paper: white page, black frame, serif
//! text, blue and red lines with markers, as in a LaTeX plot. The sizes are in
//! pixels at 300 dpi for a figure 7 inches wide.

use crate::Series;
use crate::display::{Bounds, Tick, bounds, nice_ticks, short_number, thin};
use crate::layout::Rect;

/// The width of the figure in pixels: 7 inches at 300 dpi.
pub const PAPER_WIDTH: u32 = 2100;
/// The height of the figure in pixels: 4 2/3 inches at 300 dpi.
pub const PAPER_HEIGHT: u32 = 1400;
/// The resolution the file says it has.
pub const PAPER_DPI: u32 = 300;

/// The page and the ink.
pub const PAGE: &str = "#ffffff";
pub const INK: &str = "#000000";
/// The faint grid lines.
pub const GRID: &str = "#d9d9d9";
/// The font stack: Latin Modern or Computer Modern if the system has one, a
/// Times-like serif otherwise.
pub const FONT_FAMILY: &str =
    "\"Latin Modern Roman\", \"CMU Serif\", \"Computer Modern\", \"Times New Roman\", Times, serif";

/// A length in points (1/72 inch) as pixels at `PAPER_DPI`.
pub fn points(value: f64) -> f64 {
    value * f64::from(PAPER_DPI) / 72.0
}

/// The text of the tick numbers, the axis labels and the legend, in points.
pub const TICK_POINTS: f64 = 9.0;
pub const LABEL_POINTS: f64 = 10.0;
/// The frame, the grid and a line, in points.
pub const FRAME_POINTS: f64 = 0.4;
pub const GRID_POINTS: f64 = 0.3;
pub const LINE_POINTS: f64 = 0.8;
/// The radius of a marker, in points.
pub const MARKER_POINTS: f64 = 2.2;

const LEFT: f64 = 250.0;
const RIGHT: f64 = 80.0;
const TOP: f64 = 70.0;
const BOTTOM: f64 = 220.0;
const X_TICKS: usize = 6;
const Y_TICKS: usize = 6;
const MARKS: usize = 10;
const RANGE_PAD: f64 = 0.04;

/// The shape of a marker.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Marker {
    Circle,
    Cross,
    Square,
    Diamond,
    Triangle,
    Plus,
}

/// The colour and marker of line number `index`: blue circles, then red
/// crosses, and four more. After six the same pairs return, dashed.
pub fn line_look(index: usize) -> (&'static str, Marker, bool) {
    const LOOKS: [(&str, Marker); 6] = [
        ("#0000ff", Marker::Circle),
        ("#ff0000", Marker::Cross),
        ("#007f00", Marker::Square),
        ("#ff8000", Marker::Diamond),
        ("#7f00ff", Marker::Triangle),
        ("#000000", Marker::Plus),
    ];
    let (colour, marker) = LOOKS[index % LOOKS.len()];
    (colour, marker, index >= LOOKS.len())
}

/// One line of the figure.
#[derive(Clone, Debug, PartialEq)]
pub struct PaperLine {
    pub name: String,
    pub colour: &'static str,
    pub marker: Marker,
    pub dashed: bool,
    /// The line as runs of points; a gap in the data starts a new run.
    pub runs: Vec<Vec<(f64, f64)>>,
    /// Where the markers sit: about ten points spread along the line.
    pub marks: Vec<(f64, f64)>,
}

/// The figure worked out for drawing.
#[derive(Clone, Debug, PartialEq)]
pub struct PaperChart {
    pub plot: Rect,
    pub x_ticks: Vec<Tick>,
    pub y_ticks: Vec<Tick>,
    pub lines: Vec<PaperLine>,
    pub x_label: String,
    /// The line's name when there is only one line; with several the legend
    /// names them and the axis has no label.
    pub y_label: Option<String>,
}

/// A number as a paper prints it: three digits, a true minus sign.
pub fn paper_number(value: f64) -> String {
    short_number(value).replace('-', "\u{2212}")
}

/// Lay out `lines`, each a name and a series, in a figure of `PAPER_WIDTH` by
/// `PAPER_HEIGHT`. None if no series has a number in it.
pub fn paper_chart(lines: &[(&str, &Series)]) -> Option<PaperChart> {
    let plot = Rect {
        x: LEFT,
        y: TOP,
        w: f64::from(PAPER_WIDTH) - LEFT - RIGHT,
        h: f64::from(PAPER_HEIGHT) - TOP - BOTTOM,
    };
    let Bounds { low, high, x_max } = bounds(lines.iter().map(|(_, series)| *series), RANGE_PAD)?;
    let to_x = |index: f64| plot.x + index / x_max * plot.w;
    let to_y = |value: f64| plot.y + plot.h - (value - low) / (high - low) * plot.h;
    let ticks = |values: Vec<f64>, to: &dyn Fn(f64) -> f64| -> Vec<Tick> {
        values
            .into_iter()
            .map(|tick| Tick {
                at: to(tick),
                label: paper_number(tick),
            })
            .collect()
    };
    let x_ticks = ticks(nice_ticks(0.0, x_max, X_TICKS), &to_x);
    let y_ticks = ticks(nice_ticks(low, high, Y_TICKS), &to_y);
    let made = lines
        .iter()
        .enumerate()
        .map(|(index, (name, series))| {
            let (colour, marker, dashed) = line_look(index);
            let runs = thin(&series.values, plot.w as usize)
                .into_iter()
                .map(|run| {
                    run.into_iter()
                        .map(|(i, value)| (to_x(i), to_y(value)))
                        .collect()
                })
                .collect();
            PaperLine {
                name: (*name).to_string(),
                colour,
                marker,
                dashed,
                runs,
                marks: marks_of(&series.values, &to_x, &to_y),
            }
        })
        .collect();
    Some(PaperChart {
        plot,
        x_ticks,
        y_ticks,
        lines: made,
        x_label: "index".to_string(),
        y_label: match lines {
            [(name, _)] => Some((*name).to_string()),
            _ => None,
        },
    })
}

/// About `MARKS` points spread evenly along the values, skipping any that is
/// not a number.
fn marks_of(
    values: &[Option<f64>],
    to_x: &dyn Fn(f64) -> f64,
    to_y: &dyn Fn(f64) -> f64,
) -> Vec<(f64, f64)> {
    let count = values.len();
    let wanted = count.min(MARKS);
    (0..wanted)
        .filter_map(|k| {
            let index = if wanted > 1 {
                (k * (count - 1) + (wanted - 1) / 2) / (wanted - 1)
            } else {
                0
            };
            let value = values[index].filter(|number| number.is_finite())?;
            Some((to_x(index as f64), to_y(value)))
        })
        .collect()
}

/// The size of a picture of `width` by `height` pixels enlarged by a whole
/// number so it fills the figure without blur: the factor, and the new width and
/// height. A picture already larger than the figure keeps its size.
pub fn picture_scale(width: u32, height: u32) -> (u32, u32, u32) {
    let factor = (PAPER_WIDTH / width.max(1))
        .min(PAPER_HEIGHT / height.max(1))
        .max(1);
    (factor, width * factor, height * factor)
}
