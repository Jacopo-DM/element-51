"""The live session: node types and the graph being edited."""

from collections.abc import Callable, Iterable
from pathlib import Path

from element_51.edits import AddLink, AddNode, apply_edit
from element_51.engine import RunResult, run_graph
from element_51.model import Graph, NodeType, PortRef, PortValue
from element_51.nodes import Registered
from element_51.protocol import Snapshot
from element_51.store import load_graph, save_graph


class App:
    """Holds the node types and the authoritative copy of the graph.

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
        node_id = self.graph.next_id()
        self.apply(AddNode(id=node_id, node_type=name, x=x, y=y))
        self._changed()
        return node_id

    def apply(self, edit: AddNode | AddLink) -> None:
        """Apply an edit from the browser, without telling listeners.

        An edit the rules refuse raises `EditError` and leaves the graph
        unchanged.

        Parameters
        ----------
        edit
            The edit to apply.
        """
        self.graph = apply_edit(self.graph, self._node_types(), edit)

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
        self.graph = load_graph(Path(path), self._node_types())
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
        self._listeners.append(listener)

    def _changed(self) -> None:
        """Tell every listener the state after a change."""
        snapshot = self.snapshot()
        for listener in self._listeners:
            listener(snapshot)

    def execute(self) -> RunResult:
        """Run the graph and report what happened to every node.

        Returns
        -------
        RunResult
            One outcome per node; nothing is raised for a failing node.
        """
        return run_graph(self.graph, self.node_functions, raise_errors=False)

    def run(self) -> dict[str, dict[str, PortValue]]:
        """Run the graph and return every node's outputs.

        A node that fails raises its own exception, with a note naming the
        node.

        Returns
        -------
        dict[str, dict[str, PortValue]]
            For each node id, the value of each of its output ports by name.
        """
        result = run_graph(self.graph, self.node_functions, raise_errors=True)
        return {
            outcome.node: dict(outcome.outputs) for outcome in result.outcomes
        }

    def snapshot(self) -> Snapshot:
        """Describe the whole state for a browser that just connected.

        Returns
        -------
        Snapshot
            Every node type and the current graph.
        """
        return Snapshot(
            node_types=tuple(f.node_type for f in self.node_functions.values()),
            graph=self.graph,
        )
