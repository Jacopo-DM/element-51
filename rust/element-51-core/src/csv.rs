//! A chart's series as CSV text.

use crate::Series;

/// The CSV of `lines`, each a name and a series: a header `index,name,...`,
/// then one row per index up to the longest series. A gap, a number that is
/// not finite and the end of a shorter series are empty cells. Numbers are the
/// shortest text that reads back as the same number. A name with a comma, a
/// quote or a line break is quoted. None if no series has a finite number.
pub fn chart_csv(lines: &[(&str, &Series)]) -> Option<String> {
    let finite = |value: &Option<f64>| value.filter(|number| number.is_finite());
    if !lines
        .iter()
        .any(|(_, series)| series.values.iter().any(|value| finite(value).is_some()))
    {
        return None;
    }
    let mut text = String::from("index");
    for (name, _) in lines {
        text.push(',');
        text.push_str(&cell(name));
    }
    text.push('\n');
    let longest = lines.iter().map(|(_, series)| series.values.len()).max()?;
    for index in 0..longest {
        text.push_str(&index.to_string());
        for (_, series) in lines {
            text.push(',');
            if let Some(number) = series.values.get(index).and_then(finite) {
                text.push_str(&number.to_string());
            }
        }
        text.push('\n');
    }
    Some(text)
}

/// `text` as one CSV cell: in quotes, with quotes doubled, if it needs them.
fn cell(text: &str) -> String {
    if text.contains([',', '"', '\n', '\r']) {
        format!("\"{}\"", text.replace('"', "\"\""))
    } else {
        text.to_string()
    }
}

/// `text` as a `data:` URL a browser can download as a CSV file: UTF-8, with
/// every byte that is not a letter, digit or one of `-_.~` written as `%XX`.
pub fn csv_data_url(text: &str) -> String {
    let mut url = String::from("data:text/csv;charset=utf-8,");
    for byte in text.bytes() {
        if byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_' | b'.' | b'~') {
            url.push(char::from(byte));
        } else {
            url.push_str(&format!("%{byte:02X}"));
        }
    }
    url
}
