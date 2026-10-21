"""Run a job in a child process, so Stop can kill it and a crash stays there."""

import importlib
import importlib.machinery
import multiprocessing
import sys
import time
from collections.abc import Mapping
from queue import Empty
from typing import Protocol

from element_51.engine import describe_error
from element_51.job_exec import Sink, StopFlag, execute
from element_51.job_node import JobNode, Stopped
from element_51.model import Model
from element_51.visuals import Carried

KILL_AFTER = 0.5
"""Seconds after Stop before the child is killed if it has not ended."""

POLL = 0.05
"""Seconds the parent waits for a message before it looks at the child."""

type JobClass = type[JobNode[object]]


class Began(Model):
    """The job has this many steps."""

    total: int


class Updated(Model):
    """New values for some outputs."""

    fields: Mapping[str, Carried]


class Advanced(Model):
    """Steps are done: `step` of them."""

    step: int


class Ended(Model):
    """The job's code returned, or Stop ended it."""


class Failed(Model):
    """The job's code raised; `error` is worded for a person."""

    error: str


type ChildMessage = Began | Updated | Advanced | Ended | Failed


class ChildFailedError(Exception):
    """The job in the child process failed.

    Parameters
    ----------
    reason
        Why, worded for a person.
    """

    def __init__(self, reason: str) -> None:
        super().__init__(reason)
        self.reason = reason


class Outbox(Protocol):
    """Where the child puts its messages."""

    def put(self, message: ChildMessage) -> None:
        """Send a message to the parent."""


class _QueueSink:
    """Turns a job's progress into messages for the parent."""

    def __init__(self, outbox: Outbox) -> None:
        self._outbox = outbox

    def begin(self, total: int) -> None:
        """Send the number of steps."""
        self._outbox.put(Began(total=total))

    def update(self, fields: Mapping[str, Carried]) -> None:
        """Send new output values."""
        self._outbox.put(Updated(fields=dict(fields)))

    def advance(self, step: int) -> None:
        """Send the number of steps done."""
        self._outbox.put(Advanced(step=step))


def child_main(
    module: str,
    name: str,
    inputs: Mapping[str, Carried],
    outbox: Outbox,
    stop: StopFlag,
) -> None:
    """Import the job's class, run it and send what it does to `outbox`.

    This is the child process's entry point. It sends `Ended` when the job
    returns or Stop ends it, and `Failed` when the job's code or the import
    raises. A process killed from outside sends neither.

    Parameters
    ----------
    module
        The module that defines the class.
    name
        The class's name in that module.
    inputs
        The value of each input.
    outbox
        Where the messages go.
    stop
        The flag Stop sets.
    """
    try:
        function: JobClass = getattr(importlib.import_module(module), name)
        execute(function, inputs, stop, _QueueSink(outbox))
    except Stopped:
        outbox.put(Ended())
    except Exception as error:  # ruff: ignore[blind-except]  # job code can raise anything; the card shows it
        outbox.put(Failed(error=describe_error(error)))
    else:
        outbox.put(Ended())


def check_importable(function: type) -> str | None:
    """Say why a child process could not import a job's class.

    Parameters
    ----------
    function
        The job's class.

    Returns
    -------
    str | None
        The reason, or None if a fresh interpreter should find it.
    """
    name = function.__name__
    module = function.__module__
    if "<locals>" in function.__qualname__:
        return (
            f"{name} is defined inside a function, "
            "so a separate process cannot import it"
        )
    if module == "__main__":
        if getattr(sys.modules[module], "__file__", None) is None:
            return (
                f"{name} is defined in an interactive session (a notebook "
                "or a prompt), so a separate process cannot import it; "
                "define it in a script or a module"
            )
        return None
    if importlib.machinery.PathFinder.find_spec(module.split(".")[0], sys.path):
        return None
    return (
        f"{name} is in module {module}, "
        "which a separate process cannot find on the import path"
    )


def _apply(message: ChildMessage, sink: Sink) -> bool:
    """Pass a child's message to the sink.

    Returns
    -------
    bool
        Whether the message was the last one.

    Raises
    ------
    ChildFailedError
        If the message says the job's code raised.
    """
    if isinstance(message, Began):
        sink.begin(message.total)
    elif isinstance(message, Updated):
        sink.update(message.fields)
    elif isinstance(message, Advanced):
        sink.advance(message.step)
    elif isinstance(message, Failed):
        raise ChildFailedError(message.error)
    return isinstance(message, Ended)


def run_in_child(
    function: JobClass,
    inputs: Mapping[str, Carried],
    stop: StopFlag,
    sink: Sink,
) -> None:
    """Run a job in a child process and pass what it reports to `sink`.

    Blocks until the job ends. When `stop` is set the child is asked to end,
    and killed `KILL_AFTER` seconds later if it has not. A killed job counts
    as stopped.

    Parameters
    ----------
    function
        The job's class; the child imports it by module and name.
    inputs
        The value of each input, pickled to the child.
    stop
        The flag Stop sets in this process.
    sink
        Where the child's progress and outputs go.

    Raises
    ------
    ChildFailedError
        If the job's code raised, or the process ended without saying so.
    """
    context = multiprocessing.get_context("spawn")
    outbox = context.Queue()
    child_stop = context.Event()
    process = context.Process(
        target=child_main,
        args=(
            function.__module__,
            function.__name__,
            inputs,
            outbox,
            child_stop,
        ),
        daemon=True,
    )
    process.start()
    kill_at: float | None = None
    gone = False
    try:
        while True:
            if stop.is_set() and kill_at is None:
                child_stop.set()
                kill_at = time.monotonic() + KILL_AFTER
            if kill_at is not None and time.monotonic() >= kill_at:
                return
            try:
                message: ChildMessage = outbox.get(timeout=POLL)
            except Empty:
                if gone:
                    process.join()
                    msg = (
                        "The job's process ended without a result "
                        f"(exit code {process.exitcode})"
                    )
                    raise ChildFailedError(msg) from None
                gone = not process.is_alive()
                continue
            if _apply(message, sink):
                return
    finally:
        process.kill()
        process.join()
        outbox.close()
        outbox.cancel_join_thread()
