//! The theme: every colour, size and font in one place.

use element_51_core::theme::{THEME_NAMES, Theme};

const DRAW: &str = include_str!("../../element-51-web/src/draw.rs");
const WEB: &str = include_str!("../../element-51-web/src/lib.rs");
const PAGE: &str = include_str!("../../element-51-web/static/index.html");

#[test]
fn every_built_in_theme_parses_and_carries_its_own_name() {
    assert!(!THEME_NAMES.is_empty());
    for name in THEME_NAMES {
        let theme = Theme::builtin(name).unwrap_or_else(|| panic!("{name} does not parse"));
        assert_eq!(theme.name, name);
    }
}

#[test]
fn an_unknown_theme_name_gives_nothing() {
    assert!(Theme::builtin("nope").is_none());
}

#[test]
fn the_default_theme_is_the_dark_antimony_one() {
    assert_eq!(Theme::default().name, "antimony");
}

#[test]
fn the_sizes_are_sensible() {
    let metrics = Theme::default().metrics;
    assert!(metrics.node_width > 0.0);
    assert!(metrics.header_height > 0.0);
    assert!(metrics.row_height > 0.0);
    assert!(metrics.wire_width_selected > metrics.wire_width);
    assert!(metrics.card_radius * 2.0 < metrics.header_height + metrics.row_height);
}

#[test]
fn the_html_parts_get_named_css_values_from_the_theme() {
    let theme = Theme::default();
    let variables = theme.css_variables();
    let value = |name: &str| {
        variables
            .iter()
            .find(|(key, _)| key == name)
            .map(|(_, value)| value.as_str())
    };
    assert_eq!(value("--canvas"), Some(theme.colours.canvas.as_str()));
    assert_eq!(value("--field-bg"), Some(theme.colours.field.as_str()));
    assert_eq!(value("--menu-bg"), Some(theme.colours.menu.as_str()));
    assert_eq!(value("--font"), Some(theme.fonts.family.as_str()));
    assert_eq!(
        value("--value-font"),
        Some(theme.fonts.value_family.as_str())
    );
    assert!(value("--nope").is_none());
}

/// Whether `text` holds something that looks like a hex colour, `#` then 3, 4,
/// 6 or 8 hex digits that end the word. Element ids like `#fields` do not count.
fn has_hex_colour(text: &str) -> Option<String> {
    let bytes = text.as_bytes();
    for (index, byte) in bytes.iter().enumerate() {
        if *byte != b'#' {
            continue;
        }
        let digits = bytes[index + 1..]
            .iter()
            .take_while(|byte| byte.is_ascii_hexdigit())
            .count();
        let after = bytes.get(index + 1 + digits).copied().unwrap_or(b' ');
        if [3, 4, 6, 8].contains(&digits) && !after.is_ascii_alphanumeric() {
            return Some(text[index..index + 1 + digits].to_string());
        }
    }
    None
}

#[test]
fn no_colour_is_written_in_the_drawing_code_or_the_page() {
    for (name, text) in [("draw.rs", DRAW), ("lib.rs", WEB), ("index.html", PAGE)] {
        assert_eq!(has_hex_colour(text), None, "{name} holds a colour literal");
    }
}

#[test]
fn the_scanner_finds_colours_and_skips_ids() {
    assert_eq!(
        has_hex_colour("fill: #1e1e1e;"),
        Some("#1e1e1e".to_string())
    );
    assert_eq!(has_hex_colour("color: #fff }"), Some("#fff".to_string()));
    assert_eq!(has_hex_colour("#fields input { }"), None);
    assert_eq!(has_hex_colour("#menu, #error, #canvas"), None);
}

const NAMES: &str = include_str!("../../../tests/fixtures/theme_names.json");

#[test]
fn rust_and_python_offer_the_same_themes() {
    let python: Vec<String> = serde_json::from_str(NAMES).unwrap();
    assert_eq!(python, THEME_NAMES);
}

#[test]
fn the_light_theme_has_a_light_canvas_and_a_wire_selection_that_shows_on_it() {
    let dark = Theme::builtin("antimony").unwrap();
    let light = Theme::builtin("antimony-light").unwrap();
    assert_ne!(dark.colours.canvas, light.colours.canvas);
    assert_ne!(light.colours.selected_wire, light.colours.canvas);
    assert_ne!(dark.colours.selected_wire, dark.colours.canvas);
    // Only the look changes between these two: sizes and fonts are shared.
    assert_eq!(dark.metrics, light.metrics);
    assert_eq!(dark.fonts, light.fonts);
}

fn changed(base: &str, changes: &str) -> Result<Theme, String> {
    Theme::with_changes(base, &serde_json::from_str(changes).unwrap())
}

#[test]
fn no_changes_gives_the_built_in_theme() {
    assert_eq!(
        Theme::with_changes("antimony-light", &serde_json::Value::Null).unwrap(),
        Theme::builtin("antimony-light").unwrap()
    );
    assert_eq!(
        changed("antimony", "{}").unwrap(),
        Theme::builtin("antimony").unwrap()
    );
}

#[test]
fn a_change_replaces_only_the_named_values() {
    let base = Theme::builtin("antimony").unwrap();
    let theme = changed(
        "antimony",
        r##"{"colours": {"canvas": "#101010"},
             "metrics": {"node_width": 260},
             "fonts": {"value": "14px Menlo", "field_px": 14}}"##,
    )
    .unwrap();
    assert_eq!(theme.colours.canvas, "#101010");
    assert_eq!(theme.metrics.node_width, 260.0);
    assert_eq!(theme.fonts.value, "14px Menlo");
    assert_eq!(theme.fonts.field_px, 14.0);
    assert_eq!(theme.colours.card, base.colours.card);
    assert_eq!(theme.metrics.row_height, base.metrics.row_height);
    assert_eq!(theme.fonts.card, base.fonts.card);
    assert_eq!(theme.name, "antimony");
}

#[test]
fn changes_apply_on_top_of_the_base_they_name() {
    let theme = changed("antimony-light", r#"{"fonts": {"family": "Georgia"}}"#).unwrap();
    assert_eq!(
        theme.colours.canvas,
        Theme::builtin("antimony-light").unwrap().colours.canvas
    );
    assert_eq!(theme.fonts.family, "Georgia");
}

#[test]
fn an_unknown_base_names_the_themes() {
    let error = changed("nope", "{}").unwrap_err();
    assert!(
        error.contains("nope") && error.contains("antimony-light"),
        "{error}"
    );
}

#[test]
fn an_unknown_key_names_the_section_and_lists_the_keys() {
    let error = changed("antimony", r#"{"colours": {"canvass": "red"}}"#).unwrap_err();
    assert!(
        error.contains("colours") && error.contains("canvass"),
        "{error}"
    );
    assert!(error.contains("card_border"), "{error}");
}

#[test]
fn an_unknown_section_lists_the_sections() {
    let error = changed("antimony", r#"{"sizes": {}}"#).unwrap_err();
    assert!(
        error.contains("sizes") && error.contains("metrics"),
        "{error}"
    );
}

#[test]
fn a_value_of_the_wrong_kind_says_what_it_must_be() {
    let error = changed("antimony", r#"{"metrics": {"node_width": "wide"}}"#).unwrap_err();
    assert!(
        error.contains("node_width") && error.contains("number"),
        "{error}"
    );
    let error = changed("antimony", r#"{"fonts": {"card": 12}}"#).unwrap_err();
    assert!(error.contains("card") && error.contains("text"), "{error}");
}

#[test]
fn changes_that_are_not_objects_are_refused() {
    assert!(changed("antimony", "[]").unwrap_err().contains("object"));
    assert!(
        changed("antimony", r#"{"colours": 5}"#)
            .unwrap_err()
            .contains("colours")
    );
}

const EXAMPLE_THEME: &str = include_str!("../../../examples/my_theme.json");
const THEME_DOCS: &str = include_str!("../../../docs/themes.md");

#[test]
fn the_example_theme_file_uses_only_keys_that_exist() {
    let mut file: serde_json::Value = serde_json::from_str(EXAMPLE_THEME).unwrap();
    let base = file["base"].as_str().unwrap().to_string();
    file.as_object_mut().unwrap().remove("base");
    assert!(Theme::with_changes(&base, &file).is_ok());
}

#[test]
fn the_theme_docs_list_every_key_a_theme_can_change() {
    let theme = serde_json::to_value(Theme::default()).unwrap();
    for section in ["colours", "metrics", "fonts"] {
        for key in theme[section].as_object().unwrap().keys() {
            assert!(
                THEME_DOCS.contains(&format!("| `{key}` |")),
                "docs/themes.md has no row for {section} key {key}"
            );
        }
    }
}
