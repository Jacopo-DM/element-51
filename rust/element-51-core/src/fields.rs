//! Reading and showing the values typed into input boxes.

use serde_json::Value;

use crate::PortType;

/// Read what a person typed for an input of type `kind`.
///
/// Empty text means the node has no value of its own (`Ok(None)`), so the
/// input falls back to its default. Anything else must fit the type.
pub fn parse_input(text: &str, kind: PortType) -> Result<Option<Value>, String> {
    if text.trim().is_empty() {
        return Ok(None);
    }
    let trimmed = text.trim();
    match kind {
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
