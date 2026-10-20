//! The add-node menu: nested categories, and search across all of them.

use crate::{Edit, NodeType};

/// One line of a menu level.
#[derive(Clone, Debug, PartialEq)]
pub enum Entry {
    /// A node type, by name.
    Node(String),
    /// A submenu.
    Category { name: String, entries: Vec<Entry> },
}

/// A node type that matches a search, with where it sits in the menu.
#[derive(Clone, Debug, PartialEq)]
pub struct Match {
    pub name: String,
    /// The category levels and the name, joined by `/`.
    pub path: String,
}

/// The menu for `node_types`. A level lists its categories first, in the order
/// they first appear, then its node types in the order given.
pub fn build_menu(node_types: &[NodeType]) -> Vec<Entry> {
    let mut menu = Vec::new();
    for node_type in node_types {
        let levels: Vec<&str> = if node_type.category.is_empty() {
            Vec::new()
        } else {
            node_type.category.split('/').collect()
        };
        insert(&mut menu, &levels, &node_type.name);
    }
    menu
}

/// Put `name` into `entries`, `levels` of categories down.
fn insert(entries: &mut Vec<Entry>, levels: &[&str], name: &str) {
    let Some((first, rest)) = levels.split_first() else {
        entries.push(Entry::Node(name.to_string()));
        return;
    };
    let position = entries
        .iter()
        .position(|entry| matches!(entry, Entry::Category { name, .. } if name == first));
    let position = position.unwrap_or_else(|| {
        // A new category goes after the last category and before every node.
        let at = entries
            .iter()
            .position(|entry| matches!(entry, Entry::Node(_)))
            .unwrap_or(entries.len());
        entries.insert(
            at,
            Entry::Category {
                name: (*first).to_string(),
                entries: Vec::new(),
            },
        );
        at
    });
    if let Entry::Category { entries, .. } = &mut entries[position] {
        insert(entries, rest, name);
    }
}

/// The node types whose name or whole path holds `text`, ignoring case, in the
/// order given. Empty text finds nothing.
pub fn search(node_types: &[NodeType], text: &str) -> Vec<Match> {
    let wanted = text.trim().to_lowercase();
    if wanted.is_empty() {
        return Vec::new();
    }
    node_types
        .iter()
        .filter_map(|node_type| {
            let path = if node_type.category.is_empty() {
                node_type.name.clone()
            } else {
                format!("{}/{}", node_type.category, node_type.name)
            };
            path.to_lowercase().contains(&wanted).then(|| Match {
                name: node_type.name.clone(),
                path,
            })
        })
        .collect()
}

/// Something the menu on a card's header icon can do to that card's node.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CardAction {
    DeleteNode,
}

/// The entries of the card menu, in the order shown.
pub fn card_actions() -> Vec<CardAction> {
    vec![CardAction::DeleteNode]
}

impl CardAction {
    /// The words on the menu entry.
    pub fn label(self) -> &'static str {
        match self {
            CardAction::DeleteNode => "Delete node",
        }
    }

    /// The edit that carries out this action on node `id`.
    pub fn edit(self, id: &str) -> Edit {
        match self {
            CardAction::DeleteNode => Edit::DeleteNode { id: id.to_string() },
        }
    }
}
