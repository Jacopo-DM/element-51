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

/// Where to put a popup of `size` that was asked for at `want`, so that it
/// lies inside a window of `window` size with `margin` to spare. A popup too
/// large for the window sticks to the top and left margin.
pub fn clamp_into(
    want: (f64, f64),
    size: (f64, f64),
    window: (f64, f64),
    margin: f64,
) -> (f64, f64) {
    let x = want.0.min(window.0 - size.0 - margin).max(margin);
    let y = want.1.min(window.1 - size.1 - margin).max(margin);
    (x, y)
}

/// Where a flyout of `size` goes for the menu entry with edges `entry`, which
/// is `(left, top, right)` in window pixels. The flyout opens against the entry's
/// right edge, level with it and raised by `raise` (the menu's padding), or
/// against its left edge if the right has no room. It is kept inside the window
/// with `margin` to spare.
pub fn flyout_at(
    entry: (f64, f64, f64),
    size: (f64, f64),
    window: (f64, f64),
    raise: f64,
    margin: f64,
) -> (f64, f64) {
    let (left, top, right) = entry;
    let x = if right + size.0 + margin > window.0 {
        left - size.0
    } else {
        right
    };
    let (_, y) = clamp_into((x, top - raise), size, window, margin);
    (x, y)
}
