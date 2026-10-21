"""Train a small network on MNIST and watch it learn.

Work in progress: the notes for this example come with its last task.
"""

# Standard library
import gzip
import hashlib
import shutil
import struct
import urllib.request
from collections.abc import Callable
from pathlib import Path

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
TIMEOUT_SECONDS = 60

# --- Warnings --- #

# --- Typing --- #
type Bytes = NDArray[np.uint8]
type Floats = NDArray[np.float64]
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
            Part(name=TRAIN_IMAGES, size=9_912_422, sha256=""),
            Part(name=TRAIN_LABELS, size=28_881, sha256=""),
            Part(name=TEST_IMAGES, size=1_648_877, sha256=""),
            Part(name=TEST_LABELS, size=4_542, sha256=""),
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
            Part(name=TRAIN_IMAGES, size=26_421_880, sha256=""),
            Part(name=TRAIN_LABELS, size=29_515, sha256=""),
            Part(name=TEST_IMAGES, size=4_422_102, sha256=""),
            Part(name=TEST_LABELS, size=5_148, sha256=""),
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


def build_app() -> element_51.App:
    """Build the app.

    Returns
    -------
    element_51.App
        An empty app, until the nodes are added.
    """
    return element_51.App([])


def main() -> None:
    """Entry point."""
    element_51.serve(build_app())


if __name__ == "__main__":  # pragma: no cover
    main()
