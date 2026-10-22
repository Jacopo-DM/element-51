"""Tests for the messages between Python and the browser."""

import json
import math
from pathlib import Path

import pytest
from hypothesis import example, given
from hypothesis import strategies as st
from pydantic import ValidationError

from element_51.edits import (
    AddLink,
    AddNode,
    DeleteLink,
    DeleteNode,
    MoveNode,
    SetValue,
)
from element_51.engine import RunResult
from element_51.model import Graph, PortRef
from element_51.protocol import (
    Ack,
    ClientMessage,
    EditRequest,
    JobRefusal,
    JobUpdate,
    JobView,
    PngChartRequest,
    PngPictureRequest,
    PngReply,
    Reject,
    RunRequest,
    Snapshot,
    SourceReply,
    SourceRequest,
    StartRequest,
    StopRequest,
)
from element_51.visuals import Image, Series

FIXTURE = Path(__file__).parent / "fixtures" / "messages.json"
MESSAGES = json.loads(FIXTURE.read_text())


def round_trip(
    model: type[
        EditRequest
        | Ack
        | Reject
        | RunRequest
        | RunResult
        | SourceRequest
        | SourceReply
        | StartRequest
        | StopRequest
        | JobUpdate
        | JobRefusal
    ],
    key: str,
) -> object:
    """Parse the fixture message `key` and check it dumps back unchanged.

    Returns
    -------
    object
        The parsed message.
    """
    parsed = model.model_validate(MESSAGES[key])
    assert json.loads(parsed.model_dump_json()) == MESSAGES[key]
    return parsed


def test_add_node_request_round_trips() -> None:
    """The browser's add_node request is the shape Rust writes."""
    request = round_trip(EditRequest, "edit_add_node")
    assert isinstance(request, EditRequest)
    expected_id = 17
    assert request.id == expected_id
    assert isinstance(request.edit, AddNode)


def test_add_link_request_round_trips() -> None:
    """The browser's add_link request is the shape Rust writes."""
    request = round_trip(EditRequest, "edit_add_link")
    assert isinstance(request, EditRequest)
    assert isinstance(request.edit, AddLink)


@pytest.mark.parametrize(
    ("key", "kind"),
    [
        ("edit_move_node", MoveNode),
        ("edit_delete_node", DeleteNode),
        ("edit_delete_link", DeleteLink),
        ("edit_set_value", SetValue),
        ("edit_clear_value", SetValue),
    ],
)
def test_move_and_delete_requests_round_trip(
    key: str, kind: type[MoveNode | DeleteNode | DeleteLink | SetValue]
) -> None:
    """The browser's move and delete requests are the shape Rust writes."""
    request = round_trip(EditRequest, key)
    assert isinstance(request, EditRequest)
    assert isinstance(request.edit, kind)


def test_ack_and_reject_round_trip() -> None:
    """The replies Python writes are the shape Rust reads."""
    assert isinstance(round_trip(Ack, "ack"), Ack)
    reject = round_trip(Reject, "reject")
    assert isinstance(reject, Reject)
    assert reject.reason == "cannot link float to str"


def test_an_unknown_edit_kind_is_refused() -> None:
    """A typo in an edit is an error, not a silent no-op."""
    broken = {**MESSAGES["edit_add_node"], "edit": {"kind": "explode"}}
    with pytest.raises(ValidationError):
        EditRequest.model_validate(broken)


def test_a_request_must_say_it_is_an_edit() -> None:
    """The `op` field is checked."""
    with pytest.raises(ValidationError):
        EditRequest.model_validate({**MESSAGES["ack"], "edit": {}})


def test_run_request_and_result_round_trip() -> None:
    """The run request and its result are the shapes Rust reads and writes."""
    assert isinstance(round_trip(RunRequest, "run"), RunRequest)
    result = round_trip(RunResult, "run_result")
    assert isinstance(result, RunResult)
    assert [o.status for o in result.outcomes] == [
        "ok",
        "error",
        "skipped",
        "ok",
    ]


def test_a_result_with_a_picture_round_trips() -> None:
    """A picture in `shown` is a list of pixels on the wire."""
    result = round_trip(RunResult, "run_result_picture")
    assert isinstance(result, RunResult)
    picture = result.outcomes[0].shown["out"]
    assert isinstance(picture, Image)
    assert picture.pixels == bytes([0, 85, 170, 255])


def test_a_result_with_a_chart_round_trips() -> None:
    """A series with a gap is a list with a null on the wire."""
    result = round_trip(RunResult, "run_result_chart")
    assert isinstance(result, RunResult)
    series = result.outcomes[0].shown["series_1"]
    assert isinstance(series, Series)
    assert math.isnan(series.values[2])


def test_a_source_request_and_its_replies_round_trip() -> None:
    """Asking for a node's source, and both ways the answer can go."""
    request = round_trip(SourceRequest, "source")
    assert isinstance(request, SourceRequest)
    assert request.node == "n2"
    reply = round_trip(SourceReply, "source_reply")
    assert isinstance(reply, SourceReply)
    assert reply.source is not None
    assert reply.source.startswith("@node\ndef add")
    assert reply.reason is None
    unreadable = round_trip(SourceReply, "source_reply_unreadable")
    assert isinstance(unreadable, SourceReply)
    assert unreadable.source is None
    assert unreadable.reason is not None
    assert "cannot be read" in unreadable.reason


def test_client_messages_are_told_apart_by_op() -> None:
    """The server parses one union: an edit, a run or a source request."""
    edit = ClientMessage.validate_python(MESSAGES["edit_add_node"])
    run = ClientMessage.validate_python(MESSAGES["run"])
    assert isinstance(edit, EditRequest)
    assert isinstance(run, RunRequest)
    source = ClientMessage.validate_python(MESSAGES["source"])
    assert isinstance(source, SourceRequest)
    start = ClientMessage.validate_python(MESSAGES["start"])
    stop = ClientMessage.validate_python(MESSAGES["stop"])
    assert isinstance(start, StartRequest)
    assert isinstance(stop, StopRequest)
    with pytest.raises(ValidationError):
        ClientMessage.validate_python(MESSAGES["ack"])


def test_job_messages_round_trip() -> None:
    """The start and stop requests, a job update and a refusal match Rust."""
    start = round_trip(StartRequest, "start")
    stop = round_trip(StopRequest, "stop")
    assert isinstance(start, StartRequest)
    assert isinstance(stop, StopRequest)
    update = round_trip(JobUpdate, "job_update")
    assert isinstance(update, JobUpdate)
    assert update.state == "running"
    assert (update.step, update.steps) == (2, 5)
    assert update.linked[0].node == "n3"
    assert math.isnan(update.linked[0].series.values[1])
    failed = round_trip(JobUpdate, "job_update_failed")
    assert isinstance(failed, JobUpdate)
    assert failed.steps is None
    assert failed.error == "RuntimeError: boom"
    refused = round_trip(JobRefusal, "job_refused")
    assert isinstance(refused, JobRefusal)
    assert refused.reason == "Node n1 is already running"


def test_a_job_in_a_snapshot_has_no_op() -> None:
    """The `op` tags a message; a job listed in a snapshot is the bare view."""
    bare = {k: v for k, v in MESSAGES["job_update"].items() if k != "op"}
    snapshot = Snapshot(
        theme="antimony",
        node_types=(),
        graph=Graph(nodes=(), links=()),
        jobs=(JobView.model_validate(bare),),
    )
    dumped = json.loads(snapshot.model_dump_json())
    assert "op" not in dumped["jobs"][0]
    assert dumped["jobs"][0]["node"] == "n1"
    assert Snapshot.model_validate(dumped) == snapshot


def test_a_partial_run_result_says_so() -> None:
    """A result with `partial` set covers some nodes; the others stay."""
    partial = round_trip(RunResult, "run_result_partial")
    assert isinstance(partial, RunResult)
    assert partial.partial
    whole = round_trip(RunResult, "run_result")
    assert isinstance(whole, RunResult)
    assert not whole.partial


@given(st.text())
@example(json.dumps(MESSAGES["edit_add_node"]))
@example(json.dumps(MESSAGES["run"]))
@example(json.dumps(MESSAGES["source"]))
@example(json.dumps(MESSAGES["start"]))
@example(json.dumps(MESSAGES["stop"]))
def test_client_message_boundary(raw: str) -> None:
    """Any text from the page is a client message or a ValidationError."""
    try:
        message = ClientMessage.validate_json(raw)
    except ValidationError:
        return
    assert isinstance(
        message,
        EditRequest | RunRequest | SourceRequest | StartRequest | StopRequest,
    )


NODE_IDS = st.from_regex(r"n[0-9]+", fullmatch=True)
COORDINATES = st.floats(allow_nan=False, allow_infinity=False)
PORT_REFS = st.builds(PortRef, node=NODE_IDS, port=st.text())
EDITS = st.one_of(
    st.builds(
        AddNode,
        id=NODE_IDS,
        node_type=st.text(),
        x=COORDINATES,
        y=COORDINATES,
    ),
    st.builds(AddLink, source=PORT_REFS, target=PORT_REFS),
    st.builds(MoveNode, id=NODE_IDS, x=COORDINATES, y=COORDINATES),
    st.builds(DeleteNode, id=NODE_IDS),
    st.builds(DeleteLink, source=PORT_REFS, target=PORT_REFS),
    st.builds(
        SetValue,
        node=NODE_IDS,
        port=st.text(),
        value=st.one_of(
            st.none(),
            COORDINATES,
            st.integers(),
            st.text(),
            st.booleans(),
        ),
    ),
)
MESSAGES_FROM_THE_PAGE = st.one_of(
    st.builds(EditRequest, id=st.integers(min_value=0), edit=EDITS),
    st.builds(RunRequest),
    st.builds(SourceRequest, node=st.text()),
)


@given(MESSAGES_FROM_THE_PAGE)
def test_client_message_round_trip(
    message: EditRequest | RunRequest | SourceRequest,
) -> None:
    """What the page's message dumps to reads back as the same message."""
    assert ClientMessage.validate_json(message.model_dump_json()) == message


@pytest.mark.parametrize(
    ("model", "key"),
    [
        (PngChartRequest, "png_chart"),
        (PngPictureRequest, "png_picture"),
        (PngReply, "png_reply"),
        (PngReply, "png_reply_refused"),
    ],
)
def test_png_messages_round_trip(
    model: type[PngChartRequest | PngPictureRequest | PngReply], key: str
) -> None:
    """The PNG requests and replies are the shape Rust writes."""
    parsed = model.model_validate(MESSAGES[key])
    assert json.loads(parsed.model_dump_json()) == MESSAGES[key]


def test_a_png_request_is_a_client_message() -> None:
    """The server reads both PNG requests, and refuses a reply from a page."""
    chart = ClientMessage.validate_python(MESSAGES["png_chart"])
    picture = ClientMessage.validate_python(MESSAGES["png_picture"])
    assert isinstance(chart, PngChartRequest)
    assert isinstance(picture, PngPictureRequest)
    with pytest.raises(ValidationError):
        ClientMessage.validate_python(MESSAGES["png_reply"])


def test_a_chart_line_needs_a_lower_case_rrggbb_colour() -> None:
    """The colour is checked when the request is read."""
    bad = json.loads(json.dumps(MESSAGES["png_chart"]))
    bad["lines"][0]["colour"] = "orange"
    with pytest.raises(ValidationError):
        PngChartRequest.model_validate(bad)
