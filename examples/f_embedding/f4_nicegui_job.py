"""A NiceGUI panel that starts a background job and follows its progress.

The page is a splitter: the editor on the left, a panel written in NiceGUI on
the right. The graph has a job node, `Ticker`, that adds one number to a
running total every 0.3 seconds, and a Plot that draws the totals. The panel
starts and stops the job, shows a progress bar and the job's state, and
counts the nodes in the graph.

A job runs in its own thread and tells the `App` as it goes. The panel
listens with `App.subscribe_jobs`, keeps the latest update, and a NiceGUI
timer redraws the widgets five times a second. The listener never touches a
widget itself, because it is called from the job's thread.

Run it with one worker only: the `App` lives in this process.

Build the browser layer first: `uv run python tools/build_wasm.py`.

Notes
-----
    * Run the script and open http://127.0.0.1:8765 if it does not open for you.
    * Press `Start` in the panel: the bar fills in about 6 seconds and the
        Plot in the editor grows one point a step.
    * Press `Start` again while it runs: a message says the job is already
        running, and stays until you press `Dismiss`.
    * Press `Stop` during a run: the state shows `stopped` and the Plot keeps
        what it has.
    * Press `Start` on the `Ticker` card in the editor instead: the panel
        follows, because both talk to the same `App`.
    * Add or delete a node in the editor: the node count in the panel follows
        within a second.

"""

# Standard library
import time
from typing import override

# Third-party libraries
import uvicorn
from fastapi import FastAPI
from nicegui import ui
from rich.console import Console
from rich.traceback import install

# Local libraries
import element_51
from element_51.jobs import JobRefusedError
from element_51.protocol import JobUpdate
from element_51.session import JobEvent

# --- Globals --- #
PORT = 8765
JOB = "n1"
POLL_SECONDS = 0.2
FIRST_X = 60.0
FIRST_Y = 60.0

# --- Warnings --- #

# --- Typing --- #

# --- TERMINAL OUTPUT SETUP --- #
install(show_locals=False)
console = Console()


class Tick(element_51.Model):
    """The series `Ticker` makes, one new point per step."""

    total: element_51.Series


class Ticker(element_51.JobNode[list[int]]):
    """A pretend job: a running total that grows a little more each step."""

    ticks: int = 20
    pause: float = 0.3

    @override
    def steps(self) -> int:
        """Say how many steps there are.

        Returns
        -------
        int
            One step per tick.
        """
        return self.ticks

    @override
    def begin(self) -> list[int]:
        """Make the state the steps share: the total so far.

        Returns
        -------
        list[int]
            A list holding the starting total, 0.
        """
        return [0]

    @override
    def step(self, state: list[int], i: int) -> Tick:
        """Add `i + 1` to the total.

        Returns
        -------
        Tick
            The new total: a series grows by what a step returns.
        """
        time.sleep(self.pause)
        state[0] += i + 1
        return Tick(total=element_51.Series(values=(float(state[0]),)))


def describe(update: JobUpdate | None) -> str:
    """Say in a few words how a job stands.

    Parameters
    ----------
    update
        The latest update the panel heard, or `None` if there is none yet.

    Returns
    -------
    str
        For example `running, step 4 of 20`, or `idle`.
    """
    if update is None:
        return "idle"
    text = f"{update.state}, step {update.step} of {update.steps}"
    if update.error is not None:
        text = f"{text}: {update.error}"
    return text


def fraction(update: JobUpdate | None) -> float:
    """Work out how far a job has got.

    Parameters
    ----------
    update
        The latest update the panel heard, or `None` if there is none yet.

    Returns
    -------
    float
        From 0.0 to 1.0.
    """
    if update is None or not update.steps:
        return 0.0
    return update.step / update.steps


def build_app() -> element_51.App:
    """Build the graph: the job, and a Plot that draws its totals.

    Returns
    -------
    element_51.App
        An app whose first node, `n1`, is a `Ticker`, with a Plot, `n2`,
        linked to its series.
    """
    app = element_51.App([Ticker, element_51.PlotNode])
    ticker = app.add_node(Ticker, x=FIRST_X, y=FIRST_Y)
    chart = app.add_node(element_51.PlotNode, x=360.0, y=FIRST_Y)
    app.add_link(ticker, "total", chart, "series_1")
    return app


def build_host(app: element_51.App) -> FastAPI:
    """Build the FastAPI app that hosts NiceGUI and the editor.

    Parameters
    ----------
    app
        The session the editor and the panel share.

    Returns
    -------
    FastAPI
        The host, with the editor mounted at `/editor` and NiceGUI at `/`.
    """
    host = FastAPI()
    host.mount("/editor", element_51.asgi_app(app))
    latest: dict[str, JobUpdate] = {}

    def listen(event: JobEvent) -> None:
        if isinstance(event, JobUpdate):
            latest[event.node] = event

    app.subscribe_jobs(listen)

    @ui.page("/")
    def page() -> None:
        ui.add_css("html { color-scheme: light; }")

        def ask(action: str) -> None:
            try:
                if action == "start":
                    app.start_job(JOB)
                else:
                    app.stop_job(JOB)
            except JobRefusedError as error:
                ui.notify(
                    error.reason,
                    type="warning",
                    close_button="Dismiss",
                    timeout=0,
                )

        def refresh() -> None:
            update = latest.get(JOB)
            state.set_text(describe(update))
            bar.set_value(fraction(update))
            nodes.set_text(f"{len(app.graph.nodes)} nodes in the graph")

        with ui.splitter(value=75).classes("w-full h-screen") as splitter:
            with splitter.before:
                ui.element("iframe").props("src=/editor/").style(
                    "width: 100%; height: 100%; border: 0"
                )
            with splitter.after, ui.column().classes("p-4 w-full"):
                ui.label("Side panel").classes("text-lg")
                with ui.row():
                    ui.button("Start", on_click=lambda: ask("start"))
                    ui.button("Stop", on_click=lambda: ask("stop"))
                bar = ui.linear_progress(value=0.0, show_value=False)
                state = ui.label()
                nodes = ui.label()
        ui.timer(POLL_SECONDS, refresh)

    ui.run_with(host, mount_path="/")
    return host


def main() -> None:
    """Entry point."""
    uvicorn.run(build_host(build_app()), host="127.0.0.1", port=PORT)


if __name__ == "__main__":  # pragma: no cover
    main()
