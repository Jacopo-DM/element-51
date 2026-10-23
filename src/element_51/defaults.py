"""The node types every editor starts with, and where the menu puts them."""

from element_51.inputs import Flag, Float, Integer, Slider, Text
from element_51.model import NodeType
from element_51.nodes import PlotNode, Registered

DEFAULT_NODES: tuple[Registered, ...] = (
    Float,
    Integer,
    Text,
    Flag,
    Slider,
    PlotNode,
)
"""The nodes an `App` adds before the user's own, unless told not to."""

DEFAULT_FOLDER = "Default nodes"
USER_FOLDER = "User defined nodes"


def in_menu(node_type: NodeType, *, default: bool) -> NodeType:
    """Put a node type in the folder the add-node menu shows it under.

    Parameters
    ----------
    node_type
        The type as its function declares it; left unchanged.
    default
        True for one of `DEFAULT_NODES`, false for a node the user listed.

    Returns
    -------
    NodeType
        A copy whose category starts with the folder.
    """
    folder = DEFAULT_FOLDER if default else USER_FOLDER
    category = (
        f"{folder}/{node_type.category}" if node_type.category else folder
    )
    return node_type.model_copy(update={"category": category})
