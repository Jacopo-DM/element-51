//! What Python sends to the page: snapshots, acknowledgements, results and job updates.

use crate::Shared;
use crate::a11y_ui::announce;
use crate::apply_theme;
use crate::autorun_ui::changed;
use crate::hide_error;
use crate::mark_ready;
use crate::menu_ui::fill_menu;
use crate::pictures::Pictures;
use crate::redraw;
use crate::run_ui::{clear_run, on_job_update, set_running, start_run};
use crate::show_error;
use crate::source_ui::show_source;
use element_51_core::announce::{refusal_text, run_text};
use element_51_core::theme::Theme;
use element_51_core::{Message, Replica};
use web_sys::MessageEvent;

pub(crate) fn on_message(shared: &Shared, event: &MessageEvent) {
    let parsed = event
        .data()
        .as_string()
        .ok_or_else(|| "Python sent a message that is not text".to_string())
        .and_then(|text| Message::from_json(&text).map_err(|error| error.to_string()));
    let mut source_reply = None;
    let mut graph_changed = false;
    {
        let mut state = shared.borrow_mut();
        match parsed {
            Ok(Message::Snapshot(snapshot)) => {
                state.jobs = snapshot
                    .jobs
                    .iter()
                    .map(|job| (job.node.clone(), job.clone()))
                    .collect();
                clear_run(&mut state);
                graph_changed = true;
                state.selected = None;
                state.tree_stale = true;
                let theme_key = format!("{} {}", snapshot.theme, snapshot.theme_changes);
                if theme_key != state.theme_key {
                    match Theme::with_changes(&snapshot.theme, &snapshot.theme_changes) {
                        Ok(theme) => {
                            apply_theme(&state.document, &theme);
                            state.theme = theme;
                            state.theme_key = theme_key;
                        }
                        Err(reason) => show_error(
                            &state.document,
                            &format!("The theme cannot be used: {reason}"),
                        ),
                    }
                }
                mark_ready(&state.document);
                fill_menu(&state.document, &snapshot);
                match state.replica.as_mut() {
                    Some(replica) => replica.reset(snapshot),
                    None => state.replica = Some(Replica::new(snapshot)),
                }
            }
            Ok(Message::Ack(ack)) => {
                if let Some(replica) = state.replica.as_mut() {
                    replica.ack(ack.id);
                }
            }
            Ok(Message::Reject(reject)) => {
                let known = state
                    .replica
                    .as_mut()
                    .is_some_and(|replica| replica.reject(reject.id));
                if known {
                    state.tree_stale = true;
                    clear_run(&mut state);
                    graph_changed = true;
                    state.run_failed = false;
                    show_error(
                        &state.document,
                        &format!("Python refused the edit: {}", reject.reason),
                    );
                    announce(&state.document, &refusal_text(&reject.reason));
                }
            }
            Ok(Message::RunResult(result)) => {
                if !result.partial {
                    set_running(&state.document, false);
                }
                if let Some(summary) = result.error_summary() {
                    show_error(&state.document, &summary);
                    state.run_failed = true;
                } else if !result.partial && state.run_failed {
                    // A passing run clears the banner a failing run left.
                    hide_error(&state.document);
                    state.run_failed = false;
                }
                let run_ended = !result.partial;
                // The run after a job follows the job's own announcement; it is only
                // spoken when something in it failed.
                if !result.partial || result.error_summary().is_some() {
                    announce(&state.document, &run_text(&result));
                }
                // A partial result covers the nodes after a job that ended; it
                // joins the last result instead of replacing it.
                let whole = match (result.partial, state.run.take()) {
                    (true, Some(mut run)) => {
                        run.merge(&result);
                        run
                    }
                    _ => result,
                };
                state.pictures = Pictures::from_result(&state.document, &whole);
                state.tree_stale = true;
                state.run = Some(whole);
                // A change that came in during the run starts one more.
                if run_ended && state.auto.finished() {
                    start_run(&state);
                }
            }
            Ok(Message::SourceReply(reply)) => source_reply = Some(reply),
            Ok(Message::JobUpdate(update)) => on_job_update(&mut state, update),
            Ok(Message::JobRefused(refusal)) => {
                show_error(&state.document, &refusal.reason);
                announce(&state.document, &refusal_text(&refusal.reason));
            }
            Ok(
                Message::Edit(_)
                | Message::Run(_)
                | Message::Source(_)
                | Message::Start(_)
                | Message::Stop(_),
            ) => {
                show_error(
                    &state.document,
                    "Python sent a message only the browser should send.",
                );
            }
            Err(error) => show_error(&state.document, &error),
        }
    }
    if graph_changed {
        changed(shared);
    }
    redraw(shared);
    // The panel moves the focus, which runs handlers that borrow the state, so
    // it is shown once the borrow above has ended.
    if let Some(reply) = source_reply {
        show_source(shared, &reply);
    }
}
