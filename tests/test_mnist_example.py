"""The MNIST example: its data reader, its downloads, its network and nodes."""

import gzip
import hashlib
import struct
from pathlib import Path
from types import ModuleType

import numpy as np
import pytest
from test_examples import load

PICTURES = 200
PARTS = 4
SIDE = 28


@pytest.fixture(scope="module")
def mnist() -> ModuleType:
    """Load the example as a module.

    Returns
    -------
    ModuleType
        `g3_mnist.py`.
    """
    return load("g_experiments/g3_mnist.py")


def write_images(path: Path, count: int = PICTURES) -> None:
    """Write `count` pictures as a gzipped IDX file, pixels counting up."""
    body = bytes(i % 256 for i in range(count * SIDE * SIDE))
    head = struct.pack(">IIII", 0x0803, count, SIDE, SIDE)
    path.write_bytes(gzip.compress(head + body))


def write_labels(path: Path, count: int = PICTURES) -> None:
    """Write `count` labels as a gzipped IDX file, 0 to 9 round and round."""
    head = struct.pack(">II", 0x0801, count)
    path.write_bytes(gzip.compress(head + bytes(i % 10 for i in range(count))))


def test_images_are_read_one_row_per_picture(
    mnist: ModuleType, tmp_path: Path
) -> None:
    """The header says how many pictures; each row holds 784 pixels."""
    write_images(tmp_path / "i.gz")
    images = mnist.load_images(tmp_path / "i.gz")
    assert images.shape == (PICTURES, SIDE * SIDE)
    assert images[1, 0] == (SIDE * SIDE) % 256


def test_labels_are_read_one_per_picture(
    mnist: ModuleType, tmp_path: Path
) -> None:
    """The labels come back as a flat array of 0 to 9."""
    write_labels(tmp_path / "l.gz")
    labels = mnist.load_labels(tmp_path / "l.gz")
    assert labels.shape == (PICTURES,)
    assert list(labels[:12]) == [0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 0, 1]


def test_a_file_that_is_not_gzip_is_refused(
    mnist: ModuleType, tmp_path: Path
) -> None:
    """Plain bytes are named as not gzip."""
    (tmp_path / "x.gz").write_bytes(b"not gzip at all")
    with pytest.raises(ValueError, match="not a gzip file"):
        mnist.load_labels(tmp_path / "x.gz")


def test_a_file_too_short_for_a_header_is_refused(
    mnist: ModuleType, tmp_path: Path
) -> None:
    """Two bytes cannot hold an IDX header."""
    (tmp_path / "x.gz").write_bytes(gzip.compress(b"ab"))
    with pytest.raises(ValueError, match="too short"):
        mnist.load_images(tmp_path / "x.gz")


def test_labels_are_not_accepted_as_images(
    mnist: ModuleType, tmp_path: Path
) -> None:
    """The magic number tells the two kinds apart."""
    write_labels(tmp_path / "l.gz")
    with pytest.raises(ValueError, match="magic number"):
        mnist.load_images(tmp_path / "l.gz")


def test_a_header_that_promises_more_bytes_is_refused(
    mnist: ModuleType, tmp_path: Path
) -> None:
    """A count that the data cannot fill is named with both numbers."""
    head = struct.pack(">II", 0x0801, 50)
    (tmp_path / "l.gz").write_bytes(gzip.compress(head + b"123"))
    with pytest.raises(ValueError, match="3 bytes, its header says 50"):
        mnist.load_labels(tmp_path / "l.gz")


def test_a_zero_count_is_refused(mnist: ModuleType, tmp_path: Path) -> None:
    """The header record wants at least one item."""
    head = struct.pack(">II", 0x0801, 0)
    (tmp_path / "l.gz").write_bytes(gzip.compress(head))
    with pytest.raises(ValueError, match="count"):
        mnist.load_labels(tmp_path / "l.gz")


def fake_dataset(mnist: ModuleType, payloads: dict[str, bytes]) -> object:
    """Build a dataset whose parts are `payloads`, with their real sha256.

    Returns
    -------
    object
        The dataset record.
    """
    parts = tuple(
        mnist.Part(
            name=name,
            size=len(data),
            sha256=hashlib.sha256(data).hexdigest(),
        )
        for name, data in payloads.items()
    )
    return mnist.Dataset(
        name="fake", folder="fake", base="https://example.test/d", parts=parts
    )


def test_fetch_downloads_checks_and_keeps_each_missing_file(
    mnist: ModuleType, tmp_path: Path
) -> None:
    """Each part is written as `.part`, checked, renamed; none is left over."""
    payloads = {"a.gz": b"first", "b.gz": b"second"}
    asked: list[str] = []

    def download(url: str, path: Path) -> None:
        asked.append(url)
        path.write_bytes(payloads[url.rsplit("/", 1)[1]])

    dataset = fake_dataset(mnist, payloads)
    fetched = mnist.fetch(dataset, tmp_path, download)
    assert [part.name for part in fetched] == ["a.gz", "b.gz"]
    assert asked == [
        "https://example.test/d/a.gz",
        "https://example.test/d/b.gz",
    ]
    assert sorted(p.name for p in (tmp_path / "fake").iterdir()) == [
        "a.gz",
        "b.gz",
    ]


def test_fetch_leaves_a_file_that_is_already_there(
    mnist: ModuleType, tmp_path: Path
) -> None:
    """Nothing is downloaded for a file that exists."""
    dataset = fake_dataset(mnist, {"a.gz": b"first"})
    (tmp_path / "fake").mkdir()
    (tmp_path / "fake" / "a.gz").write_bytes(b"first")

    def download(
        url: str, path: Path
    ) -> None:  # pragma: no cover  # must not run
        raise AssertionError(url, path)

    assert mnist.fetch(dataset, tmp_path, download) == []


def test_fetch_deletes_a_download_that_is_not_the_pinned_file(
    mnist: ModuleType, tmp_path: Path
) -> None:
    """A wrong file is refused and neither it nor its `.part` stays."""
    dataset = fake_dataset(mnist, {"a.gz": b"first"})

    def download(url: str, path: Path) -> None:
        del url
        path.write_bytes(b"tampered")

    with pytest.raises(ValueError, match="not the pinned file"):
        mnist.fetch(dataset, tmp_path, download)
    assert list((tmp_path / "fake").iterdir()) == []


def test_fetch_refuses_a_part_with_no_pinned_hash(
    mnist: ModuleType, tmp_path: Path
) -> None:
    """An empty sha256 means nothing may be downloaded for it."""
    dataset = mnist.Dataset(
        name="fake",
        folder="fake",
        base="https://example.test/d",
        parts=(mnist.Part(name="a.gz", size=1, sha256=""),),
    )

    def download(
        url: str, path: Path
    ) -> None:  # pragma: no cover  # must not run
        raise AssertionError(url, path)

    with pytest.raises(ValueError, match="no sha256 is pinned"):
        mnist.fetch(dataset, tmp_path, download)


def test_download_file_refuses_an_address_that_is_not_https(
    mnist: ModuleType, tmp_path: Path
) -> None:
    """Plain http and file addresses are never opened."""
    with pytest.raises(ValueError, match="only https"):
        mnist.download_file("http://example.test/x", tmp_path / "x")


def test_the_table_names_both_datasets_with_four_parts_each(
    mnist: ModuleType,
) -> None:
    """MNIST and Fashion-MNIST each list the same four file names."""
    assert sorted(mnist.DATASETS) == ["fashion", "mnist"]
    names = [
        [part.name for part in dataset.parts]
        for dataset in mnist.DATASETS.values()
    ]
    assert names[0] == names[1]
    assert len(names[0]) == len(np.unique(names[0])) == PARTS
