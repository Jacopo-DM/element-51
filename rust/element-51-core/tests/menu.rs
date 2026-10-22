//! The add-node menu: nested categories, and search across all of them.

use element_51_core::NodeType;
use element_51_core::menu::{Entry, build_menu, search, step_menu};

fn kind(name: &str, category: &str) -> NodeType {
    NodeType {
        job: false,
        name: name.to_string(),
        inputs: vec![],
        outputs: vec![],
        defaults: Default::default(),
        category: category.to_string(),
        display: Default::default(),
        variadic: Default::default(),
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

#[test]
fn a_card_offers_to_view_its_source_to_save_and_to_delete_the_node_last() {
    use element_51_core::Edit;
    use element_51_core::menu::{CardAction, card_actions};
    let actions = card_actions();
    assert_eq!(
        actions,
        vec![
            CardAction::ViewSource,
            CardAction::SavePng,
            CardAction::SaveCsv,
            CardAction::DeleteNode
        ]
    );
    assert_eq!(CardAction::ViewSource.label(), "View source code");
    assert_eq!(CardAction::SavePng.label(), "Save as PNG");
    assert_eq!(CardAction::SaveCsv.label(), "Save as CSV");
    assert_eq!(CardAction::DeleteNode.label(), "Delete node");
    assert_eq!(
        CardAction::DeleteNode.edit("n3"),
        Some(Edit::DeleteNode { id: "n3".into() })
    );
}

#[test]
fn viewing_the_source_and_saving_are_not_edits() {
    use element_51_core::menu::CardAction;
    assert_eq!(CardAction::ViewSource.edit("n3"), None);
    assert_eq!(CardAction::SavePng.edit("n3"), None);
    assert_eq!(CardAction::SaveCsv.edit("n3"), None);
}

#[test]
fn save_as_png_is_offered_only_for_a_picture_or_a_chart() {
    use element_51_core::CardDisplay;
    use element_51_core::menu::{CardAction, card_actions};
    for action in card_actions() {
        let saves = matches!(action, CardAction::SavePng | CardAction::SaveCsv);
        assert_eq!(action.offered(CardDisplay::None), !saves, "{action:?}");
        assert_eq!(
            action.offered(CardDisplay::Image),
            action != CardAction::SaveCsv,
            "{action:?}"
        );
        assert!(action.offered(CardDisplay::Chart));
    }
}

#[test]
fn save_as_csv_is_offered_only_for_a_chart() {
    use element_51_core::CardDisplay;
    use element_51_core::menu::CardAction;
    assert!(CardAction::SaveCsv.offered(CardDisplay::Chart));
    assert!(!CardAction::SaveCsv.offered(CardDisplay::Image));
    assert!(!CardAction::SaveCsv.offered(CardDisplay::None));
}

#[test]
fn the_png_file_is_named_after_the_node_and_what_it_shows() {
    use element_51_core::menu::png_file_name;
    assert_eq!(png_file_name("n1", "out"), "n1-out.png");
    assert_eq!(png_file_name("n5", "chart"), "n5-chart.png");
    // A port name with a slash, a dot or a space cannot reach a path.
    assert_eq!(png_file_name("n2", "../a b.c"), "n2-___a_b_c.png");
    assert_eq!(png_file_name("n2", "é"), "n2-_.png");
}

mod placement {
    use element_51_core::menu::{clamp_into, flyout_at};

    const WINDOW: (f64, f64) = (1000.0, 800.0);

    #[test]
    fn a_menu_inside_the_window_stays_where_it_was_asked_for() {
        assert_eq!(
            clamp_into((100.0, 120.0), (160.0, 200.0), WINDOW, 8.0),
            (100.0, 120.0)
        );
    }

    #[test]
    fn a_menu_that_hangs_out_is_moved_back_inside_with_the_margin() {
        assert_eq!(
            clamp_into((950.0, 700.0), (160.0, 200.0), WINDOW, 8.0),
            (832.0, 592.0)
        );
        assert_eq!(
            clamp_into((-30.0, -5.0), (160.0, 200.0), WINDOW, 8.0),
            (8.0, 8.0)
        );
    }

    #[test]
    fn a_menu_taller_than_the_window_sticks_to_the_top_margin() {
        assert_eq!(
            clamp_into((10.0, 300.0), (160.0, 900.0), WINDOW, 8.0).1,
            8.0
        );
    }

    #[test]
    fn a_flyout_opens_beside_its_entry_to_the_right_when_there_is_room() {
        // The entry spans x 100 to 260 and starts at y 300; the flyout is level
        // with it, raised by the menu padding of 4.
        let at = flyout_at((100.0, 300.0, 260.0), (160.0, 120.0), WINDOW, 4.0, 8.0);
        assert_eq!(at, (260.0, 296.0));
    }

    #[test]
    fn a_flyout_opens_to_the_left_when_the_right_has_no_room() {
        let at = flyout_at((780.0, 300.0, 940.0), (160.0, 120.0), WINDOW, 4.0, 8.0);
        assert_eq!(at, (620.0, 296.0));
    }

    #[test]
    fn a_flyout_near_the_bottom_is_lifted_to_fit_and_never_leaves_the_top() {
        let low = flyout_at((100.0, 780.0, 260.0), (160.0, 300.0), WINDOW, 4.0, 8.0);
        assert_eq!(low, (260.0, 492.0));
        let high = flyout_at((100.0, 2.0, 260.0), (160.0, 120.0), WINDOW, 4.0, 8.0);
        assert_eq!(high, (260.0, 8.0));
    }
}

#[test]
fn the_arrow_keys_step_through_a_list_and_stop_at_its_ends() {
    assert_eq!(step_menu(Some(1), 4, true), Some(2));
    assert_eq!(step_menu(Some(1), 4, false), Some(0));
    assert_eq!(step_menu(Some(3), 4, true), Some(3));
    assert_eq!(step_menu(Some(0), 4, false), Some(0));
}

#[test]
fn with_nothing_focused_down_goes_to_the_first_and_up_to_the_last() {
    assert_eq!(step_menu(None, 4, true), Some(0));
    assert_eq!(step_menu(None, 4, false), Some(3));
}

#[test]
fn an_empty_list_has_nowhere_to_step() {
    assert_eq!(step_menu(None, 0, true), None);
    assert_eq!(step_menu(Some(0), 0, false), None);
}

#[test]
fn a_stale_position_past_the_end_is_brought_back_in() {
    assert_eq!(step_menu(Some(9), 3, true), Some(2));
    assert_eq!(step_menu(Some(9), 3, false), Some(2));
}
