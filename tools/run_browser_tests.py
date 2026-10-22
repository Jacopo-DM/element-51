"""Run the keyboard walk-throughs of `tools/ui_keys.js` in a real browser.

Each walk-through gets a fresh `tools/ui_states.py` server on a free port (the
rows delete and move things, so one server serves one walk), a fresh page in
Playwright's headless Chromium, and a click on the empty canvas for focus.
The runner loads `tools/ui_keys.js` into the page, calls the walk-through's
function and reads the rows it returns. One line is printed per walk-through:
its name, `PASS` or `FAIL`, the seconds and the number of rows that passed. A
failure adds each failed row. The exit code is 1 when any walk-through failed.

Run it from the repo root, after `uv run python tools/build_wasm.py`::

    uv run python tools/run_browser_tests.py

Notes
-----
    * Chromium is a separate download: `uv run playwright install
        --only-shell chromium`. Without it the run fails and prints the command.
    * The servers use free ports, never 8765 (the examples) or 8768
        (`tools/ui_states.py` by hand).
    * The rows are the checks: nothing here rewrites them. A walk-through that
        throws, times out or returns no rows counts as failed.
"""

# Standard library
import argparse
import contextlib
import socket
import subprocess  # ruff: ignore[suspicious-subprocess-import]
import sys
import time
from collections.abc import Generator, Sequence
from pathlib import Path
from typing import Literal

# Third-party libraries
from playwright.sync_api import Browser, Page, ViewportSize, sync_playwright
from playwright.sync_api import Error as PlaywrightError
from pydantic import BaseModel, ConfigDict

# --- Globals --- #
ROOT = Path(__file__).resolve().parent.parent
KEYS_SCRIPT = ROOT / "tools" / "ui_keys.js"
STATES_SCRIPT = ROOT / "tools" / "ui_states.py"
WEB_INDEX = ROOT / "src" / "element_51" / "web" / "index.html"
DEFAULT_TIMEOUT = 120
START_SECONDS = 30.0
POLL_SECONDS = 0.1
STOP_SECONDS = 10.0
VIEWPORT = ViewportSize(width=1280, height=800)
EMPTY_CANVAS = (1150, 700)
INSTALL_COMMAND = "uv run playwright install --only-shell chromium"
BUILD_COMMAND = "uv run python tools/build_wasm.py"
NAME_WIDTH = 16
MILLISECONDS = 1000

type Graph = Literal["standard", "display", "job"]
type Status = Literal["PASS", "FAIL"]
type Row = tuple[str, bool, str]


class Strict(BaseModel):
    """A frozen record that refuses fields it does not know."""

    model_config = ConfigDict(frozen=True, extra="forbid")


class Walk(Strict):
    """One walk-through: the function in `ui_keys.js` and the graph it needs."""

    function: str
    graph: Graph


class Result(BaseModel):
    """What a walk-through function returns (other fields are ignored)."""

    model_config = ConfigDict(frozen=True, extra="ignore")

    passed: int
    failed: int
    rows: list[Row]


class Outcome(Strict):
    """The verdict on one walk-through."""

    name: str
    status: Status
    seconds: float
    passed: int
    detail: tuple[str, ...] = ()


WALKS = (Walk(function="uiKeys", graph="standard"),)


def expression(function: str, timeout: int) -> str:
    """Build the script that runs a walk-through with a time limit.

    Parameters
    ----------
    function
        The function in `ui_keys.js`, for example `uiKeys`.
    timeout
        Seconds the walk may take before the page rejects it.

    Returns
    -------
    str
        A script that resolves to the walk's result.
    """
    limit = f'new Error("timed out after {timeout} s")'
    late = f"setTimeout(() => no({limit}), {timeout * MILLISECONDS})"
    return f"Promise.race([{function}(), new Promise((_, no) => {late})])"


def failed_rows(result: Result) -> tuple[str, ...]:
    """List the rows that did not pass.

    Parameters
    ----------
    result
        The walk's result.

    Returns
    -------
    tuple[str, ...]
        One line per failed row: the step and what was seen.
    """
    return tuple(
        f"{step}: saw {seen}"
        for step, passed, seen in result.rows
        if not passed
    )


def judge(walk: Walk, result: Result, seconds: float) -> Outcome:
    """Turn a walk-through's result into a verdict.

    Parameters
    ----------
    walk
        The walk that ran.
    result
        What it returned.
    seconds
        How long it took.

    Returns
    -------
    Outcome
        `PASS` when no row failed and at least one row ran, else `FAIL`.
    """
    detail = failed_rows(result)
    ok = result.failed == 0 and result.passed > 0 and not detail
    if result.passed == 0 and not detail:
        detail = ("no rows ran",)
    return Outcome(
        name=walk.function,
        status="PASS" if ok else "FAIL",
        seconds=seconds,
        passed=result.passed,
        detail=detail,
    )


def broke(walk: Walk, error: str, seconds: float) -> Outcome:
    """Make the verdict for a walk-through that threw or timed out.

    Parameters
    ----------
    walk
        The walk that ran.
    error
        The error text.
    seconds
        How long it took.

    Returns
    -------
    Outcome
        A `FAIL` that carries the first line of the error.
    """
    first = error.strip().splitlines()[0] if error.strip() else "unknown error"
    return Outcome(
        name=walk.function,
        status="FAIL",
        seconds=seconds,
        passed=0,
        detail=(first,),
    )


def report(outcome: Outcome) -> str:
    """Format an outcome as the lines printed for it.

    Parameters
    ----------
    outcome
        The verdict on one walk-through.

    Returns
    -------
    str
        `name  STATUS  seconds  rows passed`, then one indented line per
        failed row.
    """
    name = f"{outcome.name:<{NAME_WIDTH}}"
    took = f"{outcome.seconds:6.1f}s"
    line = f"{name} {outcome.status}  {took}  {outcome.passed} rows passed"
    return "\n".join([line, *(f"    {row}" for row in outcome.detail)])


def exit_code(outcomes: Sequence[Outcome]) -> int:
    """Decide the exit code of the run.

    Parameters
    ----------
    outcomes
        The verdicts so far.

    Returns
    -------
    int
        0 when none failed, else 1.
    """
    return 1 if any(outcome.status == "FAIL" for outcome in outcomes) else 0


def is_missing_browser(error: str) -> bool:
    """Say whether a Playwright error means Chromium is not installed.

    Parameters
    ----------
    error
        The error text.

    Returns
    -------
    bool
        True when Playwright asks for `playwright install`.
    """
    return "playwright install" in error


def free_port() -> int:
    """Ask the system for a port nobody is using.

    Returns
    -------
    int
        A free port on 127.0.0.1.
    """
    with socket.socket() as probe:
        probe.bind(("127.0.0.1", 0))
        port: int = probe.getsockname()[1]
    return port


def _is_listening(port: int) -> bool:
    """Say whether something accepts connections on `port`.

    Parameters
    ----------
    port
        The port on 127.0.0.1.

    Returns
    -------
    bool
        True when a connection succeeds.
    """
    with socket.socket() as probe:
        probe.settimeout(POLL_SECONDS)
        return probe.connect_ex(("127.0.0.1", port)) == 0


@contextlib.contextmanager
def serving(graph: Graph, port: int) -> Generator[str]:
    """Run `tools/ui_states.py` for a graph and stop it afterwards.

    Parameters
    ----------
    graph
        The graph the server shows.
    port
        The port to serve on.

    Yields
    ------
    str
        The address of the page.

    Raises
    ------
    RuntimeError
        When the server does not listen within `START_SECONDS`.
    """
    command = [
        sys.executable,
        str(STATES_SCRIPT),
        "--graph",
        graph,
        "--port",
        str(port),
    ]
    server = subprocess.Popen(  # ruff: ignore[subprocess-without-shell-equals-true]
        command,
        cwd=ROOT,
        stdout=subprocess.DEVNULL,
        stderr=subprocess.DEVNULL,
    )
    try:
        deadline = time.monotonic() + START_SECONDS
        while not _is_listening(port):
            if server.poll() is not None or time.monotonic() > deadline:
                message = f"the {graph} server did not start on port {port}"
                raise RuntimeError(message)
            time.sleep(POLL_SECONDS)
        yield f"http://127.0.0.1:{port}"
    finally:
        server.terminate()
        try:
            server.wait(STOP_SECONDS)
        except subprocess.TimeoutExpired:  # pragma: no cover
            server.kill()


def _drive(page: Page, walk: Walk, address: str, timeout: int) -> Result:
    """Open the page, give it focus and run the walk's function in it.

    Parameters
    ----------
    page
        A fresh page.
    walk
        The walk to run.
    address
        The address of the server.
    timeout
        Seconds the walk may take.

    Returns
    -------
    Result
        What the function returned.
    """
    page.goto(address)
    page.wait_for_function(
        "document.documentElement.classList.contains('ready')"
    )
    page.mouse.click(*EMPTY_CANVAS)
    page.add_script_tag(content=KEYS_SCRIPT.read_text())
    return Result.model_validate(
        page.evaluate(expression(walk.function, timeout))
    )


def run_walk(browser: Browser, walk: Walk, timeout: int) -> Outcome:
    """Run one walk-through in a fresh page against a fresh server.

    Parameters
    ----------
    browser
        The running browser.
    walk
        The walk to run.
    timeout
        Seconds the walk may take.

    Returns
    -------
    Outcome
        The verdict.
    """
    began = time.monotonic()
    try:
        with serving(walk.graph, free_port()) as address:
            page = browser.new_page(viewport=VIEWPORT)
            result = _drive(page, walk, address, timeout)
            page.close()
    except (PlaywrightError, RuntimeError, ValueError) as error:
        return broke(walk, str(error), time.monotonic() - began)
    return judge(walk, result, time.monotonic() - began)


def _parse(argv: Sequence[str] | None) -> argparse.Namespace:
    """Read the command line.

    Parameters
    ----------
    argv
        The arguments, or None for `sys.argv`.

    Returns
    -------
    argparse.Namespace
        The parsed arguments.
    """
    parser = argparse.ArgumentParser(description=__doc__.split("\n")[0])
    parser.add_argument(
        "--timeout",
        type=int,
        default=DEFAULT_TIMEOUT,
        help="seconds allowed per walk-through (default: %(default)s)",
    )
    return parser.parse_args(argv)


def main(argv: Sequence[str] | None = None) -> int:
    """Run the walk-throughs and print one line for each.

    Parameters
    ----------
    argv
        The arguments, or None for `sys.argv`.

    Returns
    -------
    int
        0 when no walk-through failed, 1 otherwise.

    Raises
    ------
    Error
        When Chromium cannot start for a reason other than being missing.
    """
    args = _parse(argv)
    if not WEB_INDEX.exists():
        sys.stdout.write(
            f"The browser layer is not built. Run: {BUILD_COMMAND}\n"
        )
        return 1
    outcomes: list[Outcome] = []
    with sync_playwright() as playwright:
        try:
            browser = playwright.chromium.launch()
        except PlaywrightError as error:
            if not is_missing_browser(str(error)):
                raise
            sys.stdout.write(
                f"Chromium is not installed. Run: {INSTALL_COMMAND}\n"
            )
            return 1
        with contextlib.closing(browser):
            for walk in WALKS:
                outcome = run_walk(browser, walk, args.timeout)
                sys.stdout.write(report(outcome) + "\n")
                sys.stdout.flush()
                outcomes.append(outcome)
    return exit_code(outcomes)


if __name__ == "__main__":  # pragma: no cover
    sys.exit(main())
