"""The names of the editor's themes.

A theme is a complete look: colours, sizes and fonts. The look itself lives in
the Rust core (`rust/element-51-core/themes/`); Python only picks one by name,
and a shared test file keeps the two lists of names equal.
"""

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
