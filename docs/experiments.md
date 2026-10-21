# An experiment: reading digits

The example `examples/g_experiments/g3_mnist.py` trains a small neural network
on handwritten digits, live, in the editor. It uses a picture on a card, a job
that reports as it works, a Plot, and `Run automatically`, so it shows what the
pieces do together.

Run it from the repository root, after building the browser layer:

```bash
uv run examples/g_experiments/g3_mnist.py
```

The first start downloads the data: MNIST (60,000 training pictures of digits
and 10,000 test pictures) and Fashion-MNIST (the same sizes, pictures of
clothes), 42.5 MB in all, into `__data__/` in the folder you run it from. Each
file is checked against a sha256 pinned in the example before it is kept.

## The three nodes

`MnistSample` has an `index` box and a `dataset` box that holds `mnist` or
`fashion`. It outputs the picture (shown on its card), its label and the
dataset word.

`Train` is a job node. Its boxes are `hidden` (the size of the hidden layer,
128), `learning_rate` (0.1), `epochs` (30) and `dataset`, which is linked to
the sample's so the two agree. It learns by plain minibatch gradient descent
on batches of 64 pictures, written in numpy. Every 100 batches it reports one
point of the mean loss and one of the accuracy on the first 1,000 test
pictures; its last point uses all 10,000.

A Plot node draws both series as they arrive.

## What to expect

On the default settings MNIST ends at about 97.7% test accuracy in about 7
seconds, and Fashion-MNIST at about 87.8%. A network with 16 hidden units ends
at about 95.4% in 3 seconds. These are measured on one Mac with numpy, and a
run is the same every time because the random generator starts from a fixed
seed.

The example does not use a GPU, a convolution or a saved model. Its network is
the smallest one that reads digits well, so you can see every number it uses.
