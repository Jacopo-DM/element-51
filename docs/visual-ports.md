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

## Default nodes

Every editor starts with a few built-in nodes, in the `Default nodes` folder of the
right-click menu. At present that is `Plot`, in `Output nodes`. The nodes you list
go under `User defined nodes`, with their own category inside it. You do not list
the defaults, and listing `PlotNode` or a node named `Plot` raises a `ValueError`
that says what to change.

Pass `defaults=False` to leave them out: the editor then offers only your nodes, in
the folders they declare, with no `Default nodes` and no `User defined nodes`.

```python
app = element_51.App([ramp, training])                  # Plot is there
plain = element_51.App([ramp, training], defaults=False)  # only your nodes
```

## Plot

`Plot` draws the series linked to it as lines on its card. It is a default node, so
`element_51.App([ramp, training])` offers it. To place one from Python, use the
class: `app.add_node(element_51.PlotNode, x=380.0, y=60.0)`.

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

## Resizing a card

A card that shows a picture or a chart has a grip in its bottom right corner
(three short diagonal lines). Drag it to make the card wider and the picture or
chart taller; the boxes on the card widen with it. A card never gets smaller
than its default size. The size is part of the graph: Python sees it in
`app.graph`, `app.save` writes it, a reload and a loaded file bring it back, and
Ctrl+Z undoes a resize. A saved file with a card size is format version 3;
version 1 and 2 files still load (version 4 adds series colours; all
earlier versions still load).

## Save as CSV

The card menu of a Plot card also has Save as CSV, which downloads the numbers
of the chart as `n3-chart.csv`: an `index` column, then one column per line,
named as the chart's legend names it (`loss`, or `n3 loss` when two lines share
a name), and one row per point. A gap, a number that is not finite, and the end
of a shorter line are empty cells. Numbers are written with all their digits.
Before a run there is nothing to save, and the page says so with a message you
can dismiss.

## Line colours

Beside each `series_N` name of a Plot card is a small colour box in the colour of
that line. Click it and pick a colour: the line and its legend entry change. A
small `x` then appears beside the box, and pressing it gives the line the
theme's colour again. A colour that is hard to see on the chart (below 3:1
against its background) is drawn as you chose it, and the box gets a red outline
with the ratio as its hover text. The colour belongs to the series input: Python
holds it (`app.graph`, `SetColour`), Ctrl+Z undoes it, and a saved file keeps it
(format version 4). `examples/d_editor/d7_series_colours.py` sets two colours
from the script.

## Save as PNG

The card menu (the three bars on a card, or Shift+F10 with the node focused)
has Save as PNG on a card that shows a picture or a chart. The page asks Python
for the file, so the editor must be connected, and the browser downloads
`n1-out.png` for a picture and `n5-chart.png` for a chart. The first chart takes
a second longer, while matplotlib loads.

A chart is drawn by matplotlib: 2100 by 1400 pixels (7 by 4 2/3 inches at 300
dpi) on a white page, a light grid, the lines in the colours of the card (a
colour you picked is used exactly; a theme colour that is too pale for white is
darkened, keeping its hue, to reach 3 to 1), a legend when there is more than
one line, and the line's name as the y label when there is only one. A picture
is enlarged by a whole number with crisp pixels (a 28 by 28 digit becomes 1400
by 1400). The file says it is 300 dpi, so Word and LaTeX place it at its real
size. Before a run there is nothing to save, and a chart with no numbers cannot
be drawn; the page says so with a message you can dismiss.

## Save as CSV

The card menu of a Plot card also has Save as CSV, which downloads the numbers
of the chart as `n3-chart.csv`: an `index` column, then one column per line,
named as the chart's legend names it (`loss`, or `n3 loss` when two lines share
a name), and one row per point. A gap, a number that is not finite, and the end
of a shorter line are empty cells. Numbers are written with all their digits.
Before a run there is nothing to save, and the page says so with a message you
can dismiss.

## Line colours

Beside each `series_N` name of a Plot card is a small colour box in the colour of
that line. Click it and pick a colour: the line and its legend entry change. A
small `x` then appears beside the box, and pressing it gives the line the
theme's colour again. A colour that is hard to see on the chart (below 3:1
against its background) is drawn as you chose it, and the box gets a red outline
with the ratio as its hover text. The colour belongs to the series input: Python
holds it (`app.graph`, `SetColour`), Ctrl+Z undoes it, and a saved file keeps it
(format version 4). `examples/d_editor/d7_series_colours.py` sets two colours
from the script.

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
