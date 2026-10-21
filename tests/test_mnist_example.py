"""The MNIST example: its data reader, its downloads, its network and nodes."""

import gzip
import hashlib
import struct
from pathlib import Path
from types import ModuleType

import numpy as np
import pytest
from numpy.typing import NDArray
from test_examples import load

PICTURES = 200
CLASSES = 10
SEED = 51
HALF = 0.5
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


def make_set(
    count: int, rng: np.random.Generator
) -> tuple[NDArray[np.uint8], NDArray[np.uint8]]:
    """Make pictures that one lit pixel tells apart: pixel k means class k.

    Returns
    -------
    tuple[NDArray[np.uint8], NDArray[np.uint8]]
        The pictures and their labels.
    """
    labels = rng.integers(0, CLASSES, count).astype(np.uint8)
    images = np.zeros((count, CLASSES), dtype=np.uint8)
    images[np.arange(count), labels] = 255
    return images, labels


def test_the_gradients_match_finite_differences(mnist: ModuleType) -> None:
    """Every array's analytic gradient agrees with a numeric one."""
    rng = np.random.default_rng(SEED)
    weights = mnist.init_weights(5, rng, inputs=6)
    x = rng.random((12, 6))
    labels = rng.integers(0, CLASSES, 12).astype(np.uint8)
    _, grads = mnist.loss_and_gradients(weights, x, labels)
    step = 1e-6
    for name in ("w1", "b1", "w2", "b2"):
        array = getattr(weights, name)
        analytic = getattr(grads, name)
        for position in [(0,) * array.ndim, tuple(s - 1 for s in array.shape)]:
            kept = array[position]
            array[position] = kept + step
            up = mnist.loss_and_gradients(weights, x, labels)[0]
            array[position] = kept - step
            down = mnist.loss_and_gradients(weights, x, labels)[0]
            array[position] = kept
            numeric = (up - down) / (2 * step)
            assert analytic[position] == pytest.approx(
                numeric, rel=1e-5, abs=1e-8
            )


def test_the_same_seed_gives_the_same_weights(mnist: ModuleType) -> None:
    """Two networks made from one seed are equal, and another seed differs."""
    first = mnist.init_weights(8, np.random.default_rng(SEED))
    second = mnist.init_weights(8, np.random.default_rng(SEED))
    other = mnist.init_weights(8, np.random.default_rng(SEED + 1))
    assert np.array_equal(first.w1, second.w1)
    assert not np.array_equal(first.w1, other.w1)
    assert first.w1.shape == (784, 8)
    assert first.w2.shape == (8, CLASSES)


def test_probabilities_sum_to_one_for_every_picture(mnist: ModuleType) -> None:
    """Each softmax row is a probability and the ReLU layer is not negative."""
    rng = np.random.default_rng(SEED)
    weights = mnist.init_weights(6, rng, inputs=10)
    hidden, probs = mnist.forward(weights, rng.random((7, 10)))
    assert probs.shape == (7, CLASSES)
    assert probs.sum(axis=1) == pytest.approx(np.ones(7))
    assert hidden.min() >= 0.0


def test_a_network_learns_a_set_one_pixel_tells_apart(
    mnist: ModuleType,
) -> None:
    """Plain steps drive the loss down and the accuracy to 100%."""
    rng = np.random.default_rng(SEED)
    images, labels = make_set(400, rng)
    weights = mnist.init_weights(8, rng, inputs=CLASSES)
    x = mnist.to_floats(images)
    first = mnist.loss_and_gradients(weights, x[:32], labels[:32])[0]
    for step in range(300):
        rows = slice((step * 32) % 368, (step * 32) % 368 + 32)
        _, grads = mnist.loss_and_gradients(weights, x[rows], labels[rows])
        mnist.apply_gradients(weights, grads, 0.5)
    last = mnist.loss_and_gradients(weights, x[:32], labels[:32])[0]
    assert last < first / 4
    assert mnist.accuracy(weights, images, labels) == pytest.approx(1.0)


def test_a_network_with_no_weights_always_says_class_zero(
    mnist: ModuleType,
) -> None:
    """Equal scores pick class 0, so accuracy is the share of zeros."""
    weights = mnist.Weights(
        w1=np.zeros((CLASSES, 3)),
        b1=np.zeros(3),
        w2=np.zeros((3, CLASSES)),
        b2=np.zeros(CLASSES),
    )
    images = np.zeros((4, CLASSES), dtype=np.uint8)
    labels = np.array([0, 0, 3, 7], dtype=np.uint8)
    assert mnist.accuracy(weights, images, labels) == pytest.approx(HALF)
