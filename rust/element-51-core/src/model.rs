//! Records for node types, graphs and the messages that carry them.

use std::collections::BTreeMap;

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

impl std::fmt::Display for PortType {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let name = match self {
            PortType::Float => "float",
            PortType::Int => "int",
            PortType::Str => "str",
            PortType::Bool => "bool",
        };
        formatter.write_str(name)
    }
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
    /// What an unlinked input takes when the node has no value of its own.
    #[serde(default)]
    pub defaults: BTreeMap<String, serde_json::Value>,
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
pub struct PlacedNode {
    pub id: String,
    pub node_type: String,
    pub x: f64,
    pub y: f64,
    /// Values the user gave to unlinked inputs, by input name.
    #[serde(default)]
    pub values: BTreeMap<String, serde_json::Value>,
}

/// The nodes and links the user has built.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Graph {
    pub nodes: Vec<PlacedNode>,
    pub links: Vec<Link>,
}

/// The whole state, sent when a browser connects.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Snapshot {
    pub version: u32,
    /// The name of the theme to draw in.
    pub theme: String,
    pub node_types: Vec<NodeType>,
    pub graph: Graph,
}

impl Snapshot {
    /// The protocol version this build reads.
    pub const VERSION: u32 = 2;

    /// Find a node type by name.
    pub fn node_type(&self, name: &str) -> Option<&NodeType> {
        self.node_types
            .iter()
            .find(|node_type| node_type.name == name)
    }
}

/// A change to the graph, tagged by its `kind` field.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum Edit {
    /// Place a node. The browser proposes the id.
    AddNode {
        id: String,
        node_type: String,
        x: f64,
        y: f64,
    },
    /// Link an output port to an input port.
    AddLink { source: PortRef, target: PortRef },
    /// Put a node at a new position.
    MoveNode { id: String, x: f64, y: f64 },
    /// Remove a node, and every link that touches it.
    DeleteNode { id: String },
    /// Remove one link.
    DeleteLink { source: PortRef, target: PortRef },
    /// Give an input its own value, or take it away with `None`.
    SetValue {
        node: String,
        port: String,
        value: Option<serde_json::Value>,
    },
}

/// The browser asks Python to confirm an edit it already applied.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct EditRequest {
    pub id: u64,
    pub edit: Edit,
}

/// Python confirms the edit with this request id.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Ack {
    pub id: u64,
}

/// Python refuses the edit with this request id.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Reject {
    pub id: u64,
    pub reason: String,
}

/// The browser asks Python to run the graph.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RunRequest {}

/// What happened to a node in a run.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Status {
    Ok,
    Error,
    Skipped,
}

/// The outcome of one node in a run.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct NodeOutcome {
    pub node: String,
    pub status: Status,
    /// The value of each output port, by port name. Empty unless `status` is ok.
    pub outputs: BTreeMap<String, serde_json::Value>,
    pub error: Option<String>,
}

impl NodeOutcome {
    /// The value of output `port` as JSON text, for showing beside the port.
    /// Strings keep their quotes. None if the port has no value.
    pub fn output_text(&self, port: &str) -> Option<String> {
        self.outputs.get(port).map(ToString::to_string)
    }
}

/// The outcome of every node, in the order they ran.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RunResult {
    pub outcomes: Vec<NodeOutcome>,
}

impl RunResult {
    /// The outcome for node `id`, if it is in the result.
    pub fn outcome_for(&self, id: &str) -> Option<&NodeOutcome> {
        self.outcomes.iter().find(|outcome| outcome.node == id)
    }

    /// One line naming every failed node and why, or None if none failed.
    pub fn error_summary(&self) -> Option<String> {
        let errors: Vec<String> = self
            .outcomes
            .iter()
            .filter(|outcome| outcome.status == Status::Error)
            .map(|outcome| {
                format!(
                    "{}: {}",
                    outcome.node,
                    outcome.error.as_deref().unwrap_or("failed")
                )
            })
            .collect();
        if errors.is_empty() {
            None
        } else {
            Some(errors.join("; "))
        }
    }
}

/// A message in either direction, tagged by its `op` field.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(tag = "op", rename_all = "snake_case")]
pub enum Message {
    Snapshot(Snapshot),
    Edit(EditRequest),
    Ack(Ack),
    Reject(Reject),
    Run(RunRequest),
    RunResult(RunResult),
}

impl Message {
    /// Write the message as JSON text.
    pub fn to_json(&self) -> String {
        serde_json::to_string(self).expect("a message always serialises")
    }

    /// Parse a message and check that a snapshot's protocol version is one we read.
    pub fn from_json(text: &str) -> Result<Message, serde_json::Error> {
        let message: Message = serde_json::from_str(text)?;
        if let Message::Snapshot(snapshot) = &message
            && snapshot.version != Snapshot::VERSION
        {
            return Err(serde::de::Error::custom(format!(
                "protocol version {} is not supported (this build reads version {})",
                snapshot.version,
                Snapshot::VERSION
            )));
        }
        Ok(message)
    }
}

impl Graph {
    /// Whether a link ends at input `port` of node `node`.
    pub fn is_linked(&self, node: &str, port: &str) -> bool {
        self.links
            .iter()
            .any(|link| link.target.node == node && link.target.port == port)
    }

    /// An unused node id, one above the highest `n<number>` in use.
    pub fn next_id(&self) -> String {
        let highest = self
            .nodes
            .iter()
            .filter_map(|node| node.id.strip_prefix('n')?.parse::<u64>().ok())
            .max()
            .unwrap_or(0);
        format!("n{}", highest + 1)
    }
}
