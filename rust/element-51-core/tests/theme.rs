//! The theme: every colour, size and font in one place.

use element_51_core::theme::{THEME_NAMES, Theme};

const DRAW: &str = include_str!("../../element-51-web/src/draw.rs");
const WEB: &str = include_str!("../../element-51-web/src/lib.rs");
const FIELDS_UI: &str = include_str!("../../element-51-web/src/fields_ui.rs");
const INPUT: &str = include_str!("../../element-51-web/src/input.rs");
const DRAW_CARD: &str = include_str!("../../element-51-web/src/draw_card.rs");
const DRAW_CHART: &str = include_str!("../../element-51-web/src/draw_chart.rs");
const DRAW_FOCUS: &str = include_str!("../../element-51-web/src/draw_focus.rs");
const DRAW_WIRES: &str = include_str!("../../element-51-web/src/draw_wires.rs");
const RUN_UI: &str = include_str!("../../element-51-web/src/run_ui.rs");
const EDIT_UI: &str = include_str!("../../element-51-web/src/edit_ui.rs");
const MESSAGES: &str = include_str!("../../element-51-web/src/messages.rs");
const CANVAS_UI: &str = include_str!("../../element-51-web/src/canvas_ui.rs");
const MENU_UI: &str = include_str!("../../element-51-web/src/menu_ui.rs");
const MENU_KEYS: &str = include_str!("../../element-51-web/src/menu_keys.rs");
const THEME_UI: &str = include_str!("../../element-51-web/src/theme_ui.rs");
const PAPER_UI: &str = include_str!("../../element-51-web/src/paper_ui.rs");
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
fn the_default_theme_is_purple_synthwave() {
    assert_eq!(Theme::default().name, "purple-synthwave");
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
fn a_wire_is_see_through_over_a_card_in_every_theme_and_a_file_can_change_it() {
    for name in THEME_NAMES {
        let theme = Theme::builtin(name).unwrap();
        let opacity = theme.metrics.wire_over_card;
        assert!(opacity > 0.0 && opacity < 1.0, "{name}: {opacity}");
    }
    let theme = changed("antimony", r#"{"metrics": {"wire_over_card": 0.25}}"#).unwrap();
    assert_eq!(theme.metrics.wire_over_card, 0.25);
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
    for (name, text) in [
        ("draw.rs", DRAW),
        ("draw_card.rs", DRAW_CARD),
        ("draw_chart.rs", DRAW_CHART),
        ("draw_focus.rs", DRAW_FOCUS),
        ("draw_wires.rs", DRAW_WIRES),
        ("lib.rs", WEB),
        ("run_ui.rs", RUN_UI),
        ("canvas_ui.rs", CANVAS_UI),
        ("edit_ui.rs", EDIT_UI),
        ("messages.rs", MESSAGES),
        ("fields_ui.rs", FIELDS_UI),
        ("input.rs", INPUT),
        ("menu_ui.rs", MENU_UI),
        ("menu_keys.rs", MENU_KEYS),
        ("theme_ui.rs", THEME_UI),
        ("paper_ui.rs", PAPER_UI),
        ("index.html", PAGE),
    ] {
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

const EXAMPLE_THEME: &str = include_str!("../../../examples/e_look/my_theme.json");
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
    for kind in theme["ports"].as_object().unwrap().keys() {
        assert!(
            THEME_DOCS.contains(&format!("| `{kind}` |")),
            "docs/themes.md has no row for port type {kind}"
        );
    }
    for shape in ["box", "circle", "diamond"] {
        assert!(
            THEME_DOCS.contains(&format!("`{shape}`")),
            "no mention of {shape}"
        );
    }
    for section in ["colours", "metrics", "page", "fonts"] {
        for key in theme[section].as_object().unwrap().keys() {
            assert!(
                THEME_DOCS.contains(&format!("| `{key}` |")),
                "docs/themes.md has no row for {section} key {key}"
            );
        }
    }
}

#[test]
fn every_port_type_has_a_style_for_each_side_and_by_default_all_are_boxes_but_pictures_are_diamonds_and_series_circles()
 {
    use element_51_core::PortType;
    use element_51_core::theme::PortShape;
    for name in THEME_NAMES {
        let theme = Theme::builtin(name).unwrap();
        for kind in [
            PortType::Float,
            PortType::Int,
            PortType::Str,
            PortType::Bool,
            PortType::Image,
            PortType::Series,
        ] {
            for output in [false, true] {
                let style = theme.port_style(kind, output);
                let shape = match kind {
                    PortType::Image => PortShape::Diamond,
                    PortType::Series => PortShape::Circle,
                    _ => PortShape::Box,
                };
                assert_eq!(style.shape, shape, "{name} {kind} {output}");
                assert!(!style.colour.is_empty());
            }
        }
    }
}

#[test]
fn by_default_an_input_and_an_output_of_one_type_share_a_colour() {
    use element_51_core::PortType;
    let theme = Theme::default();
    for kind in [
        PortType::Float,
        PortType::Int,
        PortType::Str,
        PortType::Bool,
        PortType::Image,
        PortType::Series,
    ] {
        assert_eq!(
            theme.port_style(kind, false).colour,
            theme.port_style(kind, true).colour
        );
    }
    assert_eq!(
        theme.port_style(PortType::Int, true),
        theme.port_style(PortType::Float, true)
    );
}

#[test]
fn a_change_reaches_one_side_of_one_type_and_leaves_the_rest() {
    use element_51_core::PortType;
    use element_51_core::theme::PortShape;
    let base = Theme::default();
    let theme = changed(
        "antimony",
        r##"{"ports": {"float": {"out": {"shape": "circle"},
                                "in": {"colour": "#ff0000", "shape": "diamond"}}}}"##,
    )
    .unwrap();
    let out = theme.port_style(PortType::Float, true);
    assert_eq!(out.shape, PortShape::Circle);
    assert_eq!(out.colour, base.port_style(PortType::Float, true).colour);
    let input = theme.port_style(PortType::Float, false);
    assert_eq!(
        (input.colour.as_str(), input.shape),
        ("#ff0000", PortShape::Diamond)
    );
    assert_eq!(
        theme.port_style(PortType::Int, false),
        base.port_style(PortType::Int, false)
    );
}

#[test]
fn a_port_change_with_a_wrong_name_says_where_and_lists_the_names() {
    let error = changed("antimony", r#"{"ports": {"number": {}}}"#).unwrap_err();
    assert!(
        error.contains("ports") && error.contains("number") && error.contains("float"),
        "{error}"
    );
    let error = changed("antimony", r#"{"ports": {"float": {"output": {}}}}"#).unwrap_err();
    assert!(
        error.contains("ports.float") && error.contains("output") && error.contains("out"),
        "{error}"
    );
    let error = changed(
        "antimony",
        r#"{"ports": {"float": {"in": {"color": "red"}}}}"#,
    )
    .unwrap_err();
    assert!(
        error.contains("ports.float.in") && error.contains("colour"),
        "{error}"
    );
}

#[test]
fn a_port_shape_must_be_one_of_the_known_shapes() {
    let error = changed(
        "antimony",
        r#"{"ports": {"bool": {"in": {"shape": "star"}}}}"#,
    )
    .unwrap_err();
    assert!(
        error.contains("star") && error.contains("diamond"),
        "{error}"
    );
    let error = changed("antimony", r#"{"ports": {"bool": {"in": {"shape": 3}}}}"#).unwrap_err();
    assert!(error.contains("shape") && error.contains("text"), "{error}");
    let error = changed("antimony", r#"{"ports": {"bool": {"in": "box"}}}"#).unwrap_err();
    assert!(
        error.contains("ports.bool") && error.contains("object"),
        "{error}"
    );
}

fn declared(theme: &Theme) -> Vec<String> {
    theme
        .css_variables()
        .into_iter()
        .map(|(name, _)| name)
        .collect()
}

#[test]
fn every_variable_the_page_reads_is_one_the_theme_provides() {
    let declared = declared(&Theme::default());
    let mut rest = PAGE;
    while let Some(at) = rest.find("var(--") {
        rest = &rest[at + 4..];
        let name: String = rest
            .chars()
            .take_while(|c| c.is_ascii_lowercase() || *c == '-')
            .collect();
        assert!(
            declared.contains(&name),
            "the page reads {name}, which no theme sets"
        );
    }
}

#[test]
fn the_page_has_no_font_size_or_radius_written_in_pixels() {
    for property in ["font:", "font-size:", "border-radius:"] {
        let mut rest = PAGE;
        while let Some(at) = rest.find(property) {
            rest = &rest[at + property.len()..];
            let value = &rest[..rest.find(';').unwrap()];
            let digit_px = value
                .as_bytes()
                .windows(3)
                .any(|w| w[0].is_ascii_digit() && w[1] == b'p' && w[2] == b'x');
            assert!(
                !digit_px,
                "{property} {value} is written in the page, not the theme"
            );
        }
    }
}

#[test]
fn the_page_keys_come_out_as_pixel_values() {
    let theme = changed(
        "antimony",
        r#"{"page": {"button_radius": 12, "menu_pad": 2.5}}"#,
    )
    .unwrap();
    let value = |name: &str| {
        theme
            .css_variables()
            .into_iter()
            .find(|(key, _)| key == name)
            .map(|(_, value)| value)
    };
    assert_eq!(value("--button-radius").as_deref(), Some("12px"));
    assert_eq!(value("--menu-pad").as_deref(), Some("2.5px"));
    assert_eq!(value("--font-px").as_deref(), Some("14px"));
    assert_eq!(value("--run-bg"), Some(theme.colours.run_bg.clone()));
    assert_eq!(value("--accent"), Some(theme.colours.accent.clone()));
}

#[test]
fn a_page_key_that_does_not_exist_lists_the_page_keys() {
    let error = changed("antimony", r#"{"page": {"button_radious": 3}}"#).unwrap_err();
    assert!(
        error.contains("page") && error.contains("button_radious") && error.contains("menu_pad"),
        "{error}"
    );
    assert!(
        changed("antimony", r#"{"page": {"menu_pad": "big"}}"#)
            .unwrap_err()
            .contains("number")
    );
}

#[test]
fn the_page_values_the_soft_modern_look_did_not_change_keep_their_sizes() {
    let page = Theme::default().page;
    assert_eq!((page.font_px, page.button_font_px), (14.0, 13.0));
    assert_eq!((page.button_radius, page.field_radius), (6.0, 2.0));
    assert_eq!((page.button_x, page.button_y), (16.0, 16.0));
}

#[test]
fn the_page_stays_hidden_until_the_first_snapshot_has_set_the_theme() {
    assert!(
        PAGE.contains("html:not(.ready)"),
        "the page paints before its theme is known"
    );
    assert!(
        CANVAS_UI.contains("fn mark_ready"),
        "nothing reveals the page"
    );
    // A failure must also reveal it, or the error banner would stay invisible.
    let show_error = &WEB[WEB.find("fn show_error").unwrap()..];
    let body = &show_error[..show_error.find("\n}\n").unwrap()];
    assert!(body.contains("mark_ready"), "{body}");
}

#[test]
fn every_built_in_theme_meets_aa_wherever_it_draws_text_or_graphics() {
    for name in THEME_NAMES {
        let failures = Theme::builtin(name).unwrap().contrast_failures();
        assert!(failures.is_empty(), "{name}: {failures:?}");
    }
}

#[test]
fn a_pair_below_aa_is_named_in_the_failures() {
    let mut theme = Theme::default();
    theme.colours.placeholder = theme.colours.field.clone();
    theme.colours.selected = theme.colours.canvas.clone();
    let failures = theme.contrast_failures();
    assert_eq!(failures.len(), 2, "{failures:?}");
    assert!(failures[0].contains("placeholder on field"), "{failures:?}");
    assert!(failures[1].contains("selected on canvas"), "{failures:?}");
}

#[test]
fn a_resize_grip_that_matches_its_card_is_named_in_the_failures() {
    let mut theme = Theme::default();
    theme.colours.label = theme.colours.card.clone();
    let failures = theme.contrast_failures();
    assert!(
        failures
            .iter()
            .any(|line| line.contains("resize grip (label) on card")),
        "{failures:?}"
    );
}

#[test]
fn a_colour_that_is_not_hex_is_a_failure_too() {
    let mut theme = Theme::default();
    theme.colours.title = "red".to_string();
    let failures = theme.contrast_failures();
    assert!(
        failures.iter().any(|line| line.contains("title on header")),
        "{failures:?}"
    );
}

#[test]
fn every_page_control_has_a_visible_focus_style() {
    for selector in [
        "#run:focus-visible",
        "#menu button:focus-visible",
        "#card-menu button:focus-visible",
        "#menu-search:focus-visible",
        "#error button:focus-visible",
        "#fields input:focus-visible",
    ] {
        assert!(PAGE.contains(selector), "no focus style for {selector}");
    }
    assert!(
        PAGE.contains("var(--focus)"),
        "the focus ring does not use the theme's focus colour"
    );
}

#[test]
fn the_page_has_press_states_and_respects_reduced_motion() {
    for selector in [
        "#run:active",
        "#menu button:active",
        "#card-menu button:active",
    ] {
        assert!(PAGE.contains(selector), "no press style for {selector}");
    }
    let motion = PAGE
        .find("prefers-reduced-motion")
        .expect("no prefers-reduced-motion rule");
    let first_transition = PAGE.find("transition:").expect("no transition at all");
    assert!(
        first_transition < motion,
        "the rule must come after the transitions"
    );
    assert!(
        PAGE[motion..].contains("transition: none"),
        "reduced motion keeps transitions"
    );
}

#[test]
fn the_focus_colour_reaches_the_page_as_a_variable() {
    let theme = Theme::default();
    let variables = theme.css_variables();
    let focus = variables
        .iter()
        .find(|(name, _)| name == "--focus")
        .map(|(_, value)| value);
    assert_eq!(focus, Some(&theme.colours.focus));
}

#[test]
fn the_page_has_a_landmark_a_heading_and_names() {
    assert!(PAGE.contains("<main"), "no <main> landmark");
    assert!(PAGE.contains("<h1"), "no heading");
    assert!(
        PAGE.contains("role=\"alert\""),
        "the error banner is not announced"
    );
    assert!(
        PAGE.contains("id=\"menu-search\"") && PAGE.contains("aria-label=\"Search nodes\""),
        "the search box has no name"
    );
    let heading = PAGE.find("<h1").unwrap();
    assert!(
        PAGE[heading..].starts_with("<h1 class=\"visually-hidden\">"),
        "the heading is visible"
    );
    assert!(
        PAGE.contains(".visually-hidden"),
        "no rule hides the heading"
    );
}

#[test]
fn the_page_parts_use_the_soft_modern_look() {
    let theme = Theme::default();
    assert!(
        theme.fonts.family.starts_with("system-ui"),
        "the page font is {}",
        theme.fonts.family
    );
    assert_eq!(theme.page.menu_radius, 8.0);
    assert_eq!(theme.page.banner_radius, 8.0);
    assert!(theme.page.menu_item_radius > 0.0, "menu entries are square");
    let variables = theme.css_variables();
    for name in ["--menu-border", "--menu-item-radius"] {
        assert!(
            variables.iter().any(|(known, _)| known == name),
            "{name} does not reach the page"
        );
    }
    for rule in ["var(--menu-border)", "var(--menu-item-radius)"] {
        assert!(PAGE.contains(rule), "the page does not use {rule}");
    }
}

#[test]
fn the_focus_ring_is_the_themes_focus_colour_everywhere() {
    assert!(
        !PAGE.contains("outline: 2px solid var(--menu-hover-bg)"),
        "a menu ring still uses the hover colour"
    );
}

#[test]
fn the_page_has_a_menu_button_with_a_palette_box_and_a_theme_list() {
    for needle in [
        "id=\"app-menu-button\"",
        "aria-label=\"Menu\"",
        "aria-haspopup=\"menu\"",
        "id=\"app-menu\"",
        "id=\"try-palette\"",
        ">Load palette from Coolors link<",
        "id=\"select-theme\"",
        ">Select theme<",
        "id=\"theme-menu\"",
        "id=\"palette-box\"",
        "id=\"palette-input\"",
        "aria-label=\"Coolors palette link\"",
    ] {
        assert!(PAGE.contains(needle), "the page has no {needle}");
    }
    assert!(
        PAGE.contains("#app-menu-button { position: fixed; right: var(--button-x)"),
        "the menu button sits at the right edge"
    );
    for name in THEME_NAMES {
        let item = format!("data-theme=\"{name}\"");
        assert!(PAGE.contains(&item), "the theme list has no {name}");
    }
}

#[test]
fn the_new_controls_have_focus_and_press_styles() {
    for selector in [
        "#app-menu-button:focus-visible",
        "#app-menu button:focus-visible",
        "#theme-menu button:focus-visible",
        "#palette-input:focus-visible",
        "#palette-box button:focus-visible",
        "#app-menu-button:active",
        "#app-menu button:active",
    ] {
        assert!(PAGE.contains(selector), "no style for {selector}");
    }
}

#[test]
fn the_page_code_applies_a_palette_to_the_page_theme_only() {
    assert!(
        THEME_UI.contains("with_palette"),
        "the palette box does not apply a palette"
    );
    assert!(
        !THEME_UI.contains("send(") && !THEME_UI.contains("socket"),
        "a tried palette must not be sent to Python"
    );
}
