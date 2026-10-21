//! Reading and showing the values typed into input boxes.

use serde_json::Value;

use crate::{NodeOutcome, PortRef, PortType, Status};

/// Read what a person typed for an input of type `kind`.
///
/// Empty text means the node has no value of its own (`Ok(None)`), so the
/// input falls back to its default. Anything else must fit the type. A picture
/// has no typed value, so anything but empty text is refused.
pub fn parse_input(text: &str, kind: PortType) -> Result<Option<Value>, String> {
    if text.trim().is_empty() {
        return Ok(None);
    }
    let trimmed = text.trim();
    match kind {
        PortType::Image => Err("a picture can only be linked".to_string()),
        PortType::Series => Err("a series can only be linked".to_string()),
        PortType::Str => Ok(Some(Value::String(text.to_string()))),
        PortType::Float => trimmed
            .parse::<f64>()
            .ok()
            .filter(|number| number.is_finite())
            .and_then(serde_json::Number::from_f64)
            .map(|number| Some(Value::Number(number)))
            .ok_or_else(|| format!("'{trimmed}' is not a number")),
        PortType::Int => trimmed
            .parse::<i64>()
            .map(|number| Some(Value::from(number)))
            .map_err(|_| format!("'{trimmed}' is not a whole number")),
        PortType::Bool => match trimmed {
            "true" => Ok(Some(Value::Bool(true))),
            "false" => Ok(Some(Value::Bool(false))),
            _ => Err(format!("'{trimmed}' is not true or false")),
        },
    }
}

/// The name of the input box for port `port` of node `node`, for screen readers.
pub fn field_label(node: &str, port: &str) -> String {
    format!("{port} of node {node}")
}

/// The text an input box shows for `value`.
pub fn format_value(value: &Value) -> String {
    match value {
        Value::String(text) => text.clone(),
        other => other.to_string(),
    }
}

/// What a linked input's box shows: the value its source produced, if the last
/// run gave one, and the source's name.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LinkedText {
    /// The source's value as JSON text, such as `21.0` or `"hi"`.
    pub value: Option<String>,
    /// The source as `n1.out`, or `[n1.out]` when a value comes before it.
    pub source: String,
}

/// The text for an input linked to output `source`, given the source node's
/// outcome in the last run (`None` before a run or after an edit). A source that
/// failed, was skipped or produced no plain value (a picture or a series) has no
/// value to show.
pub fn linked_text(source: &PortRef, outcome: Option<&NodeOutcome>) -> LinkedText {
    let name = format!("{}.{}", source.node, source.port);
    let value = outcome
        .filter(|outcome| outcome.status == Status::Ok)
        .and_then(|outcome| outcome.output_text(&source.port));
    LinkedText {
        source: if value.is_some() {
            format!("[{name}]")
        } else {
            name
        },
        value,
    }
}

/// How the room in a box is shared by a value and a source of the given widths,
/// with `gap` between them: the value keeps what it needs, the source gets what
/// is left and is cut first. Returns `(value_room, source_room)`.
pub fn share_room(value_width: f64, source_width: f64, room: f64, gap: f64) -> (f64, f64) {
    if value_width + gap + source_width <= room {
        return (value_width, source_width);
    }
    let value_room = value_width.min(room).max(0.0);
    (value_room, (room - value_room - gap).max(0.0))
}
