"""Tests for the live session."""

import sys
import threading
from collections.abc import Callable, Generator
from pathlib import Path
from typing import TYPE_CHECKING, override

import pytest
from sample_jobs import FIRST_TOTAL, WAIT, Pause, Steps
from sample_nodes import add, show

from element_51.edits import AddLink, AddNode, DeleteNode, EditError
from element_51.engine import NO_OUTPUT, RunResult
from element_51.jobs import JobRefusedError, JobState
from element_51.model import Link, PortRef, PortType
from element_51.nodes import (
    Node,
    NodeFunction,
    PlotNode,
    Registered,
    node,
)
from element_51.session import App

if TYPE_CHECKING:
    from element_51.protocol import Snapshot


def test_snapshot_lists_node_types_in_registration_order() -> None:
    """The browser gets every type, whether or not a node uses it."""
    app = App([add, show])
    snapshot = app.snapshot()
    assert [t.name for t in snapshot.node_types] == [
        "Plot",
        "add",
        "show",
    ]
    assert snapshot.node_types[1].outputs[0].kind is PortType.FLOAT
    assert snapshot.graph.nodes == ()


def test_add_node_places_a_node_with_a_fresh_id() -> None:
    """Ids count up from n1 and positions are kept."""
    app = App([add, show])
    first = app.add_node(add, x=10.0, y=20.0)
    second = app.add_node(show, x=30.0, y=40.0)
    assert (first, second) == ("n1", "n2")
    nodes = app.graph.nodes
    assert [(n.id, n.node_type, n.x, n.y) for n in nodes] == [
        ("n1", "add", 10.0, 20.0),
        ("n2", "show", 30.0, 40.0),
    ]


def test_snapshot_includes_the_graph() -> None:
    """A snapshot taken after an edit shows the edit."""
    app = App([add])
    app.add_node(add, x=1.0, y=2.0)
    assert app.snapshot().graph == app.graph
    assert len(app.snapshot().graph.nodes) == 1


def test_adding_a_node_of_an_unregistered_type_is_refused() -> None:
    """A node needs a type the browser knows about."""
    app = App([add])
    with pytest.raises(ValueError, match="show"):
        app.add_node(show, x=0.0, y=0.0)


def make_clashing_add() -> Registered:
    """Build a second node function that is also called `add`.

    Returns
    -------
    Registered
        The new node function.
    """

    @node
    def add(a: float, b: float) -> float:
        return a - b

    expected = 2.0
    assert add(3.0, 1.0) == expected  # a different function, same name
    return add


def test_two_node_types_with_one_name_are_refused() -> None:
    """The name is the identity of a type."""
    with pytest.raises(ValueError, match="add"):
        App([add, make_clashing_add()])


def test_apply_changes_the_graph_and_rejects_bad_edits() -> None:
    """`apply` is the one way an edit, from a script or a browser, lands."""
    app = App([add, show])
    app.apply(AddNode(id="n1", node_type="add", x=0.0, y=0.0))
    app.apply(AddNode(id="n2", node_type="show", x=9.0, y=9.0))
    link = AddLink(
        source=PortRef(node="n1", port="out"),
        target=PortRef(node="n2", port="value"),
    )
    app.apply(link)
    expected_nodes = 2
    assert len(app.graph.nodes) == expected_nodes
    assert app.graph.links == (Link(source=link.source, target=link.target),)
    with pytest.raises(EditError):
        app.apply(link)
    assert len(app.graph.links) == 1


@node
def one() -> float:
    return 1.0


@node
def divide(a: float, b: float = 0.0) -> float:
    return a / b


def test_add_link_connects_two_ports_of_existing_nodes() -> None:
    """The script-friendly way to link nodes."""
    app = App([one, show])
    source = app.add_node(one, x=0.0, y=0.0)
    target = app.add_node(show, x=1.0, y=1.0)
    app.add_link(source, "out", target, "value")
    assert app.graph.links[0].target == PortRef(node=target, port="value")
    with pytest.raises(EditError):
        app.add_link(source, "out", target, "value")


def test_run_returns_every_nodes_value_by_id() -> None:
    """The computed values, keyed by node id."""
    app = App([one, show])
    source = app.add_node(one, x=0.0, y=0.0)
    target = app.add_node(show, x=1.0, y=1.0)
    app.add_link(source, "out", target, "value")
    assert app.run() == {source: {"out": 1.0}, target: {"out": 1.0}}


def test_run_raises_the_nodes_own_error() -> None:
    """A node that raises makes `run` raise the same error."""
    app = App([one, divide])
    source = app.add_node(one, x=0.0, y=0.0)
    target = app.add_node(divide, x=1.0, y=1.0)
    app.add_link(source, "out", target, "a")
    with pytest.raises(ZeroDivisionError):
        app.run()


def test_execute_reports_instead_of_raising() -> None:
    """The browser's run: every node gets an outcome, nothing raises."""
    app = App([one, divide])
    source = app.add_node(one, x=0.0, y=0.0)
    target = app.add_node(divide, x=1.0, y=1.0)
    app.add_link(source, "out", target, "a")
    statuses = [o.status for o in app.execute().outcomes]
    assert statuses == ["ok", "error"]


def test_script_edits_tell_every_subscriber_the_new_snapshot() -> None:
    """`add_node` and `add_link` call each listener with the new state."""
    app = App([add, show])
    heard: list[Snapshot] = []
    app.subscribe(heard.append)
    first = app.add_node(add, x=0.0, y=0.0)
    second = app.add_node(show, x=1.0, y=1.0)
    app.add_link(first, "out", second, "value")
    assert [len(s.graph.nodes) for s in heard] == [1, 2, 2]
    assert [len(s.graph.links) for s in heard] == [0, 0, 1]
    assert heard[-1] == app.snapshot()


def test_a_refused_edit_tells_nobody() -> None:
    """A listener hears about changes, not about attempts."""
    app = App([add])
    heard: list[Snapshot] = []
    app.subscribe(heard.append)
    with pytest.raises(EditError):
        app.add_link("n1", "out", "n2", "a")
    assert heard == []


def test_apply_is_silent() -> None:
    """The browser's route, `apply`, is silent; the server tells the others."""
    app = App([add])
    heard: list[Snapshot] = []
    app.subscribe(heard.append)
    app.apply(AddNode(id="n1", node_type="add", x=0.0, y=0.0))
    assert heard == []


def test_the_theme_starts_as_the_default_and_reaches_the_snapshot() -> None:
    """A new app uses the purple-synthwave theme."""
    app = App([add])
    assert app.theme == "purple-synthwave"
    assert app.snapshot().theme == "purple-synthwave"


def test_setting_the_theme_tells_listeners_and_changes_the_snapshot() -> None:
    """An open page is told at once, in the same way as for an edit."""
    app = App([add])
    heard: list[Snapshot] = []
    app.subscribe(heard.append)
    app.theme = "antimony-light"
    assert app.theme == "antimony-light"
    assert [s.theme for s in heard] == ["antimony-light"]


def test_an_unknown_theme_is_refused_and_changes_nothing() -> None:
    """The message lists the valid names; the old theme stays."""
    app = App([add])
    with pytest.raises(ValueError, match="antimony-light"):
        app.theme = "dark"
    assert app.theme == "purple-synthwave"


def test_a_theme_file_changes_reach_the_snapshot(tmp_path: Path) -> None:
    """The base name stays the theme; the file's values ride along."""
    path = tmp_path / "t.json"
    path.write_text('{"fonts": {"value": "14px Menlo"}}', encoding="utf-8")
    app = App([add])
    heard: list[Snapshot] = []
    app.subscribe(heard.append)
    app.theme = path
    assert app.theme == "purple-synthwave"
    assert heard[0].theme_changes.fonts == {"value": "14px Menlo"}
    assert app.snapshot().theme_changes.fonts == {"value": "14px Menlo"}
    app.theme = "antimony-light"
    assert not app.snapshot().theme_changes.fonts


THREADS = 8
NODES_PER_THREAD = 60


def _hammer(work: Callable[[int], None]) -> None:
    """Run `work(thread_number)` in several threads started together."""
    start = threading.Barrier(THREADS)

    def body(number: int) -> None:
        start.wait()
        work(number)

    threads = [
        threading.Thread(target=body, args=(number,))
        for number in range(THREADS)
    ]
    for thread in threads:
        thread.start()
    for thread in threads:
        thread.join()


@pytest.fixture
def fast_switching() -> Generator[None]:
    """Make Python switch threads as often as it can, to provoke races.

    Yields
    ------
    None
        Nothing; the interval is put back afterwards.
    """
    before = sys.getswitchinterval()
    sys.setswitchinterval(1e-6)
    yield
    sys.setswitchinterval(before)


@pytest.mark.usefixtures("fast_switching")
def test_threads_adding_nodes_get_distinct_ids_and_lose_none() -> None:
    """`add_node` picks an id and applies the edit as one step."""
    app = App([add])
    ids: list[str] = []

    def work(_: int) -> None:
        ids.extend(
            app.add_node(add, x=0.0, y=0.0) for _ in range(NODES_PER_THREAD)
        )

    _hammer(work)
    total = THREADS * NODES_PER_THREAD
    assert len(ids) == len(set(ids)) == total
    assert len(app.graph.nodes) == total


@pytest.mark.usefixtures("fast_switching")
def test_edits_from_several_threads_do_not_overwrite_each_other() -> None:
    """`apply`, the browser's route, replaces the graph in one step."""
    app = App([add])

    def work(number: int) -> None:
        for index in range(NODES_PER_THREAD):
            node_id = f"n{number * NODES_PER_THREAD + index + 1}"
            app.apply(AddNode(id=node_id, node_type="add", x=0.0, y=0.0))

    _hammer(work)
    assert len(app.graph.nodes) == THREADS * NODES_PER_THREAD


@pytest.mark.usefixtures("fast_switching")
def test_listeners_hear_the_changes_in_the_order_they_were_made() -> None:
    """A later snapshot never arrives before an earlier one."""
    app = App([add])
    sizes: list[int] = []
    app.subscribe(lambda snapshot: sizes.append(len(snapshot.graph.nodes)))

    def work(_: int) -> None:
        for _ in range(NODES_PER_THREAD):
            app.add_node(add, x=0.0, y=0.0)

    _hammer(work)
    assert sizes == sorted(sizes)
    assert len(sizes) == THREADS * NODES_PER_THREAD


class Doubler(Node):
    x: float = 1.0

    @override
    def run(self) -> float:  # pragma: no cover  # only its source is read
        return self.x * 2


def test_the_source_of_a_function_node_is_its_code() -> None:
    """The reply holds the function as written, decorator included."""
    app = App([add])
    node_id = app.add_node(add, x=0.0, y=0.0)
    reply = app.source(node_id)
    assert reply.node == node_id
    assert reply.reason is None
    assert reply.source is not None
    assert reply.source.startswith("@node\ndef add(a: float, b: float)")
    assert "return a + b" in reply.source


def test_the_source_of_a_class_node_is_its_class() -> None:
    """A `Node` subclass shows the whole class."""
    app = App([Doubler])
    node_id = app.add_node(Doubler, x=0.0, y=0.0)
    reply = app.source(node_id)
    assert reply.source is not None
    assert reply.source.startswith("class Doubler(Node):")
    assert "return self.x * 2" in reply.source


def test_the_source_of_the_plot_node_is_its_class() -> None:
    """The built-in node can be read like any other."""
    app = App([])
    node_id = app.add_node(PlotNode, x=0.0, y=0.0)
    reply = app.source(node_id)
    assert reply.source is not None
    assert "class PlotNode" in reply.source


def test_source_inside_a_function_is_dedented() -> None:
    """A node defined in a function still reads from the left margin."""

    @node
    def inner(a: float) -> float:  # pragma: no cover  # only its source is read
        return a

    app = App([inner])
    node_id = app.add_node(inner, x=0.0, y=0.0)
    reply = app.source(node_id)
    assert reply.source is not None
    assert reply.source.startswith("@node\ndef inner")


def test_an_unreadable_source_says_why_and_shows_no_code() -> None:
    """A function made by `exec` has no file to read."""
    namespace: dict[str, object] = {"node": node}
    exec("@node\ndef made(a: float) -> float:\n    return a", namespace)  # ruff: ignore[exec-builtin]  # a node with no source file is the case under test
    made = namespace["made"]
    assert isinstance(made, NodeFunction)
    app = App([made])
    node_id = app.add_node(made, x=0.0, y=0.0)
    reply = app.source(node_id)
    assert reply.source is None
    assert reply.reason is not None
    assert "made" in reply.reason
    assert "cannot be read" in reply.reason


def test_the_source_of_a_node_that_is_not_there_says_so() -> None:
    """The browser may ask after another page deleted the node."""
    reply = App([add]).source("n9")
    assert reply.source is None
    assert reply.reason == "There is no node n9"


def job_app() -> tuple[App, list[JobState], list[RunResult], threading.Event]:
    """Build an app with a step job feeding `show`, and listen to its jobs.

    Returns
    -------
    tuple[App, list[JobState], list[RunResult], threading.Event]
        The app, the states and run results heard, and an event set once the
        run after the job has been heard.
    """
    app = App([Steps, show])
    app.add_node(Steps, x=0.0, y=0.0)
    app.add_node(show, x=200.0, y=0.0)
    app.add_link("n1", "total", "n2", "value")
    states: list[JobState] = []
    runs: list[RunResult] = []
    heard = threading.Event()

    def listener(event: JobState | RunResult) -> None:
        if isinstance(event, JobState):
            states.append(event)
        else:
            runs.append(event)
            heard.set()

    app.subscribe_jobs(listener)
    return app, states, runs, heard


def test_a_plain_run_skips_a_job_that_has_not_run_and_what_follows() -> None:
    """Run never starts a job, so a job node has no output until Start."""
    app, _, _, _ = job_app()
    result = app.execute()
    by_node = {o.node: o for o in result.outcomes}
    assert by_node["n1"].status == "skipped"
    assert by_node["n1"].error == NO_OUTPUT
    assert by_node["n2"].status == "skipped"
    with pytest.raises(ValueError, match="no output yet"):
        app.run()


def test_start_job_runs_it_then_the_nodes_after_it_once() -> None:
    """Listeners hear the states, then the nodes after the job run once."""
    app, states, runs, heard = job_app()
    app.start_job("n1")
    assert heard.wait(WAIT)
    assert states[0].state == "running"
    assert states[-1].state == "finished"
    assert [o.node for o in runs[0].outcomes] == ["n2"]
    assert runs[0].outcomes[0].outputs == {"out": FIRST_TOTAL + 2}
    assert [s.state for s in app.job_states()] == ["finished"]


def test_a_plain_run_after_a_job_uses_its_latest_values() -> None:
    """Run and `run` take a job's outputs as if it had run."""
    app, _, _, heard = job_app()
    app.start_job("n1")
    assert heard.wait(WAIT)
    by_node = {o.node: o for o in app.execute().outcomes}
    assert by_node["n1"].outputs == {"total": FIRST_TOTAL + 2}
    assert by_node["n2"].outputs == {"out": FIRST_TOTAL + 2}
    assert app.run()["n2"]["out"] == FIRST_TOTAL + 2


def test_stop_job_stops_it_and_the_nodes_after_it_still_run() -> None:
    """A stopped job leaves partial results; the run after it still happens."""
    app, states, runs, heard = job_app()
    Pause.reset()
    Steps.pause_at = 1
    try:
        app.start_job("n1")
        assert Pause.entered.wait(WAIT)
        app.stop_job("n1")
        Pause.release.set()
        assert heard.wait(WAIT)
    finally:
        Steps.pause_at = -1
    assert states[-1].state == "stopped"
    assert runs[0].outcomes[0].outputs == {"out": FIRST_TOTAL + 1}


def test_start_and_stop_are_refused_with_a_reason() -> None:
    """Refusals come as `JobRefusedError`, whose reason is for the person."""
    app, _, _, _ = job_app()
    with pytest.raises(JobRefusedError, match="not a job"):
        app.start_job("n2")
    with pytest.raises(JobRefusedError, match="no running job"):
        app.stop_job("n1")


def test_deleting_a_job_node_forgets_its_job() -> None:
    """A deleted node leaves no job state and no results."""
    app, _, _, heard = job_app()
    app.start_job("n1")
    assert heard.wait(WAIT)
    app.apply(DeleteNode(id="n1"))
    assert app.job_states() == ()
    assert {o.node for o in app.execute().outcomes} == {"n2"}


def test_loading_a_graph_clears_every_job(tmp_path: Path) -> None:
    """Jobs belong to the graph that was replaced."""
    app, _, _, heard = job_app()
    app.start_job("n1")
    assert heard.wait(WAIT)
    path = tmp_path / "graph.json"
    app.save(path)
    app.load(path)
    assert app.job_states() == ()


def test_a_job_with_nothing_after_it_sends_no_run() -> None:
    """When no node follows, the end of the job is only a state."""
    app = App([Steps])
    app.add_node(Steps, x=0.0, y=0.0)
    heard: list[JobState | RunResult] = []
    finished = threading.Event()

    def listener(event: JobState | RunResult) -> None:
        heard.append(event)
        if isinstance(event, JobState) and event.state == "finished":
            finished.set()

    app.subscribe_jobs(listener)
    app.start_job("n1")
    assert finished.wait(WAIT)
    assert all(isinstance(event, JobState) for event in heard)
