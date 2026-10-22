"""A module with a mix of functions, for the scan tests."""

from typing import override

from sample_nodes import add as imported_add

from element_51.nodes import Node, node

THRESHOLD = 3.0
"""A value that is not a function."""

imported = imported_add


def zscore(
    x: float, mean: float, std: float
) -> float:  # pragma: no cover  # scanned only
    return (x - mean) / std


def shout(
    text: str, *, loud: bool = False
) -> str:  # pragma: no cover  # scanned only
    return text.upper() if loud else text


def untyped(a, b) -> float:  # type: ignore[no-untyped-def]  # the missing hints are the test  # ruff: ignore[missing-type-function-argument]  # same  # pragma: no cover
    return float(a is b)


def spread(
    *values: float,
) -> float:  # pragma: no cover  # scanned only
    return max(values) - min(values)


def positional(a: float, /) -> float:  # pragma: no cover  # scanned only
    return a


def _hidden(x: float) -> float:  # pragma: no cover  # scanned only
    return x


plus = zscore


@node(category="Own")
def doubled(x: float) -> float:  # pragma: no cover  # scanned only
    return x * 2


class Greeter(Node):
    """A node written as a class."""

    name: str = "world"

    @override
    def run(self) -> str:  # pragma: no cover  # scanned only
        return f"hello {self.name}"


class Helper:
    """A class that is not a node."""
