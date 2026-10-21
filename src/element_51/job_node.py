"""A node that is started and stopped by hand and works in the background."""

from collections.abc import Mapping
from typing import ClassVar, Protocol, get_type_hints, override

from element_51.nodes import Node
from element_51.visuals import Carried


class Stopped(BaseException):
    """Raised by `Report` after Stop, to end a job.

    A job may catch it to clean up and must then end. It derives from
    `BaseException`, so a job's `except Exception` does not swallow it.
    """


class Report(Protocol):
    """What a job calls to show partial outputs while it works."""

    @property
    def stopped(self) -> bool:
        """Whether Stop was pressed. The next call raises `Stopped`."""

    def __call__(self, **fields: Carried) -> None:
        """Show new values for some of the job's outputs.

        A series output grows by the points given; any other output is
        replaced. Raises `Stopped` if Stop was pressed.
        """


class JobNode[State](Node):
    """Base class for a node that is started and stopped by hand.

    Fields are the inputs, as in `Node`. A job works in steps: `steps` says how
    many, `begin` makes the job's state, and `step` does step `i` and returns
    the new values of the outputs. A job that cannot be split overrides `work`
    instead and calls `report` itself. Define one form, not both. Run never
    starts a job; the Start button does.

    Set `isolated = True` in the class body to run the job in a separate
    process. Stop then ends it even if it never steps or reports, and a crash
    stays in that process. The class must be importable by module and name,
    and the inputs and everything the job reports must be picklable.

    The return hint of `step` (or `work`) gives the outputs, as `run`'s does for
    a `Node`.
    """

    is_job: ClassVar[bool] = True
    isolated: ClassVar[bool] = False
    is_stepped: ClassVar[bool]

    def steps(self) -> int:
        """Say how many steps the job takes. A step job overrides this.

        Raises
        ------
        NotImplementedError
            Always, here.
        """
        raise NotImplementedError

    def begin(self) -> State:
        """Make the state the steps share. A step job overrides this.

        Raises
        ------
        NotImplementedError
            Always, here.
        """
        raise NotImplementedError

    def step(self, state: State, i: int) -> object:
        """Do step `i` and return new output values. A step job sets this.

        Raises
        ------
        NotImplementedError
            Always, here.
        """
        raise NotImplementedError

    def work(self, report: Report) -> object:
        """Do the whole job, calling `report`. A work job overrides this.

        Raises
        ------
        NotImplementedError
            Always, here.
        """
        raise NotImplementedError

    @classmethod
    @override
    def _hints(cls) -> dict[str, object]:
        """Return the type hints of `step`, or of `work` for a work job.

        Returns
        -------
        dict[str, object]
            The hints by parameter name, and `return`.

        Raises
        ------
        TypeError
            If the class defines both forms, neither, or a step form without
            `begin` and `steps`.
        """
        stepped = cls.step is not JobNode.step
        if stepped == (cls.work is not JobNode.work):
            msg = (
                f"{cls.__name__}: define `step` (with `begin` and `steps`) "
                "or `work`, one of them"
            )
            raise TypeError(msg)
        if stepped and (
            cls.begin is JobNode.begin or cls.steps is JobNode.steps
        ):
            msg = f"{cls.__name__}: a step job needs `begin` and `steps` too"
            raise TypeError(msg)
        cls.is_stepped = stepped
        return dict(get_type_hints(cls.step if stepped else cls.work))

    @classmethod
    @override
    def call(cls, inputs: Mapping[str, Carried]) -> object:
        """Refuse: a job is started, not called.

        Raises
        ------
        TypeError
            Always.
        """
        msg = f"{cls.__name__} is a job: start it, it is not called"
        raise TypeError(msg)
