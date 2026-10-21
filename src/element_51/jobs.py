"""Run job nodes in the background and keep what they report."""

import threading
import time
from collections.abc import Callable, Mapping
from functools import partial
from typing import Literal

from pydantic import BaseModel, ConfigDict, Field

from element_51.engine import (
    MissingInputError,
    ancestors,
    describe_error,
    resolve_inputs,
    run_graph,
)
from element_51.isolated import ChildFailedError, check_importable, run_in_child
from element_51.job_exec import CallbackSink, execute
from element_51.job_node import JobNode, Stopped
from element_51.model import Graph, Model, NodeType, PlacedNode, PortValue
from element_51.nodes import Registered
from element_51.visuals import Carried, Image, Series, Shown

PUBLISH_INTERVAL = 0.05
"""Seconds between two updates sent for one job: at most 20 a second."""

type JobStateName = Literal[
    "running", "stopping", "finished", "stopped", "failed"
]
"""Where a job is. A node with no job state is idle."""


class JobState(Model):
    """What a page needs to show one job: its state, progress and outputs.

    `steps` is None for a job that works in one call. `outputs` holds the
    plain values and `shown` the pictures and series, by output port name, as
    in `NodeOutcome`.
    """

    node: str
    state: JobStateName
    step: int = 0
    steps: int | None = None
    error: str | None = None
    outputs: Mapping[str, PortValue] = {}
    shown: Mapping[str, Shown] = {}


type Publish = Callable[[JobState], None]
type Schedule = Callable[[float, Callable[[], None]], None]


class JobRefusedError(Exception):
    """A request about a job was refused.

    Parameters
    ----------
    reason
        Why, worded for a person.
    """

    def __init__(self, reason: str) -> None:
        super().__init__(reason)
        self.reason = reason


class _Job(BaseModel):
    """One job's state while it runs and after it ends.

    The job's thread and the runner both change it, under the runner's lock,
    so it is a mutable record.
    """

    model_config = ConfigDict(extra="forbid", arbitrary_types_allowed=True)

    node: str
    node_type: NodeType
    state: JobStateName = "running"
    step: int = 0
    steps: int | None = None
    error: str | None = None
    results: dict[str, Carried] = {}
    stop: threading.Event = Field(default_factory=threading.Event)
    last_sent: float = float("-inf")
    dirty: bool = False
    flush_armed: bool = False

    def as_state(self) -> JobState:
        """Describe the job for a page.

        Returns
        -------
        JobState
            Its state, progress and the outputs reported so far.
        """
        outputs: dict[str, PortValue] = {}
        shown: dict[str, Shown] = {}
        for name, value in self.results.items():
            if isinstance(value, Image | Series):
                shown[name] = value
            else:
                outputs[name] = value
        return JobState(
            node=self.node,
            state=self.state,
            step=self.step,
            steps=self.steps,
            error=self.error,
            outputs=outputs,
            shown=shown,
        )


def _merged(
    old: Mapping[str, Carried], new: Mapping[str, Carried]
) -> dict[str, Carried]:
    """Add an update to the results: series grow, anything else is replaced.

    Returns
    -------
    dict[str, Carried]
        The results after the update.
    """
    merged = dict(old)
    for name, value in new.items():
        before = merged.get(name)
        if isinstance(before, Series) and isinstance(value, Series):
            merged[name] = Series(values=before.values + value.values)
        else:
            merged[name] = value
    return merged


def _after(delay: float, action: Callable[[], None]) -> None:
    """Call `action` once, `delay` seconds from now, on a daemon thread."""
    timer = threading.Timer(delay, action)
    timer.daemon = True
    timer.start()


class JobRunner:
    """Starts jobs, keeps their results and tells a listener as they change.

    Each job runs on a daemon thread, so Ctrl+C still ends the process. Its
    state goes to `publish` at most `PUBLISH_INTERVAL` apart; the last state
    of a burst follows when the interval is up, and a job's end is sent at
    once. `publish` may be called from any thread and must not block.

    Parameters
    ----------
    functions
        The node types, by name.
    publish
        Called with a job's state after each change.
    on_end
        Called with the node id after a job finishes or is stopped, on the
        job's thread, after its last state was published.
    clock
        Seconds, for the rate limit; tests pass a fake.
    schedule
        Calls an action after a delay; tests pass a fake.
    """

    def __init__(
        self,
        functions: Mapping[str, Registered],
        publish: Publish,
        on_end: Callable[[str], None],
        *,
        clock: Callable[[], float] = time.monotonic,
        schedule: Schedule = _after,
    ) -> None:
        self._functions = functions
        self._publish = publish
        self._on_end = on_end
        self._clock = clock
        self._schedule = schedule
        self._jobs: dict[str, _Job] = {}
        self._lock = threading.RLock()
        self._send_lock = threading.Lock()

    def start(self, graph: Graph, node: str) -> None:
        """Start the job of `node`, with the values its inputs have now.

        The nodes the job depends on run once first. The job's earlier results
        are dropped when it starts, and kept if the start is refused.

        Parameters
        ----------
        graph
            The graph as it is now.
        node
            The id of the job node.

        Raises
        ------
        JobRefusedError
            If there is no such node, it is not a job, its job is running or
            stopping, or a node it depends on fails.
        """
        placed = next((n for n in graph.nodes if n.id == node), None)
        if placed is None:
            msg = f"There is no node {node}"
            raise JobRefusedError(msg)
        function = self._functions[placed.node_type]
        if not (isinstance(function, type) and issubclass(function, JobNode)):
            msg = f"Node {node} ({placed.node_type}) is not a job"
            raise JobRefusedError(msg)
        if function.isolated and (why := check_importable(function)):
            raise JobRefusedError(why)
        job = _Job(node=node, node_type=function.node_type)
        with self._lock:
            previous = self._jobs.get(node)
            if previous is not None and previous.state in {
                "running",
                "stopping",
            }:
                msg = f"Node {node} is already {previous.state}"
                raise JobRefusedError(msg)
            self._jobs[node] = job
        try:
            inputs = self._inputs(graph, placed, function)
        except JobRefusedError:
            with self._lock:
                if previous is None:
                    del self._jobs[node]
                else:
                    self._jobs[node] = previous
            raise
        self._emit(job, force=True)
        threading.Thread(
            target=self._work, args=(job, function, inputs), daemon=True
        ).start()

    def stop(self, node: str) -> None:
        """Ask the job of `node` to end.

        It ends at its next step or report. The state is `stopping` until it
        does.

        Parameters
        ----------
        node
            The id of the job node.

        Raises
        ------
        JobRefusedError
            If the node has no running job.
        """
        with self._lock:
            job = self._jobs.get(node)
            if job is None or job.state != "running":
                msg = f"Node {node} has no running job to stop"
                raise JobRefusedError(msg)
            job.stop.set()
            job.state = "stopping"
        self._emit(job, force=True)

    def states(self) -> tuple[JobState, ...]:
        """Describe every job that has been started and not dropped.

        Returns
        -------
        tuple[JobState, ...]
            One state per job node, in the order they were first started.
        """
        with self._lock:
            return tuple(job.as_state() for job in self._jobs.values())

    def results(self) -> dict[str, dict[str, Carried]]:
        """Return what each job has reported so far, to stand in for its run.

        Returns
        -------
        dict[str, dict[str, Carried]]
            The outputs by port name, for each job node that has reported.
        """
        with self._lock:
            return {
                node: dict(job.results)
                for node, job in self._jobs.items()
                if job.results
            }

    def drop(self, node: str) -> None:
        """Stop the job of `node` if it runs, and forget it and its results.

        Parameters
        ----------
        node
            The id of the job node.
        """
        with self._lock:
            job = self._jobs.pop(node, None)
        if job is not None:
            job.stop.set()

    def clear(self) -> None:
        """Do `drop` for every job."""
        with self._lock:
            jobs = list(self._jobs.values())
            self._jobs.clear()
        for job in jobs:
            job.stop.set()

    def _inputs(
        self, graph: Graph, placed: PlacedNode, function: type[JobNode[object]]
    ) -> dict[str, Carried]:
        """Run the nodes a job needs and read its inputs from them.

        Returns
        -------
        dict[str, Carried]
            The value of each input of the job node.

        Raises
        ------
        JobRefusedError
            If a node the job needs fails or is skipped, or an input has no
            value.
        """
        result = run_graph(
            graph,
            self._functions,
            raise_errors=False,
            only=ancestors(graph, placed.id),
            fixed=self.results(),
        )
        for outcome in result.outcomes:
            if outcome.status != "ok":
                why = outcome.error or "a node before it failed"
                msg = f"Node {outcome.node} did not run ({why})"
                raise JobRefusedError(msg)
        outputs = {o.node: {**o.outputs, **o.shown} for o in result.outcomes}
        incoming = [lk for lk in graph.links if lk.target.node == placed.id]
        try:
            return resolve_inputs(placed, incoming, function, outputs)
        except MissingInputError as error:
            raise JobRefusedError(str(error)) from error

    def _work(
        self,
        job: _Job,
        function: type[JobNode[object]],
        inputs: Mapping[str, Carried],
    ) -> None:
        """Run a job on its own thread, then record how it ended."""
        outcome: JobStateName = "finished"
        try:
            self._run(job, function, inputs)
        except Stopped:
            pass
        except ChildFailedError as error:
            job.error = error.reason
            outcome = "failed"
        except Exception as error:  # ruff: ignore[blind-except]  # job code can raise anything; the card shows it
            job.error = describe_error(error)
            outcome = "failed"
        with self._lock:
            if outcome != "failed" and job.stop.is_set():
                outcome = "stopped"
            job.state = outcome
            current = self._jobs.get(job.node) is job
        if current:
            self._emit(job, force=True)
            if outcome != "failed":
                self._on_end(job.node)

    def _run(
        self,
        job: _Job,
        function: type[JobNode[object]],
        inputs: Mapping[str, Carried],
    ) -> None:
        """Run the job's code on this thread."""
        sink = CallbackSink(
            begin=partial(self._set_steps, job),
            update=partial(self._update, job),
            advance=partial(self._set_step, job),
        )
        if function.isolated:
            run_in_child(function, inputs, job.stop, sink)
        else:
            execute(function, inputs, job.stop, sink)

    def _set_steps(self, job: _Job, total: int) -> None:
        """Record how many steps a job has and publish."""
        with self._lock:
            job.steps = total
        self._emit(job)

    def _set_step(self, job: _Job, step: int) -> None:
        """Record how many steps are done and publish."""
        with self._lock:
            job.step = step
        self._emit(job)

    def _update(self, job: _Job, fields: Mapping[str, Carried]) -> None:
        """Add new values to a job's results and publish."""
        with self._lock:
            job.results = _merged(job.results, fields)
        self._emit(job)

    def _emit(self, job: _Job, *, force: bool = False) -> None:
        """Publish a job's state now, or when the interval is up."""
        with self._lock:
            if self._jobs.get(job.node) is not job:
                return
            wait = job.last_sent + PUBLISH_INTERVAL - self._clock()
            late = not force and wait > 0
            arm = late and not job.flush_armed
            if late:
                job.dirty = True
                job.flush_armed = True
        if arm:
            self._schedule(wait, partial(self._flush, job))
        if not late:
            self._send(job)

    def _flush(self, job: _Job) -> None:
        """Publish a job's last state if nothing sent it since."""
        with self._lock:
            job.flush_armed = False
            due = job.dirty and self._jobs.get(job.node) is job
        if due:
            self._send(job)

    def _send(self, job: _Job) -> None:
        """Publish the job's state as it is now, in order with the others."""
        with self._send_lock:
            with self._lock:
                state = job.as_state()
                job.last_sent = self._clock()
                job.dirty = False
            self._publish(state)
