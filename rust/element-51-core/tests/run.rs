//! Reading what Python reports about a run.

use std::collections::BTreeMap;

use element_51_core::{Message, NodeOutcome, RunResult, Status};
use serde_json::{Value, json};

const MESSAGES: &str = include_str!("../../../tests/fixtures/messages.json");

fn fixture_result() -> RunResult {
    let messages: serde_json::Value = serde_json::from_str(MESSAGES).unwrap();
    let Message::RunResult(result) =
        Message::from_json(&messages["run_result"].to_string()).unwrap()
    else {
        panic!("not a run result");
    };
    result
}

#[test]
fn the_fixture_result_has_ok_error_skipped_and_a_multi_output_node() {
    let result = fixture_result();
    let statuses: Vec<Status> = result.outcomes.iter().map(|o| o.status).collect();
    assert_eq!(
        statuses,
        [Status::Ok, Status::Error, Status::Skipped, Status::Ok]
    );
    let multi = result.outcome_for("n4").unwrap();
    assert_eq!(multi.outputs.len(), 2);
    assert_eq!(multi.outputs["q"], json!(3));
}

#[test]
fn run_request_is_a_bare_op() {
    let messages: serde_json::Value = serde_json::from_str(MESSAGES).unwrap();
    let message = Message::from_json(&messages["run"].to_string()).unwrap();
    assert!(matches!(message, Message::Run(_)));
    assert_eq!(message.to_json(), r#"{"op":"run"}"#);
}

#[test]
fn error_summary_lists_only_the_errors_with_their_node() {
    let summary = fixture_result().error_summary();
    assert_eq!(
        summary.as_deref(),
        Some("n2: ZeroDivisionError: float division by zero")
    );
}

#[test]
fn error_summary_joins_several_errors_and_is_none_without_any() {
    let mut result = fixture_result();
    result.outcomes.push(NodeOutcome {
        node: "n4".to_string(),
        status: Status::Error,
        outputs: BTreeMap::new(),
        shown: BTreeMap::new(),
        error: Some("input 'a' has no value".to_string()),
    });
    assert_eq!(
        result.error_summary().as_deref(),
        Some("n2: ZeroDivisionError: float division by zero; n4: input 'a' has no value")
    );
    result
        .outcomes
        .retain(|outcome| outcome.status != Status::Error);
    assert_eq!(result.error_summary(), None);
}

#[test]
fn outcome_for_finds_a_node_by_id() {
    let result = fixture_result();
    assert_eq!(result.outcome_for("n3").unwrap().status, Status::Skipped);
    assert!(result.outcome_for("n9").is_none());
}

fn ok_with(outputs: &[(&str, Value)]) -> NodeOutcome {
    NodeOutcome {
        node: "n1".to_string(),
        status: Status::Ok,
        outputs: outputs
            .iter()
            .map(|(port, value)| (port.to_string(), value.clone()))
            .collect(),
        shown: BTreeMap::new(),
        error: None,
    }
}

#[test]
fn output_text_shows_numbers_strings_and_bools_as_json() {
    let outcome = ok_with(&[("a", json!(3.0)), ("b", json!("HI")), ("c", json!(true))]);
    assert_eq!(outcome.output_text("a").as_deref(), Some("3.0"));
    assert_eq!(outcome.output_text("b").as_deref(), Some("\"HI\""));
    assert_eq!(outcome.output_text("c").as_deref(), Some("true"));
}

#[test]
fn output_text_is_none_for_a_port_without_a_value() {
    let outcome = ok_with(&[("q", json!(3))]);
    assert_eq!(outcome.output_text("r"), None);
    let none = NodeOutcome {
        outputs: BTreeMap::new(),
        ..outcome
    };
    assert_eq!(none.output_text("q"), None);
}
