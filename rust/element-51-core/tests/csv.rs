//! A chart's numbers as CSV text, and the name of the file.

use element_51_core::Series;
use element_51_core::csv::{chart_csv, csv_data_url};
use element_51_core::menu::csv_file_name;

fn series(values: &[Option<f64>]) -> Series {
    Series {
        values: values.to_vec(),
    }
}

#[test]
fn one_column_per_series_with_the_index_first() {
    let loss = series(&[Some(1.0), Some(0.5), Some(0.25)]);
    let accuracy = series(&[Some(0.1), Some(0.5), Some(0.9)]);
    let text = chart_csv(&[("loss", &loss), ("accuracy", &accuracy)]).unwrap();
    assert_eq!(
        text,
        "index,loss,accuracy\n0,1,0.1\n1,0.5,0.5\n2,0.25,0.9\n"
    );
}

#[test]
fn a_gap_a_shorter_series_and_a_number_that_is_not_finite_are_empty_cells() {
    let first = series(&[Some(1.0), None, Some(f64::NAN), Some(4.0)]);
    let second = series(&[Some(2.0), Some(f64::INFINITY)]);
    let text = chart_csv(&[("a", &first), ("b", &second)]).unwrap();
    assert_eq!(text, "index,a,b\n0,1,2\n1,,\n2,,\n3,4,\n");
}

#[test]
fn a_number_is_written_so_that_it_reads_back_the_same() {
    let values = [0.1 + 0.2, 1e-7, 123456789.125, -0.0, 5e-324];
    let line = series(&values.map(Some));
    let text = chart_csv(&[("x", &line)]).unwrap();
    for (row, expected) in text.lines().skip(1).zip(values) {
        let cell = row.split(',').nth(1).unwrap();
        assert_eq!(
            cell.parse::<f64>().unwrap().to_bits(),
            expected.to_bits(),
            "{cell}"
        );
    }
}

#[test]
fn a_name_with_a_comma_a_quote_or_a_line_break_is_quoted() {
    let line = series(&[Some(1.0)]);
    let text = chart_csv(&[("a,b", &line), ("say \"hi\"", &line), ("n3\nloss", &line)]).unwrap();
    assert_eq!(
        text,
        "index,\"a,b\",\"say \"\"hi\"\"\",\"n3\nloss\"\n0,1,1,1\n"
    );
}

#[test]
fn a_chart_with_no_number_has_nothing_to_save() {
    assert_eq!(chart_csv(&[]), None);
    let empty = series(&[]);
    let gaps = series(&[None, Some(f64::NAN)]);
    assert_eq!(chart_csv(&[("a", &empty), ("b", &gaps)]), None);
}

#[test]
fn the_csv_file_is_named_after_the_node() {
    assert_eq!(csv_file_name("n3"), "n3-chart.csv");
    assert_eq!(csv_file_name("../a b"), "___a_b-chart.csv");
}

#[test]
fn the_data_url_escapes_every_byte_a_url_cannot_carry() {
    assert_eq!(
        csv_data_url("index,a\n0,1.5\n"),
        "data:text/csv;charset=utf-8,index%2Ca%0A0%2C1.5%0A"
    );
    assert_eq!(
        csv_data_url("\"é\""),
        "data:text/csv;charset=utf-8,%22%C3%A9%22"
    );
}
