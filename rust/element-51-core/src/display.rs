//! What a card shows below its ports: where the picture goes and how it fits.

use crate::layout::Rect;
use crate::theme::Metrics;
use crate::{CardDisplay, NodeType};

/// The side of the square picture area: the card's width less the inset on
/// each side.
pub fn picture_side(metrics: &Metrics) -> f64 {
    metrics.node_width - 2.0 * metrics.title_pad
}

/// How much taller a card of this type is because of what it shows: the
/// picture area and a margin under it. It depends on the type only, so a card
/// does not change size when results arrive.
pub fn display_height(node_type: &NodeType, metrics: &Metrics) -> f64 {
    match node_type.display {
        CardDisplay::None => 0.0,
        CardDisplay::Image => picture_side(metrics) + metrics.title_pad,
    }
}

/// The picture area of the card at `card`, below the port rows. None if the
/// node type shows nothing.
pub fn picture_rect(card: &Rect, node_type: &NodeType, metrics: &Metrics) -> Option<Rect> {
    if node_type.display != CardDisplay::Image {
        return None;
    }
    let rows = node_type.inputs.len() + node_type.outputs.len();
    let side = picture_side(metrics);
    Some(Rect {
        x: card.x + metrics.title_pad,
        y: card.y + metrics.header_height + metrics.row_height * rows as f64,
        w: side,
        h: side,
    })
}

/// The largest rectangle of a `width` by `height` picture's shape that fits in
/// `area`, centred in it.
pub fn fit_picture(area: &Rect, width: u32, height: u32) -> Rect {
    let scale = (area.w / f64::from(width)).min(area.h / f64::from(height));
    let (w, h) = (f64::from(width) * scale, f64::from(height) * scale);
    Rect {
        x: area.x + (area.w - w) / 2.0,
        y: area.y + (area.h - h) / 2.0,
        w,
        h,
    }
}
