//! The wire messages, read from the file the Python tests also use.

use element_51_core::{Edit, JobStatus, Message};
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

#[test]
fn job_requests_and_a_refusal_carry_their_node() {
    let fixture = fixture();
    let start = Message::from_json(&fixture["start"].to_string()).unwrap();
    assert!(matches!(start, Message::Start(ref s) if s.node == "n1"));
    let stop = Message::from_json(&fixture["stop"].to_string()).unwrap();
    assert!(matches!(stop, Message::Stop(ref s) if s.node == "n1"));
    let refused = Message::from_json(&fixture["job_refused"].to_string()).unwrap();
    let Message::JobRefused(refusal) = refused else {
        panic!("not a refusal")
    };
    assert_eq!(
        (refusal.node.as_str(), refusal.reason.as_str()),
        ("n1", "Node n1 is already running")
    );
}

#[test]
fn a_job_update_carries_progress_outputs_and_the_series_it_feeds_a_chart() {
    let fixture = fixture();
    let running = Message::from_json(&fixture["job_update"].to_string()).unwrap();
    let Message::JobUpdate(update) = running else {
        panic!("not a job update")
    };
    assert_eq!(update.state, JobStatus::Running);
    assert_eq!((update.step, update.steps), (2, Some(5)));
    assert_eq!(update.outputs["total"], 11);
    assert_eq!(update.linked.len(), 1);
    assert_eq!(update.linked[0].node, "n3");
    assert_eq!(
        update.linked[0].series.values,
        [Some(0.9), None, Some(0.25)]
    );
    let failed = Message::from_json(&fixture["job_update_failed"].to_string()).unwrap();
    let Message::JobUpdate(update) = failed else {
        panic!("not a job update")
    };
    assert_eq!(update.state, JobStatus::Failed);
    assert_eq!(update.steps, None);
    assert_eq!(update.error.as_deref(), Some("RuntimeError: boom"));
}

#[test]
fn a_snapshot_carries_the_jobs_and_one_without_them_still_reads() {
    let fixture = fixture();
    // A job in a snapshot is the update without its `op`, which only tags a message.
    let mut job = fixture["job_update"].clone();
    job.as_object_mut().unwrap().remove("op");
    let update = job.to_string();
    let snapshot = format!(
        r#"{{"op": "snapshot", "version": 2, "theme": "antimony", "node_types": [],
            "graph": {{"nodes": [], "links": []}}, "jobs": [{update}]}}"#
    );
    let Message::Snapshot(with_jobs) = Message::from_json(&snapshot).unwrap() else {
        panic!("not a snapshot")
    };
    assert_eq!(with_jobs.jobs.len(), 1);
    assert_eq!(with_jobs.jobs[0].node, "n1");
    let bare = r#"{"op": "snapshot", "version": 2, "theme": "antimony", "node_types": [],
        "graph": {"nodes": [], "links": []}}"#;
    let Message::Snapshot(without) = Message::from_json(bare).unwrap() else {
        panic!("not a snapshot")
    };
    assert!(without.jobs.is_empty());
}

#[test]
fn a_partial_result_is_marked_and_merging_replaces_in_place_and_adds_the_rest() {
    let fixture = fixture();
    let whole = Message::from_json(&fixture["run_result"].to_string()).unwrap();
    let Message::RunResult(mut last) = whole else {
        panic!("not a run result")
    };
    assert!(!last.partial);
    let partial = Message::from_json(&fixture["run_result_partial"].to_string()).unwrap();
    let Message::RunResult(after) = partial else {
        panic!("not a run result")
    };
    assert!(after.partial);
    let before: Vec<String> = last.outcomes.iter().map(|o| o.node.clone()).collect();
    last.merge(&after);
    let nodes: Vec<String> = last.outcomes.iter().map(|o| o.node.clone()).collect();
    assert_eq!(
        nodes, before,
        "n2 was already there, so it is replaced in place"
    );
    assert_eq!(
        last.outcome_for("n2").unwrap().outputs["out"],
        serde_json::json!(12)
    );
    let mut empty = last.clone();
    empty.outcomes.clear();
    empty.merge(&after);
    assert_eq!(
        empty.outcomes.len(),
        1,
        "a node that was not there is added"
    );
}
