"""Tests for the graph records."""

import json
from pathlib import Path

import pytest
from pydantic import ValidationError

from element_51.model import Graph, Node, NodeType, Port, PortType
from element_51.protocol import Snapshot

FIXTURE = Path(__file__).parent / "fixtures" / "snapshot.json"


def test_shared_fixture_parses_into_a_snapshot() -> None:
    """The file the Rust side also parses is valid on the Python side."""
    snapshot = Snapshot.model_validate_json(FIXTURE.read_text())
    assert [t.name for t in snapshot.node_types] == ["add", "show"]
    assert snapshot.graph.links[0].source.node == "n1"


def test_snapshot_serialises_back_to_the_same_json() -> None:
    """A parse and a dump change nothing, so both sides agree on the shape."""
    raw = FIXTURE.read_text()
    snapshot = Snapshot.model_validate_json(raw)
    assert json.loads(snapshot.model_dump_json()) == json.loads(raw)


def test_records_are_frozen() -> None:
    """A record cannot be changed after it is built."""
    port = Port(name="a", kind=PortType.FLOAT)
    with pytest.raises(ValidationError):
        port.name = "b"  # type: ignore[misc]  # ty: ignore[invalid-assignment]  # frozen on purpose


def test_unknown_fields_are_refused() -> None:
    """A typo in a message is an error, not a silent drop."""
    with pytest.raises(ValidationError):
        Node.model_validate({
            "id": "n1",
            "node_type": "add",
            "x": 0,
            "y": 0,
            "z": 1,
        })


def test_unknown_port_kind_is_refused() -> None:
    """Only float, int, str and bool exist."""
    with pytest.raises(ValidationError):
        Port.model_validate({"name": "a", "kind": "complex"})


def test_next_id_follows_the_highest_existing_id() -> None:
    """Ids are not reused after a gap."""
    graph = Graph(
        nodes=(
            Node(id="n1", node_type="add", x=0, y=0),
            Node(id="n7", node_type="add", x=0, y=0),
        ),
        links=(),
    )
    assert graph.next_id() == "n8"
    assert Graph(nodes=(), links=()).next_id() == "n1"


def test_node_type_holds_ordered_ports() -> None:
    """Input order is the order the browser draws the rows."""
    node_type = NodeType(
        name="add",
        inputs=(
            Port(name="a", kind=PortType.FLOAT),
            Port(name="b", kind=PortType.INT),
        ),
        outputs=(Port(name="out", kind=PortType.FLOAT),),
    )
    assert [p.name for p in node_type.inputs] == ["a", "b"]
