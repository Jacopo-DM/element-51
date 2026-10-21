"""Shared pytest setup: the Hypothesis profile and the server fixtures."""

import os
from pathlib import Path

import pytest
from hypothesis import settings
from server_support import WASM_MAGIC

from element_51 import server

settings.register_profile("default", deadline=None)
settings.register_profile("ci", deadline=None, derandomize=True)
settings.load_profile("ci" if os.environ.get("CI") else "default")


@pytest.fixture
def web_dir(tmp_path: Path, monkeypatch: pytest.MonkeyPatch) -> Path:
    """Build a fake output folder in place of `src/element_51/web/`.

    Returns
    -------
    Path
        The folder, holding `index.html` and one `.wasm` file.
    """
    (tmp_path / "index.html").write_text("<p>element-51</p>")
    (tmp_path / "element_51_web_bg.wasm").write_bytes(WASM_MAGIC)
    monkeypatch.setattr(server, "WEB_DIR", tmp_path)
    return tmp_path
