"""The live session: node types and the graph being edited."""

from collections.abc import Callable, Iterable

from element_51.edits import AddLink, AddNode, apply_edit
from element_51.engine import RunResult, run_graph
from element_51.model import Graph, PortRef, PortValue
from element_51.nodes import Registered
from element_51.protocol import Snapshot


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
        node_types = {
            name: function.node_type
            for name, function in self.node_functions.items()
        }
        self.graph = apply_edit(self.graph, node_types, edit)

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
