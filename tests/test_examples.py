"""Every example builds its graph and runs to the outcome it describes."""

import importlib.util
import sys
import threading
import time
from pathlib import Path
from types import ModuleType

import pytest

from element_51.edits import SetValue
from element_51.protocol import JobUpdate
from element_51.session import App
from element_51.visuals import Image, Series

EXAMPLES = Path(__file__).parent.parent / "examples"
DEFAULT_STEPS = 60
TRAINING_STEPS = 40
WAIT = 10.0
KILLED_WITHIN = 1.5
POLL = 0.01

type Expected = list[tuple[str, str, dict[str, float | int | str | bool]]]

EXPECTED: dict[str, Expected] = {
    "a_basics/a1_hello_world.py": [("n1", "ok", {"out": "hello, world"})],
    "a_basics/a2_two_nodes.py": [
        ("n1", "ok", {"out": 21.0}),
        ("n2", "ok", {"out": 42.0}),
    ],
    "a_basics/a3_arithmetic.py": [
        ("n1", "ok", {"out": 3.0}),
        ("n2", "ok", {"out": 4.0}),
        ("n3", "ok", {"out": 7.0}),
        ("n4", "ok", {"out": 21.0}),
        ("n5", "ok", {"out": 21.0}),
    ],
    "a_basics/a4_strings.py": [
        ("n1", "ok", {"out": "hello"}),
        ("n2", "ok", {"out": "world"}),
        ("n3", "ok", {"out": 3.0}),
        ("n4", "ok", {"out": "hello world"}),
        ("n5", "ok", {"out": "HELLO WORLD"}),
        ("n6", "ok", {"out": "HELLO WORLD / HELLO WORLD"}),
    ],
    "b_node_types/b5_errors.py": [
        ("n1", "ok", {"out": 10.0}),
        ("n2", "ok", {"out": 0.0}),
        ("n3", "error", {}),
        ("n4", "skipped", {}),
        ("n5", "ok", {"out": 20.0}),
        ("n6", "error", {}),
        ("n7", "error", {}),
    ],
    "b_node_types/b4_diamond.py": [
        ("n1", "ok", {"out": 2.0}),
        ("n2", "ok", {"out": 4.0}),
        ("n3", "ok", {"out": -2.0}),
        ("n4", "ok", {"out": 2.0}),
        ("n5", "ok", {"out": 5}),
        ("n6", "ok", {"out": 2.5}),
    ],
    "b_node_types/b1_port_types.py": [
        ("n1", "ok", {"out": True}),
        ("n2", "ok", {"out": 4}),
        ("n3", "ok", {"out": 0.5}),
        ("n4", "ok", {"out": "items"}),
        ("n5", "ok", {"out": "4 items, ratio 0.5, flag no"}),
    ],
    "b_node_types/b2_multiple_outputs.py": [
        ("n1", "ok", {"out": 17}),
        ("n2", "ok", {"out": 5}),
        ("n3", "ok", {"q": 3, "r": 2}),
        ("n4", "ok", {"out": "3 remainder 2"}),
    ],
    "c_python_api/c2_live_push.py": [("n1", "ok", {"out": 1.0})],
    "d_editor/d4_menu.py": [("n1", "ok", {"out": 3.0})],
    "f_embedding/f1_fastapi_host.py": [("n1", "ok", {"out": 3.0})],
    "g_experiments/g1_picture_and_plot.py": [
        ("n1", "ok", {}),
        ("n2", "ok", {}),
        ("n3", "ok", {"out": 784}),
        ("n4", "ok", {}),
        ("n5", "ok", {}),
    ],
    "g_experiments/g3_mnist.py": [],
    "g_experiments/g2_job_nodes.py": [
        ("n1", "skipped", {}),
        ("n2", "skipped", {}),
        ("n3", "skipped", {}),
        ("n4", "skipped", {}),
        ("n5", "skipped", {}),
    ],
    "f_embedding/f2_nicegui_host.py": [("n1", "ok", {"out": 3.0})],
    "e_look/e2_theme_file.py": [
        ("n1", "ok", {"out": 3.0}),
        ("n2", "ok", {"out": 3.0}),
    ],
    "d_editor/d6_auto_run.py": [
        ("n1", "ok", {"out": 6.0}),
        ("n2", "error", {}),
    ],
    "d_editor/d5_card_menu.py": [
        ("n1", "ok", {"out": 3.0}),
        ("n2", "ok", {"out": 5.0}),
        ("n3", "ok", {"out": 5.0}),
    ],
    "e_look/e3_palette.py": [
        ("n1", "ok", {"out": 12.0}),
        ("n2", "ok", {"out": 6.0}),
    ],
    "e_look/e1_light_theme.py": [
        ("n1", "ok", {"out": 12.0}),
        ("n2", "ok", {"out": 6.0}),
    ],
    "e_look/e4_purple_synthwave.py": [
        ("n1", "ok", {"out": 12.0}),
        ("n2", "ok", {"out": 6.0}),
    ],
    "d_editor/d3_undo_redo.py": [
        ("n1", "ok", {"out": 3.0}),
        ("n2", "ok", {"out": 6.0}),
        ("n3", "ok", {"out": 7.0}),
    ],
    "d_editor/d2_values.py": [
        ("n1", "ok", {"out": 6.0}),
        ("n2", "ok", {"out": "hello world"}),
        ("n3", "ok", {"out": "hello worldhello world"}),
    ],
    "d_editor/d1_big_graph.py": [
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
    "c_python_api/c3_save_load.py": [
        ("n1", "ok", {"out": 4.0}),
        ("n2", "ok", {"out": 8.0}),
    ],
    "b_node_types/b3_class_nodes.py": [
        ("n1", "ok", {"out": "hello, world!"}),
        ("n2", "ok", {"out": "HELLO, WORLD!"}),
        ("n3", "ok", {"a": 3.0, "b": 4.5}),
        ("n4", "ok", {"total": 7.5, "mean": 3.75}),
    ],
    "c_python_api/c1_script_api.py": [
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
    # A pydantic generic such as `JobNode[int]` looks its module up here.
    sys.modules[spec.name] = module
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
    found = sorted(
        path.relative_to(EXAMPLES).as_posix()
        for path in EXAMPLES.glob("*/[a-z][0-9]*_*.py")
    )
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
    result = build("b_node_types/b5_errors.py").execute()
    errors = {o.node: o.error for o in result.outcomes if o.error}
    assert errors == {
        "n3": "ZeroDivisionError: float division by zero",
        "n6": "input 'x' has no value",
        "n7": "returned str, expected float",
    }


def test_the_push_example_grows_a_running_total() -> None:
    """Each step adds two nodes and two links; the total counts up."""
    example = load("c_python_api/c2_live_push.py")
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
    example = load("c_python_api/c3_save_load.py")
    first = example.build_app(tmp_path / "g.json")
    first.add_node(example.number, x=1.0, y=1.0)
    first.save(tmp_path / "g.json")
    second = example.build_app(tmp_path / "g.json")
    assert [n.id for n in second.graph.nodes] == ["n1", "n2", "n3"]


def test_the_light_theme_example_uses_the_light_theme() -> None:
    """Building the app already selects the light theme."""
    assert build("e_look/e1_light_theme.py").theme == "antimony-light"


def test_the_purple_synthwave_example_uses_that_theme() -> None:
    """Building the app already selects the purple-synthwave theme."""
    assert build("e_look/e4_purple_synthwave.py").theme == "purple-synthwave"


def test_the_menu_example_puts_nodes_in_nested_categories() -> None:
    """The categories of the example's node types, in menu order."""
    app = build("d_editor/d4_menu.py")
    categories = {
        name: function.node_type.category
        for name, function in app.node_functions.items()
    }
    assert categories == {
        "sine": "Math/Trig",
        "cosine": "Math/Trig",
        "square_root": "Math",
        "Shout": "Text",
        "add": "",
        "show": "",
    }


def test_the_theme_file_example_changes_the_dark_theme() -> None:
    """The app reads `my_theme.json` and sends its changes to the page."""
    snapshot = build("e_look/e2_theme_file.py").snapshot()
    assert snapshot.theme == "antimony"
    assert snapshot.theme_changes.colours["canvas"] == "#101820"
    assert snapshot.theme_changes.metrics == {"node_width": 260.0}


def test_the_picture_example_plots_a_falling_loss_and_a_rising_accuracy() -> (
    None
):
    """The Plot node shows both curves, 60 values each."""
    result = build("g_experiments/g1_picture_and_plot.py").execute()
    shown = {o.node: o for o in result.outcomes}["n5"].shown
    assert list(shown) == ["series_1", "series_2"]
    loss, accuracy = shown["series_1"], shown["series_2"]
    assert isinstance(loss, Series)
    assert isinstance(accuracy, Series)
    assert len(loss.values) == len(accuracy.values) == DEFAULT_STEPS
    assert loss.values[-1] < loss.values[0]
    assert accuracy.values[-1] > accuracy.values[0]


def test_the_picture_example_shows_a_ramp_and_its_negative() -> None:
    """The two picture nodes show 28 by 28 pictures that add up to white."""
    result = build("g_experiments/g1_picture_and_plot.py").execute()
    by_id = {outcome.node: outcome for outcome in result.outcomes}
    first, second = by_id["n1"].shown["out"], by_id["n2"].shown["out"]
    assert isinstance(first, Image)
    assert isinstance(second, Image)
    assert (first.width, first.height) == (28, 28)
    assert [
        a + b for a, b in zip(first.pixels, second.pixels, strict=True)
    ] == [255] * (28 * 28)


def test_the_job_example_runs_its_jobs_and_the_nodes_after_them() -> None:
    """Start fills the Plot's series and `double`, which runs at the end."""
    app = build("g_experiments/g2_job_nodes.py")
    ended = threading.Event()
    states: list[str] = []

    def listener(event: object) -> None:
        if isinstance(event, JobUpdate):
            states.append(event.state)
            if event.state == "finished":
                ended.set()

    app.subscribe_jobs(listener)
    app.apply(SetValue(node="n1", port="delay", value=0.0))
    app.apply(SetValue(node="n2", port="pause", value=0.0))
    app.start_job("n1")
    assert ended.wait(WAIT)
    ended.clear()
    app.start_job("n2")
    assert ended.wait(WAIT)
    outcomes = {o.node: o for o in app.execute().outcomes}
    assert outcomes["n1"].status == "ok"
    series = outcomes["n4"].shown["series_1"]
    assert isinstance(series, Series)
    assert len(series.values) == TRAINING_STEPS
    assert outcomes["n2"].outputs == {"out": 0}
    assert outcomes["n3"].outputs == {"out": 0}


def test_the_job_example_blows_up_when_asked() -> None:
    """With `blow_up` ticked the job fails at step 10 with its reason."""
    app = build("g_experiments/g2_job_nodes.py")
    failed = threading.Event()
    reasons: list[str] = []

    def listener(event: object) -> None:
        if isinstance(event, JobUpdate) and event.state == "failed":
            reasons.append(event.error or "")
            failed.set()

    app.subscribe_jobs(listener)
    app.apply(SetValue(node="n1", port="delay", value=0.0))
    app.apply(SetValue(node="n1", port="blow_up", value=True))
    app.start_job("n1")
    assert failed.wait(WAIT)
    assert reasons == ["RuntimeError: the loss blew up"]


def heard(events: list[object], state: str) -> bool:
    """Wait until a job update with this state has arrived.

    Returns
    -------
    bool
        Whether it arrived within `WAIT` seconds.
    """
    deadline = time.monotonic() + WAIT

    def arrived() -> bool:
        return any(
            isinstance(e, JobUpdate) and e.state == state for e in events
        )

    while not arrived() and time.monotonic() < deadline:
        time.sleep(POLL)
    return arrived()


def test_the_job_example_stops_its_separate_process_job_at_once(
    monkeypatch: pytest.MonkeyPatch,
) -> None:
    """Stop kills `Stubborn`, which sleeps 30 s and never reports."""
    monkeypatch.syspath_prepend(str(EXAMPLES / "g_experiments"))
    app = build("g_experiments/g2_job_nodes.py")
    events: list[object] = []

    app.subscribe_jobs(events.append)
    app.start_job("n5")
    assert heard(events, "running")
    began = time.monotonic()
    app.stop_job("n5")
    assert heard(events, "stopped")
    assert time.monotonic() - began < KILLED_WITHIN
