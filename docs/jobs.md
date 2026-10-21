# Jobs

Run presses every node once and waits. A long task, such as a training run,
needs something else: a node you start by hand, that works in the background
and shows what it has so far. That is a job node. Its class is
`element_51.JobNode`.

Run never starts a job. The Start button on the node's card does. While the job
works its card shows the state (`running`, `stopping`, `finished`, `stopped` or
`failed`), and a Plot linked to it draws the curves as they grow.

## A job in steps

A job that works in steps defines three methods. `steps` says how many, `begin`
makes the state the steps share, and `step` does step `i` and returns the new
values of the outputs. The fields are the inputs, as for any `Node`.

```python
import math
import time

import element_51


class Curves(element_51.Model):
    loss: element_51.Series
    accuracy: element_51.Series


class Training(element_51.JobNode[list[float]]):
    epochs: int = 40

    def steps(self) -> int:
        return self.epochs

    def begin(self) -> list[float]:
        return [1.0]

    def step(self, state: list[float], i: int) -> Curves:
        time.sleep(0.2)
        state[0] *= 0.93
        return Curves(
            loss=element_51.Series(values=(state[0],)),
            accuracy=element_51.Series(values=(1.0 - state[0],)),
        )
```

The return hint of `step` gives the outputs, as `run`'s does for a `Node`. The
type in brackets after `JobNode` is the type of the state that `begin` returns
and `step` receives.

A series output grows by what each step returns: step 1 returns one point of
`loss`, step 2 returns the next, and the card and the Plot show them all. Any
other output (a number, a picture) is replaced. A step that returns a series of
several points adds all of them.

The card shows a bar and `step 3 of 40`. Stop ends the job before its next step.

## A job in one call

A job that cannot be split into steps defines `work` instead, and calls `report`
when it has something to show:

```python
class Countdown(element_51.JobNode[None]):
    def work(self, report: element_51.Report) -> int:
        for number in range(5, 0, -1):
            time.sleep(1.0)
            report(out=number)
        return 0
```

`report` takes the outputs it has new values for, by name. A name that is not an
output, or a value of the wrong kind, fails the job. A `work` job shows
`running` and no bar. Stop takes effect at the next `report` call, which raises
`element_51.Stopped`. A job may catch it to clean up, and must then end. A job
that never calls `report` cannot be stopped; its card says `stopping` until it
ends by itself.

Define `step` (with `begin` and `steps`) or `work`, not both.

## What the other nodes see

A node after a job sees the job's latest values, and a Plot linked to the job
redraws on every report (at most 20 times a second). Every other node after the
job runs once, when the job ends, with its final values. Before the job has
reported anything those nodes are skipped.

The results stay until the next Start, even if you change an input of the job.
They are not saved with the graph. Deleting the job node, or loading another
graph, ends its job and drops its results.

Start first runs the nodes the job depends on, once, and gives the job their
values and the values in its own boxes as they are then. A change made while the
job runs does not reach it. If one of those nodes fails, the job does not start
and the page says which.

Two different job nodes may run together. One node never has two jobs: Start
while it runs or stops is refused, with a reason.

## From a script

```python
app.start_job("n1")
app.stop_job("n1")
app.job_states()
app.subscribe_jobs(listener)
```

`start_job` and `stop_job` raise `element_51.jobs.JobRefusedError` when the
request is refused; its `reason` is worded for a person. A listener hears a
`JobUpdate` for each change, and, when a job ends, a partial `RunResult` with
the nodes after it. It is called from the job's thread and must not block.

## Things to know

- The job runs in a thread of your Python process, so it can use anything the
    process can, and it ends when the process does. Stop is cooperative: it works
    at the next step or report.
- Define the class in a module Python can import, such as your script. A class
    made by `exec` has no module, and pydantic cannot build `JobNode[...]` for it.
- An exception in the job fails it: the card says `failed` and the banner shows
    the reason, with a Dismiss button. The editor keeps working.
- The Run button reads `Running` while a plain run is in progress.
- From the keyboard, a job node's Start and Stop come after its ports in the
    hidden list: Right enters the ports, Down reaches them, Enter presses one.
    A screen reader is told when the state changes.

See `examples/g_experiments/g2_job_nodes.py` for a runnable version of all of
this.
