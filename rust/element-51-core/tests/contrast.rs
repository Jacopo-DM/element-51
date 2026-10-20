//! WCAG contrast ratios between two hex colours.

use element_51_core::contrast::{parse_hex, ratio};

#[test]
fn black_on_white_is_twenty_one_to_one() {
    let value = ratio("#000000", "#ffffff").unwrap();
    assert!((value - 21.0).abs() < 1e-9, "{value}");
}

#[test]
fn the_ratio_does_not_depend_on_which_colour_is_in_front() {
    assert_eq!(ratio("#9a9a9a", "#303030"), ratio("#303030", "#9a9a9a"));
}

#[test]
fn a_colour_against_itself_is_one_to_one() {
    let value = ratio("#4a90d9", "#4a90d9").unwrap();
    assert!((value - 1.0).abs() < 1e-9, "{value}");
}

#[test]
fn a_known_pair_matches_the_published_ratio() {
    // Computed with the WCAG formula: grey 9a on grey 30.
    let value = ratio("#9a9a9a", "#303030").unwrap();
    assert!((value - 4.69).abs() < 0.01, "{value}");
}

#[test]
fn hex_is_read_in_either_case_and_wrong_forms_are_refused() {
    assert_eq!(parse_hex("#FFaa00"), Some((255, 170, 0)));
    for bad in [
        "ffaa00",
        "#ffaa0",
        "#ffaa000",
        "#gg0000",
        "",
        "rgb(0, 0, 0)",
    ] {
        assert_eq!(parse_hex(bad), None, "{bad}");
        assert_eq!(ratio(bad, "#000000"), None, "{bad}");
    }
}
