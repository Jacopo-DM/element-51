//! The theme: every colour, size and font the editor draws with.
//!
//! A theme is plain data, read from JSON. Drawing and layout take it as an
//! argument, so nothing else in the program says what a colour or a size is.

use crate::PortType;
use crate::contrast::ratio;
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};

/// The names of the built-in themes, the default first.
pub const THEME_NAMES: [&str; 3] = ["antimony", "antimony-light", "purple-synthwave"];

/// The theme the editor starts in when nothing picks another.
pub const DEFAULT_THEME: &str = "purple-synthwave";

const ANTIMONY: &str = include_str!("../themes/antimony.json");
const ANTIMONY_LIGHT: &str = include_str!("../themes/antimony-light.json");
const PURPLE_SYNTHWAVE: &str = include_str!("../themes/purple-synthwave.json");

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
    /// The line round a node card whose last run failed.
    pub card_error_border: String,
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
    /// Muted text in the right-click menu, such as `No matching node`.
    pub menu_muted: String,
    /// The line round the right-click menu, its flyouts and its search box.
    pub menu_border: String,
    /// The search box in the right-click menu.
    pub menu_field: String,
    /// The shadow under the right-click menu.
    pub menu_shadow: String,
    /// The menu entry under the pointer.
    pub menu_hover: String,
    /// Text of the menu entry under the pointer.
    pub menu_hover_text: String,
    /// A Python keyword (`def`, `return`, `None`) in the source panel.
    pub code_keyword: String,
    /// A string, docstrings included, in the source panel.
    pub code_string: String,
    /// A comment in the source panel.
    pub code_comment: String,
    /// A number in the source panel.
    pub code_number: String,
    /// A decorator, or the name after `def` or `class`, in the source panel.
    pub code_name: String,
    /// The Run button's background.
    pub run_bg: String,
    /// The Run button's text and triangle.
    pub run_text: String,
    /// The line round the Run button.
    pub run_border: String,
    /// The Run button's background under the pointer.
    pub run_hover: String,
    /// The Run button's text under the pointer.
    pub run_hover_text: String,
    /// The ring round the control that has the keyboard focus.
    pub focus: String,
    /// The tint of a ticked checkbox in a value box.
    pub accent: String,
    /// The error banner's background.
    pub error: String,
    /// Text in the error banner.
    pub error_text: String,
}

/// The shape of a port's marker.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum PortShape {
    Box,
    Circle,
    Diamond,
}

/// How one port marker looks.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PortStyle {
    /// The marker's colour; the wire leaving an output takes it too.
    pub colour: String,
    /// The marker's shape: `box`, `circle` or `diamond`.
    pub shape: PortShape,
}

/// The styles of the inputs and the outputs of one port type.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PortSides {
    /// An input of this type.
    #[serde(rename = "in")]
    pub input: PortStyle,
    /// An output of this type.
    #[serde(rename = "out")]
    pub output: PortStyle,
}

/// The port styles, by port type.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Ports {
    /// Integer ports.
    pub int: PortSides,
    /// Floating point ports.
    pub float: PortSides,
    /// Text ports.
    pub text: PortSides,
    /// Bool ports.
    pub bool: PortSides,
    /// Picture ports.
    pub image: PortSides,
    /// Series ports.
    pub series: PortSides,
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
    /// How solid a wire is where it crosses a card: 0 shows only the card, 1
    /// shows only the wire. Off the cards a wire is always solid.
    pub wire_over_card: f64,
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

/// Sizes of the page's own parts (menus, Run button, input boxes, error
/// banner), in CSS pixels.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Page {
    /// Size of text in the menus, the search box and the error banner.
    pub font_px: f64,
    /// Size of the text on the Run button.
    pub button_font_px: f64,
    /// Radius of a menu's corners.
    pub menu_radius: f64,
    /// Radius of the highlight on a menu entry.
    pub menu_item_radius: f64,
    /// Space above and below the entries of a menu.
    pub menu_pad: f64,
    /// Space left and right of a menu entry's text.
    pub menu_item_pad_x: f64,
    /// Space above and below a menu entry's text.
    pub menu_item_pad_y: f64,
    /// Narrowest a menu may be.
    pub menu_min_width: f64,
    /// How far a menu's shadow spreads.
    pub shadow_blur: f64,
    /// How far a menu's shadow sits below it.
    pub shadow_y: f64,
    /// Radius of the Run button's corners.
    pub button_radius: f64,
    /// Space left and right of the text on the Run button.
    pub button_pad_x: f64,
    /// Space above and below the text on the Run button.
    pub button_pad_y: f64,
    /// Distance of the Run button from the left edge of the window.
    pub button_x: f64,
    /// Distance of the Run button from the top edge of the window.
    pub button_y: f64,
    /// Space between the Run button's triangle and its text.
    pub button_gap: f64,
    /// How far the Run button's shadow spreads.
    pub button_shadow_blur: f64,
    /// How far the Run button's shadow sits below it.
    pub button_shadow_y: f64,
    /// Radius of the input boxes' corners.
    pub field_radius: f64,
    /// Radius of the menu search box's corners.
    pub search_radius: f64,
    /// Radius of the error banner's corners.
    pub banner_radius: f64,
    /// Distance of the error banner from the window's edges.
    pub banner_margin: f64,
}

/// A complete look.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Theme {
    pub name: String,
    pub colours: Colours,
    pub ports: Ports,
    pub metrics: Metrics,
    pub page: Page,
    pub fonts: Fonts,
}

impl Theme {
    /// The built-in theme called `name`, if there is one.
    pub fn builtin(name: &str) -> Option<Theme> {
        let text = match name {
            "antimony" => ANTIMONY,
            "antimony-light" => ANTIMONY_LIGHT,
            "purple-synthwave" => PURPLE_SYNTHWAVE,
            _ => return None,
        };
        Some(serde_json::from_str(text).expect("a built-in theme is valid JSON"))
    }

    /// The built-in theme `base` with some of its values replaced.
    ///
    /// `changes` is an object with up to four members, `colours`, `ports`,
    /// `metrics` and `fonts`, each an object of the keys to replace (`ports`
    /// nests: type, then `in` or `out`, then `colour` or `shape`). A value must have the
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
            let known =
                ["colours", "ports", "metrics", "page", "fonts"].contains(&section.as_str());
            let current = whole.get_mut(section).and_then(Value::as_object_mut);
            let (true, Some(current)) = (known, current) else {
                return Err(format!(
                    "unknown theme section {section:?}; the sections are: colours, ports, metrics, page, fonts"
                ));
            };
            let Value::Object(entries) = entries else {
                return Err(format!("{section} must be an object of changes"));
            };
            merge(current, entries, section)?;
        }
        serde_json::from_value(whole).map_err(|error| error.to_string())
    }

    /// The style of an input (`output` false) or an output port of type `kind`.
    pub fn port_style(&self, kind: PortType, output: bool) -> &PortStyle {
        let sides = match kind {
            PortType::Int => &self.ports.int,
            PortType::Float => &self.ports.float,
            PortType::Str => &self.ports.text,
            PortType::Bool => &self.ports.bool,
            PortType::Image => &self.ports.image,
            PortType::Series => &self.ports.series,
        };
        if output { &sides.output } else { &sides.input }
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
            ("--menu-muted", &colours.menu_muted),
            ("--menu-border", &colours.menu_border),
            ("--menu-field-bg", &colours.menu_field),
            ("--menu-shadow", &colours.menu_shadow),
            ("--menu-hover-bg", &colours.menu_hover),
            ("--menu-hover-text", &colours.menu_hover_text),
            ("--code-keyword", &colours.code_keyword),
            ("--code-string", &colours.code_string),
            ("--code-comment", &colours.code_comment),
            ("--code-number", &colours.code_number),
            ("--code-name", &colours.code_name),
            ("--error-bg", &colours.error),
            ("--error-text", &colours.error_text),
            ("--font", &self.fonts.family),
            ("--value-font", &self.fonts.value_family),
            ("--run-bg", &colours.run_bg),
            ("--run-text", &colours.run_text),
            ("--run-border", &colours.run_border),
            ("--run-hover-bg", &colours.run_hover),
            ("--run-hover-text", &colours.run_hover_text),
            ("--accent", &colours.accent),
            ("--focus", &colours.focus),
        ]
        .into_iter()
        .map(|(name, value)| (name.to_string(), value.clone()))
        .chain(self.page_variables())
        .collect()
    }

    /// The page sizes as CSS custom properties, in pixels.
    fn page_variables(&self) -> Vec<(String, String)> {
        let page = &self.page;
        [
            ("--font-px", page.font_px),
            ("--button-font-px", page.button_font_px),
            ("--menu-radius", page.menu_radius),
            ("--menu-item-radius", page.menu_item_radius),
            ("--menu-pad", page.menu_pad),
            ("--menu-item-pad-x", page.menu_item_pad_x),
            ("--menu-item-pad-y", page.menu_item_pad_y),
            ("--menu-min-width", page.menu_min_width),
            ("--shadow-blur", page.shadow_blur),
            ("--shadow-y", page.shadow_y),
            ("--button-radius", page.button_radius),
            ("--button-pad-x", page.button_pad_x),
            ("--button-pad-y", page.button_pad_y),
            ("--button-x", page.button_x),
            ("--button-y", page.button_y),
            ("--button-gap", page.button_gap),
            ("--button-shadow-blur", page.button_shadow_blur),
            ("--button-shadow-y", page.button_shadow_y),
            ("--field-radius", page.field_radius),
            ("--search-radius", page.search_radius),
            ("--banner-radius", page.banner_radius),
            ("--banner-margin", page.banner_margin),
        ]
        .into_iter()
        .map(|(name, value)| (name.to_string(), format!("{value}px")))
        .collect()
    }
}

/// WCAG 2.2 AA for text.
pub const TEXT_RATIO: f64 = 4.5;
/// WCAG 2.2 AA for icons, outlines and other graphics.
pub const GRAPHIC_RATIO: f64 = 3.0;

impl Theme {
    /// The pairs of colours the editor draws text or graphics in, that fall
    /// below WCAG 2.2 AA, each as a line that names the pair and its ratio.
    /// Nothing is returned when every pair passes.
    pub fn contrast_failures(&self) -> Vec<String> {
        let mut failures = Vec::new();
        for (what, front, back, needed) in self.contrast_pairs() {
            match ratio(&front, &back) {
                Some(value) if value >= needed => {}
                Some(value) => failures.push(format!("{what} is {value:.2}:1, needs {needed}:1")),
                None => failures.push(format!("{what}: a colour is not #rrggbb")),
            }
        }
        failures
    }

    /// Every pair as (name, foreground, background, the ratio it needs): text
    /// pairs first, then graphics.
    fn contrast_pairs(&self) -> Vec<(String, String, String, f64)> {
        let c = &self.colours;
        let text = |what: &str, front: &String, back: &String| {
            (what.to_string(), front.clone(), back.clone(), TEXT_RATIO)
        };
        let graphic = |what: String, front: &String, back: &String| {
            (what, front.clone(), back.clone(), GRAPHIC_RATIO)
        };
        let mut pairs = vec![
            text("title on header", &c.title, &c.header),
            text("title on header_error", &c.title, &c.header_error),
            text("text_dim on header_skipped", &c.text_dim, &c.header_skipped),
            text("label on card", &c.label, &c.card),
            text("field_text on field", &c.field_text, &c.field),
            text("placeholder on field", &c.placeholder, &c.field),
            text("menu_text on menu", &c.menu_text, &c.menu),
            text("menu_muted on menu", &c.menu_muted, &c.menu),
            text("menu_text on menu_field", &c.menu_text, &c.menu_field),
            text("code_keyword on menu_field", &c.code_keyword, &c.menu_field),
            text("code_string on menu_field", &c.code_string, &c.menu_field),
            text("code_comment on menu_field", &c.code_comment, &c.menu_field),
            text("code_number on menu_field", &c.code_number, &c.menu_field),
            text("code_name on menu_field", &c.code_name, &c.menu_field),
            text(
                "menu_hover_text on menu_hover",
                &c.menu_hover_text,
                &c.menu_hover,
            ),
            text("run_text on run_bg", &c.run_text, &c.run_bg),
            text(
                "run_hover_text on run_hover",
                &c.run_hover_text,
                &c.run_hover,
            ),
            text("error_text on error", &c.error_text, &c.error),
            graphic("icon on header".to_string(), &c.icon, &c.header),
            graphic("resize grip (label) on card".to_string(), &c.label, &c.card),
            graphic("selected on canvas".to_string(), &c.selected, &c.canvas),
            graphic(
                "card_error_border on canvas".to_string(),
                &c.card_error_border,
                &c.canvas,
            ),
            graphic(
                "selected_wire on canvas".to_string(),
                &c.selected_wire,
                &c.canvas,
            ),
            graphic("allowed on canvas".to_string(), &c.allowed, &c.canvas),
            graphic("refused on canvas".to_string(), &c.refused, &c.canvas),
            graphic("focus on canvas".to_string(), &c.focus, &c.canvas),
            graphic("focus on menu".to_string(), &c.focus, &c.menu),
            graphic("focus on menu_field".to_string(), &c.focus, &c.menu_field),
        ];
        for (kind, name) in [
            (PortType::Int, "int"),
            (PortType::Float, "float"),
            (PortType::Str, "text"),
            (PortType::Bool, "bool"),
            (PortType::Image, "image"),
            (PortType::Series, "series"),
        ] {
            for (output, side) in [(false, "in"), (true, "out")] {
                let colour = &self.port_style(kind, output).colour;
                pairs.push(graphic(
                    format!("port {name} {side} on card"),
                    colour,
                    &c.card,
                ));
            }
        }
        pairs
    }
}

impl Default for Theme {
    /// The `purple-synthwave` theme.
    fn default() -> Theme {
        Theme::builtin(DEFAULT_THEME).expect("the default theme exists")
    }
}

/// Replace the values in `current` with those in `entries`, level by level.
/// `path` names the level in an error, such as `ports.float.in`.
fn merge(
    current: &mut Map<String, Value>,
    entries: &Map<String, Value>,
    path: &str,
) -> Result<(), String> {
    for (key, value) in entries {
        let Some(old) = current.get_mut(key) else {
            let keys: Vec<&str> = current.keys().map(String::as_str).collect();
            return Err(format!(
                "unknown {path} key {key:?}; the keys are: {}",
                keys.join(", ")
            ));
        };
        if let Some(inner) = old.as_object_mut() {
            let Value::Object(changes) = value else {
                return Err(format!("{path}.{key} must be an object of changes"));
            };
            merge(inner, changes, &format!("{path}.{key}"))?;
        } else if std::mem::discriminant(old) == std::mem::discriminant(value) {
            *old = value.clone();
        } else {
            let kind = if old.is_number() { "a number" } else { "text" };
            return Err(format!("{path} key {key:?} must be {kind}"));
        }
    }
    Ok(())
}
