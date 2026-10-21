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
from element_51.model import PortRef
from element_51.protocol import (
    Ack,
    ClientMessage,
    EditRequest,
    Reject,
    RunRequest,
    SourceReply,
    SourceRequest,
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
    with pytest.raises(ValidationError):
        ClientMessage.validate_python(MESSAGES["ack"])


@given(st.text())
@example(json.dumps(MESSAGES["edit_add_node"]))
@example(json.dumps(MESSAGES["run"]))
@example(json.dumps(MESSAGES["source"]))
def test_client_message_boundary(raw: str) -> None:
    """Any text from the page is a client message or a ValidationError."""
    try:
        message = ClientMessage.validate_json(raw)
    except ValidationError:
        return
    assert isinstance(message, EditRequest | RunRequest | SourceRequest)


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
