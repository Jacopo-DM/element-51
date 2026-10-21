//! Pictures on a card: the wire record, the card's geometry and the rules for
//! a picture port.

use element_51_core::display::{fit_picture, picture_rect, picture_side};
use element_51_core::fields::parse_input;
use element_51_core::layout::node_rect;
use element_51_core::theme::{Metrics, Theme};
use element_51_core::{
    CardDisplay, Image, Message, NodeType, PlacedNode, Port, PortType, can_link, value_fits,
};
use serde_json::json;

const SNAPSHOT: &str = include_str!("../../../tests/fixtures/snapshot.json");
const MESSAGES: &str = include_str!("../../../tests/fixtures/messages.json");

fn m() -> Metrics {
    Theme::default().metrics
}

fn card(display: CardDisplay) -> (PlacedNode, NodeType) {
    let node = PlacedNode {
        id: "n1".to_string(),
        node_type: "t".to_string(),
        x: 40.0,
        y: 60.0,
        values: Default::default(),
    };
    let kind = if display == CardDisplay::Image {
        PortType::Image
    } else {
        PortType::Float
    };
    let node_type = NodeType {
        job: false,
        name: "t".to_string(),
        inputs: vec![Port {
            name: "a".to_string(),
            kind: PortType::Float,
        }],
        outputs: vec![Port {
            name: "out".to_string(),
            kind,
        }],
        defaults: Default::default(),
        category: String::new(),
        display,
        variadic: Default::default(),
    };
    (node, node_type)
}

fn picture(width: u32, height: u32, channels: u8) -> Image {
    Image {
        width,
        height,
        channels,
        pixels: (0..width * height * u32::from(channels))
            .map(|n| (n % 256) as u8)
            .collect(),
    }
}

#[test]
fn the_picture_area_is_a_square_inside_the_card() {
    assert_eq!(picture_side(&m()), m().node_width - 2.0 * m().title_pad);
}

#[test]
fn a_card_with_a_picture_is_taller_by_the_area_and_a_margin() {
    let (node, plain) = card(CardDisplay::None);
    let (_, with_picture) = card(CardDisplay::Image);
    let without = node_rect(&node, &plain, &m());
    let with = node_rect(&node, &with_picture, &m());
    assert_eq!(with.h - without.h, picture_side(&m()) + m().title_pad);
    assert_eq!((with.x, with.y, with.w), (without.x, without.y, without.w));
}

#[test]
fn a_card_without_a_picture_has_no_picture_area() {
    let (node, plain) = card(CardDisplay::None);
    let rect = node_rect(&node, &plain, &m());
    assert_eq!(picture_rect(&rect, &plain, &m()), None);
}

#[test]
fn the_picture_area_sits_below_the_port_rows() {
    let (node, with_picture) = card(CardDisplay::Image);
    let rect = node_rect(&node, &with_picture, &m());
    let area = picture_rect(&rect, &with_picture, &m()).unwrap();
    assert_eq!(area.x, rect.x + m().title_pad);
    assert_eq!(area.y, rect.y + m().header_height + 2.0 * m().row_height);
    assert_eq!((area.w, area.h), (picture_side(&m()), picture_side(&m())));
    assert_eq!(area.y + area.h + m().title_pad, rect.y + rect.h);
}

#[test]
fn a_wide_picture_fills_the_width_and_is_centred_upright() {
    let (node, with_picture) = card(CardDisplay::Image);
    let rect = node_rect(&node, &with_picture, &m());
    let area = picture_rect(&rect, &with_picture, &m()).unwrap();
    let fitted = fit_picture(&area, 4, 2);
    assert_eq!(fitted.w, area.w);
    assert_eq!(fitted.h, area.h / 2.0);
    assert_eq!(fitted.x, area.x);
    assert_eq!(fitted.y, area.y + area.h / 4.0);
}

#[test]
fn a_tall_picture_fills_the_height_and_is_centred_sideways() {
    let (node, with_picture) = card(CardDisplay::Image);
    let rect = node_rect(&node, &with_picture, &m());
    let area = picture_rect(&rect, &with_picture, &m()).unwrap();
    let fitted = fit_picture(&area, 1, 4);
    assert_eq!(fitted.h, area.h);
    assert_eq!(fitted.w, area.w / 4.0);
    assert_eq!(fitted.x, area.x + area.w * 3.0 / 8.0);
    assert_eq!(fitted.y, area.y);
}

#[test]
fn a_square_picture_fills_the_area() {
    let (node, with_picture) = card(CardDisplay::Image);
    let rect = node_rect(&node, &with_picture, &m());
    let area = picture_rect(&rect, &with_picture, &m()).unwrap();
    assert_eq!(fit_picture(&area, 28, 28), area);
}

#[test]
fn a_grey_picture_becomes_opaque_rgba() {
    let image = Image {
        width: 2,
        height: 1,
        channels: 1,
        pixels: vec![10, 200],
    };
    assert_eq!(image.rgba().unwrap(), [10, 10, 10, 255, 200, 200, 200, 255]);
}

#[test]
fn a_colour_picture_becomes_opaque_rgba() {
    let image = Image {
        width: 1,
        height: 2,
        channels: 3,
        pixels: vec![1, 2, 3, 4, 5, 6],
    };
    assert_eq!(image.rgba().unwrap(), [1, 2, 3, 255, 4, 5, 6, 255]);
}

#[test]
fn a_picture_whose_pixels_do_not_fill_it_has_no_rgba() {
    let mut image = picture(3, 3, 1);
    image.pixels.pop();
    assert_eq!(image.rgba(), None);
    let odd_channels = Image {
        channels: 2,
        ..picture(2, 2, 1)
    };
    assert_eq!(odd_channels.rgba(), None);
}

#[test]
fn a_picture_is_a_list_of_integers_on_the_wire() {
    let image: Image = serde_json::from_value(json!({
        "width": 2, "height": 1, "channels": 1, "pixels": [0, 255]
    }))
    .unwrap();
    assert_eq!(image.pixels, [0, 255]);
    assert_eq!(
        serde_json::to_value(&image).unwrap(),
        json!({"width": 2, "height": 1, "channels": 1, "pixels": [0, 255]})
    );
}

#[test]
fn only_a_picture_feeds_a_picture() {
    assert!(can_link(PortType::Image, PortType::Image));
    assert!(!can_link(PortType::Image, PortType::Float));
    assert!(!can_link(PortType::Float, PortType::Image));
    assert!(!can_link(PortType::Int, PortType::Image));
}

#[test]
fn a_picture_input_has_no_box_and_takes_no_typed_value() {
    let error = parse_input("5", PortType::Image).unwrap_err();
    assert!(error.contains("linked"), "{error}");
    assert!(!value_fits(&json!(5), PortType::Image));
    assert!(!value_fits(&json!("x"), PortType::Image));
    assert!(!value_fits(&json!(true), PortType::Image));
    assert!(!value_fits(&json!(1.5), PortType::Image));
}

#[test]
fn an_empty_picture_box_means_no_value() {
    assert_eq!(parse_input("  ", PortType::Image), Ok(None));
}

#[test]
fn the_shared_snapshot_has_a_picture_node_with_a_display() {
    let Message::Snapshot(snapshot) = Message::from_json(SNAPSHOT).unwrap() else {
        panic!("not a snapshot");
    };
    let picture = snapshot.node_type("picture").unwrap();
    assert_eq!(picture.display, CardDisplay::Image);
    assert_eq!(picture.outputs[0].kind, PortType::Image);
    assert_eq!(
        snapshot.node_type("add").unwrap().display,
        CardDisplay::None
    );
}

#[test]
fn the_shared_result_shows_a_picture() {
    let fixture: serde_json::Value = serde_json::from_str(MESSAGES).unwrap();
    let Message::RunResult(result) =
        Message::from_json(&fixture["run_result_picture"].to_string()).unwrap()
    else {
        panic!("not a result");
    };
    let shown = result.outcomes[0].shown["out"].as_image().unwrap();
    assert_eq!((shown.width, shown.height, shown.channels), (2, 2, 1));
    assert_eq!(shown.pixels, [0, 85, 170, 255]);
    assert!(result.outcomes[0].outputs.is_empty());
}

#[test]
fn an_older_message_without_display_or_shown_still_reads() {
    let node_type: NodeType =
        serde_json::from_value(json!({"name": "t", "inputs": [], "outputs": []})).unwrap();
    assert_eq!(node_type.display, CardDisplay::None);
    let message = json!({"op": "run_result", "outcomes": [
        {"node": "n1", "status": "ok", "outputs": {}, "error": null}
    ]});
    let Message::RunResult(result) = Message::from_json(&message.to_string()).unwrap() else {
        panic!("not a result");
    };
    assert!(result.outcomes[0].shown.is_empty());
}

// ---- series and the chart ----

use element_51_core::display::{
    CHART_LINE_STYLES, chart_layout, chart_rect, last_value, legend_slots, line_style, nice_ticks,
    short_number, shown_summary, shown_tooltip, thin,
};
use element_51_core::{Series, Shown};

fn series(values: &[Option<f64>]) -> Series {
    Series {
        values: values.to_vec(),
    }
}

fn some(values: &[f64]) -> Vec<Option<f64>> {
    values.iter().map(|value| Some(*value)).collect()
}

fn close(a: f64, b: f64) -> bool {
    (a - b).abs() < 1e-9
}

#[test]
fn series_is_its_own_port_kind_that_links_only_to_itself() {
    assert_eq!(PortType::Series.to_string(), "series");
    assert!(can_link(PortType::Series, PortType::Series));
    assert!(!can_link(PortType::Series, PortType::Float));
    assert!(!can_link(PortType::Float, PortType::Series));
    assert!(!can_link(PortType::Image, PortType::Series));
    assert!(
        parse_input("1", PortType::Series)
            .unwrap_err()
            .contains("linked")
    );
    assert!(!value_fits(&json!(1.0), PortType::Series));
}

#[test]
fn a_series_reads_null_as_a_gap_and_writes_it_back() {
    let read: Series = serde_json::from_value(json!({"values": [0.9, null, 3]})).unwrap();
    assert_eq!(read.values, [Some(0.9), None, Some(3.0)]);
    assert_eq!(
        serde_json::to_value(&read).unwrap(),
        json!({"values": [0.9, null, 3.0]})
    );
}

#[test]
fn shown_tells_a_picture_from_a_series() {
    let picture: Shown =
        serde_json::from_value(json!({"width": 1, "height": 1, "channels": 1, "pixels": [7]}))
            .unwrap();
    assert!(matches!(picture, Shown::Image(_)));
    let chart: Shown = serde_json::from_value(json!({"values": [1.0]})).unwrap();
    assert!(matches!(chart, Shown::Series(_)));
    assert!(serde_json::from_value::<Shown>(json!({"values": [1.0], "x": 1})).is_err());
}

#[test]
fn the_shared_result_shows_a_chart_with_a_gap() {
    let fixture: serde_json::Value = serde_json::from_str(MESSAGES).unwrap();
    let Message::RunResult(result) =
        Message::from_json(&fixture["run_result_chart"].to_string()).unwrap()
    else {
        panic!("not a result");
    };
    let Shown::Series(series) = &result.outcomes[0].shown["series_1"] else {
        panic!("not a series");
    };
    assert_eq!(series.values, [Some(0.9), Some(0.5), None, Some(0.25)]);
}

#[test]
fn the_shared_snapshot_has_a_plot_node_that_shows_a_chart() {
    let Message::Snapshot(snapshot) = Message::from_json(SNAPSHOT).unwrap() else {
        panic!("not a snapshot");
    };
    let plot = snapshot.node_type("Plot").unwrap();
    assert_eq!(plot.display, CardDisplay::Chart);
    assert_eq!(plot.inputs[0].kind, PortType::Series);
    assert!(plot.outputs.is_empty());
}

fn plot_card() -> (PlacedNode, NodeType) {
    let node = PlacedNode {
        id: "n1".to_string(),
        node_type: "Plot".to_string(),
        x: 0.0,
        y: 0.0,
        values: Default::default(),
    };
    let node_type = NodeType {
        job: false,
        name: "Plot".to_string(),
        inputs: vec![Port {
            name: "series_1".to_string(),
            kind: PortType::Series,
        }],
        outputs: vec![],
        defaults: Default::default(),
        category: String::new(),
        display: CardDisplay::Chart,
        variadic: Default::default(),
    };
    (node, node_type)
}

#[test]
fn a_chart_card_is_taller_by_the_chart_area_and_a_margin() {
    let (node, plot) = plot_card();
    let rect = node_rect(&node, &plot, &m());
    assert_eq!(
        rect.h,
        m().header_height + m().row_height + 0.85 * m().node_width + m().title_pad
    );
    let area = chart_rect(&rect, &plot, &m()).unwrap();
    assert_eq!(area.x, rect.x + m().title_pad);
    assert_eq!(area.y, rect.y + m().header_height + m().row_height);
    assert_eq!(
        (area.w, area.h),
        (picture_side(&m()), 0.85 * m().node_width)
    );
    assert_eq!(area.y + area.h + m().title_pad, rect.y + rect.h);
    assert_eq!(picture_rect(&rect, &plot, &m()), None);
}

#[test]
fn only_a_chart_card_has_a_chart_area() {
    let (node, plain) = card(CardDisplay::None);
    let rect = node_rect(&node, &plain, &m());
    assert_eq!(chart_rect(&rect, &plain, &m()), None);
    let (_, with_picture) = card(CardDisplay::Image);
    assert_eq!(chart_rect(&rect, &with_picture, &m()), None);
}

#[test]
fn numbers_are_short_with_about_three_significant_digits() {
    assert_eq!(short_number(0.0), "0");
    assert_eq!(short_number(0.3), "0.3");
    assert_eq!(short_number(0.31415), "0.314");
    assert_eq!(short_number(12.345), "12.3");
    assert_eq!(short_number(1.5), "1.5");
    assert_eq!(short_number(100.0), "100");
    assert_eq!(short_number(1234.7), "1235");
    assert_eq!(short_number(-2.5), "-2.5");
    assert_eq!(short_number(0.00012), "0.00012");
    assert_eq!(short_number(0.000012), "1.2e-5");
    assert_eq!(short_number(f64::NAN), "nan");
    assert_eq!(short_number(f64::INFINITY), "inf");
}

#[test]
fn ticks_are_round_numbers_inside_the_range() {
    let ticks = nice_ticks(0.0, 1.0, 5);
    assert_eq!(ticks.len(), 6);
    for (tick, expected) in ticks.iter().zip([0.0, 0.2, 0.4, 0.6, 0.8, 1.0]) {
        assert!(close(*tick, expected), "{tick} vs {expected}");
    }
    assert_eq!(nice_ticks(0.0, 100.0, 4), [0.0, 50.0, 100.0]);
    let inside = nice_ticks(0.13, 0.87, 4);
    assert!(inside.iter().all(|tick| (0.13..=0.87).contains(tick)));
    assert!(inside.len() >= 2);
    assert_eq!(nice_ticks(3.0, 3.0, 4), [3.0]);
    assert_eq!(nice_ticks(5.0, 1.0, 4), [5.0]);
}

#[test]
fn a_short_series_is_not_thinned_and_a_gap_splits_the_line() {
    let values = [Some(1.0), Some(2.0), None, Some(4.0)];
    let segments = thin(&values, 10);
    assert_eq!(segments, [vec![(0.0, 1.0), (1.0, 2.0)], vec![(3.0, 4.0)]]);
    assert!(thin(&[], 10).is_empty());
    assert!(thin(&[None, None], 10).is_empty());
}

#[test]
fn a_long_series_keeps_the_min_and_max_of_each_bucket() {
    let values: Vec<Option<f64>> = (0..1000)
        .map(|n| {
            Some(if n % 100 == 50 {
                9.0
            } else if n % 100 == 0 {
                -9.0
            } else {
                0.0
            })
        })
        .collect();
    let segments = thin(&values, 10);
    assert_eq!(segments.len(), 1);
    let points = &segments[0];
    assert!(points.len() <= 20);
    assert!(points.iter().any(|point| point.1 == 9.0));
    assert!(points.iter().any(|point| point.1 == -9.0));
    assert!(points.windows(2).all(|pair| pair[0].0 < pair[1].0));
}

#[test]
fn a_bucket_with_no_values_breaks_the_line() {
    let mut values: Vec<Option<f64>> = some(&vec![1.0; 100]);
    for value in &mut values[40..60] {
        *value = None;
    }
    let segments = thin(&values, 5);
    assert_eq!(segments.len(), 2);
}

#[test]
fn the_last_value_is_the_last_one_that_is_a_number() {
    assert_eq!(last_value(&series(&some(&[1.0, 2.0, 0.31]))), Some(0.31));
    assert_eq!(last_value(&series(&[Some(1.0), None])), Some(1.0));
    assert_eq!(last_value(&series(&[None])), None);
    assert_eq!(last_value(&series(&[])), None);
}

#[test]
fn lines_take_the_port_colours_in_turn_and_dash_from_the_sixth() {
    assert_eq!(CHART_LINE_STYLES.len(), 5);
    assert_eq!(line_style(0), (PortType::Float, false));
    assert_eq!(line_style(1), (PortType::Str, false));
    assert_eq!(line_style(4), (PortType::Series, false));
    assert_eq!(line_style(5), (PortType::Float, true));
    assert_eq!(line_style(10), (PortType::Float, true));
}

fn area() -> element_51_core::layout::Rect {
    element_51_core::layout::Rect {
        x: 100.0,
        y: 200.0,
        w: 200.0,
        h: 132.0,
    }
}

#[test]
fn the_plot_sits_inside_the_area_with_room_for_labels() {
    let loss = series(&some(&[0.9, 0.5, 0.25, 0.1]));
    let chart = chart_layout(&area(), &m(), &[&loss]).unwrap();
    let (a, p) = (area(), chart.plot);
    assert!(p.x > a.x && p.y >= a.y);
    assert!(p.x + p.w <= a.x + a.w && p.y + p.h < a.y + a.h);
    for tick in chart.x_ticks.iter().chain(&chart.y_ticks) {
        assert!(!tick.label.is_empty());
    }
    assert!(
        chart
            .x_ticks
            .iter()
            .all(|t| t.at >= p.x - 1e-9 && t.at <= p.x + p.w + 1e-9)
    );
    assert!(
        chart
            .y_ticks
            .iter()
            .all(|t| t.at >= p.y - 1e-9 && t.at <= p.y + p.h + 1e-9)
    );
}

#[test]
fn a_falling_series_runs_from_upper_left_to_lower_right() {
    let loss = series(&some(&[0.9, 0.5, 0.25, 0.1]));
    let chart = chart_layout(&area(), &m(), &[&loss]).unwrap();
    assert_eq!(chart.lines.len(), 1);
    let points = &chart.lines[0][0];
    assert_eq!(points.len(), 4);
    let p = chart.plot;
    assert!(close(points[0].0, p.x) && close(points[3].0, p.x + p.w));
    assert!(
        points[0].1 < points[3].1,
        "higher value is higher on the screen"
    );
    assert!(
        points
            .iter()
            .all(|point| point.1 >= p.y && point.1 <= p.y + p.h)
    );
    let ys: Vec<f64> = points.iter().map(|point| point.1).collect();
    assert!(ys.windows(2).all(|pair| pair[0] < pair[1]));
}

#[test]
fn two_lines_share_one_scale() {
    let low = series(&some(&[0.0, 0.0]));
    let high = series(&some(&[10.0, 10.0]));
    let chart = chart_layout(&area(), &m(), &[&low, &high]).unwrap();
    assert!(chart.lines[1][0][0].1 < chart.lines[0][0][0].1);
}

#[test]
fn a_flat_series_gets_a_scale_instead_of_dividing_by_zero() {
    let flat = series(&some(&[2.0, 2.0, 2.0]));
    let chart = chart_layout(&area(), &m(), &[&flat]).unwrap();
    let ys: Vec<f64> = chart.lines[0][0].iter().map(|point| point.1).collect();
    assert!(ys.iter().all(|y| y.is_finite()));
    assert!(close(ys[0], ys[2]));
}

#[test]
fn a_single_point_is_drawn_and_a_gap_splits_a_line() {
    let one = series(&some(&[5.0]));
    let chart = chart_layout(&area(), &m(), &[&one]).unwrap();
    assert_eq!(chart.lines[0][0].len(), 1);
    let gappy = series(&[Some(1.0), None, Some(3.0)]);
    let chart = chart_layout(&area(), &m(), &[&gappy]).unwrap();
    assert_eq!(chart.lines[0].len(), 2);
}

#[test]
fn nothing_to_draw_gives_no_layout() {
    assert!(chart_layout(&area(), &m(), &[]).is_none());
    assert!(chart_layout(&area(), &m(), &[&series(&[])]).is_none());
    assert!(chart_layout(&area(), &m(), &[&series(&[None, None])]).is_none());
}

#[test]
fn a_very_long_series_is_thinned_to_about_twice_the_plot_width() {
    let values: Vec<f64> = (0..100_000).map(|n| (f64::from(n) / 100.0).sin()).collect();
    let big = series(&some(&values));
    let chart = chart_layout(&area(), &m(), &[&big]).unwrap();
    let points: usize = chart.lines[0].iter().map(Vec::len).sum();
    assert!(points <= 2 * chart.plot.w as usize + 2, "{points}");
    assert!(points > chart.plot.w as usize);
}

#[test]
fn a_picture_summary_gives_its_size_and_whether_it_is_grey() {
    let grey = Shown::Image(Image {
        width: 28,
        height: 20,
        channels: 1,
        pixels: vec![0; 560],
    });
    assert_eq!(shown_summary(&grey), "28 x 20 grey");
    let colour = Shown::Image(Image {
        width: 1,
        height: 2,
        channels: 3,
        pixels: vec![0; 6],
    });
    assert_eq!(shown_summary(&colour), "1 x 2 colour");
}

#[test]
fn a_series_summary_is_its_last_number() {
    let series = |values: Vec<Option<f64>>| Shown::Series(Series { values });
    assert_eq!(shown_summary(&series(vec![Some(1.0), Some(0.05)])), "0.05");
    // A trailing gap or NaN is skipped, as the chart's end label does.
    assert_eq!(shown_summary(&series(vec![Some(2.0), None])), "2");
    assert_eq!(shown_summary(&series(vec![None, None])), "2 gaps");
    assert_eq!(shown_summary(&series(vec![None])), "1 gap");
    assert_eq!(shown_summary(&series(vec![])), "0 gaps");
}

#[test]
fn the_legend_row_sits_above_the_plot_inside_the_area() {
    let loss = series(&some(&[0.9, 0.5, 0.25, 0.1]));
    let chart = chart_layout(&area(), &m(), &[&loss]).unwrap();
    let (a, l, p) = (area(), chart.legend, chart.plot);
    assert!(l.x >= a.x && l.x + l.w <= a.x + a.w && l.y >= a.y);
    assert!(
        close(l.y + l.h, p.y),
        "the plot starts where the legend ends"
    );
    assert!(p.h > 0.0);
}

#[test]
fn legend_slots_split_the_row_equally_from_the_left() {
    let row = element_51_core::layout::Rect {
        x: 10.0,
        y: 20.0,
        w: 150.0,
        h: 26.0,
    };
    assert!(legend_slots(&row, 0).is_empty());
    let slots = legend_slots(&row, 3);
    assert_eq!(slots.len(), 3);
    for (index, slot) in slots.iter().enumerate() {
        assert!(close(slot.w, 50.0) && close(slot.h, 26.0) && close(slot.y, 20.0));
        assert!(close(slot.x, 10.0 + 50.0 * index as f64));
    }
}

#[test]
fn a_series_that_runs_from_nothing_to_one_gets_at_least_four_y_labels() {
    let accuracy = series(&some(&[0.05, 0.3, 0.6, 0.9, 0.95]));
    let chart = chart_layout(&area(), &m(), &[&accuracy]).unwrap();
    assert!(chart.y_ticks.len() >= 4, "{:?}", chart.y_ticks);
}

#[test]
fn the_tooltip_says_what_the_box_is_a_summary_of() {
    let series = |values: Vec<Option<f64>>| Shown::Series(Series { values });
    assert_eq!(
        shown_tooltip(&series(vec![Some(1.0), Some(0.05)])),
        "last of 2 points"
    );
    assert_eq!(shown_tooltip(&series(vec![Some(1.0)])), "last of 1 point");
    assert_eq!(
        shown_tooltip(&series(vec![None, None])),
        "2 points, none of them a number"
    );
    let grey = Shown::Image(Image {
        width: 28,
        height: 20,
        channels: 1,
        pixels: vec![0; 560],
    });
    assert_eq!(shown_tooltip(&grey), "grey picture, 28 x 20");
    let colour = Shown::Image(Image {
        width: 1,
        height: 2,
        channels: 3,
        pixels: vec![0; 6],
    });
    assert_eq!(shown_tooltip(&colour), "colour picture, 1 x 2");
}
