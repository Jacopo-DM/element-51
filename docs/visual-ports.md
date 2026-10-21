# Pictures and plots

A node can pass a picture or a list of numbers to another node, and a card can
show them. Two port types do this: `Image` and `Series`. Both ports link only
to ports of their own kind, so a picture cannot be wired into a number input,
and the editor refuses the link and says why.

## A picture

An `Image` is a grey or colour picture. `channels` is 1 for grey and 3 for red,
green and blue, and `pixels` holds `width * height * channels` bytes, row by
row from the top. The largest picture is 256 by 256.

```python
import element_51


@element_51.node
def ramp(side: int = 28) -> element_51.Image:
    row = bytes(column * 255 // max(side - 1, 1) for column in range(side))
    return element_51.Image(
        width=side, height=side, channels=1, pixels=row * side
    )
```

After a run the card of `ramp` shows the picture below its ports, with crisp
pixels, and its output box says `28 x 28 grey`. A node that takes a picture
(`def invert(source: element_51.Image) -> element_51.Image`) has no value box for
that input, because a picture cannot be typed.

## A list of numbers

A `Series` is a list of numbers, such as a loss for each training step. A
`NaN` in it is a gap, not an error: the line stops there and starts again at the
next number. A series holds at most 100,000 numbers. A node with several series
outputs returns a pydantic model, as for any node with several outputs:

```python
class Curves(element_51.Model):
    loss: element_51.Series
    accuracy: element_51.Series


@element_51.node
def training(steps: int = 60) -> Curves: ...
```

The output box of a series shows the last number it holds, such as `0.0842`;
hover over the box to read what it is (`last of 282 points`). A linked input
shows the same number, and the name of the output it is linked to after it,
such as `0.0842 [n4.loss]`.

## Plot

`element_51.PlotNode` draws the series linked to it as lines on its card. Register
the class itself:

```python
app = element_51.App([ramp, training, element_51.PlotNode])
```

The card starts with one input, `series_1`. Every series you link adds a row,
and a new empty row waits below for the next one, so a plot of three lines has
`series_1` to `series_3` and an empty `series_4`. Removing a link removes its
row and the card shrinks. A row keeps its name, so `series_1` and `series_3` can
stay when `series_2` goes.

Each line has the colour of its port type and is named after the output port it
comes from, with its last value at its end. Two lines with the same name get
their node id in front (`n4 loss`), and from the sixth line on, the lines are
dashed, so colour is never the only way to tell them apart. A long series is
thinned to about twice the width of the chart in pixels, keeping the lowest and
highest number of each part, so a spike is never lost.

## Save as PNG

The card menu (the three bars on a card, or Shift+F10 with the node focused)
has Save as PNG on a card that shows a picture or a chart. The browser downloads
`n1-out.png` for a picture, and `n5-chart.png` for a chart. Both are made to be
put in a paper, whatever theme the editor has: a chart is 2100 by 1400 pixels
(7 by 4 2/3 inches at 300 dpi) on a white page, with a thin black frame, serif
numbers and labels, a faint grid, blue and red lines with different markers so
they differ in greyscale, and a boxed legend when there is more than one line. A
picture is enlarged by a whole number, with crisp pixels and a thin frame (a 28
by 28 digit becomes 1400 by 1400). The file says it is 300 dpi, so Word and
LaTeX place it at its real size. Before a run there is nothing to save, and the page
says so with a message you can dismiss.

## Keyboard and screen readers

Pictures and plots are on a canvas, so the editor says what they show in the
hidden list of nodes that a screen reader reads: `n1, ramp. 1 input, 1 output.
Shows a 28 by 28 picture.` and `n5, Plot. 3 inputs, 0 outputs. Shows a plot of 2
lines, loss last 0.0523 and accuracy last 0.906.` The rows of a Plot are listed
as ports, with the empty one last. The picture and chart states, the card menu
and the source panel were scanned with axe-core in all three built-in themes,
with no violation.

## Try it

`examples/g_experiments/g1_picture_and_plot.py` has a picture, its negative, a
pixel count and a plot, and its notes say what to try:

```console
$ uv run python examples/g_experiments/g1_picture_and_plot.py
```
