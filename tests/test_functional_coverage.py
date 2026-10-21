"""Every public module of the package has a row in the notebook table."""

import re
from pathlib import Path

import element_51

ROOT = Path(__file__).parent
TABLE = ROOT / "functional" / "README.md"
ROW = re.compile(r"^\| `(?P<module>\w+)` \| (?P<notebook>.+) \|$", re.MULTILINE)


def _rows() -> dict[str, str]:
    """Read the coverage table.

    Returns
    -------
    dict[str, str]
        The notebook cell (or "none yet") for each module named in the table.
    """
    text = TABLE.read_text("utf-8")
    return {row["module"]: row["notebook"] for row in ROW.finditer(text)}


def test_every_public_module_has_a_row() -> None:
    package = Path(element_51.__file__).parent
    modules = {
        path.stem
        for path in package.glob("*.py")
        if not path.name.startswith("_")
    }
    assert modules - _rows().keys() == set()


def test_every_row_names_a_module_and_a_notebook_that_exists() -> None:
    package = Path(element_51.__file__).parent
    rows = _rows()
    no_module = [
        name for name in rows if not (package / f"{name}.py").is_file()
    ]
    notebooks = {name.strip("`") for name in rows.values()} - {"none yet"}
    no_file = [
        name for name in notebooks if not (TABLE.parent / name).is_file()
    ]
    assert (no_module, no_file) == ([], [])
