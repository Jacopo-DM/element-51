"""The live session: node types and the graph being edited."""

import threading
from collections.abc import Callable, Iterable
from pathlib import Path

from element_51.edits import AddLink, AddNode, AnyEdit, DeleteNode, apply_edit
from element_51.engine import RunResult, closure, descendants, run_graph
from element_51.jobs import JobRunner, JobState
from element_51.model import Graph, NodeType, PortRef
from element_51.nodes import Registered
from element_51.protocol import (
    JobUpdate,
    JobView,
    LinkedSeries,
    Snapshot,
    SourceReply,
)
from element_51.store import load_graph, save_graph
from element_51.theme import ThemeChanges, ThemeFile, resolve_theme
from element_51.visuals import Carried, Series

type JobEvent = JobUpdate | RunResult
"""What `App.subscribe_jobs` listeners hear: a job's update, or the outcomes of
the nodes after a job that has ended."""


class App:
    """Holds the node types and the authoritative copy of the graph.

    Every change and every snapshot takes one lock, so scripts running in
    several threads and the server can edit at the same time: ids are never
    reused, no edit overwrites another, and listeners hear the changes in the
    order they were made. Running the graph takes no lock, because a graph is
    immutable and a run reads it once; node functions run unlocked.

    Parameters
    ----------
    node_functions
        The node types the editor offers, in the order it lists them.

    Raises
    ------
    ValueError
        If two node functions share a name.
    """

    def __init__(self, node_functions: Iterable[Registered]) -> None:
        self.node_functions: dict[str, Registered] = {}
        for function in node_functions:
            name = function.node_type.name
            if name in self.node_functions:
                msg = f"two node types are named {name!r}"
                raise ValueError(msg)
            self.node_functions[name] = function
        self.graph = Graph(nodes=(), links=())
        self._listeners: list[Callable[[Snapshot], None]] = []
        self._theme = ThemeFile()
        self._lock = threading.RLock()
        self._job_listeners: list[Callable[[JobEvent], None]] = []
        self._jobs = JobRunner(
            self.node_functions, self._job_published, self._job_ended
        )

    def add_node(self, function: Registered, *, x: float, y: float) -> str:
        """Place a new node of a registered type.

        Parameters
        ----------
        function
            The node function, registered with this app.
        x
            Horizontal position on the canvas.
        y
            Vertical position on the canvas.

        Returns
        -------
        str
            The id of the new node.

        Raises
        ------
        ValueError
            If the function's type is not registered with this app.
        """
        name = function.node_type.name
        if self.node_functions.get(name) is not function:
            msg = f"node type {name!r} is not registered with this app"
            raise ValueError(msg)
        with self._lock:
            node_id = self.graph.next_id()
            self.apply(AddNode(id=node_id, node_type=name, x=x, y=y))
            self._changed()
        return node_id

    def apply(self, edit: AnyEdit) -> None:
        """Apply an edit from the browser, without telling listeners.

        An edit the rules refuse raises `EditError` and leaves the graph
        unchanged.

        Parameters
        ----------
        edit
            The edit to apply.
        """
        with self._lock:
            self.graph = apply_edit(self.graph, self._node_types(), edit)
            if isinstance(edit, DeleteNode):
                self._jobs.drop(edit.id)

    def add_link(
        self, source: str, source_port: str, target: str, target_port: str
    ) -> None:
        """Link an output port to an input port, by node id and port name.

        An edit the rules refuse raises `EditError`.

        Parameters
        ----------
        source
            Id of the node the link starts at.
        source_port
            Name of its output port.
        target
            Id of the node the link ends at.
        target_port
            Name of its input port.
        """
        with self._lock:
            self.apply(
                AddLink(
                    source=PortRef(node=source, port=source_port),
                    target=PortRef(node=target, port=target_port),
                )
            )
            self._changed()

    def save(self, path: Path | str) -> None:
        """Write the graph to a JSON file, replacing any file there.

        Parameters
        ----------
        path
            The file to write.
        """
        save_graph(self.graph, Path(path))

    def load(self, path: Path | str) -> None:
        """Replace the graph with one read from a JSON file.

        Open pages show the new graph at once. If the file cannot be loaded,
        `StoreError` is raised and the current graph stays as it is.

        Parameters
        ----------
        path
            The file to read.
        """
        graph = load_graph(Path(path), self._node_types())
        with self._lock:
            self.graph = graph
            self._jobs.clear()
            self._changed()

    def _node_types(self) -> dict[str, NodeType]:
        """Return every node type by name.

        Returns
        -------
        dict[str, NodeType]
            The node types this app offers.
        """
        return {
            name: function.node_type
            for name, function in self.node_functions.items()
        }

    @property
    def theme(self) -> str:
        """The name of the built-in theme the editor is drawn in."""
        return self._theme.base

    @theme.setter
    def theme(self, choice: str | Path) -> None:
        """Change the theme. Open pages are told at once.

        `choice` is a built-in theme name or the path of a theme file (see
        `element_51.theme`). Raises `ValueError`, naming the valid themes, for
        anything else.
        """
        theme = resolve_theme(choice)
        with self._lock:
            self._theme = theme
            self._changed()

    def subscribe(self, listener: Callable[[Snapshot], None]) -> None:
        """Call `listener` with a new snapshot after each change made here.

        Only changes made through `add_node` and `add_link` count. An edit
        applied with `apply` (the browser's route) is silent, because the
        server answers the sender itself and tells the other pages.

        Parameters
        ----------
        listener
            Called with the whole state after each change. It may be called
            from any thread, so it must not block.
        """
        with self._lock:
            self._listeners.append(listener)

    def _changed(self) -> None:
        """Tell every listener the state after a change."""
        with self._lock:
            snapshot = self.snapshot()
            for listener in self._listeners:
                listener(snapshot)

    def subscribe_jobs(self, listener: Callable[[JobEvent], None]) -> None:
        """Call `listener` as jobs change.

        It hears a `JobUpdate` after each change of a job, at most 20 a second
        for one job and at once when a job ends, and then, when a job finished
        or was stopped, a partial `RunResult` with the nodes after it run once.

        Parameters
        ----------
        listener
            Called from the job's thread, so it must not block.
        """
        with self._lock:
            self._job_listeners.append(listener)

    def start_job(self, node: str) -> None:
        """Start the job of a job node.

        The nodes it depends on run once first; its inputs are their values and
        the node's own boxes as they are now. Raises `JobRefusedError`, with a
        reason, if there is no such node, it is not a job, it is already
        running or stopping, or a node it depends on fails.

        Parameters
        ----------
        node
            The node's id.
        """
        with self._lock:
            graph = self.graph
        self._jobs.start(graph, node)

    def stop_job(self, node: str) -> None:
        """Ask the job of a node to end at its next step or report.

        Raises `JobRefusedError` if the node has no running job.

        Parameters
        ----------
        node
            The node's id.
        """
        self._jobs.stop(node)

    def job_states(self) -> tuple[JobView, ...]:
        """Describe every job that has been started.

        Returns
        -------
        tuple[JobView, ...]
            One view per job node that has a job.
        """
        return tuple(self._job_view(s) for s in self._jobs.states())

    def _job_view(self, state: JobState) -> JobView:
        """Add to a job's state the series that reach chart nodes from it.

        Returns
        -------
        JobView
            The state with its `linked` series.
        """
        with self._lock:
            graph = self.graph
            charts = {
                n.id
                for n in graph.nodes
                if self.node_functions[n.node_type].node_type.display == "chart"
            }
        linked = tuple(
            LinkedSeries(
                node=link.target.node, port=link.target.port, series=series
            )
            for link in graph.links
            if link.source.node == state.node
            and link.target.node in charts
            and isinstance(series := state.shown.get(link.source.port), Series)
        )
        return JobView(
            node=state.node,
            state=state.state,
            step=state.step,
            steps=state.steps,
            error=state.error,
            outputs=state.outputs,
            shown=state.shown,
            linked=linked,
        )

    def _job_published(self, state: JobState) -> None:
        """Tell the job listeners a job's update."""
        view = self._job_view(state)
        update = JobUpdate(**{
            name: getattr(view, name) for name in JobView.model_fields
        })
        with self._lock:
            listeners = tuple(self._job_listeners)
        for listener in listeners:
            listener(update)

    def _job_ended(self, node: str) -> None:
        """Run the nodes after an ended job once and tell the listeners."""
        with self._lock:
            graph = self.graph
            listeners = tuple(self._job_listeners)
        after = descendants(graph, node)
        if not after:
            return
        result = run_graph(
            graph,
            self.node_functions,
            raise_errors=False,
            only=closure(graph, after),
            fixed=self._jobs.results(),
        )
        shown = RunResult(
            outcomes=tuple(o for o in result.outcomes if o.node in after),
            partial=True,
        )
        for listener in listeners:
            listener(shown)

    def execute(self) -> RunResult:
        """Run the graph and report what happened to every node.

        Returns
        -------
        RunResult
            One outcome per node; nothing is raised for a failing node.
        """
        return run_graph(
            self.graph,
            self.node_functions,
            raise_errors=False,
            fixed=self._jobs.results(),
        )

    def run(self) -> dict[str, dict[str, Carried]]:
        """Run the graph and return every node's outputs.

        A node that fails raises its own exception, with a note naming the
        node.

        Returns
        -------
        dict[str, dict[str, Carried]]
            For each node id, the value of each of its output ports by name.
            A picture is an `Image`.
        """
        result = run_graph(
            self.graph,
            self.node_functions,
            raise_errors=True,
            fixed=self._jobs.results(),
        )
        return {
            outcome.node: {**outcome.outputs, **outcome.shown}
            for outcome in result.outcomes
        }

    def source(self, node: str) -> SourceReply:
        """Read the source code of the type of node `node`.

        Parameters
        ----------
        node
            The node's id.

        Returns
        -------
        SourceReply
            The code, or the reason there is none: the node is not in the
            graph, or its code cannot be read (a function made by `exec`, or a
            file that is gone).
        """
        with self._lock:
            placed = next((n for n in self.graph.nodes if n.id == node), None)
            function = (
                self.node_functions.get(placed.node_type) if placed else None
            )
        if placed is None or function is None:
            return SourceReply(node=node, reason=f"There is no node {node}")
        try:
            return SourceReply(node=node, source=function.source_text())
        except (OSError, TypeError) as error:
            reason = (
                f"The source of node type {placed.node_type} "
                f"cannot be read: {error}"
            )
            return SourceReply(node=node, reason=reason)

    def snapshot(self) -> Snapshot:
        """Describe the whole state for a browser that just connected.

        Returns
        -------
        Snapshot
            Every node type and the current graph.
        """
        with self._lock:
            return Snapshot(
                theme=self._theme.base,
                theme_changes=ThemeChanges(
                    colours=self._theme.colours,
                    ports=self._theme.ports,
                    metrics=self._theme.metrics,
                    page=self._theme.page,
                    fonts=self._theme.fonts,
                ),
                node_types=tuple(
                    f.node_type for f in self.node_functions.values()
                ),
                graph=self.graph,
                jobs=self.job_states(),
            )
