"""Train a small network on handwritten digits and watch it learn.

Three nodes. `MnistSample` shows one picture of the MNIST training set and
its label. `Train` is a job: a network with one hidden layer, written in
numpy, that learns to read the digits. A Plot draws its loss and its
accuracy as they change.

The first start downloads the data, 42.5 MB for MNIST (digits) and
Fashion-MNIST (clothes), into `__data__/mnist` and `__data__/fashion`, which
git ignores. Each file is checked against a sha256 pinned in this file.

Build the browser layer first: `uv run python tools/build_wasm.py`.

Notes
-----
    * Tick `Run automatically`. The graph runs once: `MnistSample` shows a
        28 by 28 picture, a 5, and its label.
    * Change `index` (0 to 59999): another picture, and its label follows
        by itself.
    * Press Start on `Train`. The Plot fills as it learns, one point every
        100 batches, 282 in all. On this Mac it takes about 7 seconds. The
        loss falls to about 0.01 and the accuracy on the 10,000 test pictures
        rises to about 97.7%.
    * Type `fashion` into the `dataset` box of `MnistSample`: it shows a
        piece of clothing, and the linked `dataset` box of `Train` follows.
        Press Start again: about 87.8%, because clothes are harder to tell
        apart than digits.
    * Set `hidden` to 16 and press Start: three seconds, about 95.4%. A
        smaller network learns less.
    * Set `learning_rate` to 0.01: the first points are poor (about 61%
        accuracy) and it ends near 96.2%. The step size matters.
    * Set `epochs` to 0 and press Start: the job fails at once and the banner
        says why. The results of the last Start stay on the Plot until the next
        one.
    * After a run, open the card menu of the `Plot` and choose Save as CSV: the
        browser downloads `n3-chart.csv`, one row per point with a column for
        the loss and one for the accuracy, ready for your own script.

"""

# Standard library
import gzip
import hashlib
import math
import shutil
import struct
import urllib.request
from collections.abc import Callable
from functools import cache
from pathlib import Path
from typing import override

# Third-party libraries
import numpy as np
from numpy.typing import NDArray
from pydantic import BaseModel, ConfigDict, Field
from rich.console import Console
from rich.traceback import install

# Local libraries
import element_51

# --- Globals --- #
DATA_ROOT = Path("__data__")
IMAGES_MAGIC = 0x0803
LABELS_MAGIC = 0x0801
IMAGE_HEADER = 16
LABEL_HEADER = 8
PIXELS = 784
CLASSES = 10
SEED = 51
MAX_PIXEL = 255.0
TINY = 1e-12
SIDE = 28
BATCH = 64
STEP_BATCHES = 100
QUICK_TEST = 1000
TIMEOUT_SECONDS = 60

# --- Warnings --- #

# --- Typing --- #
type Bytes = NDArray[np.uint8]
type Floats = NDArray[np.float64]
type Indices = NDArray[np.intp]
type Downloader = Callable[[str, Path], None]

# --- TERMINAL OUTPUT SETUP --- #
install(show_locals=False)
console = Console()

TRAIN_IMAGES = "train-images-idx3-ubyte.gz"
TRAIN_LABELS = "train-labels-idx1-ubyte.gz"
TEST_IMAGES = "t10k-images-idx3-ubyte.gz"
TEST_LABELS = "t10k-labels-idx1-ubyte.gz"


class Part(element_51.Model):
    """One file of a dataset: its name, its size and the sha256 it must have."""

    name: str
    size: int
    sha256: str


class Dataset(element_51.Model):
    """A dataset: where its four files come from and where they are kept."""

    name: str
    folder: str
    base: str
    parts: tuple[Part, ...]


DATASETS = {
    "mnist": Dataset(
        name="mnist",
        folder="mnist",
        base="https://storage.googleapis.com/cvdf-datasets/mnist",
        parts=(
            Part(
                name=TRAIN_IMAGES,
                size=9_912_422,
                sha256="440fcabf73cc546fa21475e81ea370265605f56be210a4024d2ca8f203523609",
            ),
            Part(
                name=TRAIN_LABELS,
                size=28_881,
                sha256="3552534a0a558bbed6aed32b30c495cca23d567ec52cac8be1a0730e8010255c",
            ),
            Part(
                name=TEST_IMAGES,
                size=1_648_877,
                sha256="8d422c7b0a1c1c79245a5bcf07fe86e33eeafee792b84584aec276f5a2dbc4e6",
            ),
            Part(
                name=TEST_LABELS,
                size=4_542,
                sha256="f7ae60f92e00ec6debd23a6088c31dbd2371eca3ffa0defaefb259924204aec6",
            ),
        ),
    ),
    "fashion": Dataset(
        name="fashion",
        folder="fashion",
        base=(
            "https://github.com/zalandoresearch/fashion-mnist"
            "/raw/master/data/fashion"
        ),
        parts=(
            Part(
                name=TRAIN_IMAGES,
                size=26_421_880,
                sha256="3aede38d61863908ad78613f6a32ed271626dd12800ba2636569512369268a84",
            ),
            Part(
                name=TRAIN_LABELS,
                size=29_515,
                sha256="a04f17134ac03560a47e3764e11b92fc97de4d1bfaf8ba1a3aa29af54cc90845",
            ),
            Part(
                name=TEST_IMAGES,
                size=4_422_102,
                sha256="346e55b948d973a97e58d2351dde16a484bd415d4595297633bb08f03db6a073",
            ),
            Part(
                name=TEST_LABELS,
                size=5_148,
                sha256="67da17c76eaffca5446c3361aaab5c3cd6d1c2608764d35dfb1850b086bf8dd5",
            ),
        ),
    ),
}


class IdxHeader(element_51.Model):
    """The numbers at the start of an IDX file."""

    magic: int
    count: int = Field(ge=1)
    rows: int = Field(default=1, ge=1)
    columns: int = Field(default=1, ge=1)


def read_idx(path: Path, magic: int) -> tuple[IdxHeader, Bytes]:
    """Read a gzipped IDX file: its header and its bytes.

    Parameters
    ----------
    path
        The `.gz` file.
    magic
        What the file must be: `IMAGES_MAGIC` or `LABELS_MAGIC`.

    Returns
    -------
    tuple[IdxHeader, Bytes]
        The header, and the data bytes after it.

    Raises
    ------
    ValueError
        If the file is not gzip, is too short, is the wrong kind or does not
        hold as many bytes as its header says.
    """
    try:
        raw = gzip.decompress(path.read_bytes())
    except (OSError, EOFError) as error:
        msg = f"{path.name}: not a gzip file"
        raise ValueError(msg) from error
    size = IMAGE_HEADER if magic == IMAGES_MAGIC else LABEL_HEADER
    if len(raw) < size:
        msg = f"{path.name}: too short to be an IDX file"
        raise ValueError(msg)
    numbers = struct.unpack(
        ">IIII" if size == IMAGE_HEADER else ">II", raw[:size]
    )
    header = IdxHeader.model_validate({
        "magic": numbers[0],
        "count": numbers[1],
        **(
            {"rows": numbers[2], "columns": numbers[3]}
            if size == IMAGE_HEADER
            else {}
        ),
    })
    if header.magic != magic:
        msg = f"{path.name}: magic number {header.magic:#06x}, not {magic:#06x}"
        raise ValueError(msg)
    body: Bytes = np.frombuffer(raw, dtype=np.uint8, offset=size)
    expected = header.count * header.rows * header.columns
    if body.size != expected:
        msg = f"{path.name}: {body.size} bytes, its header says {expected}"
        raise ValueError(msg)
    return header, body


def load_images(path: Path) -> Bytes:
    """Read an images file.

    Parameters
    ----------
    path
        The `.gz` file of pictures.

    Returns
    -------
    Bytes
        One row per picture, one byte per pixel (784 for 28 by 28).
    """
    header, body = read_idx(path, IMAGES_MAGIC)
    return body.reshape(header.count, header.rows * header.columns)


def load_labels(path: Path) -> Bytes:
    """Read a labels file.

    Parameters
    ----------
    path
        The `.gz` file of labels.

    Returns
    -------
    Bytes
        One label, 0 to 9, per picture.
    """
    return read_idx(path, LABELS_MAGIC)[1]


def download_file(url: str, path: Path) -> None:
    """Download `url` into `path`.

    Parameters
    ----------
    url
        An https address.
    path
        Where to write the bytes.

    Raises
    ------
    ValueError
        If `url` is not an https address.
    """
    if not url.startswith("https://"):
        msg = f"{url}: only https addresses are downloaded"
        raise ValueError(msg)
    with (
        urllib.request.urlopen(  # ruff: ignore[suspicious-url-open-usage]  # the address is a fixed https constant from DATASETS, checked above
            url, timeout=TIMEOUT_SECONDS
        ) as reply,
        path.open("wb") as out,
    ):
        shutil.copyfileobj(reply, out)


def fetch(
    dataset: Dataset, root: Path, download: Downloader = download_file
) -> list[Part]:
    """Download the files of `dataset` that are not in `root` yet.

    A file is written under a `.part` name, checked against its pinned
    sha256 and renamed only if it matches, so a file that exists is a checked
    one.

    Parameters
    ----------
    dataset
        The dataset to complete.
    root
        The folder that holds one folder per dataset.
    download
        Writes an address into a file; replaced in tests.

    Returns
    -------
    list[Part]
        The parts that were downloaded.

    Raises
    ------
    ValueError
        If a part has no pinned sha256, or a download does not match it (the
        download is deleted).
    """
    folder = root / dataset.folder
    folder.mkdir(parents=True, exist_ok=True)
    fetched: list[Part] = []
    for part in dataset.parts:
        target = folder / part.name
        if target.exists():
            continue
        if not part.sha256:
            msg = f"{dataset.name}/{part.name}: no sha256 is pinned"
            raise ValueError(msg)
        partial = target.with_name(part.name + ".part")
        download(f"{dataset.base}/{part.name}", partial)
        if hashlib.sha256(partial.read_bytes()).hexdigest() != part.sha256:
            partial.unlink()
            msg = f"{part.name}: the download is not the pinned file"
            raise ValueError(msg)
        partial.replace(target)
        fetched.append(part)
    return fetched


class Weights(BaseModel):
    """The four arrays of a one-hidden-layer network.

    The same record holds the gradients, which have the same shapes. The
    arrays are updated in place, so the record itself is frozen.
    """

    model_config = ConfigDict(
        frozen=True, extra="forbid", arbitrary_types_allowed=True
    )

    w1: Floats
    b1: Floats
    w2: Floats
    b2: Floats


def init_weights(
    hidden: int, rng: np.random.Generator, inputs: int = PIXELS
) -> Weights:
    """Make a network with small random weights and zero biases.

    Parameters
    ----------
    hidden
        How many units the hidden layer has.
    rng
        The random generator that draws the weights.
    inputs
        How many numbers one picture has.

    Returns
    -------
    Weights
        Weights scaled by the square root of 2 over the layer's input size.
    """
    return Weights(
        w1=rng.normal(0.0, np.sqrt(2.0 / inputs), (inputs, hidden)),
        b1=np.zeros(hidden),
        w2=rng.normal(0.0, np.sqrt(2.0 / hidden), (hidden, CLASSES)),
        b2=np.zeros(CLASSES),
    )


def to_floats(pixels: Bytes) -> Floats:
    """Scale bytes from 0 to 255 to numbers from 0 to 1.

    Returns
    -------
    Floats
        The same shape as `pixels`.
    """
    return pixels.astype(np.float64) / MAX_PIXEL


def forward(weights: Weights, x: Floats) -> tuple[Floats, Floats]:
    """Run pictures through the network.

    Parameters
    ----------
    weights
        The network.
    x
        One row per picture.

    Returns
    -------
    tuple[Floats, Floats]
        The hidden layer after the ReLU, and the probability of each class.
    """
    hidden = np.maximum(x @ weights.w1 + weights.b1, 0.0)
    scores = hidden @ weights.w2 + weights.b2
    scores -= scores.max(axis=1, keepdims=True)
    odds = np.exp(scores)
    return hidden, odds / odds.sum(axis=1, keepdims=True)


def loss_and_gradients(
    weights: Weights, x: Floats, labels: Bytes
) -> tuple[float, Weights]:
    """Find the cross-entropy loss of a batch and how to lower it.

    Parameters
    ----------
    weights
        The network.
    x
        One row per picture.
    labels
        The right class of each picture.

    Returns
    -------
    tuple[float, Weights]
        The mean loss, and the gradient of it for every array.
    """
    count = len(x)
    hidden, probs = forward(weights, x)
    rows = np.arange(count)
    loss = float(-np.log(probs[rows, labels] + TINY).mean())
    scores = probs.copy()
    scores[rows, labels] -= 1.0
    scores /= count
    back = (scores @ weights.w2.T) * (hidden > 0.0)
    return loss, Weights(
        w1=x.T @ back,
        b1=back.sum(axis=0),
        w2=hidden.T @ scores,
        b2=scores.sum(axis=0),
    )


def apply_gradients(weights: Weights, grads: Weights, rate: float) -> None:
    """Take one step downhill, in place.

    Parameters
    ----------
    weights
        The network, changed by this call.
    grads
        What `loss_and_gradients` returned.
    rate
        How far to move along each gradient.
    """
    weights.w1[...] -= rate * grads.w1
    weights.b1[...] -= rate * grads.b1
    weights.w2[...] -= rate * grads.w2
    weights.b2[...] -= rate * grads.b2


def accuracy(weights: Weights, images: Bytes, labels: Bytes) -> float:
    """Find the share of pictures the network gets right.

    Parameters
    ----------
    weights
        The network.
    images
        One row per picture, bytes from 0 to 255.
    labels
        The right class of each picture.

    Returns
    -------
    float
        From 0 to 1.
    """
    _, probs = forward(weights, to_floats(images))
    return float((probs.argmax(axis=1) == labels).mean())


class Split(BaseModel):
    """The four arrays of a dataset: training and test pictures and labels."""

    model_config = ConfigDict(
        frozen=True, extra="forbid", arbitrary_types_allowed=True
    )

    train_images: Bytes
    train_labels: Bytes
    test_images: Bytes
    test_labels: Bytes


def dataset_named(name: str) -> Dataset:
    """Find a dataset by the word the `dataset` box holds.

    Parameters
    ----------
    name
        `mnist` or `fashion`.

    Returns
    -------
    Dataset
        The record in `DATASETS`.

    Raises
    ------
    ValueError
        If `name` is neither of the two words.
    """
    if name not in DATASETS:
        msg = f"dataset must be mnist or fashion, not {name!r}"
        raise ValueError(msg)
    return DATASETS[name]


@cache
def read_split(folder: Path) -> Split:
    """Read the four files of a dataset from `folder`, once.

    Parameters
    ----------
    folder
        The dataset's folder, as an absolute path (it is the cache key).

    Returns
    -------
    Split
        The pictures and labels.

    Raises
    ------
    ValueError
        If a file is missing; the message says how to download them.
    """
    for name in (TRAIN_IMAGES, TRAIN_LABELS, TEST_IMAGES, TEST_LABELS):
        if not (folder / name).exists():
            msg = (
                f"{folder / name} is missing: run "
                "`uv run examples/g_experiments/g3_mnist.py` once to "
                "download the data"
            )
            raise ValueError(msg)
    return Split(
        train_images=load_images(folder / TRAIN_IMAGES),
        train_labels=load_labels(folder / TRAIN_LABELS),
        test_images=load_images(folder / TEST_IMAGES),
        test_labels=load_labels(folder / TEST_LABELS),
    )


def load_split(name: str) -> Split:
    """Load a dataset by name from `DATA_ROOT`.

    Parameters
    ----------
    name
        `mnist` or `fashion`.

    Returns
    -------
    Split
        The pictures and labels.
    """
    folder = (DATA_ROOT / dataset_named(name).folder).resolve()
    return read_split(folder)


class Sample(element_51.Model):
    """What `MnistSample` outputs: a picture, its label and its dataset."""

    image: element_51.Image
    label: int
    dataset: str


class MnistSample(element_51.Node):
    """One picture from the training set, with the digit it shows."""

    index: int = 0
    dataset: str = "mnist"

    @override
    def run(self) -> Sample:
        """Pick the picture at `index`.

        Returns
        -------
        Sample
            The picture, its label and the dataset it came from.

        Raises
        ------
        ValueError
            If `index` is outside the training set.
        """
        split = load_split(self.dataset)
        count = len(split.train_images)
        if not 0 <= self.index < count:
            msg = f"index must be from 0 to {count - 1}, not {self.index}"
            raise ValueError(msg)
        return Sample(
            image=element_51.Image(
                width=SIDE,
                height=SIDE,
                channels=1,
                pixels=split.train_images[self.index].tobytes(),
            ),
            label=int(split.train_labels[self.index]),
            dataset=self.dataset,
        )


class Curves(element_51.Model):
    """The two series `Train` makes, one new point of each per step."""

    loss: element_51.Series
    accuracy: element_51.Series


class TrainState(BaseModel):
    """What the steps of one training share."""

    model_config = ConfigDict(arbitrary_types_allowed=True)

    weights: Weights
    split: Split
    rng: np.random.Generator
    order: Indices
    cursor: int = 0


class Train(element_51.JobNode[TrainState]):
    """Train the network on the training pictures, a hundred batches a step."""

    hidden: int = 128
    learning_rate: float = 0.1
    epochs: int = 30
    dataset: str = "mnist"

    def batches(self) -> int:
        """Count the batches of the whole training.

        Returns
        -------
        int
            Batches per epoch times `epochs`.

        Raises
        ------
        ValueError
            If `epochs` is below 1.
        """
        if self.epochs < 1:
            msg = f"epochs must be at least 1, not {self.epochs}"
            raise ValueError(msg)
        pictures = len(load_split(self.dataset).train_images)
        return self.epochs * math.ceil(pictures / BATCH)

    @override
    def steps(self) -> int:
        """Say how many steps there are.

        Returns
        -------
        int
            One step per `STEP_BATCHES` batches, the last one shorter.
        """
        return math.ceil(self.batches() / STEP_BATCHES)

    @override
    def begin(self) -> TrainState:
        """Make the network and shuffle the pictures.

        Returns
        -------
        TrainState
            The weights, the data and the shuffled order.

        Raises
        ------
        ValueError
            If `hidden` is below 1 or `learning_rate` is not above 0.
        """
        if self.hidden < 1:
            msg = f"hidden must be at least 1, not {self.hidden}"
            raise ValueError(msg)
        if self.learning_rate <= 0.0:
            msg = f"learning_rate must be above 0, not {self.learning_rate}"
            raise ValueError(msg)
        split = load_split(self.dataset)
        rng = np.random.default_rng(SEED)
        return TrainState(
            weights=init_weights(self.hidden, rng),
            split=split,
            rng=rng,
            order=rng.permutation(len(split.train_images)),
        )

    @override
    def step(self, state: TrainState, i: int) -> Curves:
        """Train on up to `STEP_BATCHES` batches and test the network.

        Parameters
        ----------
        state
            What `begin` made; the steps change it.
        i
            The step, from 0.

        Returns
        -------
        Curves
            The mean loss of the batches, and the accuracy on the first
            `QUICK_TEST` test pictures (on all of them in the last step).
        """
        total = self.batches()
        first = i * STEP_BATCHES
        count = min(STEP_BATCHES, total - first)
        split = state.split
        losses: list[float] = []
        for _ in range(count):
            if state.cursor >= len(state.order):
                state.order = state.rng.permutation(len(state.order))
                state.cursor = 0
            rows = state.order[state.cursor : state.cursor + BATCH]
            state.cursor += BATCH
            loss, grads = loss_and_gradients(
                state.weights,
                to_floats(split.train_images[rows]),
                split.train_labels[rows],
            )
            apply_gradients(state.weights, grads, self.learning_rate)
            losses.append(loss)
        size = (
            len(split.test_images)
            if first + count >= total
            else min(QUICK_TEST, len(split.test_images))
        )
        return Curves(
            loss=element_51.Series(values=(float(np.mean(losses)),)),
            accuracy=element_51.Series(
                values=(
                    accuracy(
                        state.weights,
                        split.test_images[:size],
                        split.test_labels[:size],
                    ),
                )
            ),
        )


def build_app() -> element_51.App:
    """Build the app: a sample, a training job and a plot.

    Returns
    -------
    element_51.App
        An app whose first node is `MnistSample` (n1), the second `Train`
        (n2) with its `dataset` linked to the sample's, and the third a Plot
        (n3) with the loss and the accuracy linked.
    """
    app = element_51.App([MnistSample, Train])
    sample = app.add_node(MnistSample, x=60.0, y=60.0)
    train = app.add_node(Train, x=380.0, y=60.0)
    chart = app.add_node(element_51.PlotNode, x=700.0, y=60.0)
    app.add_link(sample, "dataset", train, "dataset")
    app.add_link(train, "loss", chart, "series_1")
    app.add_link(train, "accuracy", chart, "series_2")
    return app


def main() -> None:
    """Download the data that is missing, then serve the editor."""
    for dataset in DATASETS.values():
        folder = DATA_ROOT / dataset.folder
        missing = [p for p in dataset.parts if not (folder / p.name).exists()]
        if missing:
            megabytes = sum(part.size for part in missing) / 1_000_000
            console.print(
                f"Downloading {dataset.name}: {megabytes:.1f} MB "
                f"from {dataset.base}"
            )
            fetch(dataset, DATA_ROOT)
    element_51.serve(build_app())


if __name__ == "__main__":  # pragma: no cover
    main()
