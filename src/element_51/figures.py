"""PNG files of a chart or a picture, for Save as PNG.

A chart is drawn by matplotlib on a white page, a picture is enlarged by a whole
number with Pillow. Both come out at 300 dpi. matplotlib and numpy are loaded
the first time a chart is drawn, so `import element_51` stays quick.
"""

import colorsys
import io
import math
from typing import Annotated

from pydantic import Field

from element_51.model import Model
from element_51.visuals import Image

DPI = 300
PAPER_WIDTH = 2100
PAPER_HEIGHT = 1400
GRAPHIC_RATIO = 3.0
LINE_WIDTH = 2.0
MARGIN = 0.02
PADDING = 1.2
GRID_COLOUR = "#dddddd"
FONT_SIZE = 9.0
WHITE = (1.0, 1.0, 1.0)
HEX = r"#[0-9a-f]{6}"
CHANNEL_MAX = 255
DARKENING_STEPS = 24
LINEAR_KNEE = 0.03928
LINEAR_SLOPE = 12.92
LINEAR_OFFSET = 0.055
LINEAR_SCALE = 1.055
LINEAR_POWER = 2.4
RED_WEIGHT = 0.2126
GREEN_WEIGHT = 0.7152
BLUE_WEIGHT = 0.0722
GLARE = 0.05

type Rgb = tuple[float, float, float]


class Line(Model):
    """One line of a chart.

    `values` holds the numbers by index, `None` where there is a gap. `colour`
    is `#rrggbb`. `chosen` says the user picked it, so it is drawn as it is;
    otherwise it is the theme's colour, which is darkened to be read on white.
    """

    name: str
    values: tuple[float | None, ...]
    colour: Annotated[str, Field(pattern=f"^{HEX}$")]
    chosen: bool


def _channels(colour: str) -> Rgb:
    """Read `#rrggbb` as three numbers from 0 to 1.

    Returns
    -------
    Rgb
        Red, green and blue.
    """
    return (
        int(colour[1:3], 16) / CHANNEL_MAX,
        int(colour[3:5], 16) / CHANNEL_MAX,
        int(colour[5:7], 16) / CHANNEL_MAX,
    )


def _hex(rgb: Rgb) -> str:
    """Write three numbers from 0 to 1 as `#rrggbb`.

    Returns
    -------
    str
        The colour in lower case.
    """
    red, green, blue = (round(part * CHANNEL_MAX) for part in rgb)
    return f"#{red:02x}{green:02x}{blue:02x}"


def _linear(part: float) -> float:
    """Undo the gamma of one colour channel (WCAG).

    Returns
    -------
    float
        The channel in linear light.
    """
    if part <= LINEAR_KNEE:
        return part / LINEAR_SLOPE
    return math.pow((part + LINEAR_OFFSET) / LINEAR_SCALE, LINEAR_POWER)


def contrast_with_white(colour: str) -> float:
    """Find the WCAG contrast ratio of `colour` against white.

    Parameters
    ----------
    colour
        A colour written `#rrggbb`.

    Returns
    -------
    float
        From 1 (white) to 21 (black).
    """
    red, green, blue = (_linear(part) for part in _channels(colour))
    light = RED_WEIGHT * red + GREEN_WEIGHT * green + BLUE_WEIGHT * blue
    return (1.0 + GLARE) / (light + GLARE)


def darken_to(colour: str, ratio: float = GRAPHIC_RATIO) -> str:
    """Darken `colour`, keeping its hue, until white shows it at `ratio`.

    Parameters
    ----------
    colour
        A colour written `#rrggbb`.
    ratio
        The contrast with white to reach.

    Returns
    -------
    str
        `colour` itself if it already has the contrast, else the lightest
        colour of the same hue and saturation that does.
    """
    if contrast_with_white(colour) >= ratio:
        return colour
    hue, light, saturation = colorsys.rgb_to_hls(*_channels(colour))
    enough, too_pale = 0.0, light
    for _ in range(DARKENING_STEPS):
        middle = (enough + too_pale) / 2.0
        darker = _hex(colorsys.hls_to_rgb(hue, middle, saturation))
        if contrast_with_white(darker) >= ratio:
            enough = middle
        else:
            too_pale = middle
    return _hex(colorsys.hls_to_rgb(hue, enough, saturation))


def chart_png(lines: tuple[Line, ...]) -> bytes:
    """Draw `lines` as one chart and return it as a PNG file.

    The file is 2100 by 1400 pixels at 300 dpi: a white page, a light grid, the
    x label `index`, the y label the line's name when there is only one line,
    and a legend when there are several.

    Parameters
    ----------
    lines
        The lines, in order.

    Returns
    -------
    bytes
        The PNG file.

    Raises
    ------
    ValueError
        If no line has a finite number to draw.
    """
    import numpy as np  # ruff: ignore[import-outside-top-level]  # loaded on first use, see the module docstring
    from matplotlib import rc_context  # ruff: ignore[import-outside-top-level]  # loaded on first use
    from matplotlib.figure import Figure  # ruff: ignore[import-outside-top-level]  # loaded on first use

    arrays = [
        np.array(
            [np.nan if value is None else value for value in line.values],
            dtype=float,
        )
        for line in lines
    ]
    if not any(np.isfinite(array).any() for array in arrays):
        msg = "the chart has no numbers to draw"
        raise ValueError(msg)
    with rc_context({"font.size": FONT_SIZE}):
        figure = Figure(
            figsize=(PAPER_WIDTH / DPI, PAPER_HEIGHT / DPI),
            dpi=DPI,
            facecolor="white",
        )
        axes = figure.subplots()
        for line, array in zip(lines, arrays, strict=True):
            colour = line.colour if line.chosen else darken_to(line.colour)
            axes.plot(
                array, color=colour, linewidth=LINE_WIDTH, label=line.name
            )
        axes.grid(visible=True, color=GRID_COLOUR, linewidth=0.6)
        axes.spines[["top", "right"]].set_visible(False)
        axes.margins(x=MARGIN)
        axes.set_axisbelow(True)
        axes.set_xlabel("index")
        if len(lines) == 1:
            axes.set_ylabel(lines[0].name)
        else:
            axes.legend(loc="best")
        figure.tight_layout(pad=PADDING)
        buffer = io.BytesIO()
        figure.savefig(buffer, format="png", dpi=DPI)
    return buffer.getvalue()


def picture_png(image: Image) -> bytes:
    """Enlarge `image` by a whole number to fill the page and return a PNG.

    The pixels stay square and sharp. A picture already larger than the page
    keeps its size.

    Parameters
    ----------
    image
        The picture.

    Returns
    -------
    bytes
        The PNG file, at 300 dpi.
    """
    from PIL import Image as PilImage  # ruff: ignore[import-outside-top-level]  # loaded on first use

    mode = "L" if image.channels == 1 else "RGB"
    picture = PilImage.frombytes(
        mode, (image.width, image.height), image.pixels
    )
    factor = max(
        1, min(PAPER_WIDTH // image.width, PAPER_HEIGHT // image.height)
    )
    enlarged = picture.resize(
        (image.width * factor, image.height * factor),
        PilImage.Resampling.NEAREST,
    )
    buffer = io.BytesIO()
    enlarged.save(buffer, format="PNG", dpi=(DPI, DPI))
    return buffer.getvalue()
