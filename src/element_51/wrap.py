"""Turn the functions of a module into node types with one call."""

import inspect
from types import ModuleType

from pydantic import ConfigDict, SkipValidation

from element_51.model import Model
from element_51.nodes import Node, NodeFunction, Registered, node


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
