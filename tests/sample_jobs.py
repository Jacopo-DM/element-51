"""Job nodes shared by the tests."""

import threading
from typing import ClassVar, override

from element_51.job_node import JobNode
from element_51.model import Model
from element_51.visuals import Series

WAIT = 5.0
FIRST_TOTAL = 10


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
