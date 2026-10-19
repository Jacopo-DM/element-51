//! The browser's copy of the graph.
//!
//! An edit applies at once and waits for Python's answer. Python confirms it
//! (`ack`) or refuses it (`reject`). A refused edit is rolled back, together
//! with any later pending edit that no longer applies without it.
//! A new snapshot from Python (another page, or a script, changed the graph)
//! replaces the confirmed state and keeps the pending edits that still apply.

use crate::{Edit, EditError, EditRequest, Graph, NodeType, Snapshot, apply_edit};

/// The last graph Python confirmed, plus the edits still waiting for an answer.
pub struct Replica {
    node_types: Vec<NodeType>,
    confirmed: Graph,
    pending: Vec<(u64, Edit)>,
    local: Graph,
    next_request: u64,
}

impl Replica {
    /// Start from a snapshot.
    pub fn new(snapshot: Snapshot) -> Replica {
        Replica {
            node_types: snapshot.node_types,
            local: snapshot.graph.clone(),
            confirmed: snapshot.graph,
            pending: Vec::new(),
            next_request: 1,
        }
    }

    /// Replace the confirmed state with a new snapshot. Pending edits stay,
    /// except those that no longer apply to it.
    pub fn reset(&mut self, snapshot: Snapshot) {
        self.node_types = snapshot.node_types;
        self.confirmed = snapshot.graph;
        self.replay();
    }

    /// What the user sees: confirmed state plus pending edits.
    pub fn graph(&self) -> &Graph {
        &self.local
    }

    /// Every node type Python announced.
    pub fn node_types(&self) -> &[NodeType] {
        &self.node_types
    }

    /// How many edits wait for an answer.
    pub fn pending_len(&self) -> usize {
        self.pending.len()
    }

    /// An unused id for a node placed now.
    pub fn next_node_id(&self) -> String {
        self.local.next_id()
    }

    /// Apply `edit` locally and return the request to send to Python.
    ///
    /// A refused edit changes nothing and uses no request number.
    pub fn submit(&mut self, edit: Edit) -> Result<EditRequest, EditError> {
        self.local = apply_edit(&self.local, &self.node_types, &edit)?;
        let id = self.next_request;
        self.next_request += 1;
        self.pending.push((id, edit.clone()));
        Ok(EditRequest { id, edit })
    }

    /// Python confirmed request `id`. False if it is not pending.
    pub fn ack(&mut self, id: u64) -> bool {
        let Some(position) = self.pending.iter().position(|(pending, _)| *pending == id) else {
            return false;
        };
        let (_, edit) = self.pending.remove(position);
        if let Ok(next) = apply_edit(&self.confirmed, &self.node_types, &edit) {
            self.confirmed = next;
        }
        true
    }

    /// Python refused request `id`. Roll it back. False if it is not pending.
    pub fn reject(&mut self, id: u64) -> bool {
        let Some(position) = self.pending.iter().position(|(pending, _)| *pending == id) else {
            return false;
        };
        self.pending.remove(position);
        self.replay();
        true
    }

    /// Rebuild the local graph from the confirmed one, dropping any pending
    /// edit that no longer applies.
    fn replay(&mut self) {
        let mut local = self.confirmed.clone();
        let mut kept = Vec::new();
        for (id, edit) in std::mem::take(&mut self.pending) {
            if let Ok(next) = apply_edit(&local, &self.node_types, &edit) {
                local = next;
                kept.push((id, edit));
            }
        }
        self.local = local;
        self.pending = kept;
    }
}
