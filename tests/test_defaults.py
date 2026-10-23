"""The default nodes every editor starts with, and the switch for them."""

from pathlib import Path
from typing import override

import pytest
from sample_nodes import add

from element_51.nodes import Node, PlotNode, node
from element_51.session import App
from element_51.store import StoreError


@node(category="Numpy")
def hypot(x1: float, x2: float) -> float:  # pragma: no cover  # never run
    return x1 + x2


class Plot(Node):
    """A user node that takes the name of a default."""

    x: float = 1.0

    @override
    def run(self) -> float:  # pragma: no cover  # the app refuses it
        return self.x


DEFAULT_MENU = {
    "Float": "Default nodes/Input nodes",
    "Integer": "Default nodes/Input nodes",
    "Text": "Default nodes/Input nodes",
    "Flag": "Default nodes/Input nodes",
    "Slider": "Default nodes/Input nodes",
    "Plot": "Default nodes/Output nodes",
}


def menu_categories(app: App) -> dict[str, str]:
    """Return the menu path of every node type the page is told about.

    Returns
    -------
    dict[str, str]
        Node type name to category, as the add-node menu reads it.
    """
    return {t.name: t.category for t in app.snapshot().node_types}


def test_the_defaults_are_offered_without_being_given() -> None:
    """A new app has the input nodes and `Plot`, in their folders."""
    app = App([])
    assert list(app.node_functions) == list(DEFAULT_MENU)
    assert menu_categories(app) == DEFAULT_MENU


def test_user_nodes_sit_under_user_defined_nodes() -> None:
    """No category goes directly under the folder, a category inside it."""
    app = App([add, hypot])
    assert menu_categories(app) == {
        **DEFAULT_MENU,
        "add": "User defined nodes",
        "hypot": "User defined nodes/Numpy",
    }


def test_the_defaults_come_first_in_the_menu_order() -> None:
    """The page lists the default nodes before the user's."""
    names = list(App([add, hypot]).node_functions)
    assert names == [*DEFAULT_MENU, "add", "hypot"]


def test_defaults_false_gives_the_plain_menu() -> None:
    """With the switch off: no `Plot`, no folder, the categories as written."""
    app = App([add, hypot], defaults=False)
    assert menu_categories(app) == {"add": "", "hypot": "Numpy"}


def test_a_node_named_like_a_default_is_refused() -> None:
    """The error names both nodes and the two ways out."""
    with pytest.raises(ValueError, match="Plot") as error:
        App([Plot])
    message = str(error.value)
    assert "default node" in message
    assert "rename" in message
    assert "defaults=False" in message


def test_a_node_named_like_a_default_is_fine_without_defaults() -> None:
    """The switch off removes the clash."""
    app = App([Plot], defaults=False)
    assert list(app.node_functions) == ["Plot"]


def test_listing_plot_node_is_refused() -> None:
    """Old scripts that list `PlotNode` are told to remove it."""
    with pytest.raises(ValueError, match="Plot is a default node") as error:
        App([add, PlotNode])
    assert "remove it from the list" in str(error.value)


def test_listing_plot_node_is_fine_without_defaults() -> None:
    """With the switch off, `PlotNode` is an ordinary node to list."""
    app = App([PlotNode], defaults=False)
    assert menu_categories(app) == {"Plot": "Output nodes"}


def test_two_user_nodes_with_one_name_still_say_so() -> None:
    """The old message stays for a clash between the user's own nodes."""
    with pytest.raises(ValueError, match="two node types are named 'add'"):
        App([add, add])


def test_the_raw_node_types_are_not_prefixed() -> None:
    """Only the snapshot adds the folders; one function serves two apps."""
    with_defaults = App([add])
    without = App([add], defaults=False)
    assert menu_categories(with_defaults)["add"] == "User defined nodes"
    assert not menu_categories(without)["add"]
    assert not add.node_type.category
    assert PlotNode.node_type.category == "Output nodes"


def test_add_node_accepts_plot_node_from_the_defaults() -> None:
    """`PlotNode` stays the way to place a `Plot` by hand."""
    app = App([])
    node_id = app.add_node(PlotNode, x=0.0, y=0.0)
    assert app.graph.nodes[0].id == node_id


def test_a_saved_plot_needs_the_defaults_to_load(tmp_path: Path) -> None:
    """A graph with a `Plot` loads with the defaults and is refused without."""
    path = tmp_path / "graph.json"
    app = App([])
    app.add_node(PlotNode, x=0.0, y=0.0)
    app.save(path)
    App([]).load(path)
    with pytest.raises(StoreError, match="Plot"):
        App([], defaults=False).load(path)
