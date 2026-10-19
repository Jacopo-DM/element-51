"""Every example builds its graph and runs to the outcome it describes."""

import importlib.util
from pathlib import Path
from types import ModuleType

import pytest

from element_51.session import App

EXAMPLES = Path(__file__).parent.parent / "examples"

type Expected = list[tuple[str, str, dict[str, float | int | str | bool]]]

EXPECTED: dict[str, Expected] = {
    "a1_two_nodes.py": [
        ("n1", "ok", {"out": 21.0}),
        ("n2", "ok", {"out": 42.0}),
    ],
    "a2_arithmetic.py": [
        ("n1", "ok", {"out": 3.0}),
        ("n2", "ok", {"out": 4.0}),
        ("n3", "ok", {"out": 7.0}),
        ("n4", "ok", {"out": 21.0}),
        ("n5", "ok", {"out": 21.0}),
    ],
    "a3_strings.py": [
        ("n1", "ok", {"out": "hello"}),
        ("n2", "ok", {"out": "world"}),
        ("n3", "ok", {"out": 3.0}),
        ("n4", "ok", {"out": "hello world"}),
        ("n5", "ok", {"out": "HELLO WORLD"}),
        ("n6", "ok", {"out": "HELLO WORLD / HELLO WORLD"}),
    ],
    "a4_errors.py": [
        ("n1", "ok", {"out": 10.0}),
        ("n2", "ok", {"out": 0.0}),
        ("n3", "error", {}),
        ("n4", "skipped", {}),
        ("n5", "ok", {"out": 20.0}),
        ("n6", "error", {}),
        ("n7", "error", {}),
    ],
    "a5_diamond.py": [
        ("n1", "ok", {"out": 2.0}),
        ("n2", "ok", {"out": 4.0}),
        ("n3", "ok", {"out": -2.0}),
        ("n4", "ok", {"out": 2.0}),
        ("n5", "ok", {"out": 5}),
        ("n6", "ok", {"out": 2.5}),
    ],
    "a6_port_types.py": [
        ("n1", "ok", {"out": True}),
        ("n2", "ok", {"out": 4}),
        ("n3", "ok", {"out": 0.5}),
        ("n4", "ok", {"out": "items"}),
        ("n5", "ok", {"out": "4 items, ratio 0.5, flag no"}),
    ],
    "a8_multiple_outputs.py": [
        ("n1", "ok", {"out": 17}),
        ("n2", "ok", {"out": 5}),
        ("n3", "ok", {"q": 3, "r": 2}),
        ("n4", "ok", {"out": "3 remainder 2"}),
    ],
    "a10_live_push.py": [("n1", "ok", {"out": 1.0})],
    "a14_undo_redo.py": [
        ("n1", "ok", {"out": 3.0}),
        ("n2", "ok", {"out": 6.0}),
        ("n3", "ok", {"out": 7.0}),
    ],
    "a13_values.py": [
        ("n1", "ok", {"out": 6.0}),
        ("n2", "ok", {"out": "hello world"}),
        ("n3", "ok", {"out": "hello worldhello world"}),
    ],
    "a12_big_graph.py": [
        ("n1", "ok", {"out": 1.0}),
        ("n2", "ok", {"out": 2.0}),
        ("n3", "ok", {"out": 4.0}),
        ("n4", "ok", {"out": 8.0}),
        ("n5", "ok", {"out": 16.0}),
        ("n6", "ok", {"out": 32.0}),
        ("n7", "ok", {"out": 1.0}),
        ("n8", "ok", {"out": 2.0}),
        ("n9", "ok", {"out": 4.0}),
        ("n10", "ok", {"out": 8.0}),
        ("n11", "ok", {"out": 16.0}),
        ("n12", "ok", {"out": 32.0}),
        ("n13", "ok", {"out": 1.0}),
        ("n14", "ok", {"out": 2.0}),
        ("n15", "ok", {"out": 4.0}),
        ("n16", "ok", {"out": 8.0}),
        ("n17", "ok", {"out": 16.0}),
        ("n18", "ok", {"out": 32.0}),
        ("n19", "ok", {"out": 1.0}),
        ("n20", "ok", {"out": 2.0}),
        ("n21", "ok", {"out": 4.0}),
        ("n22", "ok", {"out": 8.0}),
        ("n23", "ok", {"out": 16.0}),
        ("n24", "ok", {"out": 32.0}),
    ],
    "a11_save_load.py": [
        ("n1", "ok", {"out": 4.0}),
        ("n2", "ok", {"out": 8.0}),
    ],
    "a9_class_nodes.py": [
        ("n1", "ok", {"out": "hello, world!"}),
        ("n2", "ok", {"out": "HELLO, WORLD!"}),
        ("n3", "ok", {"a": 3.0, "b": 4.5}),
        ("n4", "ok", {"total": 7.5, "mean": 3.75}),
    ],
    "a7_script_api.py": [
        ("n1", "ok", {"out": 19.99}),
        ("n2", "ok", {"out": 3}),
        ("n3", "ok", {"out": 59.97}),
        ("n4", "error", {}),
    ],
}


def load(name: str) -> ModuleType:
    """Import an example file.

    Returns
    -------
    ModuleType
        The example as a module.
    """
    path = EXAMPLES / name
    spec = importlib.util.spec_from_file_location(path.stem, path)
    assert spec is not None
    assert spec.loader is not None
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module


def build(name: str) -> App:
    """Load an example file and call its `build_app`.

    Returns
    -------
    App
        The app the example builds.
    """
    app: App = load(name).build_app()
    return app


def test_every_example_has_an_expected_outcome() -> None:
    """A new example file is not forgotten here."""
    found = sorted(path.name for path in EXAMPLES.glob("a[1-9]*.py"))
    assert found == sorted(EXPECTED)


@pytest.mark.parametrize("name", sorted(EXPECTED))
def test_example_runs_to_the_outcome_it_describes(name: str) -> None:
    """The statuses and values in each example's docstring are the real ones."""
    result = build(name).execute()
    actual = {o.node: (o.status, dict(o.outputs)) for o in result.outcomes}
    expected = {node: (status, value) for node, status, value in EXPECTED[name]}
    assert actual == expected


def test_the_error_example_reports_each_kind_of_failure() -> None:
    """Three different messages, one per failing node."""
    result = build("a4_errors.py").execute()
    errors = {o.node: o.error for o in result.outcomes if o.error}
    assert errors == {
        "n3": "ZeroDivisionError: float division by zero",
        "n6": "input 'x' has no value",
        "n7": "returned str, expected float",
    }


def test_the_push_example_grows_a_running_total() -> None:
    """Each step adds two nodes and two links; the total counts up."""
    example = load("a10_live_push.py")
    app = example.build_app()
    example.keep_adding(app, steps=2, pause=0.0)
    result = app.execute()
    totals = {o.node: o.outputs["out"] for o in result.outcomes}
    assert totals == {
        "n1": 1.0,
        "n2": 1.0,
        "n3": 2.0,
        "n4": 1.0,
        "n5": 3.0,
    }


def test_the_save_example_gets_back_what_was_saved(tmp_path: Path) -> None:
    """A second run loads the file the first run saved."""
    example = load("a11_save_load.py")
    first = example.build_app(tmp_path / "g.json")
    first.add_node(example.number, x=1.0, y=1.0)
    first.save(tmp_path / "g.json")
    second = example.build_app(tmp_path / "g.json")
    assert [n.id for n in second.graph.nodes] == ["n1", "n2", "n3"]
