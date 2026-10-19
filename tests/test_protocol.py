"""Tests for the messages between Python and the browser."""

import json
from pathlib import Path

import pytest
from pydantic import ValidationError

from element_51.edits import AddLink, AddNode
from element_51.protocol import Ack, EditRequest, Reject

FIXTURE = Path(__file__).parent / "fixtures" / "messages.json"
MESSAGES = json.loads(FIXTURE.read_text())


def round_trip(model: type[EditRequest | Ack | Reject], key: str) -> object:
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
