//! Graph logic shared by the browser layer and the native tests.
//!
//! The JSON shapes here are a contract with `src/element_51/model.py` and
//! `src/element_51/protocol.py`; `tests/fixtures/snapshot.json` is read by both
//! sides.

mod edit;
pub mod layout;
mod model;
mod replica;

pub use edit::{EditError, apply_edit};
pub use model::{
    Ack, Edit, EditRequest, Graph, Link, Message, Node, NodeOutcome, NodeType, Port, PortRef,
    PortType, Reject, RunRequest, RunResult, Snapshot, Status,
};
pub use replica::Replica;

/// Whether an output of type `from` may feed an input of type `to`.
///
/// Equal types link. An int output may also feed a float input.
pub fn can_link(from: PortType, to: PortType) -> bool {
    from == to || (from == PortType::Int && to == PortType::Float)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn float_to_str_is_refused() {
        assert!(!can_link(PortType::Float, PortType::Str));
    }

    #[test]
    fn equal_types_link() {
        assert!(can_link(PortType::Str, PortType::Str));
    }

    #[test]
    fn int_feeds_float_but_not_back() {
        assert!(can_link(PortType::Int, PortType::Float));
        assert!(!can_link(PortType::Float, PortType::Int));
    }

    #[test]
    fn port_type_uses_lowercase_names_in_json() {
        let json = serde_json::to_string(&PortType::Float).unwrap();
        assert_eq!(json, "\"float\"");
        let back: PortType = serde_json::from_str("\"bool\"").unwrap();
        assert_eq!(back, PortType::Bool);
    }
}
