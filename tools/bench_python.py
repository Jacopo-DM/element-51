"""Time the Python side on a long chain of nodes.

Builds a chain of `NODES` nodes (each takes the one before it), then times
three steps, `RUNS` times each: building the chain through `App.add_node` and
`App.add_link`, running it with `App.run`, and turning the snapshot into the
JSON a page receives. Prints the median and the range of each, in
milliseconds. Run it from the repo root: `uv run python tools/bench_python.py`.

Notes
-----
    * Run it on an idle machine and compare only numbers from the same
        machine; the range is the noise, and a gain inside it does not count.
    * `--profile` runs the three steps once under `cProfile` and prints the
        20 slowest calls by cumulative time.
"""

# Standard library
import argparse
import cProfile
import pstats
import statistics
import sys
import time
from collections.abc import Callable

from element_51 import App, node

NODES = 2000
RUNS = 11
TOP_CALLS = 20


@node
def start() -> float:
    """Start the chain.

    Returns
    -------
    float
        The first value.
    """
    return 1.0


@node
def step(value: float) -> float:
    """Add one to the value before it.

    Parameters
    ----------
    value : float
        The value of the node before it.

    Returns
    -------
    float
        One more than `value`.
    """
    return value + 1.0


def say(text: str) -> None:
    """Write a line to standard output."""
    sys.stdout.write(f"{text}\n")


def build() -> App:
    """Build a chain of `NODES` nodes.

    Returns
    -------
    App
        An app holding `start` and `NODES - 1` linked `step` nodes.
    """
    app = App([start, step])
    previous = app.add_node(start, x=0.0, y=0.0)
    for index in range(1, NODES):
        current = app.add_node(step, x=float(index), y=0.0)
        app.add_link(previous, "out", current, "value")
        previous = current
    return app


def milliseconds(action: Callable[[], object]) -> float:
    """Time one call of `action`.

    Returns
    -------
    float
        How long it took, in milliseconds.
    """
    began = time.perf_counter()
    action()
    return (time.perf_counter() - began) * 1000.0


def report(name: str, times: list[float]) -> None:
    """Print the median and the range of `times`."""
    say(
        f"{name} ({NODES} nodes): median {statistics.median(times):.1f} ms, "
        f"range {min(times):.1f} to {max(times):.1f} ms"
    )


def steps(app: App) -> None:
    """Run and serialise `app` once, the way a page and a script would."""
    app.run()
    app.snapshot().model_dump_json()


def main() -> None:
    """Print the timings, or the profile with `--profile`."""
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--profile", action="store_true")
    args = parser.parse_args()
    if args.profile:
        profiler = cProfile.Profile()
        profiler.enable()
        steps(build())
        profiler.disable()
        pstats.Stats(profiler).sort_stats("cumulative").print_stats(TOP_CALLS)
        return
    built = [milliseconds(build) for _ in range(RUNS)]
    app = build()
    ran = [milliseconds(app.run) for _ in range(RUNS)]
    sent = [
        milliseconds(lambda: app.snapshot().model_dump_json())
        for _ in range(RUNS)
    ]
    report("build", built)
    report("run", ran)
    report("snapshot json", sent)


if __name__ == "__main__":
    main()
