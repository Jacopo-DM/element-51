//! The Run button and the jobs: what a run or a job report puts on screen.

use crate::Shared;
use crate::State;
use crate::a11y_ui::announce;
use crate::pictures::Pictures;
use crate::show_error;
use element_51_core::access::job_announcement;
use element_51_core::job::JobButton;
use element_51_core::{JobStatus, JobUpdate, Message, RunRequest, StartRequest, StopRequest};
use web_sys::Document;

/// Forget the last run, then show what the jobs have reported: a job's results
/// are not the result of a run, so an edit must not lose them.
pub(crate) fn clear_run(state: &mut State) {
    state.run = None;
    show_jobs(state);
}

/// Put every job's reported outputs into the run result, and draw its pictures.
pub(crate) fn show_jobs(state: &mut State) {
    if state.jobs.is_empty() {
        return;
    }
    let mut run = state.run.take().unwrap_or_default();
    for update in state.jobs.values() {
        run.apply_job(update);
    }
    state.pictures = Pictures::from_result(&state.document, &run);
    state.run = Some(run);
}

/// A job changed: keep it, show what it reported, and tell the person when its
/// state changes. A failed job also shows its error in the banner.
pub(crate) fn on_job_update(state: &mut State, update: JobUpdate) {
    let before = state.jobs.get(&update.node).map(|job| job.state);
    let changed = before != Some(update.state);
    if changed {
        announce(&state.document, &job_announcement(&update));
    }
    if update.state == JobStatus::Failed && changed {
        let why = update.error.as_deref().unwrap_or("failed");
        show_error(&state.document, &format!("{}: {why}", update.node));
    }
    // The update joins the last result; it must not replace it, or the run after
    // another job, or a plain run, would be lost with every report.
    let mut run = state.run.take().unwrap_or_default();
    run.apply_job(&update);
    state.pictures = Pictures::from_result(&state.document, &run);
    state.run = Some(run);
    state.jobs.insert(update.node.clone(), update);
    if changed {
        state.tree_stale = true;
    }
}

/// Ask Python to start or stop the job of node `id`.
pub(crate) fn send_job_request(state: &State, id: &str, button: JobButton) {
    let node = id.to_string();
    let message = match button {
        JobButton::Start => Message::Start(StartRequest { node }),
        JobButton::Stop => Message::Stop(StopRequest { node }),
    };
    if state.socket.send_with_str(&message.to_json()).is_err() {
        show_error(
            &state.document,
            "Could not send the request: not connected to Python.",
        );
    }
}

/// Press Run: send the request and show that a run is in progress until its
/// result comes. A press while a run is in progress does nothing.
pub(crate) fn on_run(shared: &Shared) {
    let state = shared.borrow();
    let busy = state
        .document
        .get_element_by_id("run")
        .is_some_and(|button| button.get_attribute("aria-busy").as_deref() == Some("true"));
    if busy {
        return;
    }
    if state
        .socket
        .send_with_str(&Message::Run(RunRequest {}).to_json())
        .is_err()
    {
        show_error(
            &state.document,
            "Could not send the run request: not connected to Python.",
        );
    } else {
        set_running(&state.document, true);
    }
}

/// Show the Run button as busy, `Running`, or as it was. It stays focusable
/// (`aria-busy`, not `disabled`), so a keyboard user keeps their place.
pub(crate) fn set_running(document: &Document, running: bool) {
    let Some(button) = document.get_element_by_id("run") else {
        return;
    };
    if running {
        let _ = button.set_attribute("aria-busy", "true");
        let _ = button.set_attribute("aria-disabled", "true");
        button.set_text_content(Some("Running"));
    } else {
        let _ = button.remove_attribute("aria-busy");
        let _ = button.remove_attribute("aria-disabled");
        button.set_text_content(Some("Run"));
    }
}
