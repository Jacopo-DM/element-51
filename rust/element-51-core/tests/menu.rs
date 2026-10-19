//! The add-node menu: nested categories, and search across all of them.

use element_51_core::NodeType;
use element_51_core::menu::{Entry, build_menu, search};

fn kind(name: &str, category: &str) -> NodeType {
    NodeType {
        name: name.to_string(),
        inputs: vec![],
        outputs: vec![],
        defaults: Default::default(),
        category: category.to_string(),
    }
}

fn types() -> Vec<NodeType> {
    vec![
        kind("add", ""),
        kind("sine", "Math/Trig"),
        kind("sqrt", "Math"),
        kind("cosine", "Math/Trig"),
        kind("shout", "Text"),
        kind("show", ""),
    ]
}

fn node(name: &str) -> Entry {
    Entry::Node(name.to_string())
}

fn category(name: &str, entries: Vec<Entry>) -> Entry {
    Entry::Category {
        name: name.to_string(),
        entries,
    }
}

#[test]
fn categories_nest_and_come_before_the_nodes_of_their_level() {
    let menu = build_menu(&types());
    assert_eq!(
        menu,
        vec![
            category(
                "Math",
                vec![
                    category("Trig", vec![node("sine"), node("cosine")]),
                    node("sqrt"),
                ]
            ),
            category("Text", vec![node("shout")]),
            node("add"),
            node("show"),
        ]
    );
}

#[test]
fn categories_keep_the_order_they_first_appear_in() {
    let menu = build_menu(&[kind("b", "Zeta"), kind("a", "Alpha"), kind("c", "Zeta")]);
    assert_eq!(
        menu,
        vec![
            category("Zeta", vec![node("b"), node("c")]),
            category("Alpha", vec![node("a")]),
        ]
    );
}

#[test]
fn no_node_types_give_an_empty_menu() {
    assert!(build_menu(&[]).is_empty());
}

#[test]
fn search_finds_a_name_whatever_the_case() {
    let found = search(&types(), "SQRT");
    let names: Vec<&str> = found.iter().map(|m| m.name.as_str()).collect();
    assert_eq!(names, ["sqrt"]);
    assert_eq!(found[0].path, "Math/sqrt");
}

#[test]
fn search_looks_through_every_category_and_matches_the_path_too() {
    let by_path = search(&types(), "trig");
    let names: Vec<&str> = by_path.iter().map(|m| m.name.as_str()).collect();
    assert_eq!(names, ["sine", "cosine"]);
    let by_category = search(&types(), "math");
    assert_eq!(by_category.len(), 3);
    let several = search(&types(), "s");
    let names: Vec<&str> = several.iter().map(|m| m.name.as_str()).collect();
    // add has no s; show and shout, sine, sqrt, cosine all do, and the paths
    // `Text` and `Math/Trig` add nothing new.
    assert_eq!(names, ["sine", "sqrt", "cosine", "shout", "show"]);
}

#[test]
fn a_top_level_node_shows_just_its_name_as_its_path() {
    let found = search(&types(), "add");
    assert_eq!(found[0].path, "add");
}

#[test]
fn empty_or_unmatched_text_finds_nothing() {
    assert!(search(&types(), "").is_empty());
    assert!(search(&types(), "   ").is_empty());
    assert!(search(&types(), "zzz").is_empty());
}
