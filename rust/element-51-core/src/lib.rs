//! Graph logic shared by the browser layer and the native tests.
//!
//! The JSON shapes here are a contract with `src/element_51/model.py` and
//! `src/element_51/protocol.py`; `tests/fixtures/snapshot.json` is read by both
//! sides.

pub mod access;
pub mod address;
pub mod announce;
pub mod autorun;
pub mod contrast;
pub mod display;
mod edit;
pub mod fields;
pub mod highlight;
pub mod history;
pub mod job;
pub mod layout;
pub mod menu;
mod model;
pub mod palette;
mod replica;
pub mod rows;
pub mod source;
pub mod theme;
pub mod view;

pub use edit::{EditError, apply_edit, value_fits};
pub use model::{
    Ack, CardDisplay, Edit, EditRequest, Graph, Image, JobRefusal, JobStatus, JobUpdate, Link,
    LinkedSeries, Message, NodeOutcome, NodeType, PlacedNode, Port, PortRef, PortType, Reject,
    RunRequest, RunResult, Series, Shown, Snapshot, SourceReply, SourceRequest, StartRequest,
    Status, StopRequest, Variadic,
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
