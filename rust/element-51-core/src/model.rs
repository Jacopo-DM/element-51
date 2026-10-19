//! Records for node types, graphs and the messages that carry them.

use serde::{Deserialize, Serialize};

/// The type of a port. A link may join two ports only if their types agree.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum PortType {
    Float,
    Int,
    Str,
    Bool,
}

/// One input or output of a node type.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Port {
    pub name: String,
    pub kind: PortType,
}

/// The shape of a node: its name and its ports, in drawing order.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct NodeType {
    pub name: String,
    pub inputs: Vec<Port>,
    pub outputs: Vec<Port>,
}

/// One end of a link: a port on a node in the graph.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PortRef {
    pub node: String,
    pub port: String,
}

/// A link from an output port to an input port.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Link {
    pub source: PortRef,
    pub target: PortRef,
}

/// A node placed in the graph.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Node {
    pub id: String,
    pub node_type: String,
    pub x: f64,
    pub y: f64,
}

/// The nodes and links the user has built.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Graph {
    pub nodes: Vec<Node>,
    pub links: Vec<Link>,
}

/// The whole state, sent when a browser connects.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Snapshot {
    pub version: u32,
    pub node_types: Vec<NodeType>,
    pub graph: Graph,
}

impl Snapshot {
    /// The protocol version this build reads.
    pub const VERSION: u32 = 1;

    /// Find a node type by name.
    pub fn node_type(&self, name: &str) -> Option<&NodeType> {
        self.node_types
            .iter()
            .find(|node_type| node_type.name == name)
    }
}

/// A message from Python, tagged by its `op` field.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(tag = "op", rename_all = "snake_case")]
pub enum Message {
    Snapshot(Snapshot),
}

impl Message {
    /// Parse a message and check that its protocol version is one we read.
    pub fn from_json(text: &str) -> Result<Message, serde_json::Error> {
        let message: Message = serde_json::from_str(text)?;
        let Message::Snapshot(snapshot) = &message;
        if snapshot.version != Snapshot::VERSION {
            return Err(serde::de::Error::custom(format!(
                "protocol version {} is not supported (this build reads version {})",
                snapshot.version,
                Snapshot::VERSION
            )));
        }
        Ok(message)
    }
}
