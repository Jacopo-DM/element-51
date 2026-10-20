"""Use the graph from plain Python, with no browser.

Run it: `uv run python examples/c_python_api/c1_script_api.py`. It builds a
graph with `add_node` and `add_link`, runs it with `app.run()`, and shows what
happens when a node fails. To look at the same graph in the editor, call
`element_51.serve(app)` instead.

"""

# Third-party libraries
from rich.console import Console
from rich.traceback import install

# Local libraries
import element_51

# --- Globals --- #

# --- Warnings --- #

# --- Typing --- #

# --- TERMINAL OUTPUT SETUP --- #
install(show_locals=False)
console = Console()


@element_51.node
def price() -> float:
    return 19.99


@element_51.node
def quantity() -> int:
    return 3


@element_51.node
def total(price: float, quantity: float) -> float:
    return round(price * quantity, 2)


@element_51.node
def split(amount: float, people: float) -> float:
    return round(amount / people, 2)


def build_app() -> element_51.App:
    """Build `total = price * quantity`, then `split(total, people)`.

    Returns
    -------
    element_51.App
        The app. `people` has no node, so `split` has no value for it.
    """
    app = element_51.App([price, quantity, total, split])
    n_price = app.add_node(price, x=60.0, y=60.0)
    n_quantity = app.add_node(quantity, x=60.0, y=200.0)
    n_total = app.add_node(total, x=320.0, y=120.0)
    n_split = app.add_node(split, x=580.0, y=120.0)
    app.add_link(n_price, "out", n_total, "price")
    app.add_link(n_quantity, "out", n_total, "quantity")
    app.add_link(n_total, "out", n_split, "amount")
    return app


def main() -> None:
    """Run the graph and print what comes back."""
    app = build_app()
    result = app.execute()
    for outcome in result.outcomes:
        detail = outcome.error if outcome.status == "error" else outcome.outputs
        console.print(f"{outcome.node}: {outcome.status} {detail}")
    try:
        app.run()
    except ValueError as error:
        console.print(f"app.run() raised: {error} ({error.__notes__[0]})")


if __name__ == "__main__":  # pragma: no cover
    main()
