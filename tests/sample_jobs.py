"""Job nodes shared by the tests."""

import os
import threading
import time
from typing import ClassVar, override

from element_51.job_node import JobNode, Report
from element_51.model import Model
from element_51.visuals import Series

WAIT = 5.0
FIRST_TOTAL = 10
LONG = 30.0
TICK = 0.02
EXIT_CODE = 3


class Out(Model):
    loss: Series
    total: int


class Pause:
    """A gate a job waits at, so a test can act while the job is running."""

    entered = threading.Event()
    release = threading.Event()

    @classmethod
    def reset(cls) -> None:
        cls.entered = threading.Event()
        cls.release = threading.Event()

    @classmethod
    def wait(cls) -> None:
        cls.entered.set()
        assert cls.release.wait(WAIT)


class Steps(JobNode[int]):
    """A step job: step i adds i to 10 and gives the point i."""

    count: int = 3
    pause_at: ClassVar[int] = -1

    @override
    def steps(self) -> int:
        return self.count

    @override
    def begin(self) -> int:
        return FIRST_TOTAL

    @override
    def step(self, state: int, i: int) -> Out:
        if i == self.pause_at:
            Pause.wait()
        return Out(loss=Series(values=(float(i),)), total=state + i)


class Stamp:
    """When `Timed` last reported, on this process's monotonic clock."""

    at = 0.0


class Timed(JobNode[None]):
    """A work job that notes the time just before it reports once."""

    @override
    def work(self, report: Report) -> int:
        Stamp.at = time.monotonic()
        report(out=1)
        return 1


class Counting(JobNode[None]):
    """An isolated work job: reports 1 to `upto`, then returns `base + upto`."""

    isolated: ClassVar[bool] = True
    upto: int = 3
    base: int = 10

    @override
    def work(self, report: Report) -> int:
        for number in range(1, self.upto + 1):
            report(out=number)
        return self.base + self.upto


class IsoSteps(JobNode[int]):
    """An isolated step job: the same steps as `Steps`."""

    isolated: ClassVar[bool] = True
    count: int = 3

    @override
    def steps(self) -> int:
        return self.count

    @override
    def begin(self) -> int:
        return FIRST_TOTAL

    @override
    def step(self, state: int, i: int) -> Out:
        return Out(loss=Series(values=(float(i),)), total=state + i)


class Sleeper(JobNode[None]):
    """An isolated job that sleeps far longer than any test waits."""

    isolated: ClassVar[bool] = True

    @override
    def work(
        self, report: Report
    ) -> int:  # pragma: no cover  # runs in the child; killed
        del report
        time.sleep(LONG)
        return 0


class Chatty(JobNode[None]):
    """An isolated job that reports until Stop makes a report raise."""

    isolated: ClassVar[bool] = True

    @override
    def work(self, report: Report) -> int:
        number = 0
        while True:
            number += 1
            report(out=number)
            time.sleep(TICK)  # pragma: no cover  # reached in the child only


class Dies(JobNode[None]):
    """An isolated job whose process ends at once, saying nothing."""

    isolated: ClassVar[bool] = True

    @override
    def work(
        self, report: Report
    ) -> int:  # pragma: no cover  # runs in the child; exits
        del report
        os._exit(EXIT_CODE)


class Raises(JobNode[None]):
    """An isolated job whose code raises."""

    isolated: ClassVar[bool] = True

    @override
    def work(self, report: Report) -> int:
        del report
        msg = "bad"
        raise ValueError(msg)
