//! Colouring Python source for the source panel.

use element_51_core::highlight::{Kind, Span, highlight};

fn kinds(code: &str) -> Vec<(Kind, String)> {
    highlight(code)
        .into_iter()
        .filter(|span| span.kind != Kind::Plain)
        .map(|span| (span.kind, span.text))
        .collect()
}

fn text_of(spans: &[Span]) -> String {
    spans.iter().map(|span| span.text.as_str()).collect()
}

#[test]
fn the_runs_put_back_together_are_the_code() {
    let code = "@element_51.node\ndef training(steps: int = 60, *, blow_up: bool = False):\n    \"\"\"Make a loss.\n\n    Returns\n    \"\"\"\n    x = f'{steps}' + 1e-5  # why\n    return x\n";
    assert_eq!(text_of(&highlight(code)), code);
    assert_eq!(text_of(&highlight("")), "");
}

#[test]
fn keywords_names_numbers_and_comments_get_their_kinds() {
    let found = kinds("def add(a, b=0x1F):  # sum\n    return None\n");
    assert_eq!(
        found,
        vec![
            (Kind::Keyword, "def".to_string()),
            (Kind::Name, "add".to_string()),
            (Kind::Number, "0x1F".to_string()),
            (Kind::Comment, "# sum".to_string()),
            (Kind::Keyword, "return".to_string()),
            (Kind::Keyword, "None".to_string()),
        ]
    );
}

#[test]
fn a_decorator_is_one_name_with_its_dots() {
    assert_eq!(
        kinds("@element_51.node(category=\"Math\")\nclass A:\n    pass"),
        vec![
            (Kind::Name, "@element_51.node".to_string()),
            (Kind::String, "\"Math\"".to_string()),
            (Kind::Keyword, "class".to_string()),
            (Kind::Name, "A".to_string()),
            (Kind::Keyword, "pass".to_string()),
        ]
    );
}

#[test]
fn a_name_that_only_contains_a_keyword_is_plain() {
    assert!(kinds("define = island + format_in").is_empty());
    // A name after `def` is a name, but the word after it is not.
    assert_eq!(
        kinds("def f(): return 1"),
        vec![
            (Kind::Keyword, "def".to_string()),
            (Kind::Name, "f".to_string()),
            (Kind::Keyword, "return".to_string()),
            (Kind::Number, "1".to_string()),
        ]
    );
}

#[test]
fn numbers_take_their_exponent_and_underscores_but_not_the_next_word() {
    let found = kinds("a = 1_000 + 0.5 + .25 + 1e-5 + 2E+3\nb = x.y");
    let numbers: Vec<&str> = found.iter().map(|(_, text)| text.as_str()).collect();
    assert_eq!(numbers, vec!["1_000", "0.5", ".25", "1e-5", "2E+3"]);
    // A hex number does not take a following minus as an exponent sign.
    assert_eq!(
        kinds("0xE-1"),
        vec![
            (Kind::Number, "0xE".to_string()),
            (Kind::Number, "1".to_string()),
        ]
    );
}

#[test]
fn strings_have_prefixes_escapes_and_triple_quotes() {
    let found = kinds("a = rb'x\\'y' + \"it's\" + F\"{a}\"");
    let strings: Vec<&str> = found.iter().map(|(_, text)| text.as_str()).collect();
    assert_eq!(strings, vec!["rb'x\\'y'", "\"it's\"", "F\"{a}\""]);
    let docstring = "\"\"\"One\n  two \\\"\"\" three\"\"\" + 1";
    assert_eq!(
        kinds(docstring),
        vec![
            (
                Kind::String,
                "\"\"\"One\n  two \\\"\"\" three\"\"\"".to_string()
            ),
            (Kind::Number, "1".to_string()),
        ]
    );
}

#[test]
fn a_word_before_a_quote_that_is_not_a_prefix_stays_a_word() {
    assert_eq!(kinds("print'x'"), vec![(Kind::String, "'x'".to_string())]);
}

#[test]
fn half_typed_code_still_shows() {
    // An open single-quote string ends with its line, an open triple quote at
    // the end of the code, and a lone quote at the end is a string.
    assert_eq!(
        kinds("a = 'open\nb = 2"),
        vec![
            (Kind::String, "'open".to_string()),
            (Kind::Number, "2".to_string()),
        ]
    );
    assert_eq!(
        kinds("\"\"\"never closed\nmore"),
        vec![(Kind::String, "\"\"\"never closed\nmore".to_string())]
    );
    assert_eq!(kinds("x = '"), vec![(Kind::String, "'".to_string())]);
    assert_eq!(kinds("\"\\"), vec![(Kind::String, "\"\\".to_string())]);
    assert_eq!(
        kinds("\"\"\"a\\"),
        vec![(Kind::String, "\"\"\"a\\".to_string())]
    );
}

#[test]
fn a_comment_ends_with_its_line_and_text_in_it_is_not_coloured_again() {
    assert_eq!(
        kinds("# def x 1 'a'\nx"),
        vec![(Kind::Comment, "# def x 1 'a'".to_string())]
    );
    assert_eq!(
        kinds("s = '# not a comment'"),
        vec![(Kind::String, "'# not a comment'".to_string())]
    );
}

#[test]
fn text_that_is_not_python_is_left_plain_and_whole() {
    let spans = highlight("é ∑ @ . → x");
    assert_eq!(text_of(&spans), "é ∑ @ . → x");
    assert!(spans.iter().all(|span| span.kind == Kind::Plain));
    assert_eq!(spans.len(), 1);
}

#[test]
fn every_coloured_kind_has_a_class_and_plain_has_none() {
    assert_eq!(Kind::Plain.class(), None);
    let classes: Vec<_> = [
        Kind::Keyword,
        Kind::String,
        Kind::Comment,
        Kind::Number,
        Kind::Name,
    ]
    .iter()
    .map(|kind| kind.class().unwrap())
    .collect();
    assert_eq!(
        classes,
        vec![
            "tok-keyword",
            "tok-string",
            "tok-comment",
            "tok-number",
            "tok-name"
        ]
    );
}
