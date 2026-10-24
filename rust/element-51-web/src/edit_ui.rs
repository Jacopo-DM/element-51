//! The edit flow: send an edit, apply it to the page's copy, undo and redo.

use crate::Shared;
use crate::State;
use crate::a11y_ui::announce;
use crate::autorun_ui::changed;
use crate::redraw;
use crate::run_ui::clear_run;
use crate::show_error;
use element_51_core::announce::{edit_text, refusal_text, undo_text};
use element_51_core::{Edit, EditRequest, Graph, Message};

pub(crate) fn send(state: &State, request: EditRequest) {
    let text = Message::Edit(request).to_json();
    if state.socket.send_with_str(&text).is_err() {
        show_error(
            &state.document,
            "Could not send the edit: not connected to Python.",
        );
    }
}

/// Apply an edit the user made, send it, and remember it so it can be undone.
pub(crate) fn submit(shared: &Shared, edit: Edit) {
    submit_edit(shared, edit, true);
}

/// Apply `edit` locally and send it. A refused edit shows its reason. With
/// `record` the edit joins the undo history; undo and redo pass false.
pub(crate) fn submit_edit(shared: &Shared, edit: Edit, record: bool) {
    let mut altered = false;
    {
        let mut state = shared.borrow_mut();
        let Some(replica) = state.replica.as_mut() else {
            return;
        };
        // Moving a node changes no result, so the shown values stay.
        let keeps_results = matches!(edit, Edit::MoveNode { .. });
        let before = replica.graph().clone();
        match replica.submit(edit.clone()) {
            Ok(request) => {
                if record {
                    state.history.record(&before, &edit);
                    announce(&state.document, &edit_text(&edit));
                }
                send(&state, request);
                state.tree_stale = true;
                if !keeps_results {
                    clear_run(&mut state);
                    altered = true;
                }
            }
            Err(error) => {
                show_error(&state.document, &error.to_string());
                announce(&state.document, &refusal_text(&error.to_string()));
            }
        }
    }
    if altered {
        changed(shared);
    }
    redraw(shared);
}

/// Put `edit`, which was already applied and sent without a record, in the undo
/// history as one step back to `before`, the graph it started from.
pub(crate) fn record_applied(shared: &Shared, before: &Graph, edit: &Edit) {
    let mut state = shared.borrow_mut();
    state.history.record(before, edit);
    announce(&state.document, &edit_text(edit));
}

/// Undo the newest step, or redo the newest undone one. The edits go to Python
/// like any others; the first one that no longer applies shows its reason and
/// ends the step.
pub(crate) fn step_history(shared: &Shared, undo: bool) {
    let edits = {
        let mut state = shared.borrow_mut();
        state.selected = None;
        // An undo in the middle of a drag ends that drag's record.
        state.slider_start = None;
        if undo {
            state.history.undo()
        } else {
            state.history.redo()
        }
    };
    let changed = edits.as_ref().is_some_and(|edits| !edits.is_empty());
    let mut interrupted = false;
    for edit in edits.unwrap_or_default() {
        let before = shared.borrow().replica.as_ref().map(|r| r.graph().clone());
        submit_edit(shared, edit, false);
        let after = shared.borrow().replica.as_ref().map(|r| r.graph().clone());
        if before == after {
            interrupted = true;
            break;
        }
    }
    // A refused step has already been announced with its reason.
    if !interrupted {
        announce(&shared.borrow().document, &undo_text(undo, changed));
    }
    redraw(shared);
}
