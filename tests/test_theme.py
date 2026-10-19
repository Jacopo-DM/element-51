"""Tests for the theme names Python offers."""

import json
from pathlib import Path

import pytest

from element_51.theme import DEFAULT_THEME, THEME_NAMES, check_theme

FIXTURE = Path(__file__).parent / "fixtures" / "theme_names.json"


def test_python_and_rust_offer_the_same_themes() -> None:
    """The Rust test reads the same file, so the two lists cannot drift."""
    assert list(THEME_NAMES) == json.loads(FIXTURE.read_text())


def test_the_default_is_the_first_name() -> None:
    """The dark Antimony theme is the default."""
    assert DEFAULT_THEME == THEME_NAMES[0] == "antimony"


@pytest.mark.parametrize("name", THEME_NAMES)
def test_a_known_name_passes(name: str) -> None:
    """A valid name comes back unchanged."""
    assert check_theme(name) == name


def test_an_unknown_name_lists_the_valid_ones() -> None:
    """The error names what was asked for and every valid theme."""
    with pytest.raises(ValueError, match="'dark'") as raised:
        check_theme("dark")
    assert "antimony, antimony-light" in str(raised.value)
