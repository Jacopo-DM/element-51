"""The browser test runner: its verdicts, its lines and its exit code.

The runner itself starts Chromium, so only its pure parts run here. The
walk-throughs are run by `uv run python tools/run_browser_tests.py`.
"""

import importlib.util
import sys
from pathlib import Path
from types import ModuleType

import pytest

RUNNER = Path(__file__).parent.parent / "tools" / "run_browser_tests.py"
ROWS = 3
FIRST_PORT = 1024
LAST_PORT = 65535


def _load() -> ModuleType:
    """Import `tools/run_browser_tests.py` by its path.

    Returns
    -------
    ModuleType
        The runner module.
    """
    spec = importlib.util.spec_from_file_location("run_browser_tests", RUNNER)
    assert spec is not None
    assert spec.loader is not None
    module = importlib.util.module_from_spec(spec)
    sys.modules["run_browser_tests"] = module
    spec.loader.exec_module(module)
    return module


@pytest.fixture(scope="module")
def runner() -> ModuleType:
    """Load the runner once.

    Returns
    -------
    ModuleType
        The runner module.
    """
    return _load()


def test_a_walk_with_no_failed_row_passes(runner: ModuleType) -> None:
    """Every row passed: the verdict is PASS and carries no detail."""
    rows = [("a", True, "x")] * ROWS
    result = runner.Result(passed=ROWS, failed=0, rows=rows)
    outcome = runner.judge(runner.WALKS[0], result, 1.5)
    assert outcome.status == "PASS"
    assert outcome.passed == ROWS
    assert outcome.detail == ()


def test_a_failed_row_fails_the_walk_and_is_listed(runner: ModuleType) -> None:
    """One failed row: FAIL, and the line names the step and what was seen."""
    result = runner.Result(
        passed=1,
        failed=1,
        rows=[("one", True, "ok"), ("two is shown", False, "hidden")],
    )
    outcome = runner.judge(runner.WALKS[0], result, 2.0)
    assert outcome.status == "FAIL"
    assert outcome.detail == ("two is shown: saw hidden",)


def test_a_walk_that_ran_no_rows_fails(runner: ModuleType) -> None:
    """A function that returns nothing must not count as a pass."""
    result = runner.Result(passed=0, failed=0, rows=[])
    outcome = runner.judge(runner.WALKS[0], result, 0.1)
    assert outcome.status == "FAIL"
    assert outcome.detail == ("no rows ran",)


def test_other_fields_of_the_result_are_ignored(runner: ModuleType) -> None:
    """`uiKeys` also returns `stoppedOn`; the runner does not need it."""
    result = runner.Result.model_validate({
        "passed": 1,
        "failed": 0,
        "rows": [["a", True, "x"]],
        "stoppedOn": "n1",
    })
    assert result.rows == [("a", True, "x")]


def test_a_walk_that_threw_fails_with_the_first_line(
    runner: ModuleType,
) -> None:
    """An error becomes a FAIL that shows its first line only."""
    outcome = runner.broke(runner.WALKS[0], "boom\nstack trace", 3.0)
    assert outcome.status == "FAIL"
    assert outcome.detail == ("boom",)


def test_an_empty_error_still_says_something(runner: ModuleType) -> None:
    """A blank error text still gives a reason."""
    outcome = runner.broke(runner.WALKS[0], "  ", 0.0)
    assert outcome.detail == ("unknown error",)


def test_the_report_has_one_line_and_one_per_failed_row(
    runner: ModuleType,
) -> None:
    """The report starts with the name and status, then indents the rows."""
    outcome = runner.Outcome(
        name="uiKeys",
        status="FAIL",
        seconds=12.34,
        passed=88,
        detail=("a: saw 1", "b: saw 2"),
    )
    lines = runner.report(outcome).splitlines()
    assert lines[0].startswith("uiKeys")
    assert "FAIL" in lines[0]
    assert "12.3s" in lines[0]
    assert "88 rows passed" in lines[0]
    assert lines[1:] == ["    a: saw 1", "    b: saw 2"]


def test_the_exit_code_is_one_when_any_walk_failed(runner: ModuleType) -> None:
    """PASS alone exits 0; a FAIL anywhere exits 1."""
    ok = runner.Outcome(name="a", status="PASS", seconds=1.0, passed=1)
    bad = runner.Outcome(name="b", status="FAIL", seconds=1.0, passed=0)
    assert runner.exit_code([ok]) == 0
    assert runner.exit_code([]) == 0
    assert runner.exit_code([ok, bad]) == 1


def test_the_script_races_the_walk_against_a_timer(runner: ModuleType) -> None:
    """The page rejects a walk that takes longer than the timeout."""
    script = runner.expression("uiKeys", 7)
    assert script.startswith("Promise.race([uiKeys(),")
    assert "7000" in script
    assert "timed out after 7 s" in script


def test_a_missing_browser_is_told_from_other_errors(
    runner: ModuleType,
) -> None:
    """Playwright's own advice marks the missing browser."""
    missing = "Executable doesn't exist. Please run: playwright install"
    assert runner.is_missing_browser(missing)
    assert not runner.is_missing_browser("net::ERR_CONNECTION_REFUSED")


def test_a_free_port_can_be_bound_and_is_not_listening(
    runner: ModuleType,
) -> None:
    """The port the runner picks has nothing on it yet."""
    port = runner.free_port()
    assert FIRST_PORT <= port <= LAST_PORT
    assert not runner._is_listening(port)  # ruff: ignore[private-member-access]  # tests the helper


def test_every_walk_names_a_function_in_the_keys_script(
    runner: ModuleType,
) -> None:
    """A walk whose function is missing from `ui_keys.js` is caught here."""
    source = runner.KEYS_SCRIPT.read_text()
    for walk in runner.WALKS:
        assert f"async function {walk.function}(" in source


def test_all_walks_run_unless_some_are_named(runner: ModuleType) -> None:
    """No names means all five; names pick walks and keep their order."""
    assert runner.select_walks(None) == runner.WALKS
    assert runner.select_walks([]) == runner.WALKS
    picked = runner.select_walks(["uiKeysJob", "uiKeys"])
    assert [walk.function for walk in picked] == ["uiKeys", "uiKeysJob"]


def test_the_five_walks_match_the_graphs_they_need(runner: ModuleType) -> None:
    """The display walk needs the display graph, the job walk the job one."""
    graphs = {walk.function: walk.graph for walk in runner.WALKS}
    assert graphs == {
        "uiKeys": "standard",
        "uiKeysDisplay": "display",
        "uiKeysJob": "job",
        "uiKeysAuto": "standard",
        "uiKeysSlider": "slider",
    }
