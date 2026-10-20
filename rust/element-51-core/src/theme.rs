//! The theme: every colour, size and font the editor draws with.
//!
//! A theme is plain data, read from JSON. Drawing and layout take it as an
//! argument, so nothing else in the program says what a colour or a size is.

use serde::{Deserialize, Serialize};
use serde_json::Value;

/// The names of the built-in themes, the default first.
pub const THEME_NAMES: [&str; 2] = ["antimony", "antimony-light"];

const ANTIMONY: &str = include_str!("../themes/antimony.json");
const ANTIMONY_LIGHT: &str = include_str!("../themes/antimony-light.json");

/// Colours, as CSS colour strings.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Colours {
    /// The canvas behind the graph.
    pub canvas: String,
    /// The dots of the grid on the canvas.
    pub dot: String,
    /// A node card's body.
    pub card: String,
    /// The line round a node card.
    pub card_border: String,
    /// A node card's title bar.
    pub header: String,
    /// The title bar of a node that failed.
    pub header_error: String,
    /// The title bar of a node that was skipped.
    pub header_skipped: String,
    /// The id and type name in a title bar.
    pub title: String,
    /// Port names, and the values shown after a run.
    pub label: String,
    /// Faded text, for a node that was skipped.
    pub text_dim: String,
    /// A greyed hint in a value box: a default, a source, `output`.
    pub placeholder: String,
    /// The three-bar icon in a title bar.
    pub icon: String,
    /// The outline of a selected node.
    pub selected: String,
    /// A selected wire, which must show on the canvas.
    pub selected_wire: String,
    /// A port a dragged link may end at.
    pub allowed: String,
    /// A port a dragged link may not end at.
    pub refused: String,
    /// Number ports and the wires that leave them.
    pub port_number: String,
    /// Text ports and the wires that leave them.
    pub port_text: String,
    /// Bool ports and the wires that leave them.
    pub port_bool: String,
    /// The background of a value box.
    pub field: String,
    /// Text typed in a value box.
    pub field_text: String,
    /// The border of an input box.
    pub field_border: String,
    /// The right-click menu's background.
    pub menu: String,
    /// Text in the right-click menu.
    pub menu_text: String,
    /// The search box in the right-click menu.
    pub menu_field: String,
    /// The shadow under the right-click menu.
    pub menu_shadow: String,
    /// The menu entry under the pointer.
    pub menu_hover: String,
    /// Text of the menu entry under the pointer.
    pub menu_hover_text: String,
    /// The error banner's background.
    pub error: String,
    /// Text in the error banner.
    pub error_text: String,
}

/// Sizes, in graph units (one unit is one pixel at zoom 1).
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Metrics {
    /// Width of a node card.
    pub node_width: f64,
    /// Height of a card's title bar.
    pub header_height: f64,
    /// Height of one row of a card: an input or an output.
    pub row_height: f64,
    /// Radius of a card's corners.
    pub card_radius: f64,
    /// Thickness of the line round a card.
    pub border_width: f64,
    /// Side of a port's square marker.
    pub port_size: f64,
    /// How far a port's centre sits inside the card's edge.
    pub port_inset: f64,
    /// How close a press must be to a port's anchor to hit it.
    pub port_hit_radius: f64,
    /// Thickness of a wire.
    pub wire_width: f64,
    /// Thickness of a selected wire.
    pub wire_width_selected: f64,
    /// Thickness of the outline round a selected node.
    pub selection_width: f64,
    /// The least a wire's curve leaves sideways.
    pub wire_min_reach: f64,
    /// The most a wire's curve swings out when its ends are in one column.
    pub wire_max_swing: f64,
    /// Height of a value box.
    pub field_height: f64,
    /// Radius of a value box's corners.
    pub box_radius: f64,
    /// Space between a port and its label.
    pub port_gap: f64,
    /// Space between the labels and the value boxes.
    pub box_gap: f64,
    /// Space between a card's edge and its title.
    pub title_pad: f64,
    /// Space between text and the edge of its value box.
    pub box_pad: f64,
    /// How far below a row's or the header's centre a text baseline sits.
    pub text_baseline: f64,
    /// Width of the three-bar icon.
    pub icon_width: f64,
    /// Thickness of one bar of the icon.
    pub icon_bar: f64,
    /// Space between the bars of the icon.
    pub icon_gap: f64,
    /// Distance between grid dots.
    pub grid_step: f64,
    /// Radius of a grid dot.
    pub dot_radius: f64,
}

/// Fonts.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Fonts {
    /// The canvas font for port names and values, as a CSS font string.
    pub card: String,
    /// The canvas font for the id and type name in a title bar.
    pub title: String,
    /// The canvas font for what sits in a value box: numbers, text, links.
    pub value: String,
    /// The family for the page's own parts: menu, banner, buttons.
    pub family: String,
    /// The family for the text typed into an input box.
    pub value_family: String,
    /// The size of text in an input box at zoom 1, in pixels.
    pub field_px: f64,
}

/// A complete look.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Theme {
    pub name: String,
    pub colours: Colours,
    pub metrics: Metrics,
    pub fonts: Fonts,
}

impl Theme {
    /// The built-in theme called `name`, if there is one.
    pub fn builtin(name: &str) -> Option<Theme> {
        let text = match name {
            "antimony" => ANTIMONY,
            "antimony-light" => ANTIMONY_LIGHT,
            _ => return None,
        };
        Some(serde_json::from_str(text).expect("a built-in theme is valid JSON"))
    }

    /// The built-in theme `base` with some of its values replaced.
    ///
    /// `changes` is an object with up to three members, `colours`, `metrics`
    /// and `fonts`, each an object of the keys to replace. A value must have the
    /// same kind (text or number) as the one it replaces. `null` means no
    /// changes. The error says which name or key is wrong and what would do.
    pub fn with_changes(base: &str, changes: &Value) -> Result<Theme, String> {
        let theme = Theme::builtin(base).ok_or_else(|| {
            format!(
                "unknown theme {base:?}; the themes are: {}",
                THEME_NAMES.join(", ")
            )
        })?;
        let sections = match changes {
            Value::Null => return Ok(theme),
            Value::Object(sections) => sections,
            _ => return Err("theme changes must be an object".to_string()),
        };
        let mut whole = serde_json::to_value(&theme).map_err(|error| error.to_string())?;
        for (section, entries) in sections {
            let known = ["colours", "metrics", "fonts"].contains(&section.as_str());
            let current = whole.get_mut(section).and_then(Value::as_object_mut);
            let (true, Some(current)) = (known, current) else {
                return Err(format!(
                    "unknown theme section {section:?}; the sections are: colours, metrics, fonts"
                ));
            };
            let Value::Object(entries) = entries else {
                return Err(format!("{section} must be an object of changes"));
            };
            for (key, value) in entries {
                let Some(old) = current.get(key) else {
                    let keys: Vec<&str> = current.keys().map(String::as_str).collect();
                    return Err(format!(
                        "unknown {section} key {key:?}; the keys are: {}",
                        keys.join(", ")
                    ));
                };
                if std::mem::discriminant(old) != std::mem::discriminant(value) {
                    let kind = if old.is_number() { "a number" } else { "text" };
                    return Err(format!("{section} key {key:?} must be {kind}"));
                }
                current.insert(key.clone(), value.clone());
            }
        }
        serde_json::from_value(whole).map_err(|error| error.to_string())
    }

    /// The values the page's own parts (input boxes, menu, error banner) read
    /// as CSS custom properties, as `(name, value)` pairs.
    pub fn css_variables(&self) -> Vec<(String, String)> {
        let colours = &self.colours;
        [
            ("--canvas", &colours.canvas),
            ("--field-bg", &colours.field),
            ("--field-text", &colours.field_text),
            ("--field-border", &colours.field_border),
            ("--placeholder", &colours.placeholder),
            ("--menu-bg", &colours.menu),
            ("--menu-text", &colours.menu_text),
            ("--menu-field-bg", &colours.menu_field),
            ("--menu-shadow", &colours.menu_shadow),
            ("--menu-hover-bg", &colours.menu_hover),
            ("--menu-hover-text", &colours.menu_hover_text),
            ("--error-bg", &colours.error),
            ("--error-text", &colours.error_text),
            ("--font", &self.fonts.family),
            ("--value-font", &self.fonts.value_family),
        ]
        .into_iter()
        .map(|(name, value)| (name.to_string(), value.clone()))
        .collect()
    }
}

impl Default for Theme {
    /// The dark Antimony theme.
    fn default() -> Theme {
        Theme::builtin(THEME_NAMES[0]).expect("the default theme exists")
    }
}
