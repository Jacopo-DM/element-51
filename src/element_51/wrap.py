"""Turn the functions of a module into node types with one call."""

import inspect
from collections.abc import Callable, Mapping, Sequence
from types import ModuleType
from typing import ClassVar, Protocol, cast

from pydantic import BaseModel, ConfigDict, SkipValidation

from element_51.model import Model, NodeType, Port, PortType, PortValue
from element_51.nodes import (
    Node,
    NodeFunction,
    Registered,
    check_category,
    node,
)

INPUT_KINDS = {
    "float64": PortType.FLOAT,
    "int64": PortType.INT,
    "bool": PortType.BOOL,
}
"""The loops a ufunc node can use, by the numpy name of each input's type."""

OUTPUT_KINDS = {
    "f": PortType.FLOAT,
    "i": PortType.INT,
    "u": PortType.INT,
    "b": PortType.BOOL,
}
"""The port type of a loop's output, by numpy's one-letter kind."""

PLAIN: dict[PortType, Callable[[float], PortValue]] = {
    PortType.FLOAT: float,
    PortType.INT: int,
    PortType.BOOL: bool,
}
"""Turn a number into the plain Python value of a port type."""


class Ufunc(Protocol):
    """What the scan reads from a numpy ufunc (numpy is loaded on first use)."""

    __name__: str

    @property
    def nin(self) -> int:
        """The number of inputs."""
        ...

    @property
    def nout(self) -> int:
        """The number of outputs."""
        ...

    @property
    def types(self) -> list[str]:
        """The loops, as numpy writes them: `dd->d`."""
        ...

    @property
    def signature(self) -> str | None:
        """The core signature of a generalised ufunc, else None."""
        ...

    def __call__(self, *args: PortValue) -> object:
        """Call the ufunc on numbers.

        Parameters
        ----------
        *args
            One number per input.
        """
        ...


class Skipped(Model):
    """A function the scan left out, and why."""

    name: str
    reason: str


class Scan(Model):
    """What a scan found: the node types, and the functions it skipped."""

    model_config = ConfigDict(
        frozen=True, extra="forbid", arbitrary_types_allowed=True
    )

    nodes: SkipValidation[tuple[Registered, ...]]
    skipped: tuple[Skipped, ...]


def _public(module: ModuleType, name: str) -> bool:
    """Say whether the scan looks at `name`.

    Returns
    -------
    bool
        True if the module lists it in `__all__`, or has no `__all__` and the
        name does not start with an underscore.
    """
    listed = getattr(module, "__all__", None)
    if listed is not None:
        return name in listed
    return not name.startswith("_")


def _is_ready_node(member: object) -> bool:
    """Say whether `member` is already a node: a decorated function or class.

    Returns
    -------
    bool
        True for a `NodeFunction` and for a `Node` subclass that has a type.
    """
    if isinstance(member, NodeFunction):
        return True
    return (
        isinstance(member, type)
        and issubclass(member, Node)
        and hasattr(member, "node_type")
    )


def _defined_in(member: object, module: ModuleType) -> bool:
    """Say whether `member` was written in `module` and not imported.

    Returns
    -------
    bool
        True if the function or class names `module` as its home.
    """
    target = member.func if isinstance(member, NodeFunction) else member
    return getattr(target, "__module__", None) == module.__name__


def nodes_from_module(module: ModuleType, category: str = "") -> Scan:
    """Make a node type from every fitting function of a module.

    The module is scanned in the order its names were defined. A plain
    function with the hints `@node` needs becomes a node, in `category`. A
    function already made a node with `@node`, and a `Node` subclass, join
    the result unchanged. Names that start with an underscore, names the
    module imported, names outside its `__all__` and anything that is not a
    function or node class are left out, and a second name for the same
    function is ignored. A function that does not fit is not an error: it
    comes back in `skipped` with the reason.

    Parameters
    ----------
    module
        The module to scan.
    category
        Where the new nodes go in the add-node menu, as for `node`.

    Returns
    -------
    Scan
        The nodes, and the functions that were skipped.
    """
    nodes: dict[str, Registered] = {}
    skipped: list[Skipped] = []
    seen: set[int] = set()
    for name, member in vars(module).items():
        if not _public(module, name) or not _defined_in(member, module):
            continue
        if id(member) in seen:
            continue  # a second name for a function already scanned
        seen.add(id(member))
        if _is_ready_node(member):
            ready: Registered = member
        elif inspect.isfunction(member):
            try:
                ready = node(category=category)(member)
            except TypeError as error:
                skipped.append(Skipped(name=name, reason=str(error)))
                continue
        else:
            continue
        taken = ready.node_type.name
        if taken in nodes:
            skipped.append(
                Skipped(
                    name=name,
                    reason=f"another function is already named {taken!r}",
                )
            )
            continue
        nodes[taken] = ready
    return Scan(nodes=tuple(nodes.values()), skipped=tuple(skipped))


def _plain(kind: PortType, value: object) -> PortValue:
    """Turn a numpy scalar, or a Python number, into the port's own type.

    Parameters
    ----------
    kind
        The port type: float, int or bool.
    value
        A number numpy returned, or a value that came in on a port.

    Returns
    -------
    PortValue
        The value as a plain `float`, `int` or `bool`.

    Raises
    ------
    TypeError
        If `value` is not a number.
    """
    import numpy as np  # ruff: ignore[import-outside-top-level]  # loaded on first use, as in figures.py

    scalar = value.item() if isinstance(value, np.generic) else value
    if isinstance(scalar, bool | int | float):
        return PLAIN[kind](scalar)
    msg = f"{type(value).__name__} is not a number"
    raise TypeError(msg)


class UfuncOutputs(BaseModel):
    """The outputs of a ufunc with more than one: `out1`, `out2`, ...

    One class serves every ufunc, so its fields are extras, set by name
    (a pydantic model made at run time is not typed). The engine reads them
    with `getattr`.
    """

    model_config = ConfigDict(frozen=True, extra="allow")


class UfuncNode:
    """A numpy ufunc as a node: one input per argument, one output per result.

    Made by `nodes_from_ufuncs`. The inputs and outputs take the types of one
    of the ufunc's documented loops, and every value is turned into that
    type before the call and back into a plain Python value after it.
    """

    defaults: ClassVar[Mapping[str, PortValue]] = {}

    def __init__(
        self,
        ufunc: Ufunc,
        node_type: NodeType,
        output_model: type[BaseModel] | None,
    ) -> None:
        self.ufunc = ufunc
        self.node_type = node_type
        self.output_model = output_model

    def call(self, inputs: Mapping[str, object]) -> object:
        """Call the ufunc with every input by name.

        Parameters
        ----------
        inputs
            One value per input.

        Returns
        -------
        object
            A plain number, or for two outputs the result model.
        """
        arguments = [
            _plain(port.kind, inputs[port.name])
            for port in self.node_type.inputs
        ]
        import numpy as np  # ruff: ignore[import-outside-top-level]  # loaded on first use

        with np.errstate(all="ignore"):
            result = self.ufunc(*arguments)
        kinds = [port.kind for port in self.node_type.outputs]
        if self.output_model is None:
            return _plain(kinds[0], result)
        parts = cast("Sequence[object]", result)
        values = {
            port.name: _plain(port.kind, value)
            for port, value in zip(self.node_type.outputs, parts, strict=True)
        }
        return UfuncOutputs(**values)

    def source_text(self) -> str:
        """Show how the ufunc is called, since it has no source file.

        Returns
        -------
        str
            For example `numpy.hypot(x1, x2)`.
        """
        names = ", ".join(port.name for port in self.node_type.inputs)
        return f"numpy.{self.ufunc.__name__}({names})"


def _loop(
    ufunc: Ufunc,
) -> tuple[tuple[PortType, ...], tuple[PortType, ...]] | None:
    """Pick the loop of a ufunc that becomes its ports.

    A loop is usable when every input is float64, int64 or bool and every
    output is a float, an integer or a bool. Of those, the first with all
    inputs float64 wins, then the first with all inputs int64, then the first
    left.

    Returns
    -------
    tuple[tuple[PortType, ...], tuple[PortType, ...]] | None
        The input and output port types, or None if no loop is usable.
    """
    import numpy as np  # ruff: ignore[import-outside-top-level]  # loaded on first use

    usable: list[tuple[list[str], list[str]]] = []
    for signature in ufunc.types:
        left, _, right = signature.partition("->")
        inputs: list[str] = [np.dtype(letter).name for letter in left]
        outputs: list[str] = [np.dtype(letter).kind for letter in right]
        if all(name in INPUT_KINDS for name in inputs) and all(
            kind in OUTPUT_KINDS for kind in outputs
        ):
            usable.append((inputs, outputs))
    if not usable:
        return None

    def rank(loop: tuple[list[str], list[str]]) -> int:
        inputs = loop[0]
        if all(name == "float64" for name in inputs):
            return 0
        return 1 if all(name == "int64" for name in inputs) else 2

    best_in, best_out = min(usable, key=rank)
    return (
        tuple(INPUT_KINDS[name] for name in best_in),
        tuple(OUTPUT_KINDS[kind] for kind in best_out),
    )


def _ufunc_node(
    ufunc: Ufunc,
    loop: tuple[tuple[PortType, ...], tuple[PortType, ...]],
    category: str,
) -> UfuncNode:
    """Build the node for a ufunc from the loop chosen for it.

    Returns
    -------
    UfuncNode
        The node, with `x` or `x1`, `x2`, ... as inputs and `out` or `out1`,
        `out2`, ... as outputs.
    """
    in_kinds, out_kinds = loop
    in_names = (
        ["x"] if ufunc.nin == 1 else [f"x{n}" for n in range(1, ufunc.nin + 1)]
    )
    out_names = (
        ["out"]
        if ufunc.nout == 1
        else [f"out{n}" for n in range(1, ufunc.nout + 1)]
    )
    outputs = tuple(
        Port(name=name, kind=kind)
        for name, kind in zip(out_names, out_kinds, strict=True)
    )
    node_type = NodeType(
        name=ufunc.__name__,
        inputs=tuple(
            Port(name=name, kind=kind)
            for name, kind in zip(in_names, in_kinds, strict=True)
        ),
        outputs=outputs,
        category=category,
    )
    model = UfuncOutputs if ufunc.nout > 1 else None
    return UfuncNode(ufunc, node_type, model)


def nodes_from_ufuncs(namespace: ModuleType, category: str = "") -> Scan:
    """Make a node type from every numpy ufunc in a namespace.

    A ufunc describes itself: `nin` and `nout` give the number of inputs and
    outputs, and `types` lists the loops it has, which give the port types
    (see `UfuncNode`). A ufunc with two names, such as `acos` and `arccos`,
    gives one node, named by the ufunc itself. A ufunc that works on whole
    arrays (`matmul`) or has no float, int or bool loop (`isnat`) is not an
    error: it comes back in `skipped` with the reason.

    Parameters
    ----------
    namespace
        A module such as `numpy`, or any other that holds ufuncs.
    category
        Where the new nodes go in the add-node menu, as for `node`.

    Returns
    -------
    Scan
        The nodes, and the ufuncs that were skipped.
    """
    import numpy as np  # ruff: ignore[import-outside-top-level]  # loaded on first use

    check_category(category, "nodes_from_ufuncs")
    nodes: list[Registered] = []
    skipped: list[Skipped] = []
    seen: set[int] = set()
    for member in vars(namespace).values():
        if not isinstance(member, np.ufunc) or id(member) in seen:
            continue
        seen.add(id(member))
        ufunc = cast("Ufunc", member)
        name = ufunc.__name__
        if ufunc.signature is not None:
            reason = "takes whole arrays, not single numbers"
            skipped.append(Skipped(name=name, reason=reason))
            continue
        loop = _loop(ufunc)
        if loop is None:
            reason = "has no float, int or bool loop"
            skipped.append(Skipped(name=name, reason=reason))
            continue
        nodes.append(_ufunc_node(ufunc, loop, category))
    return Scan(nodes=tuple(nodes), skipped=tuple(skipped))
