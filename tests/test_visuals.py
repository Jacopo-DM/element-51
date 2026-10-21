"""Tests for pictures and the values a node can carry."""

import json
from typing import override

import pytest
from pydantic import ValidationError

from element_51.engine import run_graph
from element_51.model import (
    Graph,
    Link,
    PlacedNode,
    Port,
    PortRef,
    PortType,
    can_link,
)
from element_51.nodes import Node, Registered, node
from element_51.visuals import Image, carries

WIDTH = 3
HEIGHT = 2
PIXELS_IN_COLOUR = 6
FLAG = True
SAMPLE = Image(width=1, height=1, channels=1, pixels=b"a")


def grey(width: int = WIDTH, height: int = HEIGHT) -> Image:
    """Build a grey picture whose pixels count up from zero.

    Returns
    -------
    Image
        A one-channel picture.
    """
    return Image(
        width=width,
        height=height,
        channels=1,
        pixels=bytes(range(width * height)),
    )


@node
def picture() -> Image:
    return grey()


@node
def invert(source: Image) -> Image:
    return Image(
        width=source.width,
        height=source.height,
        channels=source.channels,
        pixels=bytes(255 - pixel for pixel in source.pixels),
    )


@node
def width_of(source: Image) -> int:
    return source.width


class Photo(Node):
    @override
    def run(self) -> Image:
        return grey()


@node
def bad_picture() -> Image:
    return Image(width=1, height=1, channels=1, pixels=b"ab")


def graph_of(
    types: dict[str, str], links: list[tuple[str, str, str, str]]
) -> Graph:
    """Build a graph from `{id: node type}` and `(src, out, dst, in)` links.

    Returns
    -------
    Graph
        The graph, with every node at the origin.
    """
    return Graph(
        nodes=tuple(
            PlacedNode(id=node_id, node_type=name, x=0.0, y=0.0)
            for node_id, name in types.items()
        ),
        links=tuple(
            Link(
                source=PortRef(node=src, port=out),
                target=PortRef(node=dst, port=inp),
            )
            for src, out, dst, inp in links
        ),
    )


def test_a_picture_holds_its_size_and_pixels() -> None:
    """The record keeps what it was given."""
    image = grey()
    assert (image.width, image.height, image.channels) == (WIDTH, HEIGHT, 1)
    assert image.pixels == bytes(range(WIDTH * HEIGHT))


def test_a_colour_picture_has_three_values_per_pixel() -> None:
    """Three channels mean width times height times three bytes."""
    image = Image(width=2, height=1, channels=3, pixels=bytes(6))
    assert len(image.pixels) == PIXELS_IN_COLOUR


@pytest.mark.parametrize(
    ("fields", "reason"),
    [
        (
            {"width": 2, "height": 2, "channels": 1, "pixels": b"abc"},
            "pixels",
        ),
        (
            {"width": 1, "height": 1, "channels": 3, "pixels": b"a"},
            "pixels",
        ),
        ({"width": 1, "height": 1, "channels": 2, "pixels": b"ab"}, "channels"),
        ({"width": 0, "height": 1, "channels": 1, "pixels": b""}, "width"),
        (
            {"width": 257, "height": 1, "channels": 1, "pixels": bytes(257)},
            "width",
        ),
        (
            {"width": 1, "height": 257, "channels": 1, "pixels": bytes(257)},
            "height",
        ),
    ],
)
def test_a_bad_picture_is_refused(
    fields: dict[str, object], reason: str
) -> None:
    """The size, the channel count and the pixel count are all checked."""
    with pytest.raises(ValidationError, match=reason):
        Image.model_validate(fields)


def test_the_biggest_picture_is_256_by_256() -> None:
    """The limit is inclusive."""
    side = 256
    image = Image(
        width=side, height=side, channels=1, pixels=bytes(side * side)
    )
    assert image.width == side


def test_pixels_go_out_as_a_list_of_integers() -> None:
    """The wire format needs no base64 on either side."""
    wire = json.loads(grey().model_dump_json())
    assert wire == {
        "width": WIDTH,
        "height": HEIGHT,
        "channels": 1,
        "pixels": [0, 1, 2, 3, 4, 5],
    }


def test_pixels_come_in_as_a_list_of_integers() -> None:
    """A picture survives a trip through JSON."""
    again = Image.model_validate_json(grey().model_dump_json())
    assert again == grey()


@pytest.mark.parametrize("pixels", [[256], [-1], ["a"], [1.5]])
def test_a_pixel_outside_0_to_255_is_refused(pixels: list[object]) -> None:
    """Each pixel is one byte."""
    with pytest.raises(ValidationError):
        Image.model_validate({
            "width": 1,
            "height": 1,
            "channels": 1,
            "pixels": pixels,
        })


def test_a_picture_is_immutable() -> None:
    """Like every record here, it cannot change after it is built."""
    with pytest.raises(ValidationError):
        grey().width = 5  # type: ignore[misc]  # ty: ignore[invalid-assignment]  # frozen on purpose


def test_image_links_only_to_image() -> None:
    """Pictures join pictures and nothing else."""
    assert PortType.IMAGE.value == "image"
    assert can_link(PortType.IMAGE, PortType.IMAGE)
    assert not can_link(PortType.IMAGE, PortType.FLOAT)
    assert not can_link(PortType.FLOAT, PortType.IMAGE)
    assert not can_link(PortType.INT, PortType.IMAGE)


def test_carries_checks_the_value_against_the_kind() -> None:
    """A picture fits `image` only, and plain values keep their rules."""
    assert carries(grey(), PortType.IMAGE)
    assert not carries(1.0, PortType.IMAGE)
    assert not carries(grey(), PortType.FLOAT)
    assert carries(1, PortType.FLOAT)
    assert not carries(FLAG, PortType.INT)


def test_a_function_returning_a_picture_has_an_image_output_and_a_display() -> (
    None
):
    """`-> Image` is one output `out`, and the card shows it."""
    node_type = picture.node_type
    assert node_type.outputs == (Port(name="out", kind=PortType.IMAGE),)
    assert node_type.display == "image"
    assert Photo.node_type.display == "image"


def test_a_class_node_returning_a_picture_runs() -> None:
    """A `Node` subclass shows its picture like a function does."""
    result = run_graph(
        graph_of({"n1": "Photo"}, []), {"Photo": Photo}, raise_errors=True
    )
    assert result.outcomes[0].shown == {"out": grey()}


def test_a_node_without_a_picture_shows_nothing() -> None:
    """Only a node with an image output has a display."""
    assert width_of.node_type.display == "none"


def test_a_picture_input_has_no_default() -> None:
    """A picture can only be linked, so it takes no typed value."""
    assert invert.node_type.inputs == (
        Port(name="source", kind=PortType.IMAGE),
    )
    assert invert.node_type.defaults == {}


def test_a_default_for_a_picture_input_is_refused() -> None:
    """There is no way to type a picture, so no default fits."""

    def bad(source: Image = SAMPLE) -> int:  # pragma: no cover  # never called
        return source.width

    with pytest.raises(TypeError, match=r"source.*image"):
        node(bad)


def test_a_class_field_that_is_a_picture_is_refused_with_a_default() -> None:
    """The same rule holds for a `Node` subclass."""
    with pytest.raises(TypeError, match=r"source.*image"):

        class Bad(Node):
            source: Image = SAMPLE

            @override
            def run(self) -> int:  # pragma: no cover  # never called
                return 0


def test_a_run_shows_the_picture_and_keeps_it_out_of_outputs() -> None:
    """The page reads `shown`; `outputs` holds the plain values only."""
    functions = {"picture": picture}
    graph = graph_of({"n1": "picture"}, [])
    result = run_graph(graph, functions, raise_errors=True)
    outcome = result.outcomes[0]
    assert outcome.status == "ok"
    assert outcome.outputs == {}
    assert outcome.shown == {"out": grey()}


def test_a_picture_reaches_the_node_it_is_linked_to() -> None:
    """The downstream function gets the Image itself."""
    functions: dict[str, Registered] = {
        "picture": picture,
        "invert": invert,
        "width_of": width_of,
    }
    graph = graph_of(
        {"n1": "picture", "n2": "invert", "n3": "width_of"},
        [("n1", "out", "n2", "source"), ("n2", "out", "n3", "source")],
    )
    result = run_graph(graph, functions, raise_errors=True)
    by_id = {outcome.node: outcome for outcome in result.outcomes}
    assert by_id["n2"].shown["out"].pixels == bytes(
        255 - pixel for pixel in range(WIDTH * HEIGHT)
    )
    assert by_id["n3"].outputs == {"out": WIDTH}
    assert by_id["n3"].shown == {}


def test_an_unlinked_picture_input_is_an_error_not_a_crash() -> None:
    """There is no default to fall back on."""
    functions = {"invert": invert}
    result = run_graph(
        graph_of({"n1": "invert"}, []), functions, raise_errors=False
    )
    assert result.outcomes[0].status == "error"
    assert result.outcomes[0].error == "input 'source' has no value"


def test_a_bad_picture_from_a_node_is_reported() -> None:
    """A node that builds an invalid picture fails like any other bad return."""
    result = run_graph(
        graph_of({"n1": "bad_picture"}, []),
        {"bad_picture": bad_picture},
        raise_errors=False,
    )
    assert result.outcomes[0].status == "error"
    assert "pixels" in (result.outcomes[0].error or "")


def test_a_node_returning_the_wrong_thing_for_a_picture_is_reported() -> None:
    """A float where an Image was promised is the node's mistake."""

    @node
    def liar() -> Image:
        return 1.0  # type: ignore[return-value]  # ty: ignore[invalid-return-type]  # the mistake under test

    result = run_graph(
        graph_of({"n1": "liar"}, []), {"liar": liar}, raise_errors=False
    )
    assert result.outcomes[0].error == "returned float, expected image"


def test_the_outcome_goes_out_with_the_picture_as_a_list() -> None:
    """`shown` is on the wire in the same shape as a bare Image."""
    result = run_graph(
        graph_of({"n1": "picture"}, []),
        {"picture": picture},
        raise_errors=True,
    )
    wire = json.loads(result.model_dump_json())
    assert wire["outcomes"][0]["shown"]["out"]["pixels"] == [0, 1, 2, 3, 4, 5]
