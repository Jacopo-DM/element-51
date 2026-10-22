//! WCAG contrast between two colours written as `#rrggbb`.

/// The red, green and blue of a `#rrggbb` colour, in either case; anything
/// else, including `#rgb` and colour names, gives nothing.
pub fn parse_hex(text: &str) -> Option<(u8, u8, u8)> {
    let digits = text.strip_prefix('#')?;
    if digits.len() != 6 || !digits.is_ascii() {
        return None;
    }
    let part = |range: std::ops::Range<usize>| u8::from_str_radix(&digits[range], 16).ok();
    Some((part(0..2)?, part(2..4)?, part(4..6)?))
}

/// The relative luminance of a colour, from 0 (black) to 1 (white).
pub fn luminance((red, green, blue): (u8, u8, u8)) -> f64 {
    let linear = |channel: u8| {
        let value = f64::from(channel) / 255.0;
        if value <= 0.039_28 {
            value / 12.92
        } else {
            ((value + 0.055) / 1.055).powf(2.4)
        }
    };
    0.2126 * linear(red) + 0.7152 * linear(green) + 0.0722 * linear(blue)
}

/// The WCAG contrast ratio of two `#rrggbb` colours, from 1 to 21, or nothing
/// if either is not one. It is the same whichever is in front.
pub fn ratio(first: &str, second: &str) -> Option<f64> {
    let (a, b) = (luminance(parse_hex(first)?), luminance(parse_hex(second)?));
    let (lighter, darker) = if a >= b { (a, b) } else { (b, a) };
    Some((lighter + 0.05) / (darker + 0.05))
}

/// The least contrast a graphic needs against what it is drawn on (WCAG 2.1,
/// 1.4.11).
pub const GRAPHIC_RATIO: f64 = 3.0;

/// Whether `colour` is hard to see on `background`: below the contrast a
/// graphic needs. A colour that is not `#rrggbb` is not judged.
pub fn is_low_contrast(colour: &str, background: &str) -> bool {
    ratio(colour, background).is_some_and(|ratio| ratio < GRAPHIC_RATIO)
}
