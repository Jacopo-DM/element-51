//! The sentences the status line and the screen reader are given.

use crate::access::Direction;
use crate::job::{JobButton, can_start, can_stop, progress_text, state_text};
use crate::layout::Side;
use crate::{Edit, JobStatus, JobUpdate, PortRef, RunResult, Status};

pub(crate) fn plural(count: usize, word: &str) -> String {
    if count == 1 {
        word.to_string()
    } else {
        format!("{word}s")
    }
}

pub(crate) fn end_name(end: &PortRef) -> String {
    format!("{} {}", end.node, end.port)
}

/// `a`, `a and b`, `a, b and c`.
pub(crate) fn join_names(names: &[String]) -> String {
    match names {
        [] => String::new(),
        [only] => only.clone(),
        [first @ .., last] => format!("{} and {last}", first.join(", ")),
    }
}

/// The sentence for the status line after `edit` was applied.
pub fn edit_text(edit: &Edit) -> String {
    match edit {
        Edit::AddNode { id, node_type, .. } => format!("Added node {id}, {node_type}"),
        Edit::AddLink { source, target } => {
            format!("Linked {} to {}", end_name(source), end_name(target))
        }
        Edit::MoveNode { id, .. } => format!("Moved {id}"),
        Edit::ResizeNode { id, .. } => format!("Resized {id}"),
        Edit::DeleteNode { id } => format!("Deleted node {id}"),
        Edit::DeleteLink { source, target } => format!(
            "Deleted the link from {} to {}",
            end_name(source),
            end_name(target)
        ),
        Edit::SetValue {
            node,
            port,
            value: Some(value),
        } => {
            let shown = value
                .as_str()
                .map_or_else(|| value.to_string(), str::to_string);
            format!("Set {port} of {node} to {shown}")
        }
        Edit::SetValue {
            node,
            port,
            value: None,
        } => format!("Cleared the value of {port} of {node}"),
    }
}

/// The sentence after undo (`undo` true) or redo, whether or not there was a
/// step to take.
pub fn undo_text(undo: bool, changed: bool) -> String {
    match (undo, changed) {
        (true, true) => "Undid the last change",
        (false, true) => "Redid the last change",
        (true, false) => "Nothing to undo",
        (false, false) => "Nothing to redo",
    }
    .to_string()
}

/// The sentence when Python or the rules refused something.
pub fn refusal_text(reason: &str) -> String {
    format!("Refused: {reason}")
}

/// The sentence when node `id` was selected from the keyboard.
pub fn selected_text(id: &str) -> String {
    format!("Selected {id}")
}

/// What a screen reader is told when a job's state changes: `n1: running.`,
/// `n1: finished, step 12 of 12.`, `n1: failed: boom.`.
pub fn job_announcement(update: &JobUpdate) -> String {
    let state = state_text(Some(update));
    let mut text = format!("{}: {state}", update.node);
    match (update.state, update.error.as_deref()) {
        (JobStatus::Failed, why) => text.push_str(&format!(": {}", why.unwrap_or("failed"))),
        (JobStatus::Finished | JobStatus::Stopped, _) => {
            if let Some(progress) = progress_text(Some(update)) {
                text.push_str(&format!(", {progress}"));
            }
        }
        _ => {}
    }
    text.push('.');
    text
}

/// The label of a tree item for a job button: its name, and why it does not
/// work now if that is so.
pub fn job_button_label(button: JobButton, update: Option<&JobUpdate>) -> String {
    let name = match button {
        JobButton::Start => "Start the job",
        JobButton::Stop => "Stop the job",
    };
    match job_button_refusal(button, update) {
        Some(why) => format!("{name}, button. Not available: {why}"),
        None => format!("{name}, button."),
    }
}

/// Why a job button does nothing now, or None if it works.
pub fn job_button_refusal(button: JobButton, update: Option<&JobUpdate>) -> Option<String> {
    match button {
        JobButton::Start if !can_start(update) => {
            Some(format!("the job is {}.", state_text(update)))
        }
        JobButton::Stop if !can_stop(update) => Some(match update.map(|update| update.state) {
            Some(JobStatus::Stopping) => "the job is already stopping.".to_string(),
            _ => "no job is running.".to_string(),
        }),
        _ => None,
    }
}

/// Why Delete did nothing on a port of this side.
pub fn cannot_delete_text(side: Side) -> String {
    match side {
        Side::Output => "Delete works on a node, or on an input port with a link",
        Side::Input => "That input has no link",
    }
    .to_string()
}

/// The sentence when a keyboard link was started from `source`.
pub fn link_started_text(source: &PortRef) -> String {
    format!(
        "Link from {} started. Move to an input port and press Enter, or Escape to cancel",
        end_name(source)
    )
}

/// The sentence when a started link was cancelled.
pub fn link_cancelled_text() -> String {
    "Link cancelled".to_string()
}

/// The sentence when Enter on an unlinked input moved the focus into its value
/// box.
pub fn editing_text(node: &str, port: &str) -> String {
    format!("Type a value for {port} of {node}, then press Enter. Escape leaves it unchanged")
}

/// The sentence when Enter on a linked input finds no value box to edit.
pub fn linked_input_text() -> String {
    "That input is linked, so it has no value box. Press Delete to remove the link".to_string()
}

/// The sentence when Enter on an unlinked picture input finds no value box:
/// a picture has no typed value, so it can only be linked.
pub fn picture_input_text() -> String {
    "That input takes a picture, which can only be linked. Start a link from a picture output"
        .to_string()
}

/// The sentence after the view panned.
pub fn pan_text(direction: Direction) -> String {
    let word = match direction {
        Direction::Left => "left",
        Direction::Right => "right",
        Direction::Up => "up",
        Direction::Down => "down",
    };
    format!("Panned {word}")
}

/// The sentence after the zoom changed to `zoom` (1.0 is 100 percent).
pub fn zoom_text(zoom: f64) -> String {
    format!("Zoom {} percent", (zoom * 100.0).round())
}

/// The sentence when an edit came in during a run and one more run is queued.
pub fn queued_run_text() -> &'static str {
    "Still running, will run again"
}

/// The sentence when a run ended: how many nodes were ok, failed and were
/// skipped, and the first error.
pub fn run_text(result: &RunResult) -> String {
    let count = |wanted: Status| {
        result
            .outcomes
            .iter()
            .filter(|outcome| outcome.status == wanted)
            .count()
    };
    let (ok, errors, skipped) = (
        count(Status::Ok),
        count(Status::Error),
        count(Status::Skipped),
    );
    let mut parts = Vec::new();
    if ok > 0 {
        parts.push(format!("{ok} ok"));
    }
    if errors > 0 {
        parts.push(format!("{errors} {}", plural(errors, "error")));
    }
    if skipped > 0 {
        parts.push(format!("{skipped} skipped"));
    }
    if parts.is_empty() {
        return "Run finished: no nodes".to_string();
    }
    let lead = if result.partial {
        "After the job"
    } else {
        "Run finished"
    };
    let mut text = format!("{lead}: {}", parts.join(", "));
    if let Some(first) = result
        .outcomes
        .iter()
        .find(|outcome| outcome.status == Status::Error)
    {
        let why = first.error.as_deref().unwrap_or("failed");
        text.push_str(&format!(". First error: {}: {why}", first.node));
    }
    text
}
