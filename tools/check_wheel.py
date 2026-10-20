"""Build the wheel and fail if it does not hold the browser layer.

`src/element_51/web/` is generated and git-ignored, so `uv build` succeeds
without it and the wheel ships without the editor. Run this before publishing:
`uv run python tools/check_wheel.py`. It exits 0 when the wheel holds the files
below, and 1 with the missing names otherwise.

Notes
-----
    * The wheel is built in a temporary folder; `dist/` is not touched.
    * Fix a failure with `uv run python tools/build_wasm.py`, then run this
        again.
"""

# Standard library
import subprocess  # ruff: ignore[suspicious-subprocess-import]
import sys
import tempfile
import zipfile
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
REQUIRED = (
    "element_51/web/index.html",
    "element_51/web/element_51_web.js",
    "element_51/web/element_51_web_bg.wasm",
)


def missing_from_wheel() -> list[str]:
    """Build the wheel and list the required files it does not hold.

    Returns
    -------
    list[str]
        The names from `REQUIRED` that are absent, empty if all are present.
    """
    with tempfile.TemporaryDirectory() as out:
        subprocess.run(  # ruff: ignore[subprocess-without-shell-equals-true]  # our own uv call, fixed arguments
            ["uv", "build", "--wheel", "--out-dir", out],  # ruff: ignore[start-process-with-partial-path]
            cwd=ROOT,
            check=True,
            capture_output=True,
        )
        wheel = next(Path(out).glob("*.whl"))
        with zipfile.ZipFile(wheel) as archive:
            names = set(archive.namelist())
    return [name for name in REQUIRED if name not in names]


def say(text: str) -> None:
    """Write one line to standard output.

    Parameters
    ----------
    text
        The line, without a newline.
    """
    sys.stdout.write(f"{text}\n")


def main() -> int:
    """Print the result and return the exit code.

    Returns
    -------
    int
        0 if the wheel holds the browser layer, 1 if not.
    """
    missing = missing_from_wheel()
    if missing:
        say(f"The wheel is missing: {', '.join(missing)}")
        say("Run: uv run python tools/build_wasm.py")
        return 1
    say("The wheel holds the browser layer.")
    return 0


if __name__ == "__main__":
    sys.exit(main())
