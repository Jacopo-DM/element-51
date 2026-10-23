"""Tests for the FastAPI and NiceGUI host examples of `f_embedding`."""

import json
import threading
from http import HTTPStatus
from pathlib import Path

import pytest
from server_support import (
    delete,
    fetch,
    post_json,
    running,
    running_example,
)
from test_examples import load
from websockets.sync.client import connect

from element_51 import server
from element_51.edits import SetValue
from element_51.protocol import JobUpdate, Snapshot
from element_51.session import JobEvent


def test_the_api_example_checks_requests_and_reports_runs(
    web_dir: Path,
) -> None:
    """Nodes and links go through typed bodies; a refusal says why."""
    assert web_dir.exists()
    example = load("f_embedding/f3_fastapi_api.py")
    host = example.build_host(example.build_app())
    with running(host) as port:
        status, _, body = fetch(port, "/api/node-types")
        assert status == HTTPStatus.OK
        assert json.loads(body) == [
            "Flag",
            "Float",
            "Integer",
            "Plot",
            "Slider",
            "Text",
            "add",
            "multiply",
        ]
        with connect(f"ws://{server.HOST}:{port}/editor/ws") as websocket:
            Snapshot.model_validate_json(websocket.recv())
            made = post_json(
                port,
                "/api/nodes",
                {"node_type": "multiply", "x": 400.0, "y": 120.0},
            )
            assert made == (
                HTTPStatus.CREATED,
                {"id": "n2", "node_type": "multiply", "x": 400.0, "y": 120.0},
            )
            pushed = Snapshot.model_validate_json(websocket.recv())
            assert [n.id for n in pushed.graph.nodes] == ["n1", "n2"]
        link = {
            "source": "n1",
            "source_port": "out",
            "target": "n2",
            "target_port": "a",
        }
        assert post_json(port, "/api/links", link) == (
            HTTPStatus.CREATED,
            link,
        )
        refused_status, refused = post_json(port, "/api/links", link)
        assert refused_status == HTTPStatus.CONFLICT
        assert isinstance(refused, dict)
        assert "detail" in refused
        missing = post_json(port, "/api/nodes", {"node_type": "divide"})
        assert missing[0] == HTTPStatus.NOT_FOUND
        assert post_json(port, "/api/nodes", {"kind": "add"})[0] == (
            HTTPStatus.UNPROCESSABLE_ENTITY
        )
        listed = json.loads(fetch(port, "/api/nodes")[2])
        assert [row["id"] for row in listed] == ["n1", "n2"]
        ran_status, ran = post_json(port, "/api/run", {})
        assert ran_status == HTTPStatus.OK
        assert isinstance(ran, dict)
        outcomes = ran["outcomes"]
        assert isinstance(outcomes, list)
        assert [(o["node"], o["status"]) for o in outcomes] == [
            ("n1", "ok"),
            ("n2", "ok"),
        ]
        assert outcomes[1]["outputs"] == {"out": 6.0}


def test_the_job_panel_example_describes_a_job_as_it_runs(
    web_dir: Path,
) -> None:
    """`describe` and `fraction` read the updates a real job sends."""
    assert web_dir.exists()
    example = load("f_embedding/f4_nicegui_job.py")
    app = example.build_app()
    app.apply(SetValue(node="n1", port="ticks", value=3))
    app.apply(SetValue(node="n1", port="pause", value=0.0))
    ended = threading.Event()
    updates: list[JobUpdate] = []

    def listen(event: JobEvent) -> None:
        if isinstance(event, JobUpdate):
            updates.append(event)
            if event.state == "finished":
                ended.set()

    app.subscribe_jobs(listen)
    app.start_job("n1")
    assert ended.wait(timeout=10)
    assert example.describe(None) == "idle"
    assert example.fraction(None) == pytest.approx(0.0)
    assert example.describe(updates[-1]) == "finished, step 3 of 3"
    assert example.fraction(updates[-1]) == pytest.approx(1.0)
    failed = updates[-1].model_copy(update={"state": "failed", "error": "bad"})
    assert example.describe(failed).endswith("bad")
    unknown = failed.model_copy(update={"steps": None})
    assert example.fraction(unknown) == pytest.approx(0.0)


def test_the_job_panel_example_serves_the_page_and_the_editor(
    web_dir: Path,
) -> None:
    """The NiceGUI page and the editor answer in a process of their own."""
    assert web_dir.exists()
    with running_example("f_embedding/f4_nicegui_job.py") as port:
        assert fetch(port, "/")[0] == HTTPStatus.OK
        assert (
            fetch(port, "/editor/")[2] == (web_dir / "index.html").read_bytes()
        )


def test_the_two_doors_example_shares_its_functions(web_dir: Path) -> None:
    """`place`, `node_rows` and `result_rows` work without either framework."""
    assert web_dir.exists()
    example = load("f_embedding/f5_both_front_doors.py")
    app = example.build_app()
    assert example.place(app, "multiply").id == "n2"
    assert [row.node_type for row in example.node_rows(app)] == [
        "add",
        "multiply",
    ]
    with pytest.raises(example.UnknownTypeError, match="divide"):
        example.place(app, "divide")
    rows = example.result_rows(app)
    assert [(row.node, row.status) for row in rows] == [
        ("n1", "ok"),
        ("n2", "ok"),
    ]
    assert rows[0].text == "{'out': 3.0}"


def test_the_two_doors_example_answers_the_api(web_dir: Path) -> None:
    """The page, the list, a post, a refusal and a run, in one host."""
    assert web_dir.exists()
    with running_example("f_embedding/f5_both_front_doors.py") as port:
        assert fetch(port, "/")[0] == HTTPStatus.OK
        listed = json.loads(fetch(port, "/api/nodes")[2])
        assert [row["id"] for row in listed] == ["n1"]
        status, made = post_json(port, "/api/nodes", {"node_type": "add"})
        assert status == HTTPStatus.CREATED
        assert isinstance(made, dict)
        assert made["id"] == "n2"
        missing = post_json(port, "/api/nodes", {"node_type": "divide"})
        assert missing[0] == HTTPStatus.NOT_FOUND
        ran = post_json(port, "/api/results", {})[1]
    assert isinstance(ran, list)
    assert [(row["node"], row["status"]) for row in ran] == [
        ("n1", "ok"),
        ("n2", "ok"),
    ]


def test_the_workspace_example_makes_mounts_saves_and_removes(
    web_dir: Path, tmp_path: Path
) -> None:
    """Each workspace has its own editor and file; bad names are refused."""
    assert web_dir.exists()
    name = "f_embedding/f6_workspace_server.py"
    with running_example(name, folder=tmp_path) as port:
        listed = json.loads(fetch(port, "/api/workspaces")[2])
        assert listed == [{"name": "demo", "nodes": 1, "links": 0}]
        assert fetch(port, "/")[0] == HTTPStatus.OK
        assert fetch(port, "/editor/demo/")[0] == HTTPStatus.OK
        assert fetch(port, "/editor/sums/")[0] == HTTPStatus.NOT_FOUND
        created = post_json(port, "/api/workspaces", {"name": "sums"})
        assert created == (
            HTTPStatus.CREATED,
            {"name": "sums", "nodes": 0, "links": 0},
        )
        assert fetch(port, "/editor/sums/")[0] == HTTPStatus.OK
        again = post_json(port, "/api/workspaces", {"name": "sums"})
        assert again[0] == HTTPStatus.CONFLICT
        for bad in ("Bad Name", "../x", "", "1a", "a" * 33):
            refused = post_json(port, "/api/workspaces", {"name": bad})
            assert refused[0] == HTTPStatus.UNPROCESSABLE_ENTITY
        ran = post_json(port, "/api/workspaces/demo/run", {})
        assert ran[0] == HTTPStatus.OK
        assert post_json(port, "/api/workspaces/nope/run", {})[0] == (
            HTTPStatus.NOT_FOUND
        )
        assert post_json(port, "/api/workspaces/demo/save", {})[0] == (
            HTTPStatus.NO_CONTENT
        )
        assert (tmp_path / "demo.json").is_file()
        assert delete(port, "/api/workspaces/sums") == HTTPStatus.NO_CONTENT
        assert fetch(port, "/editor/sums/")[0] == HTTPStatus.NOT_FOUND
        assert delete(port, "/api/workspaces/sums") == HTTPStatus.NOT_FOUND
    with running_example(name, folder=tmp_path) as port:
        listed = json.loads(fetch(port, "/api/workspaces")[2])
        assert listed == [{"name": "demo", "nodes": 1, "links": 0}]


def test_a_run_is_summarised_in_one_line(web_dir: Path) -> None:
    """`summary` names each node and says what it made or why it failed."""
    assert web_dir.exists()
    example = load("f_embedding/f6_workspace_server.py")
    app = example.build_app()
    assert example.summary(app.execute()) == "n1: ok {'out': 3.0}"
