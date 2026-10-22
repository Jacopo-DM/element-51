# Embedding the editor

`element_51.serve(app)` runs the editor on its own. To put it inside a larger
web app, use `element_51.asgi_app(app)` instead: it returns an ASGI app that any
ASGI host can mount.

## FastAPI

```python
import uvicorn
from fastapi import FastAPI

import element_51

app = element_51.App([...])  # your node functions and classes

host = FastAPI()
host.mount("/editor", element_51.asgi_app(app))


@host.post("/api/run")
def run_graph() -> dict[str, dict[str, object]]:
    return app.run()


uvicorn.run(host, host="127.0.0.1", port=8765)
```

The editor is at `http://127.0.0.1:8765/editor/`, and your own routes work on the
same `app`. A node added from Python, for example `app.add_node(...)` inside an
endpoint, appears in a page that is already open, with no reload.

`examples/f_embedding/f1_fastapi_host.py` is a complete version with a side
panel that runs the graph and adds nodes.

## NiceGUI

NiceGUI runs on FastAPI, so the same host serves both. Mount the editor, call
`ui.run_with(host)`, and put the editor in an `iframe` on a NiceGUI page:

```python
ui.element("iframe").props("src=/editor/").style("width: 100%; height: 100%")
```

`examples/f_embedding/f2_nicegui_host.py` shows a splitter page with NiceGUI
buttons that run the graph and add nodes.

## More examples, simplest first

All six are in `examples/f_embedding/`. Run each on its own: they share port
8765.

| Example | Host | What it adds |
| --- | --- | --- |
| `f1_fastapi_host.py` | FastAPI | A side panel with Run and Add node. |
| `f2_nicegui_host.py` | NiceGUI on FastAPI | The same panel, written in Python widgets. |
| `f3_fastapi_api.py` | FastAPI | A typed JSON API: pydantic bodies, refusals as 404 and 409, FastAPI's `/docs` page. |
| `f4_nicegui_job.py` | NiceGUI on FastAPI | Start and Stop for a job node, a progress bar and a message that stays until you press Dismiss. |
| `f5_both_front_doors.py` | both | A NiceGUI page and a JSON API that call the same plain functions on one graph. |
| `f6_workspace_server.py` | both | Many graphs: each workspace has its own editor mounted at run time, its own file, and one dashboard and one API for all of them. |

Three things to know when you build on these:

- NiceGUI keeps one global app, so a process can call `ui.run_with` once. The
    tests for `f4` to `f6` start each host in a child process.
- NiceGUI is mounted at `/`, which matches every path. An editor mounted after
    it, as `f6` does while the server runs, is never reached; `f6` puts each new
    mount first in the router's list instead of calling `host.mount`.
- `App.apply` and the browser's edits do not call `App.subscribe` listeners,
    only `add_node` and `add_link` do. A panel that must follow the page's own
    edits polls `app.graph`, as `f4` and `f5` do with a NiceGUI timer.

## Limits

- The editor and the host share one process, and `App` holds one graph. Run
    the server with a single worker: with several workers, each would hold its
    own copy of the graph.
- `App` takes a lock around every change, so scripts in other threads may edit
    while the server runs.
- The editor finds its websocket beside its own page, so it works at the root of
    a host and under any prefix such as `/editor`.
