"""A node you start by hand that works in the background.

A job node is a `JobNode` class. Run never starts it; its own Start button
does, and it reports as it goes, so a card or a plot fills in while it works.
`Training` is a step job: `steps` says how many, `begin` makes the state the
steps share, and `step` does one step and returns the new points of the two
series it makes. `Countdown` is a work job: one method that calls `report`
itself.
`Stubborn` is a job in a separate process (`isolated = True`): it sleeps
and never reports, and Stop still ends it.
`double` is a plain node after it, and `Plot` draws `Training`'s two series.

Build the browser layer first: `uv run python tools/build_wasm.py`.

Notes
-----
    * Press Run first: nothing happens to the two jobs. Their cards say
        `idle`, and the nodes after them are dimmed, as skipped. Run never
        starts a job.
    * Press Start on `Training`. The bar fills, the card says `step 3 of 40`,
        and the `Plot` card draws both curves as they grow, one point a step.
    * Press Stop on `Training` while it runs: the card says `stopping`, then
        `stopped` after the step it was in. The curves stay as they were.
        Start again: they begin over.
    * Tick `blow_up` in `Training` and press Start: at step 10 it raises, the
        card says `failed`, and the banner shows the reason, with Dismiss.
    * Press Start on `Countdown`. Its output box counts 5, 4, 3, 2, 1 as the
        job reports. `double` stays empty until the job ends, then shows 0 (the
        last value the job returned, doubled): a node after a job runs once,
        when the job ends.
    * Press Start on `Stubborn`, then Stop: the card says `stopping` and, half
        a second later, `stopped`. The job never reports and sleeps 30 s, so
        Stop could not reach it in a thread. Its own process is killed
        instead, and the editor never noticed.
    * Press Start with the keyboard: move to a job node in the hidden tree,
        press Right to enter its ports, Down to reach the Start and Stop items,
        and Enter on one. A screen reader hears `n1: running.` and
        `n1: finished, step 40 of 40.`.

"""

# Standard library
import math
import time
from typing import override

# Third-party libraries
from rich.console import Console
from rich.traceback import install

# Local libraries
import element_51

# --- Globals --- #
DECAY = 0.93
RIPPLE = 0.02
BLOW_UP_AT = 10
COUNT_FROM = 5

# --- Warnings --- #

# --- Typing --- #

# --- TERMINAL OUTPUT SETUP --- #
install(show_locals=False)
console = Console()


class Curves(element_51.Model):
    """The two series `Training` makes, one new point of each per step."""

    loss: element_51.Series
    accuracy: element_51.Series


class Training(element_51.JobNode[list[float]]):
    """A pretend training run: a falling loss and a rising accuracy."""

    epochs: int = 40
    delay: float = 0.2
    blow_up: bool = False

    @override
    def steps(self) -> int:
        """Say how many steps there are.

        Returns
        -------
        int
            One step per epoch.
        """
        return self.epochs

    @override
    def begin(self) -> list[float]:
        """Make the state the steps share: the loss so far.

        Returns
        -------
        list[float]
            A list holding the starting loss, 1.0.
        """
        return [1.0]

    @override
    def step(self, state: list[float], i: int) -> Curves:
        """Do one epoch.

        Returns
        -------
        Curves
            The new point of each series: a series grows by what a step
            returns.

        Raises
        ------
        RuntimeError
            At step `BLOW_UP_AT` if `blow_up` is ticked.
        """
        time.sleep(self.delay)
        if self.blow_up and i == BLOW_UP_AT:
            msg = "the loss blew up"
            raise RuntimeError(msg)
        state[0] *= DECAY
        return Curves(
            loss=element_51.Series(values=(state[0] + RIPPLE * math.sin(i),)),
            accuracy=element_51.Series(values=(1.0 - state[0],)),
        )


class Countdown(element_51.JobNode[None]):
    """A work job: counts down, one report a pause."""

    pause: float = 1.0

    @override
    def work(self, report: element_51.Report) -> int:
        """Count from five to one, reporting each number.

        Parameters
        ----------
        report
            Shows a new value for the output while the job works.

        Returns
        -------
        int
            Zero, the last value of the output.
        """
        for number in range(COUNT_FROM, 0, -1):
            time.sleep(self.pause)
            report(out=number)
        return 0


class Stubborn(element_51.JobNode[None]):
    """A job in its own process that sleeps and never reports."""

    isolated = True
    seconds: float = 30.0

    @override
    def work(self, report: element_51.Report) -> int:
        """Sleep, ignoring `report`.

        Parameters
        ----------
        report
            Not used: the job never reports, so only a kill can stop it.

        Returns
        -------
        int
            Zero.
        """
        del report
        time.sleep(self.seconds)
        return 0


@element_51.node
def double(value: int) -> int:
    """Double a number.

    Returns
    -------
    int
        Twice `value`.
    """
    return value * 2


def build_app() -> element_51.App:
    """Build the app: three jobs and the nodes after them.

    A step job feeds a Plot, a work job feeds a node, and a third job runs in
    its own process.

    Returns
    -------
    element_51.App
        An app whose first node is `Training` (n1), the second `Countdown`
        (n2), the third `double` (n3) after it, and the fourth a Plot with
        both of `Training`'s series linked, and the fifth `Stubborn` (n5).
    """
    app = element_51.App([
        Training,
        Countdown,
        Stubborn,
        double,
    ])
    training = app.add_node(Training, x=60.0, y=60.0)
    countdown = app.add_node(Countdown, x=60.0, y=420.0)
    twice = app.add_node(double, x=360.0, y=420.0)
    chart = app.add_node(element_51.PlotNode, x=360.0, y=60.0)
    app.add_node(Stubborn, x=60.0, y=640.0)
    app.add_link(countdown, "out", twice, "value")
    app.add_link(training, "loss", chart, "series_1")
    app.add_link(training, "accuracy", chart, "series_2")
    return app


def main() -> None:
    """Entry point."""
    element_51.serve(build_app())


if __name__ == "__main__":  # pragma: no cover
    main()
