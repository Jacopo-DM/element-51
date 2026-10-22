"""Series colours: the shared edit cases, the file and the refusals."""

import json
from pathlib import Path

import pytest
from pydantic import TypeAdapter

from element_51 import App, PlotNode
from element_51.edits import (
    Edit,
    EditError,
    EditErrorKind,
    SetColour,
    apply_edit,
)
from element_51.model import Graph, Model, NodeType
from element_51.store import FORMAT_VERSION, StoreError

FIXTURE = Path(__file__).parent / "fixtures" / "colour_cases.json"
EDIT = TypeAdapter(Edit)
ORANGE = "#ff8800"
GREEN = "#00aa55"


class ColourCase(Model):
    """One colour edit, with the graph it gives or the reason it is refused."""

    name: str
    edit: dict[str, object]
    result: str
    graph: Graph | None = None


class ColourCases(Model):
    """The shared fixture for colour edits."""

    node_types: tuple[NodeType, ...]
    graph: Graph
    cases: tuple[ColourCase, ...]


CASES = ColourCases.model_validate_json(FIXTURE.read_text())
NODE_TYPES = {t.name: t for t in CASES.node_types}


@pytest.mark.parametrize("case", CASES.cases, ids=lambda case: case.name)
def test_colour_edits_match_the_shared_cases(case: ColourCase) -> None:
    """Python gives the graph, or the refusal, the Rust side gives."""
    edit = EDIT.validate_python(case.edit)
    if case.result == "ok":
        assert apply_edit(CASES.graph, NODE_TYPES, edit) == case.graph
        return
    with pytest.raises(EditError) as raised:
        apply_edit(CASES.graph, NODE_TYPES, edit)
    assert raised.value.kind == EditErrorKind(case.result)
    assert raised.value.reason


def plot_app() -> App:
    """Build an app with one Plot node.

    Returns
    -------
    App
        An app whose graph holds `n1`, a Plot.
    """
    app = App([PlotNode])
    app.add_node(PlotNode, x=0.0, y=0.0)
    return app


def test_a_chosen_colour_is_in_the_graph_and_the_saved_file(
    tmp_path: Path,
) -> None:
    """The file keeps the colour, and a fresh app loads it back."""
    app = plot_app()
    app.apply(SetColour(node="n1", port="series_1", colour=ORANGE))
    app.apply(SetColour(node="n1", port="series_2", colour=GREEN))
    assert app.graph.nodes[0].colours == {
        "series_1": ORANGE,
        "series_2": GREEN,
    }
    path = tmp_path / "g.json"
    app.save(path)
    saved = json.loads(path.read_text("utf-8"))
    assert saved["format_version"] == FORMAT_VERSION
    assert saved["nodes"][0]["colours"] == {
        "series_1": ORANGE,
        "series_2": GREEN,
    }
    fresh = App([PlotNode])
    fresh.load(path)
    assert fresh.graph == app.graph


@pytest.mark.parametrize("version", [1, 2, 3])
def test_an_older_file_without_colours_still_loads(
    tmp_path: Path, version: int
) -> None:
    """Files written before colours existed read as having none."""
    path = tmp_path / "g.json"
    path.write_text(
        json.dumps({
            "format_version": version,
            "nodes": [{"id": "n1", "node_type": "Plot", "x": 1.0, "y": 2.0}],
            "links": [],
        }),
        "utf-8",
    )
    app = App([PlotNode])
    app.load(path)
    assert app.graph.nodes[0].colours == {}


def test_a_file_with_a_bad_colour_is_refused(tmp_path: Path) -> None:
    """A hand-edited colour is checked by the same rule as an edit."""
    path = tmp_path / "g.json"
    path.write_text(
        json.dumps({
            "format_version": FORMAT_VERSION,
            "nodes": [
                {
                    "id": "n1",
                    "node_type": "Plot",
                    "x": 0.0,
                    "y": 0.0,
                    "colours": {"series_1": "red"},
                }
            ],
            "links": [],
        }),
        "utf-8",
    )
    with pytest.raises(StoreError, match="#rrggbb"):
        App([PlotNode]).load(path)
