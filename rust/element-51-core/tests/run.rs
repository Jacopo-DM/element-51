//! Reading what Python reports about a run.

use element_51_core::{Message, NodeOutcome, RunResult, Status};
use serde_json::json;

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
fn the_fixture_result_has_an_ok_an_error_and_a_skipped_node() {
    let result = fixture_result();
    let statuses: Vec<Status> = result.outcomes.iter().map(|o| o.status).collect();
    assert_eq!(statuses, [Status::Ok, Status::Error, Status::Skipped]);
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
        value: None,
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

#[test]
fn value_text_shows_numbers_strings_and_bools_as_json() {
    let outcome = |value| NodeOutcome {
        node: "n1".to_string(),
        status: Status::Ok,
        value,
        error: None,
    };
    assert_eq!(
        outcome(Some(json!(3.0))).value_text().as_deref(),
        Some("3.0")
    );
    assert_eq!(
        outcome(Some(json!("HI"))).value_text().as_deref(),
        Some("\"HI\"")
    );
    assert_eq!(
        outcome(Some(json!(true))).value_text().as_deref(),
        Some("true")
    );
    assert_eq!(outcome(None).value_text(), None);
}
