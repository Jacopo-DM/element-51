"""The editor's themes: a built-in name, or a file of changes to one.

A theme is a complete look: colours, sizes and fonts. The look itself lives in
the Rust core (`rust/element-51-core/themes/`); Python only picks a built-in
one by name, and a shared test file keeps the two lists of names equal. A
theme file changes some of a built-in theme's values: Python checks its shape,
and the browser checks the keys and says which one is wrong.
"""

from pathlib import Path

from pydantic import Field, field_validator

from element_51.model import Model

THEME_NAMES = ("antimony", "antimony-light")
DEFAULT_THEME = THEME_NAMES[0]


def check_theme(name: str) -> str:
    """Return `name` if it is a theme, or say which names are.

    Parameters
    ----------
    name
        The theme name to check.

    Returns
    -------
    str
        The same name.

    Raises
    ------
    ValueError
        If there is no theme with that name.
    """
    if name not in THEME_NAMES:
        valid = ", ".join(THEME_NAMES)
        msg = f"unknown theme {name!r}; the themes are: {valid}"
        raise ValueError(msg)
    return name


class ThemeChanges(Model):
    """Values that replace a built-in theme's own, by section and key.

    `ports` nests one level more: port type, `in` or `out`, then `colour` or
    `shape`.

    The keys are those in `rust/element-51-core/themes/antimony.json`.
    """

    colours: dict[str, str] = Field(default_factory=dict)
    ports: dict[str, dict[str, dict[str, str]]] = Field(default_factory=dict)
    metrics: dict[str, float] = Field(default_factory=dict)
    page: dict[str, float] = Field(default_factory=dict)
    fonts: dict[str, str | float] = Field(default_factory=dict)


class ThemeFile(ThemeChanges):
    """A theme file: the built-in theme to start from, and the changes."""

    base: str = DEFAULT_THEME

    @field_validator("base")
    @classmethod
    def _base_is_a_theme(cls, base: str) -> str:
        return check_theme(base)


def resolve_theme(choice: str | Path) -> ThemeFile:
    """Turn a theme name or the path of a theme file into a theme.

    Parameters
    ----------
    choice
        A built-in theme name, or the path of a JSON theme file.

    Returns
    -------
    ThemeFile
        The built-in theme to start from, and the changes to it.

    Raises
    ------
    ValueError
        If `choice` is neither a built-in name nor a file, or the file is not
        a theme file.
    """
    if isinstance(choice, str) and choice in THEME_NAMES:
        return ThemeFile(base=choice)
    path = Path(choice)
    if not path.is_file():
        valid = ", ".join(THEME_NAMES)
        msg = (
            f"unknown theme {str(choice)!r}: it is not a built-in theme "
            f"({valid}) and not a file"
        )
        raise ValueError(msg)
    return ThemeFile.model_validate_json(path.read_text(encoding="utf-8"))
