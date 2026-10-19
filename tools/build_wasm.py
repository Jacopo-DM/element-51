"""Build the browser layer and copy it into the Python package.

Runs `cargo build` for `wasm32-unknown-unknown` with the rustup `stable`
toolchain, then `wasm-bindgen`, then copies the output and `index.html` into
`src/element_51/web/`. Run it from the repo root: `uv run python
tools/build_wasm.py`.

Notes
-----
    * The Homebrew `rustc` on PATH has no WASM target, so the rustup binaries
        are named explicitly.
    * The rustup `rust-lld` cannot find its `libLLVM.dylib`; the build sets
        `DYLD_FALLBACK_LIBRARY_PATH` to the toolchain's `lib` folder.
    * `tools/bin/wasm-bindgen` must be 0.2.129, the version of the crate.
"""

# Standard library
import os
import shutil
import subprocess  # ruff: ignore[suspicious-subprocess-import]
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
RUST = ROOT / "rust"
WEB = ROOT / "src" / "element_51" / "web"
WASM_BINDGEN = ROOT / "tools" / "bin" / "wasm-bindgen"
TARGET = "wasm32-unknown-unknown"
CRATE = "element_51_web"


def rustup_which(tool: str) -> Path:
    """Return the path of a tool in the rustup `stable` toolchain.

    Parameters
    ----------
    tool : str
        The tool name, for example `cargo`.

    Returns
    -------
    Path
        Absolute path of the tool.
    """
    out = subprocess.run(  # ruff: ignore[subprocess-without-shell-equals-true]
        ["rustup", "which", "--toolchain", "stable", tool],  # ruff: ignore[start-process-with-partial-path]
        check=True,
        capture_output=True,
        text=True,
    )
    return Path(out.stdout.strip())


def build() -> None:
    """Compile the crate, run wasm-bindgen and fill `src/element_51/web/`."""
    cargo = rustup_which("cargo")
    rustc = rustup_which("rustc")
    env = os.environ | {
        "RUSTC": str(rustc),
        "DYLD_FALLBACK_LIBRARY_PATH": str(rustc.parent.parent / "lib"),
    }
    subprocess.run(  # ruff: ignore[subprocess-without-shell-equals-true]
        [str(cargo), "build", "--target", TARGET, "--release"],
        cwd=RUST,
        env=env,
        check=True,
    )
    wasm = RUST / "target" / TARGET / "release" / f"{CRATE}.wasm"
    shutil.rmtree(WEB, ignore_errors=True)
    WEB.mkdir(parents=True)
    subprocess.run(  # ruff: ignore[subprocess-without-shell-equals-true]
        [
            str(WASM_BINDGEN),
            "--target",
            "web",
            "--out-dir",
            str(WEB),
            str(wasm),
        ],
        check=True,
    )
    shutil.copy(RUST / "element-51-web" / "static" / "index.html", WEB)


if __name__ == "__main__":
    build()
    sys.stdout.write(f"built {WEB}\n")
