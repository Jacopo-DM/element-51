//! The job row of a card: its size, its buttons, the words, and how a job's
//! reports reach a run result.

use std::collections::BTreeMap;

use element_51_core::access::{NodeItem, job_text, with_jobs};
use element_51_core::announce::{job_announcement, job_button_label, job_button_refusal, run_text};
use element_51_core::display::{chart_rect, picture_rect};
use element_51_core::job::{
    JobButton, can_start, can_stop, job_button_at, job_height, job_layout, progress_fraction,
    progress_text, state_text,
};
use element_51_core::layout::{Rect, node_rect};
use element_51_core::theme::{Metrics, Theme};
use element_51_core::{
    CardDisplay, Graph, JobStatus, JobUpdate, LinkedSeries, NodeOutcome, NodeType, PlacedNode,
    Port, PortType, RunResult, Series, Shown, Status,
};
use serde_json::json;

fn m() -> Metrics {
    Theme::default().metrics
}

fn port(name: &str, kind: PortType) -> Port {
    Port {
        name: name.to_string(),
        kind,
    }
}

fn kind(name: &str, job: bool, display: CardDisplay) -> NodeType {
    NodeType {
        job,
        name: name.to_string(),
        inputs: vec![port("epochs", PortType::Int)],
        outputs: vec![port("loss", PortType::Series)],
        defaults: Default::default(),
        category: String::new(),
        display,
        variadic: Default::default(),
    }
}

fn placed(id: &str, node_type: &str, x: f64, y: f64) -> PlacedNode {
    PlacedNode {
        id: id.to_string(),
        node_type: node_type.to_string(),
        x,
        y,
        values: Default::default(),
        width: None,
        display_height: None,
    }
}

fn update(node: &str, state: JobStatus, step: u32, steps: Option<u32>) -> JobUpdate {
    JobUpdate {
        node: node.to_string(),
        state,
        step,
        steps,
        error: None,
        outputs: BTreeMap::new(),
        shown: BTreeMap::new(),
        linked: vec![],
    }
}

fn series(values: &[f64]) -> Series {
    Series {
        values: values.iter().map(|v| Some(*v)).collect(),
    }
}

#[test]
fn a_job_card_is_two_rows_taller_and_a_plain_card_is_not() {
    let metrics = m();
    let plain = kind("plain", false, CardDisplay::None);
    let job = kind("train", true, CardDisplay::None);
    assert_eq!(job_height(&plain, &metrics), 0.0);
    assert_eq!(job_height(&job, &metrics), 2.0 * metrics.row_height);
    let node = placed("n1", "train", 0.0, 0.0);
    let (a, b) = (
        node_rect(&node, &plain, &metrics),
        node_rect(&node, &job, &metrics),
    );
    assert_eq!(b.h - a.h, 2.0 * metrics.row_height);
}

#[test]
fn a_picture_or_a_chart_sits_below_the_job_row() {
    let metrics = m();
    let node = placed("n1", "train", 0.0, 0.0);
    for (display, area) in [
        (
            CardDisplay::Image,
            picture_rect as fn(&Rect, &NodeType, &Metrics) -> Option<Rect>,
        ),
        (CardDisplay::Chart, chart_rect),
    ] {
        let job = kind("train", true, display);
        let plain = kind("plain", false, display);
        let card = node_rect(&node, &job, &metrics);
        let with = area(&card, &job, &metrics).unwrap();
        let without = area(&card, &plain, &metrics).unwrap();
        assert_eq!(with.y - without.y, 2.0 * metrics.row_height);
        assert!(with.y + with.h <= card.y + card.h);
    }
}

#[test]
fn the_buttons_and_the_bar_sit_inside_the_card_below_the_ports() {
    let metrics = m();
    let job = kind("train", true, CardDisplay::None);
    let card = node_rect(&placed("n1", "train", 50.0, 70.0), &job, &metrics);
    let layout = job_layout(&card, &job, &metrics).unwrap();
    let ports_end = card.y + metrics.header_height + 2.0 * metrics.row_height;
    for rect in [layout.start, layout.stop, layout.bar] {
        assert!(rect.x >= card.x && rect.x + rect.w <= card.x + card.w);
        assert!(rect.y >= ports_end && rect.y + rect.h <= card.y + card.h);
    }
    assert!(
        layout.stop.x >= layout.start.x + layout.start.w,
        "Stop is right of Start"
    );
    assert!(
        layout.bar.y >= layout.start.y + layout.start.h,
        "the bar is on the second row"
    );
    assert!(layout.state_right <= card.x + card.w);
    assert!(layout.state_baseline < layout.step_baseline);
    assert!(job_layout(&card, &kind("plain", false, CardDisplay::None), &metrics).is_none());
}

#[test]
fn the_words_and_the_bar_follow_the_job() {
    assert_eq!(state_text(None), "idle");
    for (state, word) in [
        (JobStatus::Running, "running"),
        (JobStatus::Stopping, "stopping"),
        (JobStatus::Finished, "finished"),
        (JobStatus::Stopped, "stopped"),
        (JobStatus::Failed, "failed"),
    ] {
        assert_eq!(state_text(Some(&update("n1", state, 0, None))), word);
    }
    let stepping = update("n1", JobStatus::Running, 3, Some(5));
    assert_eq!(
        progress_text(Some(&stepping)).as_deref(),
        Some("step 3 of 5")
    );
    assert_eq!(progress_fraction(Some(&stepping)), Some(0.6));
    let working = update("n1", JobStatus::Running, 0, None);
    assert_eq!(progress_text(Some(&working)), None);
    assert_eq!(progress_fraction(Some(&working)), None);
    assert_eq!(progress_text(None), None);
    assert_eq!(progress_fraction(None), None);
    let empty = update("n1", JobStatus::Finished, 0, Some(0));
    assert_eq!(progress_fraction(Some(&empty)), Some(1.0));
    let over = update("n1", JobStatus::Running, 9, Some(5));
    assert_eq!(progress_fraction(Some(&over)), Some(1.0));
}

#[test]
fn start_works_unless_a_job_runs_and_stop_only_while_it_runs() {
    assert!(can_start(None));
    assert!(!can_stop(None));
    for (state, start, stop) in [
        (JobStatus::Running, false, true),
        (JobStatus::Stopping, false, false),
        (JobStatus::Finished, true, false),
        (JobStatus::Stopped, true, false),
        (JobStatus::Failed, true, false),
    ] {
        let one = update("n1", state, 0, None);
        assert_eq!(can_start(Some(&one)), start, "{state:?}");
        assert_eq!(can_stop(Some(&one)), stop, "{state:?}");
    }
}

#[test]
fn a_press_finds_the_button_that_works_in_the_state_the_job_is_in() {
    let metrics = m();
    let types = vec![
        kind("train", true, CardDisplay::None),
        kind("plain", false, CardDisplay::None),
    ];
    let graph = Graph {
        nodes: vec![
            placed("n1", "train", 0.0, 0.0),
            placed("n2", "plain", 400.0, 0.0),
        ],
        links: vec![],
    };
    let card = node_rect(&graph.nodes[0], &types[0], &metrics);
    let layout = job_layout(&card, &types[0], &metrics).unwrap();
    let centre = |r: &Rect| (r.x + r.w / 2.0, r.y + r.h / 2.0);
    let mut jobs = BTreeMap::new();
    let found =
        |jobs: &BTreeMap<String, JobUpdate>, at| job_button_at(&graph, &types, jobs, at, &metrics);
    assert_eq!(
        found(&jobs, centre(&layout.start)),
        Some(("n1".to_string(), JobButton::Start))
    );
    assert_eq!(
        found(&jobs, centre(&layout.stop)),
        None,
        "nothing runs, so no Stop"
    );
    jobs.insert(
        "n1".to_string(),
        update("n1", JobStatus::Running, 1, Some(3)),
    );
    assert_eq!(
        found(&jobs, centre(&layout.start)),
        None,
        "running, so no Start"
    );
    assert_eq!(
        found(&jobs, centre(&layout.stop)),
        Some(("n1".to_string(), JobButton::Stop))
    );
    jobs.insert(
        "n1".to_string(),
        update("n1", JobStatus::Stopping, 1, Some(3)),
    );
    assert_eq!(found(&jobs, centre(&layout.stop)), None);
    assert_eq!(
        found(&jobs, (card.x + 1.0, card.y + 1.0)),
        None,
        "the header is no button"
    );
    assert_eq!(
        found(&jobs, (-500.0, -500.0)),
        None,
        "nothing under the point"
    );
    let plain = node_rect(&graph.nodes[1], &types[1], &metrics);
    assert_eq!(
        found(&jobs, centre(&plain)),
        None,
        "a plain card has no buttons"
    );
}

#[test]
fn a_jobs_outputs_and_the_series_it_feeds_a_chart_join_the_run_result() {
    let mut run = RunResult {
        outcomes: vec![NodeOutcome {
            node: "n1".to_string(),
            status: Status::Skipped,
            outputs: BTreeMap::new(),
            shown: BTreeMap::new(),
            error: Some("no output yet: press Start".to_string()),
        }],
        partial: false,
    };
    let mut report = update("n1", JobStatus::Running, 2, Some(5));
    report.outputs.insert("total".to_string(), json!(11));
    report
        .shown
        .insert("loss".to_string(), Shown::Series(series(&[0.9, 0.5])));
    report.linked.push(LinkedSeries {
        node: "n3".to_string(),
        port: "series_1".to_string(),
        series: series(&[0.9, 0.5]),
    });
    run.apply_job(&report);
    let job = run.outcome_for("n1").unwrap();
    assert_eq!(
        job.status,
        Status::Ok,
        "a skipped job that reported is ok now"
    );
    assert_eq!(job.error, None);
    assert_eq!(job.outputs["total"], json!(11));
    assert_eq!(job.output_text("loss").as_deref(), Some("0.5"));
    let chart = run.outcome_for("n3").unwrap();
    assert_eq!(chart.status, Status::Ok);
    assert_eq!(chart.shown["series_1"], Shown::Series(series(&[0.9, 0.5])));
    // A later report replaces the lines, and a second line on the chart stays.
    let mut chart_two = run.outcome_for("n3").unwrap().clone();
    chart_two
        .shown
        .insert("series_2".to_string(), Shown::Series(series(&[1.0])));
    run.merge(&RunResult {
        outcomes: vec![chart_two],
        partial: true,
    });
    report.linked[0].series = series(&[0.9, 0.5, 0.2]);
    run.apply_job(&report);
    let chart = run.outcome_for("n3").unwrap();
    assert_eq!(
        chart.shown["series_1"],
        Shown::Series(series(&[0.9, 0.5, 0.2]))
    );
    assert_eq!(chart.shown["series_2"], Shown::Series(series(&[1.0])));
    assert_eq!(run.outcomes.len(), 2);
}

#[test]
fn a_job_that_has_reported_nothing_changes_nothing() {
    let mut run = RunResult::default();
    run.apply_job(&update("n1", JobStatus::Running, 0, Some(5)));
    assert!(run.outcomes.is_empty());
}

fn item(id: &str, label: &str) -> NodeItem {
    NodeItem {
        id: id.to_string(),
        label: label.to_string(),
        port_count: 2,
        ports: vec![],
    }
}

#[test]
fn a_job_node_label_says_where_its_job_is_and_a_plain_one_is_left_alone() {
    let types = vec![
        kind("train", true, CardDisplay::None),
        kind("plain", false, CardDisplay::None),
    ];
    let graph = Graph {
        nodes: vec![
            placed("n1", "train", 0.0, 0.0),
            placed("n2", "plain", 0.0, 100.0),
        ],
        links: vec![],
    };
    let items = vec![
        item("n1", "n1, train. 1 input, 1 output."),
        item("n2", "n2, plain. 1 input, 1 output."),
    ];
    let mut jobs = BTreeMap::new();
    let idle = with_jobs(items.clone(), &graph, &types, &jobs);
    assert_eq!(idle[0].label, "n1, train. 1 input, 1 output. Job idle.");
    assert_eq!(idle[1].label, "n2, plain. 1 input, 1 output.");
    jobs.insert(
        "n1".to_string(),
        update("n1", JobStatus::Running, 3, Some(12)),
    );
    assert!(
        with_jobs(items, &graph, &types, &jobs)[0]
            .label
            .ends_with("Job running.")
    );
}

#[test]
fn the_job_sentences_name_the_state_and_a_failure_says_why() {
    assert_eq!(job_text(None), "Job idle.");
    let mut failed = update("n1", JobStatus::Failed, 0, None);
    assert_eq!(job_text(Some(&failed)), "Job failed: failed.");
    failed.error = Some("boom".to_string());
    assert_eq!(job_text(Some(&failed)), "Job failed: boom.");
    assert_eq!(job_announcement(&failed), "n1: failed: boom.");
    let running = update("n1", JobStatus::Running, 1, Some(5));
    assert_eq!(job_announcement(&running), "n1: running.");
    let finished = update("n1", JobStatus::Finished, 5, Some(5));
    assert_eq!(job_announcement(&finished), "n1: finished, step 5 of 5.");
    let stopped = update("n1", JobStatus::Stopped, 2, None);
    assert_eq!(job_announcement(&stopped), "n1: stopped.");
    let no_reason = update("n1", JobStatus::Failed, 0, None);
    assert_eq!(job_announcement(&no_reason), "n1: failed: failed.");
}

#[test]
fn a_job_button_says_when_it_does_not_work_and_why() {
    assert_eq!(
        job_button_label(JobButton::Start, None),
        "Start the job, button."
    );
    assert_eq!(
        job_button_label(JobButton::Stop, None),
        "Stop the job, button. Not available: no job is running."
    );
    let running = update("n1", JobStatus::Running, 1, Some(5));
    assert_eq!(
        job_button_label(JobButton::Start, Some(&running)),
        "Start the job, button. Not available: the job is running."
    );
    assert_eq!(
        job_button_label(JobButton::Stop, Some(&running)),
        "Stop the job, button."
    );
    let stopping = update("n1", JobStatus::Stopping, 1, Some(5));
    assert_eq!(
        job_button_refusal(JobButton::Stop, Some(&stopping)).as_deref(),
        Some("the job is already stopping.")
    );
    assert_eq!(
        job_button_refusal(
            JobButton::Start,
            Some(&update("n1", JobStatus::Failed, 0, None))
        ),
        None
    );
}

#[test]
fn a_run_after_a_job_is_not_called_a_run() {
    let partial = RunResult {
        outcomes: vec![NodeOutcome {
            node: "n2".to_string(),
            status: Status::Ok,
            outputs: BTreeMap::new(),
            shown: BTreeMap::new(),
            error: None,
        }],
        partial: true,
    };
    assert_eq!(run_text(&partial), "After the job: 1 ok");
}
