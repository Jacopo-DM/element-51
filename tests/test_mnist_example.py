"""The MNIST example: its data reader, its downloads, its network and nodes."""

import gzip
import hashlib
import struct
import threading
from pathlib import Path
from types import ModuleType

import numpy as np
import pytest
from numpy.typing import NDArray
from test_examples import load

from element_51.edits import SetValue
from element_51.protocol import JobUpdate
from element_51.visuals import Series

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


TRAIN = 200
TEST = 100
FASHION_TRAIN = 300
FASHION_TEST = 150
WAIT = 10.0
INDEX = 3
EPOCHS = 3
BATCHES = 12
FASHION_BATCHES = 15
POINTS = 6
STEPS = 3


@pytest.fixture
def data(tmp_path: Path, monkeypatch: pytest.MonkeyPatch) -> Path:
    """Write fake MNIST and Fashion-MNIST files under `__data__` and go there.

    Returns
    -------
    Path
        The `__data__` folder.
    """
    monkeypatch.chdir(tmp_path)
    root = tmp_path / "__data__"
    for folder, train, test in (
        ("mnist", TRAIN, TEST),
        ("fashion", FASHION_TRAIN, FASHION_TEST),
    ):
        (root / folder).mkdir(parents=True)
        write_images(root / folder / "train-images-idx3-ubyte.gz", train)
        write_labels(root / folder / "train-labels-idx1-ubyte.gz", train)
        write_images(root / folder / "t10k-images-idx3-ubyte.gz", test)
        write_labels(root / folder / "t10k-labels-idx1-ubyte.gz", test)
    return root


def test_a_sample_is_a_picture_a_label_and_its_dataset(
    mnist: ModuleType, data: Path
) -> None:
    """Index 3 is the fourth picture, with label 3."""
    del data
    sample = mnist.MnistSample(index=INDEX).run()
    assert (sample.image.width, sample.image.height) == (SIDE, SIDE)
    assert sample.image.channels == 1
    assert sample.image.pixels[0] == (INDEX * SIDE * SIDE) % 256
    assert sample.label == INDEX
    assert sample.dataset == "mnist"


def test_a_sample_can_come_from_the_fashion_folder(
    mnist: ModuleType, data: Path
) -> None:
    """The fashion folder holds 300 pictures, so index 250 exists there only."""
    del data
    assert mnist.MnistSample(index=250, dataset="fashion").run().label == 0
    with pytest.raises(ValueError, match="from 0 to 199, not 250"):
        mnist.MnistSample(index=250).run()


def test_a_sample_refuses_a_dataset_word_it_does_not_know(
    mnist: ModuleType, data: Path
) -> None:
    """The error names both words."""
    del data
    with pytest.raises(ValueError, match="mnist or fashion, not 'mnst'"):
        mnist.MnistSample(dataset="mnst").run()


def test_missing_data_says_how_to_download_it(
    mnist: ModuleType, tmp_path: Path, monkeypatch: pytest.MonkeyPatch
) -> None:
    """With no folder, the error names the command that fetches the files."""
    monkeypatch.chdir(tmp_path)
    with pytest.raises(ValueError, match=r"g3_mnist\.py` once to download"):
        mnist.MnistSample().run()


def test_the_graph_runs_its_sample_and_leaves_the_job_for_start(
    mnist: ModuleType, data: Path
) -> None:
    """Run gives the sample's outputs; the job and the plot wait for Start."""
    del data
    result = mnist.build_app().execute()
    by_id = {outcome.node: outcome for outcome in result.outcomes}
    assert by_id["n1"].status == "ok"
    assert by_id["n1"].outputs == {"label": 0, "dataset": "mnist"}
    assert by_id["n2"].status == by_id["n3"].status == "skipped"


def test_training_steps_cover_every_batch_once(
    mnist: ModuleType, data: Path
) -> None:
    """200 pictures make 4 batches an epoch; 3 epochs make 12 batches."""
    del data
    train = mnist.Train(epochs=EPOCHS)
    assert train.batches() == BATCHES
    assert train.steps() == 1
    assert (
        mnist.Train(epochs=EPOCHS, dataset="fashion").batches()
        == FASHION_BATCHES
    )


def test_a_step_makes_one_point_of_each_curve(
    mnist: ModuleType, data: Path, monkeypatch: pytest.MonkeyPatch
) -> None:
    """Five batches of 100-batch steps become 3 steps; the last is shorter."""
    del data
    monkeypatch.setattr(mnist, "STEP_BATCHES", 5)
    train = mnist.Train(epochs=EPOCHS, hidden=4)
    state = train.begin()
    assert train.steps() == STEPS
    points = [train.step(state, i) for i in range(STEPS)]
    for curves in points:
        assert len(curves.loss.values) == len(curves.accuracy.values) == 1
        assert 0.0 <= curves.accuracy.values[0] <= 1.0


def test_the_last_step_tests_every_test_picture(
    mnist: ModuleType, data: Path, monkeypatch: pytest.MonkeyPatch
) -> None:
    """Early steps test QUICK_TEST pictures; the last step tests them all."""
    del data
    monkeypatch.setattr(mnist, "STEP_BATCHES", 6)
    monkeypatch.setattr(mnist, "QUICK_TEST", 10)
    train = mnist.Train(epochs=3, hidden=4)
    state = train.begin()
    early = train.step(state, 0).accuracy.values[0]
    last = train.step(state, 1).accuracy.values[0]
    assert early * 10 == pytest.approx(round(early * 10))
    assert last * TEST == pytest.approx(round(last * TEST))


def test_training_is_the_same_twice(
    mnist: ModuleType, data: Path, monkeypatch: pytest.MonkeyPatch
) -> None:
    """Two runs from the same seed draw the same curves."""
    del data
    monkeypatch.setattr(mnist, "STEP_BATCHES", 4)

    def curves() -> list[float]:
        train = mnist.Train(epochs=2, hidden=4)
        state = train.begin()
        return [
            train.step(state, i).loss.values[0] for i in range(train.steps())
        ]

    assert curves() == curves()


@pytest.mark.parametrize(
    ("field", "value", "method", "message"),
    [
        ("hidden", 0, "begin", "hidden must be at least 1"),
        ("learning_rate", 0.0, "begin", "learning_rate must be above 0"),
        ("epochs", 0, "steps", "epochs must be at least 1"),
    ],
)
@pytest.mark.usefixtures("data")
def test_a_setting_out_of_range_says_which(
    mnist: ModuleType, field: str, value: float, method: str, message: str
) -> None:
    """Each bad setting is a clear error, not a crash in numpy."""
    train = mnist.Train(**{field: value})
    with pytest.raises(ValueError, match=message):
        getattr(train, method)()


def test_start_fills_the_plot_with_one_point_a_step(
    mnist: ModuleType, data: Path, monkeypatch: pytest.MonkeyPatch
) -> None:
    """Start on `Train` ends with both curves in the Plot."""
    del data
    monkeypatch.setattr(mnist, "STEP_BATCHES", 2)
    app = mnist.build_app()
    ended = threading.Event()

    def listener(event: object) -> None:
        if isinstance(event, JobUpdate) and event.state == "finished":
            ended.set()

    app.subscribe_jobs(listener)
    app.apply(SetValue(node="n2", port="epochs", value=EPOCHS))
    app.apply(SetValue(node="n2", port="hidden", value=4))
    app.start_job("n2")
    assert ended.wait(WAIT)
    outcomes = {o.node: o for o in app.execute().outcomes}
    loss = outcomes["n3"].shown["series_1"]
    accuracy = outcomes["n3"].shown["series_2"]
    assert isinstance(loss, Series)
    assert isinstance(accuracy, Series)
    assert len(loss.values) == len(accuracy.values) == POINTS
