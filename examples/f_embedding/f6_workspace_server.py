"""A workspace server: many graphs, each with its own editor, API and file.

FastAPI hosts everything and NiceGUI draws the dashboard. Each workspace is
its own `App`, with its own graph, mounted at `/editor/<name>/` while the
server runs. Creating a workspace mounts a new editor, deleting one removes it
again, and every workspace saves to `<folder>/<name>.json` and comes back
when the server starts.

The dashboard at `/` lists the workspaces, and a JSON API at
`/api/workspaces` does the same jobs for scripts. Both doors call one class,
`Workspaces`, which owns the apps, the files and the mounted editors, takes a
lock around every change, and refuses a bad name before it can reach the file
system (a name becomes part of a path, so only `a-z`, `0-9` and `-` pass).

Run it with one worker only: the apps live in this process.

Build the browser layer first: `uv run python tools/build_wasm.py`.

Notes
-----
    * Run the script and open http://127.0.0.1:8765 if it does not open for you.
        One workspace, `demo`, is there with an `add` node.
    * Type `sums` in the box and press `Create`: a second workspace appears,
        empty. Press `Open editor` on it: a new tab shows its own editor.
    * Place a node in `sums`, press `Save`, then stop the script and start it
        again: both workspaces come back, and `sums` has your node. The files
        are in `__data__/workspaces/`, which git ignores.
    * Type `Bad Name` and press `Create`: a message says why it was refused
        and stays until you press `Dismiss`.
    * Press `Run` on `demo`: its outputs appear in the row.
    * Press `Delete` on `sums`: its editor is gone (`/editor/sums/` is a 404)
        and so is its file.
    * With a terminal open: `curl http://127.0.0.1:8765/api/workspaces`
        lists them as JSON, and `curl -X POST
        http://127.0.0.1:8765/api/workspaces/demo/run` runs one.

"""

# Standard library
import re
import threading
from collections.abc import Callable
from pathlib import Path
from typing import Literal

# Third-party libraries
import uvicorn
from fastapi import FastAPI, HTTPException, status
from nicegui import ui
from rich.console import Console
from rich.traceback import install
from starlette.routing import Mount

# Local libraries
import element_51
from element_51.engine import RunResult

# --- Globals --- #
PORT = 8765
FOLDER = Path("__data__/workspaces")
NAME = re.compile(r"[a-z][a-z0-9-]{0,31}")
DEMO = "demo"
FIRST_X = 60.0
FIRST_Y = 80.0
POLL_SECONDS = 1.0
STATUS_OF = {
    "invalid": status.HTTP_422_UNPROCESSABLE_CONTENT,
    "exists": status.HTTP_409_CONFLICT,
    "missing": status.HTTP_404_NOT_FOUND,
}

# --- Warnings --- #

# --- Typing --- #
type Kind = Literal["invalid", "exists", "missing"]

# --- TERMINAL OUTPUT SETUP --- #
install(show_locals=False)
console = Console()


class NewWorkspace(element_51.Model):
    """The body of `POST /api/workspaces`: the name to create."""

    name: str


class WorkspaceInfo(element_51.Model):
    """One workspace, as both doors show it."""

    name: str
    nodes: int
    links: int


class WorkspaceError(Exception):
    """A request about a workspace was refused.

    Parameters
    ----------
    reason
        Why, worded for a person.
    kind
        `invalid` for a bad name, `exists` for a name taken, `missing` for a
        name nobody made.
    """

    def __init__(self, reason: str, kind: Kind) -> None:
        super().__init__(reason)
        self.kind = kind
        self.reason = reason


@element_51.node
def add(a: float = 1.0, b: float = 2.0) -> float:
    return a + b


@element_51.node
def multiply(a: float = 1.0, b: float = 2.0) -> float:
    return a * b


class Workspaces:
    """The workspaces of a server: their apps, their files and their editors.

    A class because it is behaviour first: it holds no record, only the
    state the two doors share. Every method takes one lock, so the
    dashboard's callbacks and the API's threads cannot mix up a change.

    Parameters
    ----------
    host
        The FastAPI app each workspace's editor is mounted on.
    folder
        Where the files are kept. Created when the first one is saved.
    """

    def __init__(self, host: FastAPI, folder: Path) -> None:
        self._host = host
        self._folder = folder
        self._apps: dict[str, element_51.App] = {}
        self._lock = threading.RLock()

    def restore(self) -> None:
        """Make a workspace of every file in the folder."""
        if self._folder.is_dir():
            for path in sorted(self._folder.glob("*.json")):
                if NAME.fullmatch(path.stem):
                    self.create(path.stem)

    def names(self) -> list[str]:
        """List the workspaces.

        Returns
        -------
        list[str]
            Their names, in the order they were made.
        """
        with self._lock:
            return list(self._apps)

    def get(self, name: str) -> element_51.App:
        """Find a workspace.

        Parameters
        ----------
        name
            The workspace's name.

        Returns
        -------
        element_51.App
            Its session.

        Raises
        ------
        WorkspaceError
            If there is no workspace of that name.
        """
        with self._lock:
            app = self._apps.get(name)
        if app is None:
            msg = f"there is no workspace {name!r}"
            raise WorkspaceError(msg, "missing")
        return app

    def create(
        self, name: str, app: element_51.App | None = None
    ) -> element_51.App:
        """Make a workspace and mount its editor.

        If a file of that name is in the folder, the graph is loaded from it.

        Parameters
        ----------
        name
            Lower case letters, digits and `-`, starting with a letter, at
            most 32 characters.
        app
            The session to use, or `None` for an empty one with `add` and
            `multiply` registered.

        Returns
        -------
        element_51.App
            The new workspace's session.

        Raises
        ------
        WorkspaceError
            If the name is not allowed or is taken.
        """
        if not NAME.fullmatch(name):
            msg = (
                f"{name!r} is not a name: use a to z, 0 to 9 and -, starting "
                "with a letter, at most 32 characters"
            )
            raise WorkspaceError(msg, "invalid")
        with self._lock:
            if name in self._apps:
                msg = f"{name!r} already exists"
                raise WorkspaceError(msg, "exists")
            made = app or element_51.App([add, multiply])
            path = self._folder / f"{name}.json"
            if path.is_file():
                made.load(path)
            self._apps[name] = made
            # NiceGUI is mounted at "/", which matches every path, so an
            # editor mounted later would never be reached: go in first.
            self._host.router.routes.insert(
                0, Mount(f"/editor/{name}", app=element_51.asgi_app(made))
            )
        return made

    def save(self, name: str) -> None:
        """Write a workspace's graph to its file.

        Parameters
        ----------
        name
            The workspace's name.
        """
        app = self.get(name)
        self._folder.mkdir(parents=True, exist_ok=True)
        app.save(self._folder / f"{name}.json")

    def delete(self, name: str) -> None:
        """Remove a workspace: its editor, its session and its file.

        Parameters
        ----------
        name
            The workspace's name.
        """
        with self._lock:
            self.get(name)
            del self._apps[name]
            path = f"/editor/{name}"
            routes = self._host.router.routes
            routes[:] = [
                route
                for route in routes
                if not (isinstance(route, Mount) and route.path == path)
            ]
            (self._folder / f"{name}.json").unlink(missing_ok=True)

    def describe(self, name: str) -> WorkspaceInfo:
        """Count a workspace's nodes and links.

        Parameters
        ----------
        name
            The workspace's name.

        Returns
        -------
        WorkspaceInfo
            The name and the counts.
        """
        graph = self.get(name).graph
        return WorkspaceInfo(
            name=name, nodes=len(graph.nodes), links=len(graph.links)
        )

    def run(self, name: str) -> RunResult:
        """Run a workspace's graph.

        Parameters
        ----------
        name
            The workspace's name.

        Returns
        -------
        RunResult
            What happened to each node.
        """
        return self.get(name).execute()


def build_app() -> element_51.App:
    """Build the `demo` workspace's app: one `add` node that shows 3.0.

    Returns
    -------
    element_51.App
        An app with `add` and `multiply` registered and one `add` node, `n1`.
    """
    app = element_51.App([add, multiply])
    app.add_node(add, x=FIRST_X, y=FIRST_Y)
    return app


def summary(result: RunResult) -> str:
    """Say in one line what a run did.

    Parameters
    ----------
    result
        The outcome of a run.

    Returns
    -------
    str
        For example `n1: ok {'out': 3.0}; n2: division by zero`.
    """
    parts: list[str] = []
    for outcome in result.outcomes:
        what = outcome.error or f"ok {dict(outcome.outputs)}"
        parts.append(f"{outcome.node}: {what}")
    return "; ".join(parts)


def guarded[T](work: Callable[[], T]) -> T:
    """Call `work`, turning a refused request into an HTTP error.

    Parameters
    ----------
    work
        A call on the workspaces.

    Returns
    -------
    T
        What `work` returned.

    Raises
    ------
    HTTPException
        If `work` raised `WorkspaceError`: 404, 409 or 422 with the reason.
    """
    try:
        return work()
    except WorkspaceError as error:
        raise HTTPException(
            status_code=STATUS_OF[error.kind], detail=error.reason
        ) from error


def add_api(host: FastAPI, workspaces: Workspaces) -> None:
    """Add the JSON API to the host.

    Parameters
    ----------
    host
        The FastAPI app to add the routes to.
    workspaces
        The workspaces the routes work on.
    """

    @host.get("/api/workspaces")
    def listing() -> list[WorkspaceInfo]:
        return [workspaces.describe(name) for name in workspaces.names()]

    @host.post("/api/workspaces", status_code=status.HTTP_201_CREATED)
    def create(body: NewWorkspace) -> WorkspaceInfo:
        guarded(lambda: workspaces.create(body.name))
        return workspaces.describe(body.name)

    @host.post("/api/workspaces/{name}/run")
    def run(name: str) -> RunResult:
        return guarded(lambda: workspaces.run(name))

    @host.post(
        "/api/workspaces/{name}/save", status_code=status.HTTP_204_NO_CONTENT
    )
    def save(name: str) -> None:
        guarded(lambda: workspaces.save(name))

    @host.delete(
        "/api/workspaces/{name}", status_code=status.HTTP_204_NO_CONTENT
    )
    def delete(name: str) -> None:
        guarded(lambda: workspaces.delete(name))


def add_dashboard(workspaces: Workspaces) -> None:
    """Add the NiceGUI dashboard page at `/`.

    Parameters
    ----------
    workspaces
        The workspaces the page lists and changes.
    """

    @ui.page("/")
    def page() -> None:
        ui.add_css("html { color-scheme: light; }")
        last: dict[str, str] = {}

        def attempt(work: Callable[[], object]) -> None:
            try:
                work()
            except WorkspaceError as error:
                ui.notify(
                    error.reason,
                    type="warning",
                    close_button="Dismiss",
                    timeout=0,
                )
            else:
                rows.refresh()

        def show_run(name: str) -> None:
            last[name] = summary(workspaces.run(name))

        def remove(name: str) -> None:
            workspaces.delete(name)
            last.pop(name, None)

        @ui.refreshable
        def rows() -> None:
            for name in workspaces.names():
                info = workspaces.describe(name)
                with ui.row().classes("items-center"):
                    ui.label(name).classes("text-lg w-32")
                    ui.label(f"{info.nodes} nodes, {info.links} links")
                    ui.link("Open editor", f"/editor/{name}/", new_tab=True)
                    ui.button(
                        "Run",
                        on_click=lambda n=name: attempt(lambda: show_run(n)),
                    )
                    ui.button(
                        "Save",
                        on_click=lambda n=name: attempt(
                            lambda: workspaces.save(n)
                        ),
                    )
                    ui.button(
                        "Delete",
                        on_click=lambda n=name: attempt(lambda: remove(n)),
                    )
                    ui.label(last.get(name, ""))

        with ui.column().classes("p-4"):
            ui.label("Workspaces").classes("text-xl")
            with ui.row():
                field = ui.input(placeholder="new-name")
                ui.button(
                    "Create",
                    on_click=lambda: attempt(
                        lambda: workspaces.create(str(field.value))
                    ),
                )
            rows()
        ui.timer(POLL_SECONDS, rows.refresh)


def build_host(folder: Path = FOLDER) -> FastAPI:
    """Build the FastAPI app: the dashboard, the API and the editors.

    Parameters
    ----------
    folder
        Where the workspaces are saved and restored from.

    Returns
    -------
    FastAPI
        The host: NiceGUI at `/`, the API at `/api/workspaces` and one editor
        at `/editor/<name>/` per workspace. `demo` is made if the folder has
        no workspaces.
    """
    host = FastAPI()
    workspaces = Workspaces(host, folder)
    workspaces.restore()
    if not workspaces.names():
        workspaces.create(DEMO, build_app())
    add_api(host, workspaces)
    add_dashboard(workspaces)
    ui.run_with(host, mount_path="/")
    return host


def main() -> None:
    """Entry point."""
    uvicorn.run(build_host(), host="127.0.0.1", port=PORT)


if __name__ == "__main__":  # pragma: no cover
    main()
