"""Tests for running jobs in the background."""

import threading
from collections.abc import Callable
from typing import cast, override

import pytest
from sample_jobs import FIRST_TOTAL, WAIT, Out, Pause, Steps

from element_51.job_node import JobNode, Report, Stopped
from element_51.jobs import JobRefusedError, JobRunner, JobState
from element_51.model import Graph, Link, PlacedNode, PortRef
from element_51.nodes import Node, node
from element_51.visuals import Image, Series

PAUSE_AT = 1
DONE_BEFORE_PAUSE = 2


class Worker(JobNode[None]):
    """A work job that reports in pieces, then returns the last piece."""

    mode: str = "ok"

    @override
    def work(self, report: Report) -> Out:
        report(total=1)
        report(loss=Series(values=(1.0,)))
        report(loss=Series(values=(2.0, 3.0)))
        if self.mode == "boom":
            msg = "boom"
            raise RuntimeError(msg)
        if self.mode == "name":
            report(nothing=1)
        if self.mode == "kind":
            report(total=1.5)
        if self.mode == "wait":
            Pause.wait()
            report(total=2)
        if self.mode == "catch":
            Pause.wait()
            try:
                report(total=2)
            except Stopped:
                Pause.release.clear()
                Catcher.cleaned.set()
        return Out(loss=Series(values=(4.0,)), total=9)


class Catcher:
    cleaned = threading.Event()


class Silent(JobNode[None]):
    """A work job that reports nothing and returns nothing."""

    @override
    def work(self, report: Report) -> int:
        del report
        return cast("int", None)


class Picture(JobNode[None]):
    """A work job with a picture output."""

    @override
    def work(self, report: Report) -> Image:
        del report
        return Image(width=1, height=1, channels=1, pixels=bytes([7]))


class Needs(JobNode[None]):
    """A work job with an input that has no default."""

    amount: int

    @override
    def work(self, report: Report) -> int:  # pragma: no cover  # refused first
        del report
        return self.amount


class Plain(Node):
    x: float = 0.0

    @override
    def run(self) -> float:  # pragma: no cover  # a plain node, never run here
        return self.x


@node
def source() -> int:
    return 4


@node
def broken() -> int:
    msg = "no source"
    raise RuntimeError(msg)


FUNCTIONS = {
    f.node_type.name: f
    for f in (Steps, Worker, Silent, Picture, Needs, Plain, source, broken)
}


class Collector:
    """Gathers what a runner publishes and lets a test wait for a state."""

    def __init__(self) -> None:
        self.states: list[JobState] = []
        self.ended: list[str] = []
        self._ended = threading.Event()
        self._changed = threading.Condition()

    def publish(self, state: JobState) -> None:
        with self._changed:
            self.states.append(state)
            self._changed.notify_all()

    def on_end(self, node_id: str) -> None:
        self.ended.append(node_id)
        self._ended.set()

    def wait_for(self, name: str, node_id: str = "n1") -> JobState:
        """Return the last published state with this name.

        Returns
        -------
        JobState
            The state.
        """

        def seen() -> bool:
            return any(
                s.state == name and s.node == node_id for s in self.states
            )

        with self._changed:
            found = self._changed.wait_for(seen, WAIT)
        assert found, f"no {name}; saw {[s.state for s in self.states]}"
        return next(
            s
            for s in reversed(self.states)
            if s.state == name and s.node == node_id
        )

    def wait_for_end(self) -> None:
        assert self._ended.wait(WAIT)


def runner_for(
    collector: Collector,
    *,
    clock: Callable[[], float] | None = None,
    schedule: Callable[[float, Callable[[], None]], None] | None = None,
) -> JobRunner:
    """Build a runner whose output goes to `collector`.

    Returns
    -------
    JobRunner
        The runner.
    """
    Pause.reset()
    if clock is not None and schedule is not None:
        return JobRunner(
            FUNCTIONS,
            collector.publish,
            collector.on_end,
            clock=clock,
            schedule=schedule,
        )
    return JobRunner(FUNCTIONS, collector.publish, collector.on_end)


def graph_of(
    kinds: dict[str, str],
    links: tuple[tuple[str, str, str, str], ...] = (),
    values: dict[str, dict[str, int | str]] | None = None,
) -> Graph:
    """Build a graph from `{id: type}` and `(src, out, dst, in)` links.

    Returns
    -------
    Graph
        The graph.
    """
    given = values or {}
    return Graph(
        nodes=tuple(
            PlacedNode(
                id=node_id,
                node_type=kind,
                x=0.0,
                y=0.0,
                values=given.get(node_id, {}),
            )
            for node_id, kind in kinds.items()
        ),
        links=tuple(
            Link(source=PortRef(node=a, port=b), target=PortRef(node=c, port=d))
            for a, b, c, d in links
        ),
    )


def test_a_step_job_runs_to_the_end_and_series_grow() -> None:
    """Each step adds a point to `loss` and replaces `total`."""
    seen = Collector()
    runner = runner_for(seen)
    runner.start(graph_of({"n1": "Steps"}), "n1")
    done = seen.wait_for("finished")
    assert (done.step, done.steps) == (3, 3)
    assert done.outputs == {"total": FIRST_TOTAL + 2}
    assert done.shown["loss"] == Series(values=(0.0, 1.0, 2.0))
    seen.wait_for_end()
    assert seen.ended == ["n1"]
    assert seen.states[0].state == "running"


def test_a_work_job_reports_pieces_and_its_return_value_is_the_last_piece() -> (
    None
):
    """Reports and the return value merge the same way; steps stay None."""
    seen = Collector()
    runner = runner_for(seen)
    runner.start(graph_of({"n1": "Worker"}), "n1")
    done = seen.wait_for("finished")
    assert done.steps is None
    assert done.outputs == {"total": 9}
    assert done.shown["loss"] == Series(values=(1.0, 2.0, 3.0, 4.0))


def test_a_job_that_returns_nothing_finishes_with_no_outputs() -> None:
    """`None` from `work` adds nothing; the job still finishes."""
    seen = Collector()
    runner = runner_for(seen)
    runner.start(graph_of({"n1": "Silent"}), "n1")
    done = seen.wait_for("finished")
    assert done.outputs == {}
    assert runner.results() == {}


def test_a_picture_output_is_shown_not_listed_with_the_plain_values() -> None:
    """Pictures and series go in `shown`, numbers in `outputs`."""
    seen = Collector()
    runner = runner_for(seen)
    runner.start(graph_of({"n1": "Picture"}), "n1")
    done = seen.wait_for("finished")
    assert done.outputs == {}
    assert isinstance(done.shown["out"], Image)


@pytest.mark.parametrize(
    ("mode", "words"),
    [
        ("boom", "RuntimeError: boom"),
        ("name", "'nothing' is not an output of Worker"),
        ("kind", "output 'total' is float, expected"),
    ],
)
def test_a_failing_job_ends_failed_with_its_reason_and_keeps_its_results(
    mode: str, words: str
) -> None:
    """A raise, an unknown output name and a wrong kind all fail the job."""
    seen = Collector()
    runner = runner_for(seen)
    runner.start(
        graph_of({"n1": "Worker"}, values={"n1": {"mode": mode}}), "n1"
    )
    failed = seen.wait_for("failed")
    assert failed.error is not None
    assert words in failed.error
    assert failed.shown["loss"] == Series(values=(1.0, 2.0, 3.0))
    assert not seen.ended


def test_stop_ends_a_step_job_before_its_next_step() -> None:
    """The state is `stopping` at once and `stopped` when the job ends."""
    seen = Collector()
    runner = runner_for(seen)
    graph = graph_of({"n1": "Steps"}, values={"n1": {"count": 5}})
    Steps.pause_at = PAUSE_AT
    try:
        runner.start(graph, "n1")
        assert Pause.entered.wait(WAIT)
        runner.stop("n1")
        assert seen.wait_for("stopping").state == "stopping"
        Pause.release.set()
        stopped = seen.wait_for("stopped")
    finally:
        Steps.pause_at = -1
    assert stopped.step == PAUSE_AT + 1
    assert stopped.shown["loss"] == Series(values=(0.0, 1.0))
    seen.wait_for_end()
    assert seen.ended == ["n1"]


def test_stop_makes_the_next_report_raise_and_drops_that_report() -> None:
    """A work job is stopped at its next report call."""
    seen = Collector()
    runner = runner_for(seen)
    runner.start(
        graph_of({"n1": "Worker"}, values={"n1": {"mode": "wait"}}), "n1"
    )
    assert Pause.entered.wait(WAIT)
    runner.stop("n1")
    Pause.release.set()
    stopped = seen.wait_for("stopped")
    assert stopped.outputs == {"total": 1}


def test_a_job_that_catches_stopped_to_clean_up_still_ends_stopped() -> None:
    """After Stop the job may clean up and return; the state is `stopped`."""
    seen = Collector()
    runner = runner_for(seen)
    runner.start(
        graph_of({"n1": "Worker"}, values={"n1": {"mode": "catch"}}), "n1"
    )
    assert Pause.entered.wait(WAIT)
    runner.stop("n1")
    Pause.release.set()
    seen.wait_for("stopped")
    assert Catcher.cleaned.wait(WAIT)


def test_stop_is_refused_when_nothing_runs_and_after_a_stop() -> None:
    """There is no running job to stop before a start, or once it stops."""
    seen = Collector()
    runner = runner_for(seen)
    with pytest.raises(JobRefusedError, match="no running job"):
        runner.stop("n1")
    Steps.pause_at = 0
    try:
        runner.start(graph_of({"n1": "Steps"}), "n1")
        assert Pause.entered.wait(WAIT)
        runner.stop("n1")
        with pytest.raises(JobRefusedError, match="no running job"):
            runner.stop("n1")
        Pause.release.set()
        seen.wait_for("stopped")
    finally:
        Steps.pause_at = -1


def test_start_is_refused_while_the_job_runs_or_stops() -> None:
    """One node never has two jobs."""
    seen = Collector()
    runner = runner_for(seen)
    graph = graph_of({"n1": "Steps"})
    Steps.pause_at = 0
    try:
        runner.start(graph, "n1")
        assert Pause.entered.wait(WAIT)
        with pytest.raises(JobRefusedError, match="already running"):
            runner.start(graph, "n1")
        runner.stop("n1")
        with pytest.raises(JobRefusedError, match="already stopping"):
            runner.start(graph, "n1")
        Pause.release.set()
        seen.wait_for("stopped")
    finally:
        Steps.pause_at = -1


def test_start_is_refused_for_a_missing_node_and_for_a_plain_node() -> None:
    """Only a job node starts."""
    runner = runner_for(Collector())
    graph = graph_of({"n1": "Plain", "n2": "source"})
    with pytest.raises(JobRefusedError, match="no node n9"):
        runner.start(graph, "n9")
    with pytest.raises(JobRefusedError, match="not a job"):
        runner.start(graph, "n1")
    with pytest.raises(JobRefusedError, match="not a job"):
        runner.start(graph, "n2")


def test_start_runs_the_nodes_before_the_job_and_takes_their_values() -> None:
    """The job's `count` input comes from the linked node, run once."""
    seen = Collector()
    runner = runner_for(seen)
    graph = graph_of(
        {"n1": "source", "n2": "Steps"}, (("n1", "out", "n2", "count"),)
    )
    runner.start(graph, "n2")
    done = seen.wait_for("finished", "n2")
    assert done.steps == source.defaults.get("x", 4)


def test_start_is_refused_with_the_name_of_a_node_before_it_that_failed() -> (
    None
):
    """A failing ancestor stops the start, and an earlier result is kept."""
    seen = Collector()
    runner = runner_for(seen)
    runner.start(graph_of({"n2": "Steps"}), "n2")
    seen.wait_for("finished", "n2")
    bad = graph_of(
        {"n1": "broken", "n2": "Steps"}, (("n1", "out", "n2", "count"),)
    )
    with pytest.raises(JobRefusedError, match="Node n1 did not run"):
        runner.start(bad, "n2")
    assert runner.results()["n2"]["total"] == FIRST_TOTAL + 2
    fresh = runner_for(Collector())
    with pytest.raises(JobRefusedError, match="did not run"):
        fresh.start(bad, "n2")
    assert fresh.states() == ()


def test_start_is_refused_for_an_input_with_no_value() -> None:
    """A job input with no link, value or default is named."""
    runner = runner_for(Collector())
    with pytest.raises(JobRefusedError, match="input 'amount' has no value"):
        runner.start(graph_of({"n1": "Needs"}), "n1")


def test_a_job_started_again_drops_its_old_results() -> None:
    """Results stay until the next Start, which replaces them."""
    seen = Collector()
    runner = runner_for(seen)
    graph = graph_of({"n1": "Steps"})
    runner.start(graph, "n1")
    seen.wait_for("finished")
    seen.wait_for_end()
    assert runner.results()["n1"]["total"] == FIRST_TOTAL + 2
    Steps.pause_at = 0
    try:
        runner.start(graph, "n1")
        assert Pause.entered.wait(WAIT)
        assert runner.results() == {}
        Pause.release.set()
        seen.wait_for("finished")
    finally:
        Steps.pause_at = -1


def test_states_lists_each_job_and_drop_forgets_one() -> None:
    """A dropped job leaves no state and no results, and is not published."""
    seen = Collector()
    runner = runner_for(seen)
    graph = graph_of({"n1": "Steps", "n2": "Worker"})
    runner.start(graph, "n1")
    runner.start(graph, "n2")
    seen.wait_for("finished", "n1")
    seen.wait_for("finished", "n2")
    assert [s.node for s in runner.states()] == ["n1", "n2"]
    runner.drop("n1")
    runner.drop("n9")
    assert [s.node for s in runner.states()] == ["n2"]
    assert set(runner.results()) == {"n2"}
    runner.clear()
    assert runner.states() == ()


def test_a_dropped_running_job_is_stopped_and_publishes_nothing_more() -> None:
    """Deleting a node ends its job quietly."""
    seen = Collector()
    runner = runner_for(seen)
    Steps.pause_at = 0
    try:
        runner.start(graph_of({"n1": "Steps"}), "n1")
        assert Pause.entered.wait(WAIT)
        before = len(seen.states)
        runner.drop("n1")
        Pause.release.set()
        threading.Event().wait(0.05)
    finally:
        Steps.pause_at = -1
    assert len(seen.states) == before
    assert not seen.ended


def test_clear_stops_every_running_job() -> None:
    """Loading a graph ends all jobs."""
    seen = Collector()
    runner = runner_for(seen)
    Steps.pause_at = 0
    try:
        runner.start(graph_of({"n1": "Steps"}), "n1")
        assert Pause.entered.wait(WAIT)
        runner.clear()
        Pause.release.set()
        threading.Event().wait(0.05)
    finally:
        Steps.pause_at = -1
    assert runner.states() == ()
    assert not seen.ended


class FakeTime:
    """A clock and a scheduler a test steps by hand."""

    def __init__(self) -> None:
        self.now = 0.0
        self.waiting: list[tuple[float, Callable[[], None]]] = []

    def clock(self) -> float:
        return self.now

    def schedule(self, delay: float, action: Callable[[], None]) -> None:
        self.waiting.append((delay, action))


def test_updates_inside_the_interval_wait_for_one_timer_and_end_at_once() -> (
    None
):
    """Many updates in 50 ms send the first, arm one timer, and the end."""
    seen = Collector()
    fake = FakeTime()
    runner = runner_for(seen, clock=fake.clock, schedule=fake.schedule)
    runner.start(graph_of({"n1": "Steps"}), "n1")
    seen.wait_for("finished")
    states = [s.state for s in seen.states]
    assert states[0] == "running"
    assert states[-1] == "finished"
    assert len(fake.waiting) == 1
    assert fake.waiting[0][0] == pytest.approx(0.05)
    count = len(seen.states)
    fake.waiting[0][1]()
    assert len(seen.states) == count


def test_a_timer_sends_the_last_update_when_updates_stop_coming() -> None:
    """A burst, then silence: the timer publishes what the burst left."""
    seen = Collector()
    fake = FakeTime()
    runner = runner_for(seen, clock=fake.clock, schedule=fake.schedule)
    Steps.pause_at = DONE_BEFORE_PAUSE
    try:
        runner.start(graph_of({"n1": "Steps"}), "n1")
        assert Pause.entered.wait(WAIT)
        assert len(fake.waiting) == 1
        count = len(seen.states)
        fake.now = 1.0
        fake.waiting[0][1]()
        assert len(seen.states) == count + 1
        assert seen.states[-1].step == DONE_BEFORE_PAUSE
        fake.waiting[0][1]()
        assert len(seen.states) == count + 1
        Pause.release.set()
        seen.wait_for("finished")
    finally:
        Steps.pause_at = -1


def test_a_timer_for_a_dropped_job_sends_nothing() -> None:
    """The flush checks the job is still the node's current one."""
    seen = Collector()
    fake = FakeTime()
    runner = runner_for(seen, clock=fake.clock, schedule=fake.schedule)
    Steps.pause_at = DONE_BEFORE_PAUSE
    try:
        runner.start(graph_of({"n1": "Steps"}), "n1")
        assert Pause.entered.wait(WAIT)
        runner.drop("n1")
        count = len(seen.states)
        fake.waiting[0][1]()
        assert len(seen.states) == count
        Pause.release.set()
    finally:
        Steps.pause_at = -1
