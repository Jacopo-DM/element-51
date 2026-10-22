"""What the two wrapping examples scan, beyond the graph they build."""

from test_examples import load


def test_the_numpy_example_serves_every_ufunc_that_fits() -> None:
    """The app knows the five-node graph and also the rest of numpy's nodes."""
    app = load("h_wrapping/h1_numpy_ufuncs.py").build_app()
    assert {"hypot", "modf", "exp2", "sin", "isnan", "gcd"} <= set(
        app.node_functions
    )


def test_the_own_module_example_skips_the_function_without_ports() -> None:
    """`describe` takes `*values`; the scan says so and leaves it out."""
    scan = load("h_wrapping/h2_own_module.py").scan()
    assert [node.node_type.name for node in scan.nodes] == [
        "mean_of_two",
        "zscore",
        "percent",
    ]
    assert [(s.name, "*args" in s.reason) for s in scan.skipped] == [
        ("describe", True)
    ]
