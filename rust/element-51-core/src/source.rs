//! Showing a node's source code: the request, and the panel's words.

use crate::{Message, SourceReply, SourceRequest};

/// What the source panel shows for one reply.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SourcePanel {
    /// The panel's heading, such as `Source of n2 (add)`.
    pub title: String,
    /// The code, or the reason there is none.
    pub body: String,
    /// What a screen reader is told when the panel opens.
    pub announcement: String,
}

/// The message that asks Python for the source of node `node`, as JSON text.
pub fn source_request_text(node: &str) -> String {
    Message::Source(SourceRequest {
        node: node.to_string(),
    })
    .to_json()
}

/// The words for `reply`, whose node has the node type named `type_name`.
pub fn source_panel(reply: &SourceReply, type_name: &str) -> SourcePanel {
    let title = format!("Source of {} ({type_name})", reply.node);
    match &reply.source {
        Some(code) => {
            let lines = code.lines().count();
            let unit = if lines == 1 { "line" } else { "lines" };
            SourcePanel {
                title,
                body: code.clone(),
                announcement: format!(
                    "Source of {}, {type_name}, {lines} {unit}. Press Escape to close",
                    reply.node
                ),
            }
        }
        None => {
            let reason = reply.reason.as_deref().unwrap_or("Python sent no source");
            SourcePanel {
                title,
                body: reason.to_string(),
                announcement: format!(
                    "No source for {}: {reason}. Press Escape to close",
                    reply.node
                ),
            }
        }
    }
}
