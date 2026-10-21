"""Tests for jobs that run in a child process."""

import multiprocessing
import threading
import time
from typing import ClassVar, override

import pytest
from sample_jobs import (
    Chatty,
    Counting,
    Dies,
    IsoSteps,
    Raises,
    Sleeper,
)
from test_jobs import Collector, graph_of

from element_51.isolated import (
    Advanced,
    Began,
    ChildMessage,
    Ended,
    Failed,
    Updated,
    check_importable,
    child_main,
)
from element_51.job_node import JobNode, Report
from element_51.jobs import JobRefusedError, JobRunner
from element_51.visuals import Series

FUNCTIONS = {
    f.node_type.name: f
    for f in (Counting, IsoSteps, Sleeper, Chatty, Dies, Raises)
}
GRACE = 1.0
SETTLE = 0.2
LOCAL_TO_FUNCTION = "inside a function"


class Outbox:
    """A list standing in for the queue to the parent."""

    def __init__(self) -> None:
        self.messages: list[ChildMessage] = []

    def put(self, message: ChildMessage) -> None:
        self.messages.append(message)


def runner_for(collector: Collector) -> JobRunner:
    return JobRunner(FUNCTIONS, collector.publish, collector.on_end)


def no_children_left() -> bool:
    return not multiprocessing.active_children()


def test_the_child_sends_each_report_then_ended_for_a_work_job() -> None:
    outbox = Outbox()
    child_main(
        "sample_jobs", "Counting", {"upto": 2}, outbox, threading.Event()
    )
    assert outbox.messages == [
        Updated(fields={"out": 1}),
        Updated(fields={"out": 2}),
        Updated(fields={"out": 12}),
        Ended(),
    ]


def test_the_child_sends_the_steps_and_each_step_for_a_step_job() -> None:
    outbox = Outbox()
    child_main(
        "sample_jobs", "IsoSteps", {"count": 2}, outbox, threading.Event()
    )
    assert outbox.messages == [
        Began(total=2),
        Updated(fields={"loss": Series(values=(0.0,)), "total": 10}),
        Advanced(step=1),
        Updated(fields={"loss": Series(values=(1.0,)), "total": 11}),
        Advanced(step=2),
        Ended(),
    ]


def test_the_child_sends_ended_when_stop_ends_the_job() -> None:
    outbox = Outbox()
    stop = threading.Event()
    stop.set()
    child_main("sample_jobs", "Chatty", {}, outbox, stop)
    assert outbox.messages == [Ended()]


def test_the_child_sends_failed_with_the_reason_when_the_code_raises() -> None:
    outbox = Outbox()
    child_main("sample_jobs", "Raises", {}, outbox, threading.Event())
    assert outbox.messages == [Failed(error="ValueError: bad")]


def test_the_child_sends_failed_when_the_class_cannot_be_imported() -> None:
    outbox = Outbox()
    child_main("sample_jobs", "Nope", {}, outbox, threading.Event())
    (message,) = outbox.messages
    assert isinstance(message, Failed)
    assert "Nope" in message.error


def test_a_work_job_runs_in_a_child_and_its_reports_arrive() -> None:
    collector = Collector()
    runner = runner_for(collector)
    runner.start(graph_of({"n1": "Counting"}, values={"n1": {"upto": 2}}), "n1")
    done = collector.wait_for("finished")
    collector.wait_for_end()
    assert done.outputs == {"out": 12}
    assert no_children_left()


def test_a_step_job_runs_in_a_child_with_progress() -> None:
    collector = Collector()
    runner = runner_for(collector)
    runner.start(graph_of({"n1": "IsoSteps"}), "n1")
    done = collector.wait_for("finished")
    assert (done.step, done.steps) == (3, 3)
    assert done.outputs == {"total": 12}
    assert done.shown == {"loss": Series(values=(0.0, 1.0, 2.0))}
    assert no_children_left()


def test_stop_kills_a_child_that_never_reports_within_a_second() -> None:
    collector = Collector()
    runner = runner_for(collector)
    runner.start(graph_of({"n1": "Sleeper"}), "n1")
    time.sleep(SETTLE)
    began = time.monotonic()
    runner.stop("n1")
    collector.wait_for("stopped")
    assert time.monotonic() - began < GRACE
    collector.wait_for_end()
    assert no_children_left()


def test_stop_ends_a_child_that_reports_without_waiting_to_kill_it() -> None:
    collector = Collector()
    runner = runner_for(collector)
    runner.start(graph_of({"n1": "Chatty"}), "n1")
    deadline = time.monotonic() + GRACE * 5
    while not runner.results() and time.monotonic() < deadline:
        time.sleep(0.01)
    assert runner.results()
    runner.stop("n1")
    collector.wait_for("stopped")
    assert no_children_left()


def test_a_child_that_dies_makes_the_job_failed_with_its_exit_code() -> None:
    collector = Collector()
    runner = runner_for(collector)
    runner.start(graph_of({"n1": "Dies"}), "n1")
    failed = collector.wait_for("failed")
    assert failed.error is not None
    assert "exit code 3" in failed.error
    assert collector.ended == []
    assert no_children_left()


def test_a_job_whose_code_raises_in_the_child_is_failed_with_the_reason() -> (
    None
):
    collector = Collector()
    runner = runner_for(collector)
    runner.start(graph_of({"n1": "Raises"}), "n1")
    assert collector.wait_for("failed").error == "ValueError: bad"
    assert no_children_left()


def test_a_class_defined_inside_a_function_is_refused_at_start() -> None:
    class Local(JobNode[None]):
        isolated: ClassVar[bool] = True

        @override
        def work(self, report: Report) -> int:  # pragma: no cover  # refused
            del report
            return 0

    collector = Collector()
    runner = JobRunner(
        {Local.node_type.name: Local}, collector.publish, collector.on_end
    )
    with pytest.raises(JobRefusedError, match=LOCAL_TO_FUNCTION):
        runner.start(graph_of({"n1": "Local"}), "n1")
    assert runner.states() == ()
    assert check_importable(Local) is not None


def test_a_class_in_a_module_off_the_import_path_is_refused() -> None:
    class Elsewhere(JobNode[None]):
        isolated: ClassVar[bool] = True

        @override
        def work(self, report: Report) -> int:  # pragma: no cover  # refused
            del report
            return 0

    Elsewhere.__qualname__ = "Elsewhere"
    Elsewhere.__module__ = "no_such_module_for_a_child"
    reason = check_importable(Elsewhere)
    assert reason is not None
    assert "no_such_module_for_a_child" in reason


def test_a_class_in_an_importable_module_is_accepted() -> None:
    assert check_importable(Counting) is None
    assert check_importable(IsoSteps) is None


def test_a_class_in_the_main_module_is_accepted() -> None:
    class Main(JobNode[None]):
        @override
        def work(self, report: Report) -> int:  # pragma: no cover  # not run
            del report
            return 0

    Main.__qualname__ = "Main"
    Main.__module__ = "__main__"
    assert check_importable(Main) is None
