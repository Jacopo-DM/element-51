//! The panel that shows a node's source code, and the messages that ask for it.

use element_51_core::source::{SourcePanel, source_panel, source_request_text};
use element_51_core::{Message, SourceReply, SourceRequest};
use serde_json::Value;

const MESSAGES: &str = include_str!("../../../tests/fixtures/messages.json");

fn fixture() -> Value {
    serde_json::from_str(MESSAGES).unwrap()
}

#[test]
fn a_source_request_and_its_replies_round_trip_through_the_shared_fixture() {
    let fixture = fixture();
    for key in ["source", "source_reply", "source_reply_unreadable"] {
        let message = Message::from_json(&fixture[key].to_string()).unwrap();
        assert_eq!(
            serde_json::to_value(&message).unwrap(),
            fixture[key],
            "{key}"
        );
    }
    let Message::Source(request) = Message::from_json(&fixture["source"].to_string()).unwrap()
    else {
        panic!("not a source request");
    };
    assert_eq!(request.node, "n2");
    let Message::SourceReply(reply) =
        Message::from_json(&fixture["source_reply_unreadable"].to_string()).unwrap()
    else {
        panic!("not a reply");
    };
    assert_eq!(reply.source, None);
    assert!(reply.reason.unwrap().contains("cannot be read"));
}

#[test]
fn a_request_goes_out_as_json_with_its_op() {
    let text = source_request_text("n4");
    let value: Value = serde_json::from_str(&text).unwrap();
    assert_eq!(value, serde_json::json!({"op": "source", "node": "n4"}));
    assert_eq!(
        Message::from_json(&text).unwrap(),
        Message::Source(SourceRequest {
            node: "n4".to_string()
        })
    );
}

fn reply(source: Option<&str>, reason: Option<&str>) -> SourceReply {
    SourceReply {
        node: "n2".to_string(),
        source: source.map(str::to_string),
        reason: reason.map(str::to_string),
    }
}

#[test]
fn code_gets_a_title_the_code_itself_and_a_sentence_with_its_length() {
    let code = "@node\ndef add(a: float) -> float:\n    return a\n";
    let panel = source_panel(&reply(Some(code), None), "add");
    assert_eq!(
        panel,
        SourcePanel {
            title: "Source of n2 (add)".to_string(),
            body: code.to_string(),
            announcement: "Source of n2, add, 3 lines. Press Escape to close".to_string(),
        }
    );
}

#[test]
fn one_line_is_not_plural() {
    let panel = source_panel(&reply(Some("x = 1"), None), "t");
    assert!(
        panel.announcement.contains("1 line."),
        "{}",
        panel.announcement
    );
}

#[test]
fn a_reason_is_shown_in_place_of_the_code_and_said() {
    let panel = source_panel(&reply(None, Some("The source cannot be read")), "made");
    assert_eq!(panel.title, "Source of n2 (made)");
    assert_eq!(panel.body, "The source cannot be read");
    assert_eq!(
        panel.announcement,
        "No source for n2: The source cannot be read. Press Escape to close"
    );
}

#[test]
fn a_reply_with_neither_still_says_something() {
    let panel = source_panel(&reply(None, None), "t");
    assert_eq!(panel.body, "Python sent no source");
    assert!(panel.announcement.starts_with("No source for n2"));
}
