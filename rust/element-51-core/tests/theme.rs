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
