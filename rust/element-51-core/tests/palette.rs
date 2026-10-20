//! Trying a Coolors palette link as the theme's colours.

use element_51_core::contrast::{luminance, parse_hex};
use element_51_core::palette::{parse_link, with_palette};
use element_51_core::theme::{THEME_NAMES, Theme};

const EXAMPLE: &str = "https://coolors.co/acadbc-9b9ece-6665dd-473bf0-000500";

fn brightness(hex: &str) -> f64 {
    luminance(parse_hex(hex).unwrap())
}

#[test]
fn a_coolors_link_gives_its_colours_in_the_order_written() {
    let expected = ["#acadbc", "#9b9ece", "#6665dd", "#473bf0", "#000500"];
    assert_eq!(parse_link(EXAMPLE).unwrap(), expected);
}

#[test]
fn the_link_may_be_written_in_the_other_forms_coolors_uses() {
    let two = vec!["#acadbc".to_string(), "#9b9ece".to_string()];
    for text in [
        "coolors.co/acadbc-9b9ece",
        "http://www.coolors.co/ACADBC-9B9ECE",
        "https://coolors.co/palette/acadbc-9b9ece",
        "https://coolors.co/acadbc-9b9ece/",
        "  https://coolors.co/acadbc-9b9ece?ref=1#top  \n",
    ] {
        assert_eq!(parse_link(text).as_ref(), Ok(&two), "{text:?}");
    }
}

#[test]
fn anything_else_gives_an_error_that_says_what_is_wrong() {
    let cases = [
        ("", "Paste a Coolors link"),
        ("hello", "not a Coolors link"),
        ("https://example.com/acadbc-9b9ece", "not a Coolors link"),
        ("https://coolors.co/", "no colours"),
        ("https://coolors.co/generate", "no colours"),
        ("https://coolors.co/acadbc", "at least 2 colours"),
        ("https://coolors.co/acadbz-9b9ece", "acadbz"),
        (
            "https://coolors.co/000000-111111-222222-333333-444444-555555-666666-777777-888888-999999-aaaaaa",
            "at most 10 colours",
        ),
    ];
    for (text, wanted) in cases {
        let error = parse_link(text).unwrap_err();
        assert!(error.contains(wanted), "{text:?}: {error}");
    }
}

#[test]
fn the_example_palette_sets_the_canvas_the_text_and_the_accent() {
    let theme = with_palette(&Theme::default(), EXAMPLE).unwrap();
    let c = &theme.colours;
    assert_eq!(c.canvas, "#000500", "the darkest colour is the canvas");
    assert_eq!(c.title, "#acadbc", "the lightest colour is the text");
    assert_eq!(
        c.accent, "#473bf0",
        "the most saturated colour is the accent"
    );
    assert_eq!(c.run_bg, c.accent);
    assert!(brightness(&c.card) <= 0.05, "cards stay dark: {}", c.card);
    assert!(
        brightness(&c.header) <= 0.2,
        "headers stay dark: {}",
        c.header
    );
    assert_eq!(c.menu, c.card, "menus take the card colour");
}

#[test]
fn two_colours_are_enough() {
    let theme = with_palette(&Theme::default(), "coolors.co/ffffff-000000").unwrap();
    assert_eq!(theme.colours.canvas, "#000000");
    assert_eq!(theme.colours.title, "#ffffff");
    assert!(theme.contrast_failures().is_empty());
}

#[test]
fn the_order_in_the_link_does_not_matter() {
    let forward = with_palette(&Theme::default(), EXAMPLE).unwrap();
    let backward = with_palette(
        &Theme::default(),
        "https://coolors.co/000500-473bf0-6665dd-9b9ece-acadbc",
    )
    .unwrap();
    assert_eq!(forward.colours, backward.colours);
}

#[test]
fn every_palette_ends_up_readable_whatever_it_is_and_whichever_theme_it_starts_from() {
    let palettes = [
        EXAMPLE,
        "coolors.co/ffffff-f5f5f5-eeeeee",
        "coolors.co/000000-0a0a0a-111111-1a1a1a",
        "coolors.co/ff0000-cc0000-990000",
        "coolors.co/808080-818181",
        "coolors.co/ff595e-ffca3a-8ac926-1982c4-6a4c93-264653-2a9d8f-e9c46a-f4a261-e76f51",
    ];
    for name in THEME_NAMES {
        let base = Theme::builtin(name).unwrap();
        for link in palettes {
            let theme = with_palette(&base, link).unwrap();
            let failures = theme.contrast_failures();
            assert!(failures.is_empty(), "{name} with {link}: {failures:?}");
        }
    }
}

#[test]
fn a_palette_changes_colours_only() {
    let base = Theme::builtin("antimony").unwrap();
    let theme = with_palette(&base, EXAMPLE).unwrap();
    let section = |theme: &Theme, name: &str| serde_json::to_value(theme).unwrap()[name].clone();
    for name in ["metrics", "page", "fonts"] {
        assert_eq!(section(&theme, name), section(&base, name), "{name}");
    }
    for kind in ["int", "float", "text", "bool"] {
        for side in ["in", "out"] {
            assert_eq!(
                section(&theme, "ports")[kind][side]["shape"],
                section(&base, "ports")[kind][side]["shape"],
                "{kind} {side}"
            );
        }
    }
    assert_ne!(section(&theme, "colours"), section(&base, "colours"));
}

#[test]
fn a_bad_link_leaves_the_theme_as_it_was() {
    let base = Theme::default();
    assert!(with_palette(&base, "https://coolors.co/acadbc").is_err());
    assert_eq!(base.colours, Theme::default().colours);
}
