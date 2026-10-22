"""PNG files of a chart and of a picture."""

import colorsys
import io
import subprocess  # ruff: ignore[suspicious-subprocess-import]  # runs this Python with fixed arguments
import sys

import pytest
from hypothesis import given
from hypothesis import strategies as st
from PIL import Image as PilImage
from pydantic import ValidationError

from element_51 import Image
from element_51.figures import (
    DPI,
    GRAPHIC_RATIO,
    PAPER_HEIGHT,
    PAPER_WIDTH,
    Line,
    chart_png,
    contrast_with_white,
    darken_to,
    picture_png,
)

PALE = "#edc27b"
DARK = "#1a1a80"
SIDE = 28
MAGNIFICATION = 50
HUE_TOLERANCE = 0.05
BIG = 256
MIDDLE_LOW = 0.1
MIDDLE_HIGH = 0.9
PNG_DPI_ROUNDING = 0.01
rgb_values = st.integers(min_value=0, max_value=255)


def curve(count: int = 20) -> tuple[float | None, ...]:
    """Make a falling curve.

    Returns
    -------
    tuple[float | None, ...]
        `count` numbers.
    """
    return tuple(1.0 / (step + 1) for step in range(count))


def decoded(data: bytes) -> PilImage.Image:
    """Read PNG bytes back.

    Returns
    -------
    PilImage.Image
        The picture, loaded.
    """
    picture = PilImage.open(io.BytesIO(data))
    picture.load()
    return picture


def colour_count(picture: PilImage.Image, colour: str) -> int:
    """Count the pixels of exactly `colour` (`#rrggbb`).

    Returns
    -------
    int
        How many pixels have it.
    """
    wanted = bytes(int(colour[i : i + 2], 16) for i in (1, 3, 5))
    return picture.convert("RGB").tobytes().count(wanted)


def test_a_chart_is_a_white_page_of_the_paper_size_at_300_dpi() -> None:
    data = chart_png((
        Line(name="loss", values=curve(), colour=DARK, chosen=True),
    ))
    picture = decoded(data)
    assert picture.size == (PAPER_WIDTH, PAPER_HEIGHT)
    assert picture.convert("RGB").getpixel((0, 0)) == (255, 255, 255)
    dpi = picture.info["dpi"]
    assert abs(dpi[0] - DPI) < PNG_DPI_ROUNDING
    assert abs(dpi[1] - DPI) < PNG_DPI_ROUNDING


def test_a_chosen_colour_is_drawn_exactly_and_a_theme_colour_is_darkened() -> (
    None
):
    chosen = decoded(
        chart_png((Line(name="a", values=curve(), colour=PALE, chosen=True),))
    )
    themed = decoded(
        chart_png((Line(name="a", values=curve(), colour=PALE, chosen=False),))
    )
    assert colour_count(chosen, PALE) > 0
    assert colour_count(themed, PALE) == 0
    assert colour_count(themed, darken_to(PALE)) > 0


def test_a_gap_and_several_lines_still_draw() -> None:
    gappy = (1.0, None, 0.5, float("nan"), 0.25)
    data = chart_png((
        Line(name="a", values=gappy, colour=DARK, chosen=True),
        Line(name="b", values=curve(5), colour=PALE, chosen=False),
    ))
    assert decoded(data).size == (PAPER_WIDTH, PAPER_HEIGHT)


def test_a_chart_with_no_number_is_refused() -> None:
    nothing = Line(
        name="a", values=(None, float("nan")), colour=DARK, chosen=True
    )
    with pytest.raises(ValueError, match="no numbers"):
        chart_png((nothing,))
    with pytest.raises(ValueError, match="no numbers"):
        chart_png(())


def test_a_line_colour_must_be_rrggbb_in_lower_case() -> None:
    for bad in ("red", "#fff", "#FF8800", "#ff88zz", ""):
        with pytest.raises(ValidationError):
            Line(name="a", values=(1.0,), colour=bad, chosen=True)


def test_a_grey_picture_is_enlarged_by_a_whole_number_with_sharp_pixels() -> (
    None
):
    pixels = bytes(i % 256 for i in range(SIDE * SIDE))
    picture = decoded(
        picture_png(Image(width=SIDE, height=SIDE, channels=1, pixels=pixels))
    )
    assert picture.size == (SIDE * MAGNIFICATION, SIDE * MAGNIFICATION)
    assert picture.mode == "L"
    block = {
        picture.getpixel((x, y))
        for x in range(MAGNIFICATION)
        for y in range(MAGNIFICATION)
    }
    assert block == {pixels[0]}
    assert picture.getpixel((MAGNIFICATION, 0)) == pixels[1]
    assert abs(picture.info["dpi"][0] - DPI) < PNG_DPI_ROUNDING


def test_a_colour_picture_keeps_its_colours_and_a_big_one_keeps_its_size() -> (
    None
):
    side = BIG
    pixels = bytes([200, 10, 20] * (side * side))
    picture = decoded(
        picture_png(Image(width=side, height=side, channels=3, pixels=pixels))
    )
    assert picture.mode == "RGB"
    assert picture.size == (side * 5, side * 5)
    assert picture.getpixel((0, 0)) == (200, 10, 20)
    wide = decoded(
        picture_png(
            Image(width=2, height=1, channels=1, pixels=bytes([0, 255]))
        )
    )
    assert wide.size == (PAPER_WIDTH, PAPER_WIDTH // 2)


def test_a_picture_wider_than_the_page_is_not_shrunk() -> None:
    assert PAPER_WIDTH > BIG
    # No picture is ever larger than 256 pixels, so the smallest factor is 5.
    picture = decoded(
        picture_png(Image(width=BIG, height=1, channels=1, pixels=bytes(BIG)))
    )
    assert picture.size[0] >= BIG


def test_a_pale_colour_is_darkened_to_three_to_one_and_a_dark_one_is_left() -> (
    None
):
    assert contrast_with_white(PALE) < GRAPHIC_RATIO
    darker = darken_to(PALE)
    assert contrast_with_white(darker) >= GRAPHIC_RATIO
    assert darken_to(DARK) == DARK
    assert darken_to("#000000") == "#000000"


@given(rgb_values, rgb_values, rgb_values)
def test_darkening_always_reaches_the_ratio_and_does_not_change_the_hue(
    red: int, green: int, blue: int
) -> None:
    colour = f"#{red:02x}{green:02x}{blue:02x}"
    result = darken_to(colour)
    assert contrast_with_white(result) >= GRAPHIC_RATIO
    assert darken_to(result) == result
    if colour == result:
        return
    before = colorsys.rgb_to_hls(red / 255, green / 255, blue / 255)
    after_rgb = tuple(int(result[i : i + 2], 16) / 255 for i in (1, 3, 5))
    after = colorsys.rgb_to_hls(*after_rgb)
    saturated = before[2] > HUE_TOLERANCE and after[2] > HUE_TOLERANCE
    middling = (
        MIDDLE_LOW < before[1] < MIDDLE_HIGH
        and MIDDLE_LOW < after[1] < MIDDLE_HIGH
    )
    if saturated and middling:
        gap = abs(before[0] - after[0])
        assert min(gap, 1.0 - gap) < HUE_TOLERANCE


def test_importing_the_package_does_not_load_matplotlib() -> None:
    code = (
        "import sys, element_51, element_51.figures; "
        "loaded = {'matplotlib', 'numpy'} & set(sys.modules); "
        "sys.exit(1 if loaded else 0)"
    )
    result = subprocess.run(  # ruff: ignore[subprocess-without-shell-equals-true]  # fixed arguments
        [sys.executable, "-c", code], check=False, capture_output=True
    )
    assert result.returncode == 0, result.stderr.decode()
