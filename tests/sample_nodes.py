"""Node functions shared by the tests."""

from element_51.nodes import node


@node
def add(a: float, b: float) -> float:
    return a + b


@node
def show(value: float) -> float:
    return value
