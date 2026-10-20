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

## Limits

- The editor and the host share one process, and `App` holds one graph. Run
    the server with a single worker: with several workers, each would hold its
    own copy of the graph.
- `App` takes a lock around every change, so scripts in other threads may edit
    while the server runs.
- The editor finds its websocket beside its own page, so it works at the root of
    a host and under any prefix such as `/editor`.
