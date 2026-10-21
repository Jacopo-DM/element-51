"""Tests for `JobNode`."""

from typing import cast, override

import pytest

from element_51.job_node import JobNode, Report, Stopped
from element_51.model import Model, Port, PortType
from element_51.nodes import Node
from element_51.visuals import Series


class Curves(Model):
    loss: Series
    accuracy: Series


class Training(JobNode[int]):
    """A step job: five epochs by default."""

    epochs: int = 5

    @override
    def steps(self) -> int:  # pragma: no cover  # only its hints are read
        return self.epochs

    @override
    def begin(self) -> int:  # pragma: no cover  # only its hints are read
        return 0

    @override
    def step(
        self, state: int, i: int
    ) -> Curves:  # pragma: no cover  # only its hints are read
        return Curves(
            loss=Series(values=(1.0,)), accuracy=Series(values=(0.5,))
        )


class Counting(JobNode[None]):
    """A work job with one plain output."""

    @override
    def work(
        self, report: Report
    ) -> int:  # pragma: no cover  # only its hints are read
        report(out=1)
        return 2


def test_a_job_node_builds_its_type_from_its_fields_and_its_step_hint() -> None:
    """Fields are inputs, `step`'s return model gives the outputs."""
    kind = Training.node_type
    assert kind.name == "Training"
    assert kind.job
    assert kind.inputs == (Port(name="epochs", kind=PortType.INT),)
    assert [(p.name, p.kind) for p in kind.outputs] == [
        ("loss", PortType.SERIES),
        ("accuracy", PortType.SERIES),
    ]
    assert Training.defaults == {"epochs": 5}
    assert Training.is_stepped


def test_a_work_job_takes_its_outputs_from_the_hint_of_work() -> None:
    """A plain return type gives one output `out`; the job is not stepped."""
    assert Counting.node_type.job
    assert Counting.node_type.outputs == (Port(name="out", kind=PortType.INT),)
    assert not Counting.is_stepped


def test_a_plain_node_is_not_a_job() -> None:
    """Only `JobNode` sets the flag."""

    class Plain(Node):
        x: float = 0.0

        @override
        def run(self) -> float:  # pragma: no cover  # only its hints are read
            return self.x

    assert not Plain.node_type.job
    assert not Plain.is_job


def test_a_job_class_with_both_forms_or_neither_is_refused() -> None:
    """One of `step` and `work`, not both and not neither."""
    with pytest.raises(TypeError, match="one of them"):

        class Neither(JobNode[None]):
            pass

    with pytest.raises(TypeError, match="one of them"):

        class Both(JobNode[None]):
            @override
            def step(
                self, state: None, i: int
            ) -> int:  # pragma: no cover  # never runs
                return i

            @override
            def work(
                self, report: Report
            ) -> int:  # pragma: no cover  # never runs
                return 1


def test_a_step_job_needs_begin_and_steps() -> None:
    """A `step` alone is refused, naming what is missing."""
    with pytest.raises(TypeError, match="begin"):

        class Half(JobNode[None]):
            @override
            def step(
                self, state: None, i: int
            ) -> int:  # pragma: no cover  # never runs
                return i


def test_a_job_without_a_return_hint_is_refused() -> None:
    """The outputs come from the hint, as for `Node.run`."""
    with pytest.raises(TypeError, match="no type hint"):

        class Unhinted(JobNode[None]):
            @override
            def work(self, report: Report):  # type: ignore[no-untyped-def]  # ruff: ignore[missing-return-type-private-function]  # the missing return hint is the point
                return 1  # pragma: no cover  # never runs


def test_a_job_is_started_not_called() -> None:
    """`call` refuses, so nothing runs a job as a plain node."""
    with pytest.raises(TypeError, match="start it"):
        Training.call({"epochs": 1})


def test_stopped_passes_through_a_jobs_own_except_exception() -> None:
    """`Stopped` is no `Exception`, so a broad handler lets it by."""
    assert not issubclass(Stopped, Exception)
    assert issubclass(Stopped, BaseException)


def test_the_base_hooks_of_a_job_are_placeholders_that_refuse() -> None:
    """A hook a subclass leaves alone raises `NotImplementedError`."""
    report = cast("Report", None)
    with pytest.raises(NotImplementedError):
        Counting().steps()
    with pytest.raises(NotImplementedError):
        Counting().begin()
    with pytest.raises(NotImplementedError):
        Counting().step(None, 0)
    with pytest.raises(NotImplementedError):
        Training().work(report)
