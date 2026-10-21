# element-51

element-51 lets a Python author design a node-editor GUI. You describe the
nodes with ordinary typed functions or classes. The library serves an editor in
the browser, keeps the authoritative graph in your Python process, and runs it
when asked. It is for people who want a graphical way to wire up computations,
such as a pipeline builder, a small visual tool for a team, or a control panel
that a script can also drive.

## Quick start

```console
$ uv add element-51
```

```python
import element_51


@element_51.node
def number() -> float:
    return 21.0


class Double(element_51.Node):
    value: float = 0.0

    def run(self) -> float:
        return self.value * 2


app = element_51.App([number, Double])
first = app.add_node(number, x=60, y=80)
second = app.add_node(Double, x=320, y=80)
app.add_link(first, "out", second, "value")
element_51.serve(app, open_browser=True)
```

Your browser opens on the editor. Press Run: `number` shows 21.0 and `Double`
shows 42.0. The same graph is available in Python: `app.run()` returns the
value of every output, and `app.save("graph.json")` and `app.load("graph.json")`
keep it between sessions.

## What to read next

- [Themes](themes.md): choose a built-in theme, or change colours, sizes, fonts
    and port shapes with a JSON file. It lists every key.
- [Pictures and plots](visual-ports.md): pass a picture or a list of numbers
    between nodes, show it on a card, draw lines on a Plot card, save a PNG.
- [Jobs](jobs.md): a node you start and stop by hand that works in the background and
    shows what it has so far, such as a training run.
- [Embedding](embedding.md): put the editor inside a FastAPI or NiceGUI app,
    next to a side panel of your own.
- [API reference](reference.md): every public function and class.

The folder `examples/` in the repository holds runnable examples, grouped by
topic. Each one starts with notes on what to try.

```{toctree}
:maxdepth: 2
:caption: Contents

themes
visual-ports
jobs
embedding
reference
```
