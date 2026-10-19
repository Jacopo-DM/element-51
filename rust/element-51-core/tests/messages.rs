//! The wire messages, read from the file the Python tests also use.

use element_51_core::{Edit, Message};
use serde_json::Value;

const MESSAGES: &str = include_str!("../../../tests/fixtures/messages.json");

fn fixture() -> Value {
    serde_json::from_str(MESSAGES).unwrap()
}

#[test]
fn every_message_round_trips_unchanged() {
    let fixture = fixture();
    for (name, original) in fixture.as_object().unwrap() {
        let message = Message::from_json(&original.to_string()).unwrap();
        let again: Value = serde_json::to_value(&message).unwrap();
        assert_eq!(&again, original, "{name}");
    }
}

#[test]
fn edit_requests_carry_their_edit() {
    let fixture = fixture();
    let add_node = Message::from_json(&fixture["edit_add_node"].to_string()).unwrap();
    let Message::Edit(request) = add_node else {
        panic!("not an edit")
    };
    assert_eq!(request.id, 17);
    assert!(matches!(request.edit, Edit::AddNode { ref id, .. } if id == "n3"));
    let add_link = Message::from_json(&fixture["edit_add_link"].to_string()).unwrap();
    let Message::Edit(request) = add_link else {
        panic!("not an edit")
    };
    assert!(matches!(request.edit, Edit::AddLink { .. }));
}

#[test]
fn replies_carry_the_request_id() {
    let fixture = fixture();
    let ack = Message::from_json(&fixture["ack"].to_string()).unwrap();
    assert!(matches!(ack, Message::Ack(ref a) if a.id == 17));
    let reject = Message::from_json(&fixture["reject"].to_string()).unwrap();
    let Message::Reject(reject) = reject else {
        panic!("not a reject")
    };
    assert_eq!(
        (reject.id, reject.reason.as_str()),
        (18, "cannot link float to str")
    );
}

#[test]
fn unknown_edit_kind_is_refused() {
    let broken = r#"{"op": "edit", "id": 1, "edit": {"kind": "explode"}}"#;
    assert!(Message::from_json(broken).is_err());
}

#[test]
fn to_json_writes_what_from_json_reads() {
    let fixture = fixture();
    let message = Message::from_json(&fixture["edit_add_link"].to_string()).unwrap();
    assert_eq!(Message::from_json(&message.to_json()).unwrap(), message);
}
