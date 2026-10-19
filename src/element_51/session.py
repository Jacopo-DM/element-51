"""The live session: node types and the graph being edited."""

from collections.abc import Iterable

from element_51.model import Graph, Node
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
        node = Node(id=self.graph.next_id(), node_type=name, x=x, y=y)
        self.graph = Graph(
            nodes=(*self.graph.nodes, node), links=self.graph.links
        )
        return node.id

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
