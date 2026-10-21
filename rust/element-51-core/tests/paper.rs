//! The figure for a paper: its layout, markers, numbers and picture size.

use element_51_core::Series;
use element_51_core::paper::{
    Marker, PAPER_HEIGHT, PAPER_WIDTH, line_look, paper_chart, paper_number, picture_scale, points,
};

fn series(values: &[f64]) -> Series {
    Series {
        values: values.iter().map(|value| Some(*value)).collect(),
    }
}

#[test]
fn the_plot_sits_inside_the_figure_with_room_for_labels() {
    let loss = series(&[1.0, 0.5, 0.25]);
    let chart = paper_chart(&[("loss", &loss)]).unwrap();
    let plot = chart.plot;
    assert!(plot.x > 100.0 && plot.y > 20.0);
    assert!(plot.x + plot.w < f64::from(PAPER_WIDTH));
    assert!(plot.y + plot.h < f64::from(PAPER_HEIGHT) - 100.0);
}

#[test]
fn a_single_line_names_the_y_axis_and_several_do_not() {
    let a = series(&[1.0, 2.0]);
    let b = series(&[2.0, 1.0]);
    let one = paper_chart(&[("loss", &a)]).unwrap();
    assert_eq!(one.y_label.as_deref(), Some("loss"));
    assert_eq!(one.x_label, "index");
    let two = paper_chart(&[("loss", &a), ("accuracy", &b)]).unwrap();
    assert_eq!(two.y_label, None);
}

#[test]
fn the_lines_start_blue_with_circles_then_red_with_crosses() {
    assert_eq!(line_look(0), ("#0000ff", Marker::Circle, false));
    assert_eq!(line_look(1), ("#ff0000", Marker::Cross, false));
    assert_eq!(line_look(6), ("#0000ff", Marker::Circle, true));
}

#[test]
fn ticks_sit_inside_the_plot_and_use_a_true_minus() {
    let wave = series(&[-1.0, 0.0, 1.0, 0.0, -1.0]);
    let chart = paper_chart(&[("wave", &wave)]).unwrap();
    let plot = chart.plot;
    for tick in &chart.x_ticks {
        assert!(tick.at >= plot.x - 0.01 && tick.at <= plot.x + plot.w + 0.01);
    }
    for tick in &chart.y_ticks {
        assert!(tick.at >= plot.y - 0.01 && tick.at <= plot.y + plot.h + 0.01);
    }
    assert!(
        chart
            .y_ticks
            .iter()
            .any(|tick| tick.label.starts_with('\u{2212}'))
    );
    assert_eq!(paper_number(-0.5), "\u{2212}0.5");
    assert_eq!(paper_number(12.0), "12");
}

#[test]
fn about_ten_markers_are_spread_along_a_long_line_and_all_on_a_short_one() {
    let long: Vec<f64> = (0..282).map(f64::from).collect();
    let chart = paper_chart(&[("up", &series(&long))]).unwrap();
    let marks = &chart.lines[0].marks;
    assert_eq!(marks.len(), 10);
    assert!(marks.windows(2).all(|pair| pair[0].0 < pair[1].0));
    let first = marks.first().unwrap().0;
    let last = marks.last().unwrap().0;
    assert!((first - chart.plot.x).abs() < 1.0);
    assert!((last - (chart.plot.x + chart.plot.w)).abs() < 1.0);
    let short = paper_chart(&[("up", &series(&[1.0, 2.0, 3.0]))]).unwrap();
    assert_eq!(short.lines[0].marks.len(), 3);
}

#[test]
fn a_gap_breaks_the_line_and_has_no_marker() {
    let gappy = Series {
        values: vec![Some(1.0), None, Some(2.0)],
    };
    let chart = paper_chart(&[("gappy", &gappy)]).unwrap();
    assert_eq!(chart.lines[0].runs.len(), 2);
    assert_eq!(chart.lines[0].marks.len(), 2);
}

#[test]
fn no_numbers_gives_no_chart() {
    let empty = Series {
        values: vec![None, None],
    };
    assert_eq!(paper_chart(&[("empty", &empty)]), None);
}

#[test]
fn a_flat_line_still_has_a_scale() {
    let flat = series(&[3.0, 3.0, 3.0]);
    let chart = paper_chart(&[("flat", &flat)]).unwrap();
    assert!(chart.y_ticks.len() >= 2);
}

#[test]
fn points_are_pixels_at_300_dpi() {
    assert!((points(72.0) - 300.0).abs() < 1e-9);
    assert!((points(10.0) - 41.666_666).abs() < 1e-3);
}

#[test]
fn a_small_picture_is_enlarged_by_a_whole_number_to_fill_the_figure() {
    assert_eq!(picture_scale(28, 28), (50, 1400, 1400));
    assert_eq!(picture_scale(100, 10), (21, 2100, 210));
    assert_eq!(picture_scale(256, 256), (5, 1280, 1280));
    assert_eq!(picture_scale(3000, 10), (1, 3000, 10));
}
