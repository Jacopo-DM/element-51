"""Tests for pictures and the values a node can carry."""

import json
import math
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
    Variadic,
    can_link,
    find_input,
    variadic_index,
)
from element_51.nodes import Node, PlotNode, Registered, node
from element_51.visuals import MAX_POINTS, Image, Series, carries

WIDTH = 3
HEIGHT = 2
PIXELS_IN_COLOUR = 6
FLAG = True
THIRD = 3.0
LIMIT = 100_000
EMPTY = Series(values=())
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
    negative = by_id["n2"].shown["out"]
    assert isinstance(negative, Image)
    assert negative.pixels == bytes(
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


LOSS = (0.9, 0.5, 0.25)


@node
def loss_curve() -> Series:
    return Series(values=LOSS)


@node
def two_curves_and_a_number() -> Series:
    return Series(values=(1.0, float("nan"), 3.0))


@node
def last_of(history: Series) -> float:
    return history.values[-1]


def test_a_series_holds_floats_by_index() -> None:
    """The x axis is the position, so only the values are kept."""
    assert Series(values=LOSS).values == LOSS
    assert Series.model_validate({"values": [1, 2]}).values == (1.0, 2.0)
    assert Series(values=()).values == ()


def test_nan_goes_out_as_null_and_comes_back_as_nan() -> None:
    """A diverging loss is a gap in the line, not a crash."""
    series = Series(values=(1.0, float("nan"), 3.0))
    wire = json.loads(series.model_dump_json())
    assert wire == {"values": [1.0, None, 3.0]}
    again = Series.model_validate_json(series.model_dump_json())
    assert again.values[0] == pytest.approx(1.0)
    assert math.isnan(again.values[1])
    assert again.values[2] == THIRD


def test_a_series_may_hold_100000_points_and_no_more() -> None:
    """A node that returns more fails with a clear error."""
    assert MAX_POINTS == LIMIT
    assert len(Series(values=(0.0,) * MAX_POINTS).values) == MAX_POINTS
    with pytest.raises(ValidationError, match="100000"):
        Series(values=(0.0,) * (MAX_POINTS + 1))


@pytest.mark.parametrize("values", [["a"], "abc", 5])
def test_a_series_refuses_what_is_not_a_list_of_numbers(
    values: object,
) -> None:
    """Text, or a lone number, is an error."""
    with pytest.raises(ValidationError):
        Series.model_validate({"values": values})


def test_series_links_only_to_series() -> None:
    """A series is its own kind."""
    assert PortType.SERIES.value == "series"
    assert can_link(PortType.SERIES, PortType.SERIES)
    assert not can_link(PortType.SERIES, PortType.FLOAT)
    assert not can_link(PortType.FLOAT, PortType.SERIES)
    assert not can_link(PortType.IMAGE, PortType.SERIES)


def test_carries_a_series_for_a_series_port_only() -> None:
    """The kind check works for both visual kinds."""
    assert carries(Series(values=LOSS), PortType.SERIES)
    assert not carries(Series(values=LOSS), PortType.IMAGE)
    assert not carries(grey(), PortType.SERIES)
    assert not carries(1.0, PortType.SERIES)


def test_a_function_returning_a_series_has_a_series_output() -> None:
    """`-> Series` is one output `out`; the card itself shows nothing."""
    assert loss_curve.node_type.outputs == (
        Port(name="out", kind=PortType.SERIES),
    )
    assert loss_curve.node_type.display == "none"
    assert last_of.node_type.inputs == (
        Port(name="history", kind=PortType.SERIES),
    )


def test_a_default_for_a_series_input_is_refused() -> None:
    """A series cannot be typed, so no default fits."""

    def bad(history: Series = EMPTY) -> int:  # pragma: no cover  # never called
        return len(history.values)

    with pytest.raises(TypeError, match=r"history.*series"):
        node(bad)


def test_the_plot_node_is_a_chart_with_one_series_input() -> None:
    """It has no outputs and shows a chart."""
    node_type = PlotNode.node_type
    assert node_type.name == "Plot"
    assert node_type.inputs == (Port(name="series_1", kind=PortType.SERIES),)
    assert node_type.outputs == ()
    assert node_type.display == "chart"
    assert PlotNode.defaults == {}
    assert PlotNode.output_model is None


def test_a_series_reaches_the_node_it_is_linked_to() -> None:
    """The downstream function gets the Series itself."""
    functions: dict[str, Registered] = {
        "loss_curve": loss_curve,
        "last_of": last_of,
    }
    result = run_graph(
        graph_of(
            {"n1": "loss_curve", "n2": "last_of"},
            [("n1", "out", "n2", "history")],
        ),
        functions,
        raise_errors=True,
    )
    by_id = {outcome.node: outcome for outcome in result.outcomes}
    assert by_id["n1"].outputs == {}
    assert by_id["n1"].shown == {}
    assert by_id["n2"].outputs == {"out": LOSS[-1]}


def test_a_plot_shows_the_series_linked_to_it() -> None:
    """The chart's data is `shown`, by input port name."""
    functions: dict[str, Registered] = {
        "loss_curve": loss_curve,
        "Plot": PlotNode,
    }
    result = run_graph(
        graph_of(
            {"n1": "loss_curve", "n2": "Plot"},
            [("n1", "out", "n2", "series_1")],
        ),
        functions,
        raise_errors=True,
    )
    plot = result.outcomes[1]
    assert plot.status == "ok"
    assert plot.outputs == {}
    assert plot.shown == {"series_1": Series(values=LOSS)}


def test_a_plot_with_nothing_linked_is_ok_and_shows_nothing() -> None:
    """The empty row is a place to link, not a missing value."""
    result = run_graph(
        graph_of({"n1": "Plot"}, []), {"Plot": PlotNode}, raise_errors=True
    )
    assert result.outcomes[0].status == "ok"
    assert result.outcomes[0].shown == {}


def test_a_series_with_a_gap_reaches_the_page_with_a_null() -> None:
    """NaN in a node's series becomes null on the wire."""
    functions: dict[str, Registered] = {
        "gappy": two_curves_and_a_number,
        "Plot": PlotNode,
    }
    result = run_graph(
        graph_of(
            {"n1": "gappy", "n2": "Plot"}, [("n1", "out", "n2", "series_1")]
        ),
        functions,
        raise_errors=True,
    )
    wire = json.loads(result.model_dump_json())
    assert wire["outcomes"][1]["shown"]["series_1"] == {
        "values": [1.0, None, 3.0]
    }


def test_the_wire_tells_a_series_from_a_picture() -> None:
    """`shown` reads back into the right record for each."""
    result = run_graph(
        graph_of({"n1": "picture"}, []),
        {"picture": picture},
        raise_errors=True,
    )
    again = type(result).model_validate_json(result.model_dump_json())
    assert again.outcomes[0].shown == {"out": grey()}
    chart = type(result).model_validate({
        "outcomes": [
            {
                "node": "n1",
                "status": "ok",
                "shown": {"series_1": {"values": [1.0, None]}},
            }
        ]
    })
    shown = chart.outcomes[0].shown["series_1"]
    assert isinstance(shown, Series)
    assert math.isnan(shown.values[1])


def test_a_variadic_node_type_names_its_rows_by_prefix_and_number() -> None:
    """`Plot` has rows `series_1`, `series_2`, ... of the series kind."""
    assert PlotNode.node_type.variadic == Variadic(
        prefix="series", kind=PortType.SERIES
    )
    assert loss_curve.node_type.variadic is None


@pytest.mark.parametrize(
    ("name", "expected"),
    [
        ("series_1", 1),
        ("series_12", 12),
        ("series_0", None),
        ("series_01", None),
        ("series_-1", None),
        ("series_x", None),
        ("series_", None),
        ("series", None),
        ("other_1", None),
        ("xseries_1", None),
        ("series_1x", None),
    ],
)
def test_only_a_prefix_and_a_positive_whole_number_is_a_row(
    name: str, expected: int | None
) -> None:
    """The row number has no sign, no leading zero and nothing after it."""
    variadic = Variadic(prefix="series", kind=PortType.SERIES)
    assert variadic_index(variadic, name) == expected


def test_find_input_takes_listed_ports_and_any_row_of_a_variadic_type() -> None:
    """A row that is not in `inputs` still has the variadic kind."""
    node_type = PlotNode.node_type
    assert find_input(node_type, "series_1") == Port(
        name="series_1", kind=PortType.SERIES
    )
    assert find_input(node_type, "series_7") == Port(
        name="series_7", kind=PortType.SERIES
    )
    assert find_input(node_type, "series_0") is None
    assert find_input(last_of.node_type, "history") == Port(
        name="history", kind=PortType.SERIES
    )
    assert find_input(last_of.node_type, "history_2") is None


def two_curves_graph(*rows: str) -> Graph:
    """Build `loss_curve` nodes n1, n2, ... all linked to rows of a Plot.

    Returns
    -------
    Graph
        The graph; the Plot is the node after the curves.
    """
    types = {f"n{i + 1}": "loss_curve" for i in range(len(rows))}
    plot = f"n{len(rows) + 1}"
    types[plot] = "Plot"
    return graph_of(
        types,
        [(f"n{i + 1}", "out", plot, row) for i, row in enumerate(rows)],
    )


def test_a_plot_shows_every_linked_row_not_only_the_listed_one() -> None:
    """Rows 1 and 3 are linked; row 2 is nobody's."""
    functions: dict[str, Registered] = {
        "loss_curve": loss_curve,
        "Plot": PlotNode,
    }
    result = run_graph(
        two_curves_graph("series_1", "series_3"), functions, raise_errors=True
    )
    plot = result.outcomes[-1]
    assert plot.status == "ok"
    assert list(plot.shown) == ["series_1", "series_3"]


def test_a_plot_with_only_a_later_row_linked_still_shows_it() -> None:
    """Row 1 may stay empty."""
    functions: dict[str, Registered] = {
        "loss_curve": loss_curve,
        "Plot": PlotNode,
    }
    result = run_graph(
        two_curves_graph("series_2"), functions, raise_errors=True
    )
    assert list(result.outcomes[-1].shown) == ["series_2"]
