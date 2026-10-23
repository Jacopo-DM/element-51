"""The input nodes: a value you set, passed on as it is."""

import pytest
from sample_nodes import add

from element_51 import App, Flag, Float, Integer, Slider, Text
from element_51.edits import EditError, SetValue


def test_the_input_nodes_sit_in_default_nodes_input_nodes() -> None:
    """Every app offers the five, each in the `Input nodes` group."""
    app = App([])
    categories = {t.name: t.category for t in app.snapshot().node_types}
    for name in ("Float", "Integer", "Text", "Flag", "Slider"):
        assert categories[name] == "Default nodes/Input nodes"


def test_each_variable_passes_its_value_on() -> None:
    """The default is returned, and a value set on the card replaces it."""
    app = App([])
    nodes = {
        kind: app.add_node(kind, x=0.0, y=0.0)
        for kind in (Float, Integer, Text, Flag)
    }
    first = app.run()
    assert first[nodes[Float]] == {"out": 0.0}
    assert first[nodes[Integer]] == {"out": 0}
    assert first[nodes[Text]] == {"out": ""}
    assert first[nodes[Flag]] == {"out": False}
    app.apply(SetValue(node=nodes[Float], port="value", value=2.5))
    app.apply(SetValue(node=nodes[Integer], port="value", value=7))
    app.apply(SetValue(node=nodes[Text], port="value", value="hi"))
    app.apply(SetValue(node=nodes[Flag], port="value", value=True))
    second = app.run()
    assert second[nodes[Float]] == {"out": 2.5}
    assert second[nodes[Integer]] == {"out": 7}
    assert second[nodes[Text]] == {"out": "hi"}
    assert second[nodes[Flag]] == {"out": True}


def test_one_float_feeds_two_inputs_the_same_value() -> None:
    """The fan-out the examples used `three` for."""
    app = App([add])
    source = app.add_node(Float, x=0.0, y=0.0)
    app.apply(SetValue(node=source, port="value", value=3.0))
    total = app.add_node(add, x=300.0, y=0.0)
    app.add_link(source, "out", total, "a")
    app.add_link(source, "out", total, "b")
    assert app.run()[total] == {"out": 6.0}


def test_a_text_variable_cannot_feed_a_number_input() -> None:
    """The link is refused like any link between different types."""
    app = App([add])
    text = app.add_node(Text, x=0.0, y=0.0)
    total = app.add_node(add, x=300.0, y=0.0)
    with pytest.raises(EditError):
        app.add_link(text, "out", total, "a")


def test_a_slider_returns_its_value() -> None:
    """The defaults are 0 between -10 and 10; a moved value comes back."""
    app = App([])
    slider = app.add_node(Slider, x=0.0, y=0.0)
    assert app.run()[slider] == {"out": 0.0}
    app.apply(SetValue(node=slider, port="value", value=-7.5))
    assert app.run()[slider] == {"out": -7.5}


def test_a_slider_value_outside_its_range_is_an_error() -> None:
    """Nothing is clamped: the card shows the failure."""
    app = App([])
    slider = app.add_node(Slider, x=0.0, y=0.0)
    app.apply(SetValue(node=slider, port="value", value=11.0))
    with pytest.raises(ValueError, match="between"):
        app.run()


def test_a_slider_needs_a_minimum_below_its_maximum() -> None:
    """An empty range is refused, whatever the value."""
    app = App([])
    slider = app.add_node(Slider, x=0.0, y=0.0)
    app.apply(SetValue(node=slider, port="minimum", value=10.0))
    with pytest.raises(ValueError, match="below"):
        app.run()


def test_only_the_slider_declares_a_slider_card() -> None:
    """The page draws a slider control where a node says `slider`."""
    assert Slider.node_type.display == "slider"
    for kind in (Float, Integer, Text, Flag):
        assert kind.node_type.display == "none"
