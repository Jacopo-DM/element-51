"""Pictures: values a node can show on its card and pass along a link.

`PortValue` (in `model.py`) stays the four plain kinds. `Carried` adds what
cannot be typed into a box and can only travel along a link.
"""

from typing import Annotated, Literal, Self, TypeGuard

from pydantic import BeforeValidator, Field, PlainSerializer, model_validator

from element_51.model import Model, PortType, PortValue, value_fits

MAX_SIDE = 256


def _bytes_from_list(value: object) -> object:
    """Accept the JSON list of integers the browser layer reads and writes.

    Returns
    -------
    object
        `bytes` for a list or tuple of integers, else `value` unchanged.

    Raises
    ------
    ValueError
        If an item of a list is not an integer from 0 to 255.
    """
    if isinstance(value, list | tuple):
        try:
            return bytes(value)
        except TypeError as error:
            msg = "pixels must be integers from 0 to 255"
            raise ValueError(msg) from error
    return value


type Pixels = Annotated[
    bytes,
    BeforeValidator(_bytes_from_list),
    PlainSerializer(list, return_type=list[int], when_used="json"),
]


class Image(Model):
    """A grey or colour picture, at most 256 by 256 pixels.

    `pixels` holds the rows from the top, left to right, one byte per channel:
    `width * height * channels` bytes. On the wire the pixels are a JSON list
    of integers from 0 to 255.
    """

    width: int = Field(ge=1, le=MAX_SIDE)
    height: int = Field(ge=1, le=MAX_SIDE)
    channels: Literal[1, 3]
    pixels: Pixels

    @model_validator(mode="after")
    def _count_pixels(self) -> Self:
        """Check that the pixels fill the picture exactly.

        Returns
        -------
        Self
            The picture.

        Raises
        ------
        ValueError
            If there are not `width * height * channels` pixels.
        """
        expected = self.width * self.height * self.channels
        if len(self.pixels) != expected:
            msg = (
                f"pixels: {self.width} by {self.height} with {self.channels} "
                f"channel(s) needs {expected} values, got {len(self.pixels)}"
            )
            raise ValueError(msg)
        return self


type Carried = PortValue | Image


def carries(value: object, kind: PortType) -> TypeGuard[Carried]:
    """Whether `value` is something a port of type `kind` can carry.

    Parameters
    ----------
    value
        A value from a node function.
    kind
        The port type.

    Returns
    -------
    bool
        True if the value fits.
    """
    if kind is PortType.IMAGE:
        return isinstance(value, Image)
    return value_fits(value, kind)
