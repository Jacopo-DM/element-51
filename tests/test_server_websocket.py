"""Tests for the editor server's websocket."""

import asyncio
import base64
import json
import time
from collections.abc import Callable
from pathlib import Path

import pytest
from sample_jobs import WAIT, Pause, Stamp, Steps, Timed
from sample_nodes import add, show
from server_support import running
from starlette.websockets import WebSocketDisconnect
from websockets.exceptions import ConnectionClosed
from websockets.sync.client import ClientConnection, connect

from element_51 import server
from element_51.engine import RunResult
from element_51.figures import Line
from element_51.protocol import (
    Ack,
    JobRefusal,
    JobUpdate,
    PngChartRequest,
    PngPictureRequest,
    PngReply,
    Reject,
    Snapshot,
    SourceReply,
)
from element_51.session import App
from element_51.visuals import Image

REQUEST_ID_7 = 7
REQUEST_ID_8 = 8
INVALID_PAYLOAD = 1007
SILENCE = 0.3
PNG_WAIT = 30.0
PNG_MAGIC = b"\x89PNG\r\n\x1a\n"


def test_websocket_sends_a_snapshot_on_connect(web_dir: Path) -> None:
    """A browser that connects gets the node types and the graph."""
    assert web_dir.exists()
    app = App([add])
    app.add_node(add, x=5.0, y=6.0)
    with (
        running(server.asgi_app(app)) as port,
        connect(f"ws://{server.HOST}:{port}/ws") as websocket,
    ):
        message = websocket.recv()
    snapshot = Snapshot.model_validate_json(message)
    assert [t.name for t in snapshot.node_types] == [
        "Float",
        "Integer",
        "Text",
        "Flag",
        "Slider",
        "Plot",
        "add",
    ]
    assert [(n.id, n.x, n.y) for n in snapshot.graph.nodes] == [
        ("n1", 5.0, 6.0)
    ]


def test_every_new_connection_gets_the_current_graph(web_dir: Path) -> None:
    """A reload sees the edits made since the last connection."""
    assert web_dir.exists()
    app = App([add])
    with running(server.asgi_app(app)) as port:
        url = f"ws://{server.HOST}:{port}/ws"
        with connect(url) as first:
            before = Snapshot.model_validate_json(first.recv())
        app.add_node(add, x=1.0, y=1.0)
        with connect(url) as second:
            after = Snapshot.model_validate_json(second.recv())
    assert len(before.graph.nodes) == 0
    assert len(after.graph.nodes) == 1


def test_edits_made_in_the_browser_are_there_after_a_reload(
    web_dir: Path,
) -> None:
    """The browser builds a graph, goes away, and a new page gets it back."""
    assert web_dir.exists()
    app = App([add])
    requests = [
        (
            '{"op": "edit", "id": 1, "edit": {"kind": "add_node", "id": "n1",'
            ' "node_type": "add", "x": 10.0, "y": 20.0}}'
        ),
        (
            '{"op": "edit", "id": 2, "edit": {"kind": "add_node", "id": "n2",'
            ' "node_type": "add", "x": 300.0, "y": 40.0}}'
        ),
        (
            '{"op": "edit", "id": 3, "edit": {"kind": "add_link",'
            ' "source": {"node": "n1", "port": "out"},'
            ' "target": {"node": "n2", "port": "a"}}}'
        ),
    ]
    with running(server.asgi_app(app)) as port:
        url = f"ws://{server.HOST}:{port}/ws"
        with connect(url) as before_reload:
            before_reload.recv()
            for request in requests:
                before_reload.send(request)
                Ack.model_validate_json(before_reload.recv())
        with connect(url) as after_reload:
            snapshot = Snapshot.model_validate_json(after_reload.recv())
    assert [(n.id, n.x, n.y) for n in snapshot.graph.nodes] == [
        ("n1", 10.0, 20.0),
        ("n2", 300.0, 40.0),
    ]
    assert [
        (link.source.node, link.target.node, link.target.port)
        for link in snapshot.graph.links
    ] == [("n1", "n2", "a")]


def test_two_open_pages_each_get_the_whole_graph(web_dir: Path) -> None:
    """A second tab is a second connection and starts from a full snapshot."""
    assert web_dir.exists()
    app = App([add])
    app.add_node(add, x=1.0, y=2.0)
    with running(server.asgi_app(app)) as port:
        url = f"ws://{server.HOST}:{port}/ws"
        with connect(url) as first, connect(url) as second:
            one = Snapshot.model_validate_json(first.recv())
            two = Snapshot.model_validate_json(second.recv())
    assert one == two
    assert len(one.graph.nodes) == 1


def test_an_edit_from_the_browser_is_applied_and_acknowledged(
    web_dir: Path,
) -> None:
    """The browser adds a node; Python acks it and `app.graph` has it."""
    assert web_dir.exists()
    app = App([add])
    request = (
        '{"op": "edit", "id": 7, "edit": {"kind": "add_node", "id": "n1",'
        ' "node_type": "add", "x": 3.0, "y": 4.0}}'
    )
    with (
        running(server.asgi_app(app)) as port,
        connect(f"ws://{server.HOST}:{port}/ws") as websocket,
    ):
        websocket.recv()  # the snapshot
        websocket.send(request)
        reply = Ack.model_validate_json(websocket.recv())
    assert reply.id == REQUEST_ID_7
    assert [(n.id, n.x, n.y) for n in app.graph.nodes] == [("n1", 3.0, 4.0)]


def test_a_bad_edit_is_rejected_with_a_reason(web_dir: Path) -> None:
    """A link to a node that does not exist gets `reject`, not a crash."""
    assert web_dir.exists()
    app = App([add])
    request = (
        '{"op": "edit", "id": 8, "edit": {"kind": "add_link",'
        ' "source": {"node": "n1", "port": "out"},'
        ' "target": {"node": "n2", "port": "a"}}}'
    )
    with (
        running(server.asgi_app(app)) as port,
        connect(f"ws://{server.HOST}:{port}/ws") as websocket,
    ):
        websocket.recv()
        websocket.send(request)
        reply = Reject.model_validate_json(websocket.recv())
        websocket.send(request)  # the connection is still usable
        again = Reject.model_validate_json(websocket.recv())
    assert reply.id == REQUEST_ID_8
    assert "n1" in reply.reason
    assert again.id == reply.id
    assert app.graph.links == ()


def test_a_malformed_message_closes_the_connection(web_dir: Path) -> None:
    """Garbage is not guessed at: the socket closes with 1007 and a reason."""
    assert web_dir.exists()
    with (
        running(server.asgi_app(App([add]))) as port,
        connect(f"ws://{server.HOST}:{port}/ws") as websocket,
    ):
        websocket.recv()
        websocket.send("not json")
        with pytest.raises(ConnectionClosed) as closed:
            websocket.recv()
    assert closed.value.rcvd is not None
    assert closed.value.rcvd.code == INVALID_PAYLOAD
    assert "malformed" in closed.value.rcvd.reason


def test_a_run_request_returns_the_outcome_of_every_node(web_dir: Path) -> None:
    """The browser's Run button: every node comes back with a status."""
    assert web_dir.exists()
    app = App([add])
    app.add_node(add, x=0.0, y=0.0)
    with (
        running(server.asgi_app(app)) as port,
        connect(f"ws://{server.HOST}:{port}/ws") as websocket,
    ):
        websocket.recv()
        websocket.send('{"op": "run"}')
        result = RunResult.model_validate_json(websocket.recv())
    assert [(o.node, o.status) for o in result.outcomes] == [("n1", "error")]
    assert result.outcomes[0].error == "input 'a' has no value"


def test_a_source_request_is_answered_to_the_asking_page_only(
    web_dir: Path,
) -> None:
    """The reply holds the node's code and reaches no other page."""
    assert web_dir.exists()
    app = App([add])
    app.add_node(add, x=0.0, y=0.0)
    with (
        running(server.asgi_app(app)) as port,
        connect(f"ws://{server.HOST}:{port}/ws") as asker,
        connect(f"ws://{server.HOST}:{port}/ws") as other,
    ):
        asker.recv()
        other.recv()
        asker.send('{"op": "source", "node": "n1"}')
        reply = SourceReply.model_validate_json(asker.recv())
        with pytest.raises(TimeoutError):
            other.recv(timeout=SILENCE)
    assert reply.node == "n1"
    assert reply.source is not None
    assert "def add(" in reply.source


def test_a_png_request_is_answered_with_the_file_to_the_asking_page_only(
    web_dir: Path,
) -> None:
    """The reply holds the PNG as base64 and reaches no other page."""
    assert web_dir.exists()
    app = App([add])
    request = PngChartRequest(
        node="n4",
        name="n4-chart.png",
        lines=(
            Line(
                name="loss",
                values=(1.0, 0.5, None),
                colour="#1a1a80",
                chosen=True,
            ),
        ),
    )
    with (
        running(server.asgi_app(app)) as port,
        connect(f"ws://{server.HOST}:{port}/ws") as asker,
        connect(f"ws://{server.HOST}:{port}/ws") as other,
    ):
        asker.recv()
        other.recv()
        asker.send(request.model_dump_json())
        reply = PngReply.model_validate_json(asker.recv(timeout=PNG_WAIT))
        with pytest.raises(TimeoutError):
            other.recv(timeout=SILENCE)
    assert (reply.node, reply.name, reply.reason) == (
        "n4",
        "n4-chart.png",
        None,
    )
    assert reply.data is not None
    assert base64.b64decode(reply.data).startswith(PNG_MAGIC)


def test_a_picture_request_is_answered_with_a_png(web_dir: Path) -> None:
    """A picture comes back enlarged, as a PNG file in base64."""
    assert web_dir.exists()
    request = PngPictureRequest(
        node="n1",
        name="n1-out.png",
        picture=Image(width=2, height=1, channels=1, pixels=bytes([0, 255])),
    )
    with (
        running(server.asgi_app(App([add]))) as port,
        connect(f"ws://{server.HOST}:{port}/ws") as page,
    ):
        page.recv()
        page.send(request.model_dump_json())
        reply = PngReply.model_validate_json(page.recv(timeout=PNG_WAIT))
    assert reply.data is not None
    assert base64.b64decode(reply.data).startswith(PNG_MAGIC)


def test_a_chart_with_no_numbers_is_answered_with_the_reason(
    web_dir: Path,
) -> None:
    """Nothing is drawn, and the page is told why."""
    assert web_dir.exists()
    request = PngChartRequest(
        node="n4",
        name="n4-chart.png",
        lines=(
            Line(name="loss", values=(None,), colour="#1a1a80", chosen=True),
        ),
    )
    with (
        running(server.asgi_app(App([add]))) as port,
        connect(f"ws://{server.HOST}:{port}/ws") as page,
    ):
        page.recv()
        page.send(request.model_dump_json())
        reply = PngReply.model_validate_json(page.recv(timeout=PNG_WAIT))
    assert reply.data is None
    assert reply.reason is not None
    assert "no numbers" in reply.reason


def test_a_script_edit_reaches_a_page_that_is_already_open(
    web_dir: Path,
) -> None:
    """`app.add_node` after the page connected: the page gets a new snapshot."""
    assert web_dir.exists()
    app = App([add, show])
    with (
        running(server.asgi_app(app)) as port,
        connect(f"ws://{server.HOST}:{port}/ws") as websocket,
    ):
        websocket.recv()
        first = app.add_node(add, x=1.0, y=2.0)
        pushed = Snapshot.model_validate_json(websocket.recv(timeout=5))
        second = app.add_node(show, x=3.0, y=4.0)
        websocket.recv(timeout=5)
        app.add_link(first, "out", second, "value")
        linked = Snapshot.model_validate_json(websocket.recv(timeout=5))
    assert [n.id for n in pushed.graph.nodes] == ["n1"]
    assert [(n.source.node, n.target.node) for n in linked.graph.links] == [
        ("n1", "n2")
    ]


def test_an_edit_in_one_page_reaches_the_other_page_only(web_dir: Path) -> None:
    """The sender gets only its ack; the other page gets a snapshot."""
    assert web_dir.exists()
    app = App([add])
    request = (
        '{"op": "edit", "id": 1, "edit": {"kind": "add_node", "id": "n1",'
        ' "node_type": "add", "x": 3.0, "y": 4.0}}'
    )
    with running(server.asgi_app(app)) as port:
        url = f"ws://{server.HOST}:{port}/ws"
        with connect(url) as sender, connect(url) as watcher:
            sender.recv()
            watcher.recv()
            sender.send(request)
            Ack.model_validate_json(sender.recv(timeout=5))
            pushed = Snapshot.model_validate_json(watcher.recv(timeout=5))
            with pytest.raises(TimeoutError):
                sender.recv(timeout=SILENCE)
    assert [n.id for n in pushed.graph.nodes] == ["n1"]


def test_a_closed_page_is_forgotten(web_dir: Path) -> None:
    """After one page closes, a script edit still reaches the other."""
    assert web_dir.exists()
    app = App([add])
    with running(server.asgi_app(app)) as port:
        url = f"ws://{server.HOST}:{port}/ws"
        with connect(url) as stays:
            stays.recv()
            with connect(url) as leaves:
                leaves.recv()
            # The server notices the close in its own time; edit until it has.
            app.add_node(add, x=0.0, y=0.0)
            pushed = Snapshot.model_validate_json(stays.recv(timeout=5))
    assert len(pushed.graph.nodes) == 1


class _GoneSender:
    """A page whose socket closed before the message could be sent."""

    def __init__(self, error: Exception) -> None:
        self.error = error

    async def send_text(self, data: str) -> None:
        assert data == "hello"
        raise self.error


@pytest.mark.parametrize(
    "error", [WebSocketDisconnect(code=1006), RuntimeError("closed")]
)
def test_the_sender_task_ends_quietly_when_the_page_is_gone(
    error: Exception,
) -> None:
    """A send to a closed socket stops the task; nothing is raised."""

    async def scenario() -> None:
        connection = server.Connection()
        connection.send("hello")
        await connection.pump(_GoneSender(error))

    asyncio.run(scenario())


def test_loading_a_file_updates_a_page_that_is_already_open(
    web_dir: Path, tmp_path: Path
) -> None:
    """`app.load` after the page connected: the page gets the loaded graph."""
    assert web_dir.exists()
    source = App([add, show])
    first = source.add_node(add, x=1.0, y=2.0)
    second = source.add_node(show, x=3.0, y=4.0)
    source.add_link(first, "out", second, "value")
    source.save(tmp_path / "g.json")
    app = App([add, show])
    with (
        running(server.asgi_app(app)) as port,
        connect(f"ws://{server.HOST}:{port}/ws") as websocket,
    ):
        websocket.recv()
        app.load(tmp_path / "g.json")
        pushed = Snapshot.model_validate_json(websocket.recv(timeout=5))
    assert pushed.graph == source.graph


def test_move_and_delete_from_the_browser_change_the_graph(
    web_dir: Path,
) -> None:
    """A moved node and a deleted node are acknowledged and applied."""
    assert web_dir.exists()
    app = App([add])
    app.add_node(add, x=0.0, y=0.0)
    app.add_node(add, x=1.0, y=1.0)
    move = (
        '{"op": "edit", "id": 1, "edit": {"kind": "move_node",'
        ' "id": "n1", "x": 9.5, "y": 8.0}}'
    )
    delete = (
        '{"op": "edit", "id": 2, "edit": {"kind": "delete_node", "id": "n2"}}'
    )
    with (
        running(server.asgi_app(app)) as port,
        connect(f"ws://{server.HOST}:{port}/ws") as websocket,
    ):
        websocket.recv()
        for request in (move, delete):
            websocket.send(request)
            Ack.model_validate_json(websocket.recv(timeout=5))
    assert [(n.id, n.x, n.y) for n in app.graph.nodes] == [("n1", 9.5, 8.0)]


def test_a_value_set_in_the_browser_is_applied_and_pushed(
    web_dir: Path,
) -> None:
    """Typing into an input: ack to the sender, snapshot to the other page."""
    assert web_dir.exists()
    app = App([add])
    app.add_node(add, x=0.0, y=0.0)
    request = (
        '{"op": "edit", "id": 1, "edit": {"kind": "set_value",'
        ' "node": "n1", "port": "a", "value": 4.5}}'
    )
    with running(server.asgi_app(app)) as port:
        url = f"ws://{server.HOST}:{port}/ws"
        with connect(url) as sender, connect(url) as watcher:
            sender.recv()
            watcher.recv()
            sender.send(request)
            Ack.model_validate_json(sender.recv(timeout=5))
            pushed = Snapshot.model_validate_json(watcher.recv(timeout=5))
    assert app.graph.nodes[0].values == {"a": 4.5}
    assert pushed.graph.nodes[0].values == {"a": 4.5}


MAX_LATENCY = 0.1


def read_until(
    websocket: ClientConnection, wanted: Callable[[dict[str, object]], bool]
) -> dict[str, object]:
    """Read messages until one satisfies `wanted`.

    Returns
    -------
    dict[str, object]
        That message, parsed.
    """
    for _ in range(1000):
        message: dict[str, object] = json.loads(websocket.recv(timeout=WAIT))
        if wanted(message):
            return message
    pytest.fail("the message never came")  # pragma: no cover  # only on failure


def test_a_start_from_the_page_runs_the_job_and_pushes_updates(
    web_dir: Path,
) -> None:
    """The page sends start; the updates and the run after the job come back."""
    assert web_dir.exists()
    app = App([Steps, show])
    app.add_node(Steps, x=0.0, y=0.0)
    app.add_node(show, x=100.0, y=0.0)
    app.add_link("n1", "total", "n2", "value")
    with (
        running(server.asgi_app(app)) as port,
        connect(f"ws://{server.HOST}:{port}/ws") as websocket,
    ):
        websocket.recv()
        websocket.send(json.dumps({"op": "start", "node": "n1"}))
        update = read_until(websocket, lambda m: m["op"] == "job_update")
        assert update["state"] == "running"
        done = read_until(
            websocket,
            lambda m: m["op"] == "job_update" and m["state"] == "finished",
        )
        result = RunResult.model_validate(
            read_until(websocket, lambda m: m["op"] == "run_result")
        )
    assert JobUpdate.model_validate(done).step == done["steps"]
    assert result.partial
    assert [o.node for o in result.outcomes] == ["n2"]


def test_a_report_reaches_the_page_within_a_tenth_of_a_second(
    web_dir: Path,
) -> None:
    """From the job's `report` call to the page's receive: 100 ms at most."""
    assert web_dir.exists()
    app = App([Timed])
    app.add_node(Timed, x=0.0, y=0.0)
    with (
        running(server.asgi_app(app)) as port,
        connect(f"ws://{server.HOST}:{port}/ws") as websocket,
    ):
        websocket.recv()
        websocket.send(json.dumps({"op": "start", "node": "n1"}))
        read_until(
            websocket,
            lambda m: m["op"] == "job_update" and bool(m["outputs"]),
        )
        arrived = time.monotonic()
    assert 0 < arrived - Stamp.at < MAX_LATENCY


def test_a_refused_start_or_stop_goes_back_to_the_asking_page_only(
    web_dir: Path,
) -> None:
    """The reason comes in a `job_refused` message; nobody else hears it."""
    assert web_dir.exists()
    app = App([Steps, show])
    app.add_node(Steps, x=0.0, y=0.0)
    app.add_node(show, x=100.0, y=0.0)
    with running(server.asgi_app(app)) as port:
        url = f"ws://{server.HOST}:{port}/ws"
        with connect(url) as asking, connect(url) as other:
            asking.recv()
            other.recv()
            asking.send(json.dumps({"op": "start", "node": "n2"}))
            first = JobRefusal.model_validate_json(asking.recv(timeout=WAIT))
            asking.send(json.dumps({"op": "stop", "node": "n1"}))
            second = JobRefusal.model_validate_json(asking.recv(timeout=WAIT))
            with pytest.raises(TimeoutError):
                other.recv(timeout=SILENCE)
    assert "not a job" in first.reason
    assert "no running job" in second.reason


def test_a_stop_from_the_page_ends_the_job_and_a_new_page_sees_it(
    web_dir: Path,
) -> None:
    """Stop moves the job to stopping then stopped; a reload shows its state."""
    assert web_dir.exists()
    app = App([Steps])
    app.add_node(Steps, x=0.0, y=0.0)
    Pause.reset()
    Steps.pause_at = 1
    try:
        with running(server.asgi_app(app)) as port:
            url = f"ws://{server.HOST}:{port}/ws"
            with connect(url) as websocket:
                websocket.recv()
                websocket.send(json.dumps({"op": "start", "node": "n1"}))
                assert Pause.entered.wait(WAIT)
                websocket.send(json.dumps({"op": "stop", "node": "n1"}))
                read_until(
                    websocket,
                    lambda m: (
                        m["op"] == "job_update" and m["state"] == "stopping"
                    ),
                )
                with connect(url) as reloaded:
                    snapshot = Snapshot.model_validate_json(reloaded.recv())
                Pause.release.set()
                read_until(
                    websocket,
                    lambda m: (
                        m["op"] == "job_update" and m["state"] == "stopped"
                    ),
                )
    finally:
        Steps.pause_at = -1
    assert [(j.node, j.state) for j in snapshot.jobs] == [("n1", "stopping")]
