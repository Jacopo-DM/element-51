//! Pictures on a card: the wire record, the card's geometry and the rules for
//! a picture port.

use element_51_core::display::{fit_picture, picture_rect, picture_side};
use element_51_core::fields::parse_input;
use element_51_core::layout::node_rect;
use element_51_core::theme::{Metrics, Theme};
use element_51_core::{
    CardDisplay, Image, Message, NodeType, PlacedNode, Port, PortType, can_link, value_fits,
};
use serde_json::json;

const SNAPSHOT: &str = include_str!("../../../tests/fixtures/snapshot.json");
const MESSAGES: &str = include_str!("../../../tests/fixtures/messages.json");

fn m() -> Metrics {
    Theme::default().metrics
}

fn card(display: CardDisplay) -> (PlacedNode, NodeType) {
    let node = PlacedNode {
        id: "n1".to_string(),
        node_type: "t".to_string(),
        x: 40.0,
        y: 60.0,
        values: Default::default(),
    };
    let kind = if display == CardDisplay::Image {
        PortType::Image
    } else {
        PortType::Float
    };
    let node_type = NodeType {
        name: "t".to_string(),
        inputs: vec![Port {
            name: "a".to_string(),
            kind: PortType::Float,
        }],
        outputs: vec![Port {
            name: "out".to_string(),
            kind,
        }],
        defaults: Default::default(),
        category: String::new(),
        display,
    };
    (node, node_type)
}

fn picture(width: u32, height: u32, channels: u8) -> Image {
    Image {
        width,
        height,
        channels,
        pixels: (0..width * height * u32::from(channels))
            .map(|n| (n % 256) as u8)
            .collect(),
    }
}

#[test]
fn the_picture_area_is_a_square_inside_the_card() {
    assert_eq!(picture_side(&m()), m().node_width - 2.0 * m().title_pad);
}

#[test]
fn a_card_with_a_picture_is_taller_by_the_area_and_a_margin() {
    let (node, plain) = card(CardDisplay::None);
    let (_, with_picture) = card(CardDisplay::Image);
    let without = node_rect(&node, &plain, &m());
    let with = node_rect(&node, &with_picture, &m());
    assert_eq!(with.h - without.h, picture_side(&m()) + m().title_pad);
    assert_eq!((with.x, with.y, with.w), (without.x, without.y, without.w));
}

#[test]
fn a_card_without_a_picture_has_no_picture_area() {
    let (node, plain) = card(CardDisplay::None);
    let rect = node_rect(&node, &plain, &m());
    assert_eq!(picture_rect(&rect, &plain, &m()), None);
}

#[test]
fn the_picture_area_sits_below_the_port_rows() {
    let (node, with_picture) = card(CardDisplay::Image);
    let rect = node_rect(&node, &with_picture, &m());
    let area = picture_rect(&rect, &with_picture, &m()).unwrap();
    assert_eq!(area.x, rect.x + m().title_pad);
    assert_eq!(area.y, rect.y + m().header_height + 2.0 * m().row_height);
    assert_eq!((area.w, area.h), (picture_side(&m()), picture_side(&m())));
    assert_eq!(area.y + area.h + m().title_pad, rect.y + rect.h);
}

#[test]
fn a_wide_picture_fills_the_width_and_is_centred_upright() {
    let (node, with_picture) = card(CardDisplay::Image);
    let rect = node_rect(&node, &with_picture, &m());
    let area = picture_rect(&rect, &with_picture, &m()).unwrap();
    let fitted = fit_picture(&area, 4, 2);
    assert_eq!(fitted.w, area.w);
    assert_eq!(fitted.h, area.h / 2.0);
    assert_eq!(fitted.x, area.x);
    assert_eq!(fitted.y, area.y + area.h / 4.0);
}

#[test]
fn a_tall_picture_fills_the_height_and_is_centred_sideways() {
    let (node, with_picture) = card(CardDisplay::Image);
    let rect = node_rect(&node, &with_picture, &m());
    let area = picture_rect(&rect, &with_picture, &m()).unwrap();
    let fitted = fit_picture(&area, 1, 4);
    assert_eq!(fitted.h, area.h);
    assert_eq!(fitted.w, area.w / 4.0);
    assert_eq!(fitted.x, area.x + area.w * 3.0 / 8.0);
    assert_eq!(fitted.y, area.y);
}

#[test]
fn a_square_picture_fills_the_area() {
    let (node, with_picture) = card(CardDisplay::Image);
    let rect = node_rect(&node, &with_picture, &m());
    let area = picture_rect(&rect, &with_picture, &m()).unwrap();
    assert_eq!(fit_picture(&area, 28, 28), area);
}

#[test]
fn a_grey_picture_becomes_opaque_rgba() {
    let image = Image {
        width: 2,
        height: 1,
        channels: 1,
        pixels: vec![10, 200],
    };
    assert_eq!(image.rgba().unwrap(), [10, 10, 10, 255, 200, 200, 200, 255]);
}

#[test]
fn a_colour_picture_becomes_opaque_rgba() {
    let image = Image {
        width: 1,
        height: 2,
        channels: 3,
        pixels: vec![1, 2, 3, 4, 5, 6],
    };
    assert_eq!(image.rgba().unwrap(), [1, 2, 3, 255, 4, 5, 6, 255]);
}

#[test]
fn a_picture_whose_pixels_do_not_fill_it_has_no_rgba() {
    let mut image = picture(3, 3, 1);
    image.pixels.pop();
    assert_eq!(image.rgba(), None);
    let odd_channels = Image {
        channels: 2,
        ..picture(2, 2, 1)
    };
    assert_eq!(odd_channels.rgba(), None);
}

#[test]
fn a_picture_is_a_list_of_integers_on_the_wire() {
    let image: Image = serde_json::from_value(json!({
        "width": 2, "height": 1, "channels": 1, "pixels": [0, 255]
    }))
    .unwrap();
    assert_eq!(image.pixels, [0, 255]);
    assert_eq!(
        serde_json::to_value(&image).unwrap(),
        json!({"width": 2, "height": 1, "channels": 1, "pixels": [0, 255]})
    );
}

#[test]
fn only_a_picture_feeds_a_picture() {
    assert!(can_link(PortType::Image, PortType::Image));
    assert!(!can_link(PortType::Image, PortType::Float));
    assert!(!can_link(PortType::Float, PortType::Image));
    assert!(!can_link(PortType::Int, PortType::Image));
}

#[test]
fn a_picture_input_has_no_box_and_takes_no_typed_value() {
    let error = parse_input("5", PortType::Image).unwrap_err();
    assert!(error.contains("linked"), "{error}");
    assert!(!value_fits(&json!(5), PortType::Image));
    assert!(!value_fits(&json!("x"), PortType::Image));
    assert!(!value_fits(&json!(true), PortType::Image));
    assert!(!value_fits(&json!(1.5), PortType::Image));
}

#[test]
fn an_empty_picture_box_means_no_value() {
    assert_eq!(parse_input("  ", PortType::Image), Ok(None));
}

#[test]
fn the_shared_snapshot_has_a_picture_node_with_a_display() {
    let Message::Snapshot(snapshot) = Message::from_json(SNAPSHOT).unwrap() else {
        panic!("not a snapshot");
    };
    let picture = snapshot.node_type("picture").unwrap();
    assert_eq!(picture.display, CardDisplay::Image);
    assert_eq!(picture.outputs[0].kind, PortType::Image);
    assert_eq!(
        snapshot.node_type("add").unwrap().display,
        CardDisplay::None
    );
}

#[test]
fn the_shared_result_shows_a_picture() {
    let fixture: serde_json::Value = serde_json::from_str(MESSAGES).unwrap();
    let Message::RunResult(result) =
        Message::from_json(&fixture["run_result_picture"].to_string()).unwrap()
    else {
        panic!("not a result");
    };
    let shown = &result.outcomes[0].shown["out"];
    assert_eq!((shown.width, shown.height, shown.channels), (2, 2, 1));
    assert_eq!(shown.pixels, [0, 85, 170, 255]);
    assert!(result.outcomes[0].outputs.is_empty());
}

#[test]
fn an_older_message_without_display_or_shown_still_reads() {
    let node_type: NodeType =
        serde_json::from_value(json!({"name": "t", "inputs": [], "outputs": []})).unwrap();
    assert_eq!(node_type.display, CardDisplay::None);
    let message = json!({"op": "run_result", "outcomes": [
        {"node": "n1", "status": "ok", "outputs": {}, "error": null}
    ]});
    let Message::RunResult(result) = Message::from_json(&message.to_string()).unwrap() else {
        panic!("not a result");
    };
    assert!(result.outcomes[0].shown.is_empty());
}
