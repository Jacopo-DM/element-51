"""Tests for the messages between Python and the browser."""

import json
import math
from pathlib import Path

import pytest
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
from element_51.protocol import (
    Ack,
    ClientMessage,
    EditRequest,
    Reject,
    RunRequest,
)
from element_51.visuals import Image, Series

FIXTURE = Path(__file__).parent / "fixtures" / "messages.json"
MESSAGES = json.loads(FIXTURE.read_text())


def round_trip(
    model: type[EditRequest | Ack | Reject | RunRequest | RunResult], key: str
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


def test_client_messages_are_told_apart_by_op() -> None:
    """The server parses one union: an edit or a run request."""
    edit = ClientMessage.validate_python(MESSAGES["edit_add_node"])
    run = ClientMessage.validate_python(MESSAGES["run"])
    assert isinstance(edit, EditRequest)
    assert isinstance(run, RunRequest)
    with pytest.raises(ValidationError):
        ClientMessage.validate_python(MESSAGES["ack"])
