"""Run a job's code, the same way in a thread and in a child process."""

from collections.abc import Callable, Mapping
from typing import Protocol

from element_51.engine import collect_outputs
from element_51.job_node import JobNode, Stopped
from element_51.model import Model, NodeType
from element_51.visuals import Carried, carries


class StopFlag(Protocol):
    """A flag Stop sets; a thread's or a process's event fits."""

    def is_set(self) -> bool:
        """Whether Stop was pressed."""


class Sink(Protocol):
    """Where a running job's progress goes."""

    def begin(self, total: int) -> None:
        """Say how many steps the job has."""

    def update(self, fields: Mapping[str, Carried]) -> None:
        """Add new values to the job's outputs."""

    def advance(self, step: int) -> None:
        """Say that `step` steps are done."""


class CallbackSink(Model):
    """A `Sink` made of three functions.

    `begin` gets the number of steps, `update` new output values and
    `advance` the number of steps done.
    """

    begin: Callable[[int], None]
    update: Callable[[Mapping[str, Carried]], None]
    advance: Callable[[int], None]


class JobReport:
    """The `Report` a work job calls; it raises `Stopped` after Stop.

    Parameters
    ----------
    stop
        The flag Stop sets.
    node_type
        The job's type, which says what its outputs are.
    update
        Called with the fields of each report that passes the checks.
    """

    def __init__(
        self,
        stop: StopFlag,
        node_type: NodeType,
        update: Callable[[Mapping[str, Carried]], None],
    ) -> None:
        self._stop = stop
        self._node_type = node_type
        self._update = update

    @property
    def stopped(self) -> bool:
        """Whether Stop was pressed."""
        return self._stop.is_set()

    def __call__(self, **fields: Carried) -> None:
        """Show new values for some outputs.

        Raises
        ------
        Stopped
            If Stop was pressed; the values are dropped.
        ValueError
            If a name is not an output of the node.
        TypeError
            If a value does not fit its output.
        """
        if self.stopped:
            raise Stopped
        kinds = {port.name: port.kind for port in self._node_type.outputs}
        for name, value in fields.items():
            if name not in kinds:
                listed = ", ".join(kinds) or "none"
                msg = (
                    f"{name!r} is not an output of {self._node_type.name}; "
                    f"its outputs are: {listed}"
                )
                raise ValueError(msg)
            if not carries(value, kinds[name]):
                msg = (
                    f"output {name!r} is {type(value).__name__}, "
                    f"expected {kinds[name]}"
                )
                raise TypeError(msg)
        self._update(fields)


def execute(
    function: type[JobNode[object]],
    inputs: Mapping[str, Carried],
    stop: StopFlag,
    sink: Sink,
) -> None:
    """Build the node and run it, as steps or in one call.

    Stop ends it by raising `Stopped`, before a step or at a report.

    Parameters
    ----------
    function
        The job's class.
    inputs
        The value of each input.
    stop
        The flag Stop sets.
    sink
        Where progress and outputs go.
    """
    node = function(**inputs)
    if function.is_stepped:
        _steps(node, function, stop, sink)
        return
    report = JobReport(stop, function.node_type, sink.update)
    _update_from(function, node.work(report), sink)


def _steps(
    node: JobNode[object],
    function: type[JobNode[object]],
    stop: StopFlag,
    sink: Sink,
) -> None:
    """Run a step job: `begin`, then `step` for each step.

    Raises
    ------
    Stopped
        Before a step, if Stop was pressed.
    """
    total = node.steps()
    sink.begin(total)
    state = node.begin()
    for i in range(total):
        if stop.is_set():
            raise Stopped
        _update_from(function, node.step(state, i), sink)
        sink.advance(i + 1)


def _update_from(
    function: type[JobNode[object]], result: object, sink: Sink
) -> None:
    """Add a step's or a work job's return value to the outputs."""
    if result is not None:
        sink.update(collect_outputs(function, result))
