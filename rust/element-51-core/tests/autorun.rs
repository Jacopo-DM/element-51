//! The rule for `Run automatically`: when a run starts and when it waits.

use element_51_core::autorun::{AutoRun, DEBOUNCE_MS, Reaction};

fn ticked() -> AutoRun {
    let mut auto = AutoRun::new();
    auto.set_ticked(true);
    auto
}

#[test]
fn the_box_starts_unticked_and_ignores_changes() {
    let mut auto = AutoRun::new();
    assert!(!auto.ticked());
    assert_eq!(auto.change(0.0), Reaction::Ignore);
    assert!(!auto.due(1000.0));
}

#[test]
fn a_change_waits_300_ms_and_then_runs_once() {
    let mut auto = ticked();
    assert_eq!(auto.change(0.0), Reaction::Wait(DEBOUNCE_MS));
    assert!(!auto.due(299.0));
    assert!(auto.due(300.0));
    assert!(!auto.due(600.0), "the change was used up by the run");
}

#[test]
fn three_changes_in_a_second_give_one_run() {
    let mut auto = ticked();
    auto.change(0.0);
    auto.change(100.0);
    auto.change(200.0);
    assert!(!auto.due(300.0), "the first timer fires too early");
    assert!(!auto.due(400.0), "the second timer fires too early");
    assert!(auto.due(500.0), "the last change has waited 300 ms");
    assert!(!auto.due(900.0));
}

#[test]
fn a_change_during_a_run_queues_exactly_one_more_run() {
    let mut auto = ticked();
    auto.change(0.0);
    assert!(auto.due(300.0));
    assert_eq!(auto.change(350.0), Reaction::Queued);
    assert_eq!(auto.change(360.0), Reaction::Queued);
    assert!(!auto.due(900.0), "nothing starts while a run is going");
    assert!(auto.finished(), "one fresh run starts when the run ends");
    assert!(!auto.finished(), "and only one");
}

#[test]
fn a_run_with_no_change_during_it_starts_nothing_more() {
    let mut auto = ticked();
    auto.change(0.0);
    assert!(auto.due(300.0));
    assert!(!auto.finished());
}

#[test]
fn the_run_button_counts_as_a_run_in_progress() {
    let mut auto = AutoRun::new();
    auto.started();
    auto.set_ticked(true);
    assert_eq!(auto.change(0.0), Reaction::Queued);
    assert!(auto.finished());
}

#[test]
fn unticking_drops_a_waiting_change_and_a_queued_run() {
    let mut waiting = ticked();
    waiting.change(0.0);
    waiting.set_ticked(false);
    assert!(!waiting.due(1000.0));

    let mut queued = ticked();
    queued.change(0.0);
    assert!(queued.due(300.0));
    queued.change(350.0);
    queued.set_ticked(false);
    assert!(!queued.finished());
}

#[test]
fn a_run_that_ends_after_unticking_starts_nothing() {
    let mut auto = ticked();
    auto.change(0.0);
    assert!(auto.due(300.0));
    auto.change(350.0);
    auto.set_ticked(false);
    auto.set_ticked(true);
    assert!(!auto.finished());
}

#[test]
fn ticking_starts_one_run_at_once() {
    let mut auto = ticked();
    assert!(auto.tick_run());
    assert!(!auto.tick_run(), "a run is already going");
    assert!(auto.finished(), "the second tick queued one more run");
    assert!(!auto.finished());
}

#[test]
fn ticking_during_a_run_queues_one_run() {
    let mut auto = AutoRun::new();
    auto.started();
    auto.set_ticked(true);
    assert!(!auto.tick_run());
    assert!(auto.finished());
}

#[test]
fn an_unticked_box_starts_nothing() {
    let mut auto = AutoRun::new();
    assert!(!auto.tick_run());
}
