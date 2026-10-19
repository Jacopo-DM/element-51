"""Every example builds its graph and runs to the outcome it describes."""

import importlib.util
from pathlib import Path

import pytest

from element_51.session import App

EXAMPLES = Path(__file__).parent.parent / "examples"

type Expected = list[tuple[str, str, float | int | str | bool | None]]

EXPECTED: dict[str, Expected] = {
    "a1_serve.py": [],
    "a2_arithmetic.py": [
        ("n1", "ok", 3.0),
        ("n2", "ok", 4.0),
        ("n3", "ok", 7.0),
        ("n4", "ok", 21.0),
        ("n5", "ok", 21.0),
    ],
    "a3_strings.py": [
        ("n1", "ok", "hello"),
        ("n2", "ok", "world"),
        ("n3", "ok", 3.0),
        ("n4", "ok", "hello world"),
        ("n5", "ok", "HELLO WORLD"),
        ("n6", "ok", "HELLO WORLD / HELLO WORLD"),
    ],
    "a4_errors.py": [
        ("n1", "ok", 10.0),
        ("n2", "ok", 0.0),
        ("n3", "error", None),
        ("n4", "skipped", None),
        ("n5", "ok", 20.0),
        ("n6", "error", None),
        ("n7", "error", None),
    ],
    "a5_diamond.py": [
        ("n1", "ok", 2.0),
        ("n2", "ok", 4.0),
        ("n3", "ok", -2.0),
        ("n4", "ok", 2.0),
        ("n5", "ok", 5),
        ("n6", "ok", 2.5),
    ],
    "a6_port_types.py": [
        ("n1", "ok", True),
        ("n2", "ok", 4),
        ("n3", "ok", 0.5),
        ("n4", "ok", "items"),
        ("n5", "ok", "4 items, ratio 0.5, flag no"),
    ],
    "a7_script_api.py": [
        ("n1", "ok", 19.99),
        ("n2", "ok", 3),
        ("n3", "ok", 59.97),
        ("n4", "error", None),
    ],
}


def build(name: str) -> App:
    """Load an example file and call its `build_app`.

    Returns
    -------
    App
        The app the example builds.
    """
    path = EXAMPLES / name
    spec = importlib.util.spec_from_file_location(path.stem, path)
    assert spec is not None
    assert spec.loader is not None
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    app: App = module.build_app()
    return app


def test_every_example_has_an_expected_outcome() -> None:
    """A new example file is not forgotten here."""
    found = sorted(path.name for path in EXAMPLES.glob("a[1-9]*.py"))
    assert found == sorted(EXPECTED)


@pytest.mark.parametrize("name", sorted(EXPECTED))
def test_example_runs_to_the_outcome_it_describes(name: str) -> None:
    """The statuses and values in each example's docstring are the real ones."""
    result = build(name).execute()
    actual = {o.node: (o.status, o.value) for o in result.outcomes}
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
