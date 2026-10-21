"""Tests for the theme names Python offers."""

import json
import tempfile
from pathlib import Path

import pytest
from hypothesis import example, given
from hypothesis import strategies as st

from element_51.theme import (
    DEFAULT_THEME,
    THEME_NAMES,
    ThemeFile,
    check_theme,
    resolve_theme,
)

FIXTURE = Path(__file__).parent / "fixtures" / "theme_names.json"


def test_python_and_rust_offer_the_same_themes() -> None:
    """The Rust test reads the same file, so the two lists cannot drift."""
    assert list(THEME_NAMES) == json.loads(FIXTURE.read_text())


def test_the_default_is_purple_synthwave() -> None:
    """The purple-synthwave theme is the default."""
    assert DEFAULT_THEME == "purple-synthwave"
    assert DEFAULT_THEME in THEME_NAMES


@pytest.mark.parametrize("name", THEME_NAMES)
def test_a_known_name_passes(name: str) -> None:
    """A valid name comes back unchanged."""
    assert check_theme(name) == name


def test_an_unknown_name_lists_the_valid_ones() -> None:
    """The error names what was asked for and every valid theme."""
    with pytest.raises(ValueError, match="'dark'") as raised:
        check_theme("dark")
    assert "antimony, antimony-light" in str(raised.value)


def write(path: Path, text: str) -> Path:
    """Write a theme file.

    Returns
    -------
    Path
        The file written.
    """
    path.write_text(text, encoding="utf-8")
    return path


def test_a_theme_file_lists_only_what_it_changes(tmp_path: Path) -> None:
    """Missing sections are empty and the base is the default theme."""
    theme = resolve_theme(
        write(tmp_path / "t.json", '{"fonts": {"value": "14px Menlo"}}')
    )
    assert theme.base == DEFAULT_THEME
    assert theme.fonts == {"value": "14px Menlo"}
    assert theme.colours == {}
    assert theme.metrics == {}


def test_a_theme_file_may_name_another_base_and_change_all_sections(
    tmp_path: Path,
) -> None:
    """Colours are text, metrics are numbers, and `field_px` is a number."""
    text = (
        '{"base": "antimony-light", "colours": {"canvas": "#fff"},'
        ' "metrics": {"node_width": 260}, "page": {"button_radius": 12},'
        ' "fonts": {"field_px": 14}}'
    )
    theme = resolve_theme(write(tmp_path / "t.json", text))
    assert theme.base == "antimony-light"
    assert theme.colours == {"canvas": "#fff"}
    assert theme.metrics == {"node_width": 260.0}
    assert theme.page == {"button_radius": 12.0}
    assert theme.fonts == {"field_px": 14}


def test_a_path_object_works_like_a_string_path(tmp_path: Path) -> None:
    """`serve(theme=Path(...))` and `serve(theme="...")` are the same."""
    path = write(tmp_path / "t.json", "{}")
    assert resolve_theme(path) == resolve_theme(str(path))


@pytest.mark.parametrize("name", THEME_NAMES)
def test_a_built_in_name_resolves_to_it_with_no_changes(name: str) -> None:
    """A name is a theme file that changes nothing."""
    theme = resolve_theme(name)
    assert theme.base == name
    assert not theme.colours
    assert not theme.metrics
    assert not theme.fonts


def test_something_that_is_neither_a_name_nor_a_file_names_both_ways(
    tmp_path: Path,
) -> None:
    """The error lists the built-in names and says the file is missing."""
    with pytest.raises(ValueError, match=r"antimony-light.*not a file"):
        resolve_theme(str(tmp_path / "missing.json"))


def test_a_theme_file_may_restyle_the_ports_by_type_and_side(
    tmp_path: Path,
) -> None:
    """`ports` nests type, `in` or `out`, then `colour` or `shape`."""
    text = '{"ports": {"float": {"out": {"shape": "circle"}}}}'
    theme = resolve_theme(write(tmp_path / "t.json", text))
    assert theme.ports == {"float": {"out": {"shape": "circle"}}}


@pytest.mark.parametrize(
    "text",
    [
        "not json",
        '{"ports": {"float": {"out": {"shape": 3}}}}',
        '{"ports": {"float": "circle"}}',
        "[]",
        '{"colors": {}}',
        '{"colours": {"canvas": 5}}',
        '{"metrics": {"node_width": "wide"}}',
        '{"page": {"menu_pad": "big"}}',
        '{"base": "nope"}',
    ],
)
def test_a_bad_theme_file_is_refused(tmp_path: Path, text: str) -> None:
    """Wrong shapes, unknown sections and unknown bases are errors."""
    with pytest.raises(ValueError, match=r"."):
        resolve_theme(write(tmp_path / "t.json", text))


@given(st.text())
@example('{"base": "antimony", "colours": {"canvas": "#000000"}}')
@example("{}")
def test_theme_file_boundary(text: str) -> None:
    """Any file text is a theme or a ValueError, nothing else."""
    with tempfile.TemporaryDirectory() as folder:
        path = Path(folder) / "t.json"
        path.write_text(text, encoding="utf-8")
        try:
            theme = resolve_theme(path)
        except ValueError:
            return
    assert theme.base in THEME_NAMES


FINITE = st.floats(allow_nan=False, allow_infinity=False)
THEME_FILES = st.builds(
    ThemeFile,
    base=st.sampled_from(THEME_NAMES),
    colours=st.dictionaries(st.text(), st.text()),
    ports=st.dictionaries(
        st.text(),
        st.dictionaries(st.text(), st.dictionaries(st.text(), st.text())),
    ),
    metrics=st.dictionaries(st.text(), FINITE),
    page=st.dictionaries(st.text(), FINITE),
    fonts=st.dictionaries(st.text(), st.one_of(st.text(), FINITE)),
)


@given(THEME_FILES)
def test_theme_file_round_trip(theme: ThemeFile) -> None:
    """A theme file read back from what it dumps is the same theme."""
    with tempfile.TemporaryDirectory() as folder:
        path = Path(folder) / "t.json"
        path.write_text(theme.model_dump_json(), encoding="utf-8")
        assert resolve_theme(path) == theme
