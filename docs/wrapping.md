# Turning existing functions into nodes

If you already have a module of functions, you do not need to put `@node` above
each one. Two calls make nodes in bulk:

- `nodes_from_module` for your own functions that have type hints.
- `nodes_from_ufuncs` for numpy's element-wise functions (ufuncs).

Both return a `Scan`: its `nodes` go straight into `App`, and its `skipped` lists
the functions that could not become nodes, each with the reason. A function that
does not fit is never an error.

## Your own module

Say `stats.py` holds plain functions:

```python
def zscore(x: float, mean: float, std: float) -> float:
    return (x - mean) / std


def percent(part: float, whole: float) -> float:
    return 100.0 * part / whole
```

One call makes both into nodes, in the `Stats` flyout of `User defined nodes` in the add-node menu:

```python
import element_51
import stats

scan = element_51.nodes_from_module(stats, category="Stats")
app = element_51.App(scan.nodes)
element_51.serve(app)
```

The functions are not changed. `stats.zscore(82, 70, 8)` still returns 1.5, so
the same module keeps working in the rest of your code.

The ports follow the hints, exactly as with `@node`: each parameter is an input,
and the return type is the output `out` (or one output per field of a pydantic
model). The hints can be `float`, `int`, `str` or `bool`.

What the scan looks at:

- Names that start with an underscore are left out, and so are names the module
    imported from somewhere else.
- If the module has an `__all__`, only the names in it are scanned.
- A function already made a node with `@node`, and a `Node` subclass, join the
    result as they are.
- A second name for the same function (`plus = add`) makes no second node.
- Two different functions with one node name: the later one is skipped.

A function with no type hint on a parameter or on the result, with `*args` or
`**kwargs`, or with a hint that is not one of the four types is skipped. The
reason names the parameter:

```python
for item in scan.skipped:
    print(item.name, item.reason)
```

The example `examples/h_wrapping/h2_own_module.py` scans its own file and prints
the one function it has to skip.

## numpy's functions

numpy's ufuncs (`sin`, `add`, `hypot`, `isnan`, and so on) describe themselves, so
no code has to be written for them:

```python
import numpy as np

scan = element_51.nodes_from_ufuncs(np, category="Numpy")
app = element_51.App(scan.nodes)
```

On numpy 2.5 this makes 85 nodes. For each ufunc:

- The inputs come from the number of arguments: `x` for one, `x1` and `x2` for
    two.
- The outputs come from the number of results: `out`, or `out1` and `out2` for
    functions such as `modf`, which splits a number into its fractional and whole
    parts.
- The port types come from the loops numpy lists for the function. The scan takes
    the first loop that works on float64 only, then the first on int64 only, then
    the first left. So `sin` takes a float and gives a float, `isnan` takes a float
    and gives a bool, and `gcd` takes two ints.
- A number you give a port is turned into the port's type before the call, so a
    whole number on a float port cannot overflow in an integer loop. The result is
    a plain `float`, `int` or `bool`.

A ufunc with two names, such as `acos` and `arccos`, makes one node, named by the
ufunc itself (`arccos`). Five are skipped on numpy 2.5: `matmul`, `matvec`,
`vecdot` and `vecmat` work on whole arrays, and `isnat` works on dates only.

Dividing by zero gives `inf` and taking the log of a negative number gives `nan`,
as in numpy, without a warning. The page shows both as `null`.

Four nodes have a limit you may notice: `logical_and`, `logical_or`,
`logical_xor` and `logical_not` take floats, and an output of type bool cannot be
linked to a float input, so `isnan` cannot feed `logical_not`.

The example `examples/h_wrapping/h1_numpy_ufuncs.py` serves all 85 and a small
graph of three.

## What this does not do yet

- Arrays. Every port carries one number or one text, so these nodes work on single
    values. A port for arrays of any shape is the next goal.
- numpy functions that are not ufuncs, such as `np.mean` or `np.clip`. They have no
    type hints that Python can read at run time, so a scan cannot tell what they
    take. Wrap them in a typed function of your own and scan that.
- Classes and methods.
