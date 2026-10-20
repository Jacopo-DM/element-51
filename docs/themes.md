# Themes

The editor ships with two themes, `antimony` (dark, the default) and
`antimony-light`. Pick one by name:

```python
element_51.serve(app, theme="antimony-light")
```

## Your own theme

A theme file is a JSON file that lists only the values you want to change. The
rest come from a built-in theme. Pass its path instead of a name:

```python
element_51.serve(app, theme="my_theme.json")
```

```json
{
    "base": "antimony",
    "colours": {"canvas": "#101820"},
    "ports": {"float": {"out": {"colour": "#ffb000", "shape": "circle"}}},
    "metrics": {"node_width": 260},
    "fonts": {
        "value": "13px Menlo, monospace",
        "value_family": "Menlo, monospace"
    }
}
```

The file has six members, all optional:

| Member | Meaning |
| --- | --- |
| `base` | The built-in theme to start from: `antimony` (the default) or `antimony-light`. |
| `colours` | Colours to replace, by key. |
| `ports` | The colour and shape of port markers, by type and side. |
| `metrics` | Sizes on the canvas to replace, by key. |
| `page` | Sizes of the menus, the Run button and the other page parts, by key. |
| `fonts` | Fonts to replace, by key. |

`examples/e_look/e2_theme_file.py` and `examples/my_theme.json` are a working pair.
The same choice works on a running app: `app.theme = "my_theme.json"` redraws
every open page.

## What goes wrong

- A path that is not a file, a file that is not JSON, a member other than the
    four above, a colour that is not text, or a size that is not a number makes
    `serve` raise `ValueError` before anything starts.
- A key that does not exist (`"canvass"`) cannot be checked in Python, because
    the keys live in the browser's code. The page shows a banner that names the
    section and the key and lists the valid keys, and it keeps the previous theme.
- A value of the right kind but a bad content, such as `"colours": {"canvas":
    "banana"}` or a font that is not installed, is handled by the browser: an
    invalid colour is ignored and a missing font falls back to the next family in
    the list.

## Every key you can change

Each table shows the key, what it affects and the built-in values. The tables list every key, so a theme file never needs one that is not here.


### `colours`

Any CSS colour: `"#1a1a1a"`, `"rgb(26, 26, 26)"`, `"rgba(0, 0, 0, 0.5)"` or a name such as `"tomato"`.

| Key | What it colours | `antimony` | `antimony-light` |
| --- | --- | --- | --- |
| `canvas` | The canvas behind the graph. | `#1a1a1a` | `#eeeeee` |
| `dot` | The dots of the grid on the canvas. | `#2c2c2c` | `#d4d4d4` |
| `card` | A node card's body. | `#202020` | `#202020` |
| `card_border` | The line round a node card. | `#505050` | `#505050` |
| `header` | A node card's title bar. | `#505050` | `#505050` |
| `header_error` | The title bar of a node that failed. | `#8a3a3a` | `#8a3a3a` |
| `header_skipped` | The title bar of a node that was skipped. | `#3a3a3a` | `#3a3a3a` |
| `title` | The id and type name in a title bar. | `#e0e0e0` | `#e0e0e0` |
| `label` | Port names, and the values shown after a run. | `#b0b0b0` | `#b0b0b0` |
| `text_dim` | Faded text, for a node that was skipped. | `#808080` | `#808080` |
| `placeholder` | A greyed hint in a value box: a default, a source, `output`. | `#505050` | `#505050` |
| `icon` | The three-bar icon in a title bar. | `#9e9e9e` | `#9e9e9e` |
| `selected` | The outline of a selected node. | `#e0e0e0` | `#e0e0e0` |
| `selected_wire` | A selected wire, which must show on the canvas. | `#e0e0e0` | `#202020` |
| `allowed` | A port a dragged link may end at. | `#6fcf6f` | `#6fcf6f` |
| `refused` | A port a dragged link may not end at. | `#e06060` | `#e06060` |
| `field` | The background of a value box. | `#303030` | `#303030` |
| `field_text` | Text typed in a value box. | `#e0e0e0` | `#e0e0e0` |
| `field_border` | The border of an input box. | `#303030` | `#303030` |
| `menu` | The right-click menu's background. | `#ececec` | `#ececec` |
| `menu_text` | Text in the right-click menu. | `#222222` | `#222222` |
| `menu_field` | The search box in the right-click menu. | `#ffffff` | `#ffffff` |
| `menu_shadow` | The shadow under the right-click menu. | `rgba(0, 0, 0, 0.5)` | `rgba(0, 0, 0, 0.5)` |
| `menu_hover` | The menu entry under the pointer. | `#4a90d9` | `#4a90d9` |
| `menu_hover_text` | Text of the menu entry under the pointer. | `#ffffff` | `#ffffff` |
| `run_bg` | The Run button's background. | `#ececec` | `#ececec` |
| `run_text` | The Run button's text and triangle. | `#222222` | `#222222` |
| `run_border` | The line round the Run button. | `#303030` | `#303030` |
| `run_hover` | The Run button's background under the pointer. | `#4a90d9` | `#4a90d9` |
| `run_hover_text` | The Run button's text under the pointer. | `#ffffff` | `#ffffff` |
| `accent` | The tint of a ticked checkbox in a value box. | `#4a90d9` | `#4a90d9` |
| `error` | The error banner's background. | `#5a2d2d` | `#5a2d2d` |
| `error_text` | Text in the error banner. | `#f0d0d0` | `#f0d0d0` |

### `ports`

Each port type has an `in` style for its inputs and an `out` style for its outputs, and each style has a `colour` and a `shape`. A wire takes the colour of the output port it leaves. You may name only the part you change: `{"float": {"out": {"shape": "circle"}}}` keeps the colour and every other port as it was.

`shape` is one of `box` (the default), `circle` or `diamond`. A dragged link also colours the input it hovers over with `allowed` or `refused` from `colours`, in that input's shape.

| Type | Holds | Default colour (in and out) | Default shape |
| --- | --- | --- | --- |
| `int` | whole numbers | `#edc27b` | `box` |
| `float` | decimal numbers | `#edc27b` | `box` |
| `text` | text | `#94a95f` | `box` |
| `bool` | true or false | `#6fa8dc` | `box` |

### `metrics`

Numbers, in canvas units: one unit is one pixel at zoom 1.

| Key | What it sets | Default |
| --- | --- | --- |
| `node_width` | Width of a node card. | `220.0` |
| `header_height` | Height of a card's title bar. | `28.0` |
| `row_height` | Height of one row of a card: an input or an output. | `26.0` |
| `card_radius` | Radius of a card's corners. | `8.0` |
| `border_width` | Thickness of the line round a card. | `2.0` |
| `port_size` | Side of a port's square marker. | `10.0` |
| `port_inset` | How far a port's centre sits inside the card's edge. | `8.0` |
| `port_hit_radius` | How close a press must be to a port's anchor to hit it. | `9.0` |
| `wire_width` | Thickness of a wire. | `3.5` |
| `wire_width_selected` | Thickness of a selected wire. | `5.0` |
| `selection_width` | Thickness of the outline round a selected node. | `2.0` |
| `wire_min_reach` | The least a wire's curve leaves sideways. | `20.0` |
| `wire_max_swing` | The most a wire's curve swings out when its ends are in one column. | `70.0` |
| `field_height` | Height of a value box. | `20.0` |
| `box_radius` | Radius of a value box's corners. | `4.0` |
| `port_gap` | Space between a port and its label. | `8.0` |
| `box_gap` | Space between the labels and the value boxes. | `8.0` |
| `title_pad` | Space between a card's edge and its title. | `10.0` |
| `box_pad` | Space between text and the edge of its value box. | `6.0` |
| `text_baseline` | How far below a row's or the header's centre a text baseline sits. | `4.5` |
| `icon_width` | Width of the three-bar icon. | `14.0` |
| `icon_bar` | Thickness of one bar of the icon. | `2.5` |
| `icon_gap` | Space between the bars of the icon. | `2.5` |
| `grid_step` | Distance between grid dots. | `24.0` |
| `dot_radius` | Radius of a grid dot. | `1.2` |

### `page`

Numbers, in CSS pixels, for the page's own parts: the menus, the Run button, the input boxes and the error banner. Borders of 1 pixel and a few fixed paddings are not themed. The checkbox for a bool input is the browser's own; `accent` tints it when ticked.

| Key | What it sets | Default |
| --- | --- | --- |
| `font_px` | Size of text in the menus, the search box and the error banner. | `14.0` |
| `button_font_px` | Size of the text on the Run button. | `13.0` |
| `menu_radius` | Radius of a menu's corners. | `4.0` |
| `menu_pad` | Space above and below the entries of a menu. | `4.0` |
| `menu_item_pad_x` | Space left and right of a menu entry's text. | `14.0` |
| `menu_item_pad_y` | Space above and below a menu entry's text. | `4.0` |
| `menu_min_width` | Narrowest a menu may be. | `160.0` |
| `shadow_blur` | How far a menu's shadow spreads. | `8.0` |
| `shadow_y` | How far a menu's shadow sits below it. | `2.0` |
| `button_radius` | Radius of the Run button's corners. | `6.0` |
| `button_pad_x` | Space left and right of the text on the Run button. | `14.0` |
| `button_pad_y` | Space above and below the text on the Run button. | `6.0` |
| `button_x` | Distance of the Run button from the left edge of the window. | `16.0` |
| `button_y` | Distance of the Run button from the top edge of the window. | `16.0` |
| `button_gap` | Space between the Run button's triangle and its text. | `8.0` |
| `button_shadow_blur` | How far the Run button's shadow spreads. | `4.0` |
| `button_shadow_y` | How far the Run button's shadow sits below it. | `1.0` |
| `field_radius` | Radius of the input boxes' corners. | `2.0` |
| `search_radius` | Radius of the menu search box's corners. | `3.0` |
| `banner_radius` | Radius of the error banner's corners. | `6.0` |
| `banner_margin` | Distance of the error banner from the window's edges. | `16.0` |

### `fonts`

`card`, `title` and `value` are CSS font strings for the canvas: size, then family, with `bold` first if wanted. `family` and `value_family` are font families only. `field_px` is a number.

| Key | What it sets | Default |
| --- | --- | --- |
| `card` | The canvas font for port names and values, as a CSS font string. | `12px Verdana, sans-serif` |
| `title` | The canvas font for the id and type name in a title bar. | `bold 12px Verdana, sans-serif` |
| `value` | The canvas font for what sits in a value box: numbers, text, links. | `12px Monaco, Menlo, monospace` |
| `family` | The family for the page's own parts: menu, banner, buttons. | `Verdana, sans-serif` |
| `value_family` | The family for the text typed into an input box. | `Monaco, Menlo, monospace` |
| `field_px` | The size of text in an input box at zoom 1, in pixels. | `12.0` |
