//! The job row of a card: Start and Stop buttons, the state and a progress bar.
//!
//! A card of a job node has two rows below its ports: the buttons and the
//! state word on the first, the bar and `step 3 of 5` on the second. The rows
//! are there whether or not the job has started, so the card does not change
//! size while a job runs.

use crate::layout::{Rect, node_at, node_rect};
use crate::rows::type_of;
use std::collections::BTreeMap;

use crate::theme::Metrics;
use crate::{Graph, JobStatus, JobUpdate, NodeType};

/// Rows of the card's height that a job node adds.
const ROWS: f64 = 2.0;
/// A button's width, as a multiple of the value box height.
const BUTTON_WIDTH_RATIO: f64 = 3.0;
/// The bar's height, as a multiple of the card's border width.
const BAR_HEIGHT_RATIO: f64 = 3.0;
/// The share of the card's inner width that the bar takes.
const BAR_SHARE: f64 = 0.5;

/// A button on the job row.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum JobButton {
    Start,
    Stop,
}

impl JobButton {
    /// The words on the button.
    pub fn label(self) -> &'static str {
        match self {
            JobButton::Start => "Start",
            JobButton::Stop => "Stop",
        }
    }
}

/// Where the parts of the job row sit, in graph units.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct JobLayout {
    pub start: Rect,
    pub stop: Rect,
    /// The right edge and the text baseline of the state word.
    pub state_right: f64,
    pub state_baseline: f64,
    pub bar: Rect,
    /// The right edge and the text baseline of `step 3 of 5`.
    pub step_right: f64,
    pub step_baseline: f64,
}

/// How much taller a card is because it is a job's: two rows. It depends on the
/// type only.
pub fn job_height(node_type: &NodeType, metrics: &Metrics) -> f64 {
    if node_type.job {
        ROWS * metrics.row_height
    } else {
        0.0
    }
}

/// The parts of the job row of the card at `card`, below the port rows. None if
/// the node type is not a job.
pub fn job_layout(card: &Rect, node_type: &NodeType, metrics: &Metrics) -> Option<JobLayout> {
    if !node_type.job {
        return None;
    }
    let rows = node_type.inputs.len() + node_type.outputs.len();
    let left = card.x + metrics.title_pad;
    let inner = card.w - 2.0 * metrics.title_pad;
    let top = card.y + metrics.header_height + metrics.row_height * rows as f64;
    let first = top + metrics.row_height / 2.0;
    let second = top + 1.5 * metrics.row_height;
    let button = |index: f64| Rect {
        x: left + index * (BUTTON_WIDTH_RATIO * metrics.field_height + metrics.title_pad / 2.0),
        y: first - metrics.field_height / 2.0,
        w: BUTTON_WIDTH_RATIO * metrics.field_height,
        h: metrics.field_height,
    };
    let bar_height = BAR_HEIGHT_RATIO * metrics.border_width;
    Some(JobLayout {
        start: button(0.0),
        stop: button(1.0),
        state_right: left + inner,
        state_baseline: first + metrics.text_baseline,
        bar: Rect {
            x: left,
            y: second - bar_height / 2.0,
            w: inner * BAR_SHARE,
            h: bar_height,
        },
        step_right: left + inner,
        step_baseline: second + metrics.text_baseline,
    })
}

/// The state word: `idle` for a node with no job yet.
pub fn state_text(update: Option<&JobUpdate>) -> &'static str {
    match update.map(|update| update.state) {
        None => "idle",
        Some(JobStatus::Running) => "running",
        Some(JobStatus::Stopping) => "stopping",
        Some(JobStatus::Finished) => "finished",
        Some(JobStatus::Stopped) => "stopped",
        Some(JobStatus::Failed) => "failed",
    }
}

/// `step 3 of 5` for a job that works in steps, else None.
pub fn progress_text(update: Option<&JobUpdate>) -> Option<String> {
    let update = update?;
    update
        .steps
        .map(|steps| format!("step {} of {steps}", update.step))
}

/// How far a step job has come, from 0 to 1, else None.
pub fn progress_fraction(update: Option<&JobUpdate>) -> Option<f64> {
    let update = update?;
    let steps = update.steps?;
    if steps == 0 {
        return Some(1.0);
    }
    Some((f64::from(update.step) / f64::from(steps)).clamp(0.0, 1.0))
}

/// Whether Start works: the node has no job, or its job has ended.
pub fn can_start(update: Option<&JobUpdate>) -> bool {
    !matches!(
        update.map(|update| update.state),
        Some(JobStatus::Running | JobStatus::Stopping)
    )
}

/// Whether Stop is offered: the job is running (not already stopping).
pub fn can_stop(update: Option<&JobUpdate>) -> bool {
    update.map(|update| update.state) == Some(JobStatus::Running)
}

/// The button of a job node under the point `(x, y)`, with the node's id. A
/// button that does not work now (Start while running, Stop when nothing runs)
/// is not found. Only the card on top at that point counts.
pub fn job_button_at(
    graph: &Graph,
    node_types: &[NodeType],
    jobs: &BTreeMap<String, JobUpdate>,
    (x, y): (f64, f64),
    metrics: &Metrics,
) -> Option<(String, JobButton)> {
    let id = node_at(graph, node_types, x, y, metrics)?;
    let node = graph.nodes.iter().find(|node| node.id == id)?;
    let node_type = type_of(graph, node_types, node)?;
    let layout = job_layout(&node_rect(node, &node_type, metrics), &node_type, metrics)?;
    let update = jobs.get(&id);
    let inside =
        |rect: &Rect| x >= rect.x && x <= rect.x + rect.w && y >= rect.y && y <= rect.y + rect.h;
    if inside(&layout.start) && can_start(update) {
        Some((id, JobButton::Start))
    } else if inside(&layout.stop) && can_stop(update) {
        Some((id, JobButton::Stop))
    } else {
        None
    }
}
