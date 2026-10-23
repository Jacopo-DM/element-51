"""The input nodes: a value the user sets, passed on as it is."""

from typing import override

from element_51.nodes import Node

GROUP = "Input nodes"


class Float(Node):
    """A number you set. Link it to as many inputs as you like."""

    category = GROUP
    value: float = 0.0

    @override
    def run(self) -> float:
        return self.value


class Integer(Node):
    """A whole number you set."""

    category = GROUP
    value: int = 0

    @override
    def run(self) -> int:
        return self.value


class Text(Node):
    """A text you set."""

    category = GROUP
    value: str = ""

    @override
    def run(self) -> str:
        return self.value


class Flag(Node):
    """A tick box you set."""

    category = GROUP
    value: bool = False

    @override
    def run(self) -> bool:
        return self.value


class Slider(Node):
    """A number you move between a minimum and a maximum.

    Raises
    ------
    ValueError
        From `run`, if `minimum` is not below `maximum`, or `value` is outside
        them. The card shows the failure; nothing is clamped.
    """

    category = GROUP
    card = "slider"
    value: float = 0.0
    minimum: float = -10.0
    maximum: float = 10.0

    @override
    def run(self) -> float:
        if not self.minimum < self.maximum:
            msg = (
                f"the minimum {self.minimum} must be below "
                f"the maximum {self.maximum}"
            )
            raise ValueError(msg)
        if not self.minimum <= self.value <= self.maximum:
            msg = (
                f"the value {self.value} must be between "
                f"{self.minimum} and {self.maximum}"
            )
            raise ValueError(msg)
        return self.value
