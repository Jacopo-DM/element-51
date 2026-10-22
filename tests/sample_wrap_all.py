"""A module whose `__all__` names only some of its functions."""

__all__ = ["listed"]


def listed(x: float) -> float:  # pragma: no cover  # scanned only
    return x


def unlisted(x: float) -> float:  # pragma: no cover  # scanned only
    return x
