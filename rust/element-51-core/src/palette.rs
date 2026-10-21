//! Try a Coolors palette link as the theme's colours.
//!
//! The link carries the colours (`https://coolors.co/acadbc-9b9ece-...`), so
//! nothing is fetched. They are sorted by lightness, whatever order the link
//! gives: the darkest becomes the canvas, the next dark one the cards and menus,
//! a mid tone the title bars and borders, the lightest the text, and the most
//! saturated of the rest the accent. Every text and graphic colour is then
//! nudged lighter or darker until it meets WCAG AA, so any palette is readable.

use crate::contrast::{luminance, parse_hex, ratio};
use crate::theme::{GRAPHIC_RATIO, TEXT_RATIO, Theme};

/// The fewest colours a palette may have.
const MIN_COLOURS: usize = 2;
/// The most colours a palette may have (Coolors makes at most ten).
const MAX_COLOURS: usize = 10;
/// How bright a card or menu may be, as relative luminance: dark enough that
/// light text on it reads.
const CARD_BRIGHTNESS: f64 = 0.05;
/// How bright a title bar may be.
const HEADER_BRIGHTNESS: f64 = 0.2;

type Rgb = (u8, u8, u8);

/// The colours of a Coolors link as `#rrggbb`, in the order the link gives them.
///
/// The link may have `http` or `https`, `www.`, a `palette/` part, a trailing
/// slash, a query or a fragment. Anything else is an error that says what is
/// wrong.
pub fn parse_link(text: &str) -> Result<Vec<String>, String> {
    let text = text.trim();
    if text.is_empty() {
        return Err("Paste a Coolors link, such as https://coolors.co/acadbc-9b9ece".to_string());
    }
    let text = text
        .strip_prefix("https://")
        .or_else(|| text.strip_prefix("http://"))
        .unwrap_or(text);
    let text = text.strip_prefix("www.").unwrap_or(text);
    let rest = text
        .strip_prefix("coolors.co")
        .filter(|rest| rest.is_empty() || rest.starts_with(['/', '?', '#']));
    let Some(rest) = rest else {
        return Err(
            "That is not a Coolors link: it should start with https://coolors.co/".to_string(),
        );
    };
    let path = rest
        .split(['?', '#'])
        .next()
        .unwrap_or("")
        .trim_matches('/');
    let path = path.strip_prefix("palette/").unwrap_or(path);
    let tokens: Vec<&str> = path.split('-').filter(|token| !token.is_empty()).collect();
    let is_hex = |token: &&str| token.len() == 6 && token.chars().all(|c| c.is_ascii_hexdigit());
    if !tokens.iter().any(&is_hex) {
        return Err("That link has no colours in it".to_string());
    }
    if let Some(bad) = tokens.iter().find(|token| !is_hex(token)) {
        return Err(format!(
            "'{bad}' is not a colour: each one is six hex digits"
        ));
    }
    match tokens.len() {
        count if count < MIN_COLOURS => Err(format!(
            "A palette needs at least {MIN_COLOURS} colours, this link has {count}"
        )),
        count if count > MAX_COLOURS => Err(format!(
            "A palette may have at most {MAX_COLOURS} colours, this link has {count}"
        )),
        _ => Ok(tokens
            .iter()
            .map(|token| format!("#{}", token.to_lowercase()))
            .collect()),
    }
}

/// `theme` with its colours taken from the palette link, or the reason the link
/// cannot be used. Only the colours change; shapes, sizes and fonts stay.
pub fn with_palette(theme: &Theme, link: &str) -> Result<Theme, String> {
    let mut sorted: Vec<(f64, String)> = parse_link(link)?
        .into_iter()
        .map(|hex| {
            (
                luminance(parse_hex(&hex).expect("parse_link checked it")),
                hex,
            )
        })
        .collect();
    sorted.sort_by(|a, b| a.0.total_cmp(&b.0).then_with(|| a.1.cmp(&b.1)));
    let ordered: Vec<String> = sorted.into_iter().map(|(_, hex)| hex).collect();
    let count = ordered.len();
    let (canvas, text) = (ordered[0].clone(), ordered[count - 1].clone());
    let accent = most_saturated(&ordered[1..count - 1]).unwrap_or_else(|| text.clone());
    let next_dark = if count >= 3 {
        ordered[1].clone()
    } else {
        mix(&canvas, &text, 0.12)
    };
    let card = darken_to(&next_dark, &canvas, CARD_BRIGHTNESS);
    let header_source = if count >= 4 {
        ordered[2].clone()
    } else {
        mix(&card, &text, 0.3)
    };
    // The title bar darkens until the palette's text reads on it.
    let header = ensure(
        &darken_to(&header_source, &canvas, HEADER_BRIGHTNESS),
        &[&text],
        TEXT_RATIO,
    );

    let mut out = theme.clone();
    let c = &mut out.colours;
    c.canvas = canvas.clone();
    c.card = card.clone();
    c.header = header.clone();
    c.card_border = header.clone();
    c.dot = mix(&canvas, &card, 0.5);
    c.header_skipped = mix(&card, &header, 0.5);
    c.field = mix(&card, &canvas, 0.5);
    c.field_border = c.field.clone();
    c.menu = card.clone();
    c.menu_border = mix(&card, &header, 0.5);
    c.menu_field = mix(&c.menu, &canvas, 0.4);
    c.menu_hover = mix(&c.menu, &text, 0.12);
    c.run_bg = accent.clone();
    c.run_border = accent.clone();
    c.run_hover = mix(&accent, "#000000", 0.15);
    c.accent = accent.clone();

    let menu = c.menu.clone();
    c.title = ensure(&text, &[&c.header], TEXT_RATIO);
    // The red bar of a failed node is the one background that moves: the title
    // keeps the palette's text colour.
    c.header_error = ensure(&c.header_error, &[&c.title], TEXT_RATIO);
    c.text_dim = ensure(
        &mix(&text, &c.header_skipped, 0.4),
        &[&c.header_skipped],
        TEXT_RATIO,
    );
    c.label = ensure(&mix(&text, &card, 0.2), &[&card], TEXT_RATIO);
    c.field_text = ensure(&text, &[&c.field], TEXT_RATIO);
    c.placeholder = ensure(&mix(&text, &c.field, 0.45), &[&c.field], TEXT_RATIO);
    c.menu_text = ensure(&text, &[&menu, &c.menu_field], TEXT_RATIO);
    c.menu_muted = ensure(&mix(&text, &menu, 0.35), &[&menu], TEXT_RATIO);
    c.menu_hover_text = ensure(&text, &[&c.menu_hover], TEXT_RATIO);
    c.run_text = ensure(&text, &[&c.run_bg], TEXT_RATIO);
    c.run_hover_text = ensure(&text, &[&c.run_hover], TEXT_RATIO);
    c.error_text = ensure(&c.error_text, &[&c.error], TEXT_RATIO);
    c.icon = ensure(&mix(&text, &header, 0.2), &[&header], GRAPHIC_RATIO);
    c.selected = ensure(&text, &[&canvas], GRAPHIC_RATIO);
    c.selected_wire = ensure(&text, &[&canvas], GRAPHIC_RATIO);
    c.allowed = ensure(&c.allowed, &[&canvas], GRAPHIC_RATIO);
    c.refused = ensure(&c.refused, &[&canvas], GRAPHIC_RATIO);
    c.focus = ensure(&accent, &[&canvas, &menu, &c.menu_field], GRAPHIC_RATIO);

    for sides in [
        &mut out.ports.int,
        &mut out.ports.float,
        &mut out.ports.text,
        &mut out.ports.bool,
        &mut out.ports.image,
        &mut out.ports.series,
    ] {
        for style in [&mut sides.input, &mut sides.output] {
            style.colour = ensure(&style.colour, &[&card], GRAPHIC_RATIO);
        }
    }
    Ok(out)
}

/// The colour with the most saturation, the earlier one if two tie.
fn most_saturated(colours: &[String]) -> Option<String> {
    let saturation = |hex: &String| {
        let (r, g, b) = parse_hex(hex).expect("a palette colour");
        let (high, low) = (r.max(g).max(b), r.min(g).min(b));
        if high == 0 {
            0.0
        } else {
            f64::from(high - low) / f64::from(high)
        }
    };
    colours
        .iter()
        .fold(None, |best: Option<&String>, next| match best {
            Some(best) if saturation(best) >= saturation(next) => Some(best),
            _ => Some(next),
        })
        .cloned()
}

/// `first` and `second` blended, `amount` of the way from `first` to `second`.
fn mix(first: &str, second: &str, amount: f64) -> String {
    let (a, b): (Rgb, Rgb) = (
        parse_hex(first).expect("a hex colour"),
        parse_hex(second).expect("a hex colour"),
    );
    let channel = |from: u8, to: u8| {
        (f64::from(from) * (1.0 - amount) + f64::from(to) * amount).round() as u8
    };
    format!(
        "#{:02x}{:02x}{:02x}",
        channel(a.0, b.0),
        channel(a.1, b.1),
        channel(a.2, b.2)
    )
}

/// `colour`, moved toward `toward` in tenths until its luminance is at most
/// `limit`; unchanged if it already is.
fn darken_to(colour: &str, toward: &str, limit: f64) -> String {
    let mut current = colour.to_string();
    for step in 1..=10 {
        if luminance(parse_hex(&current).expect("a hex colour")) <= limit {
            break;
        }
        current = mix(colour, toward, f64::from(step) / 10.0);
    }
    current
}

/// `colour`, moved toward white or black, whichever needs the smaller move,
/// until it has at least `needed` contrast against every background; unchanged
/// if it already has.
fn ensure(colour: &str, backgrounds: &[&String], needed: f64) -> String {
    let passes = |candidate: &str| {
        backgrounds
            .iter()
            .all(|background| ratio(candidate, background).is_some_and(|value| value >= needed))
    };
    if passes(colour) {
        return colour.to_string();
    }
    let mut best: Option<(u32, String)> = None;
    for target in ["#ffffff", "#000000"] {
        for step in 1..=25_u32 {
            let candidate = mix(colour, target, f64::from(step) * 0.04);
            if passes(&candidate) {
                if best.as_ref().is_none_or(|(taken, _)| step < *taken) {
                    best = Some((step, candidate));
                }
                break;
            }
        }
    }
    match best {
        Some((_, candidate)) => candidate,
        None => {
            // No tint of it works (a mid-grey background): take the extreme
            // that contrasts most with the first background.
            let first = backgrounds[0];
            let (white, black) = (ratio("#ffffff", first), ratio("#000000", first));
            if white >= black { "#ffffff" } else { "#000000" }.to_string()
        }
    }
}
