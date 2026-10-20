"""Print the numbers the restructure pass is judged by.

Counts the tracked Python, Rust and HTML files and their lines, the files over
400 lines, and the modules that import more than five others. With
`--jscpd PATH` it also runs the clone detector at PATH over the source folders
and prints its table. Run it from the repo root: `uv run python
tools/measure_structure.py`.

Notes
-----
    * Fixtures and generated files are not tracked, so they are not counted.
    * A Python module's imports are the distinct `element_51.<module>` names it
        imports. A Rust file's are the distinct lowercase modules after `use
        element_51_core::` or `use crate::`.
    * jscpd is not a project dependency; download it by hand and pass its
        binary (see DESIGN.md, restructure pass).
"""

# Standard library
import argparse
import re
import subprocess  # ruff: ignore[suspicious-subprocess-import]
import sys
import tempfile
from pathlib import Path

LINE_LIMIT = 400
IMPORT_LIMIT = 5
SUFFIXES = (".py", ".rs", ".html")
ROOT = Path(__file__).resolve().parent.parent
SOURCE_FOLDERS = (
    "src",
    "rust/element-51-core/src",
    "rust/element-51-web/src",
)
PYTHON_IMPORT = re.compile(
    r"^(?:from|import)\s+element_51\.(\w+)", re.MULTILINE
)
RUST_IMPORT = re.compile(
    r"^use\s+(?:element_51_core|crate)::\{?\s*([a-z_]\w*)", re.MULTILINE
)


def say(text: str) -> None:
    """Write a line to standard output."""
    sys.stdout.write(f"{text}\n")


def tracked_files() -> list[Path]:
    """Return the tracked source files.

    Returns
    -------
    list[Path]
        The tracked Python, Rust and HTML files, as paths under the repo root.
    """
    listing = subprocess.run(
        ["git", "ls-files", "-z"],  # ruff: ignore[start-process-with-partial-path]
        cwd=ROOT,
        capture_output=True,
        text=True,
        check=True,
    ).stdout
    names = [name for name in listing.split("\0") if name.endswith(SUFFIXES)]
    return [ROOT / name for name in names]


def import_count(path: Path) -> int:
    """Count the other modules a file imports.

    Parameters
    ----------
    path : Path
        A tracked source file.

    Returns
    -------
    int
        The number of distinct modules it imports; 0 for a page.
    """
    text = path.read_text(encoding="utf-8")
    if path.suffix == ".py":
        return len(set(PYTHON_IMPORT.findall(text)))
    if path.suffix == ".rs":
        return len(set(RUST_IMPORT.findall(text)))
    return 0


def main() -> None:
    """Print the structure numbers, and the clone table if asked."""
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--jscpd", type=Path, help="path of the jscpd binary")
    args = parser.parse_args()
    files = tracked_files()
    lines = {
        path: len(path.read_text(encoding="utf-8").splitlines())
        for path in files
    }
    say(f"files: {len(files)}, lines: {sum(lines.values())}")
    long_files = sorted(
        (path for path in files if lines[path] > LINE_LIMIT),
        key=lambda path: -lines[path],
    )
    say(f"files over {LINE_LIMIT} lines: {len(long_files)}")
    for path in long_files:
        say(f"    {lines[path]:5d}  {path.relative_to(ROOT)}")
    wide = sorted(
        (
            (import_count(path), path)
            for path in files
            if import_count(path) > IMPORT_LIMIT
        ),
        key=lambda pair: -pair[0],
    )
    say(f"modules importing more than {IMPORT_LIMIT} others: {len(wide)}")
    for count, path in wide:
        say(f"    {count:5d}  {path.relative_to(ROOT)}")
    if args.jscpd is not None:
        with tempfile.TemporaryDirectory() as temp:
            subprocess.run(  # ruff: ignore[subprocess-without-shell-equals-true]
                [
                    str(args.jscpd),
                    "--min-lines",
                    "8",
                    "--min-tokens",
                    "70",
                    "--output",
                    temp,
                    *SOURCE_FOLDERS,
                ],
                cwd=ROOT,
                check=True,
            )


if __name__ == "__main__":
    main()
