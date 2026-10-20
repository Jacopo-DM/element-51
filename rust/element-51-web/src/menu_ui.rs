//! The add-node menu and the card menu: the DOM parts around the canvas.

use element_51_core::menu::{
    Entry, build_menu, card_actions, clamp_into, flyout_at, search as search_node_types,
};
use element_51_core::palette::with_palette;
use element_51_core::{Edit, Snapshot};
use wasm_bindgen::prelude::*;
use web_sys::{Document, Element, HtmlInputElement, KeyboardEvent, MouseEvent};

use crate::draw::Selected;
use crate::{Shared, State, apply_theme, redraw, show_error, submit};

/// The least gap between the right-click menu and the window edge, in pixels.
const MENU_MARGIN: f64 = 8.0;

/// Build the menu's tree: a button for each node type, and a submenu for each
/// category, nested as the node types' categories say.
pub(crate) fn fill_menu(document: &Document, snapshot: &Snapshot) {
    let Some(tree) = document.get_element_by_id("menu-tree") else {
        return;
    };
    tree.set_text_content(None);
    append_entries(document, &tree, &build_menu(&snapshot.node_types));
}

fn append_entries(document: &Document, parent: &Element, entries: &[Entry]) {
    for entry in entries {
        match entry {
            Entry::Node(name) => {
                if let Some(button) = node_button(document, name, name) {
                    let _ = parent.append_child(&button);
                }
            }
            Entry::Category { name, entries } => {
                let (Ok(submenu), Ok(label), Ok(panel)) = (
                    document.create_element("div"),
                    document.create_element("button"),
                    document.create_element("div"),
                ) else {
                    continue;
                };
                submenu.set_class_name("submenu");
                panel.set_class_name("panel");
                label.set_class_name("category");
                let _ = label.set_attribute("type", "button");
                let _ = label.set_attribute("data-category", name);
                label.set_text_content(Some(name));
                append_entries(document, &panel, entries);
                let _ = submenu.append_child(&label);
                let _ = submenu.append_child(&panel);
                let _ = parent.append_child(&submenu);
            }
        }
    }
}

/// A menu button that adds node type `name`, showing `label`.
fn node_button(document: &Document, name: &str, label: &str) -> Option<Element> {
    let button = document.create_element("button").ok()?;
    let _ = button.set_attribute("type", "button");
    let _ = button.set_attribute("data-node-type", name);
    button.set_text_content(Some(label));
    Some(button)
}

/// Show the node types that match what was typed in the search box, instead of
/// the tree; with nothing typed, show the tree again.
pub(crate) fn on_menu_search(shared: &Shared) {
    let state = shared.borrow();
    let document = &state.document;
    let (Some(search), Some(tree), Some(results), Some(replica)) = (
        document.get_element_by_id("menu-search"),
        document.get_element_by_id("menu-tree"),
        document.get_element_by_id("menu-results"),
        state.replica.as_ref(),
    ) else {
        return;
    };
    let text = search
        .dyn_into::<HtmlInputElement>()
        .map(|input| input.value())
        .unwrap_or_default();
    results.set_text_content(None);
    if text.trim().is_empty() {
        let _ = tree.remove_attribute("hidden");
        let _ = results.set_attribute("hidden", "");
        return;
    }
    let _ = tree.set_attribute("hidden", "");
    let _ = results.remove_attribute("hidden");
    let found = search_node_types(replica.node_types(), &text);
    if found.is_empty() {
        if let Ok(none) = document.create_element("div") {
            none.set_class_name("none");
            none.set_text_content(Some("No matching node"));
            let _ = results.append_child(&none);
        }
        return;
    }
    for entry in found {
        if let Some(button) = node_button(document, &entry.name, &entry.path) {
            let _ = results.append_child(&button);
        }
    }
}

/// Enter in the search box adds the first match; Escape closes the menu.
pub(crate) fn on_menu_search_key(shared: &Shared, event: &KeyboardEvent) {
    match event.key().as_str() {
        "Enter" => {
            event.prevent_default();
            let first = shared
                .borrow()
                .document
                .get_element_by_id("menu-results")
                .and_then(|results| results.query_selector("button").ok().flatten())
                .and_then(|button| button.get_attribute("data-node-type"));
            if let Some(name) = first {
                add_from_menu(shared, &name);
            }
        }
        "Escape" => hide_menu(&shared.borrow()),
        _ => {}
    }
}

pub(crate) fn on_context_menu(shared: &Shared, event: &MouseEvent) {
    event.prevent_default();
    let mut state = shared.borrow_mut();
    state.menu_at = state
        .view
        .to_graph(f64::from(event.offset_x()), f64::from(event.offset_y()));
    let Some(menu) = state.document.get_element_by_id("menu") else {
        return;
    };
    let (left, top) = (f64::from(event.client_x()), f64::from(event.client_y()));
    let _ = menu.set_attribute("style", &format!("left: {left}px; top: {top}px"));
    let _ = menu.remove_attribute("hidden");
    reset_menu_search(&state.document);
    // Now that it is laid out, move it back inside the window if it hangs out.
    let rect = menu.get_bounding_client_rect();
    let (left, top) = clamp_into(
        (left, top),
        (rect.width(), rect.height()),
        window_size(),
        MENU_MARGIN,
    );
    let _ = menu.set_attribute("style", &format!("left: {left}px; top: {top}px"));
}

/// Empty the search box, show the tree, and put the cursor in the box.
fn reset_menu_search(document: &Document) {
    if let Some(input) = document
        .get_element_by_id("menu-search")
        .and_then(|element| element.dyn_into::<HtmlInputElement>().ok())
    {
        input.set_value("");
        let _ = input.focus();
    }
    if let Some(tree) = document.get_element_by_id("menu-tree") {
        let _ = tree.remove_attribute("hidden");
        tree.set_scroll_top(0);
    }
    if let Some(results) = document.get_element_by_id("menu-results") {
        let _ = results.set_attribute("hidden", "");
    }
}

/// The size of the browser window, in pixels.
fn window_size() -> (f64, f64) {
    let Some(window) = web_sys::window() else {
        return (0.0, 0.0);
    };
    let pixels =
        |value: Result<JsValue, JsValue>| value.ok().and_then(|v| v.as_f64()).unwrap_or(0.0);
    (pixels(window.inner_width()), pixels(window.inner_height()))
}

/// Put the flyout of the menu entry under the pointer beside that entry. A
/// flyout is positioned against the window, not its parent, so a menu that
/// scrolls does not clip it.
pub(crate) fn on_menu_hover(shared: &Shared, event: &MouseEvent) {
    let Some(entry) = event
        .target()
        .and_then(|target| target.dyn_into::<Element>().ok())
        .and_then(|element| element.closest(".submenu").ok().flatten())
    else {
        return;
    };
    let Ok(Some(panel)) = entry.query_selector(":scope > .panel") else {
        return;
    };
    let raise = shared.borrow().theme.page.menu_pad;
    let (outer, size) = (
        entry.get_bounding_client_rect(),
        panel.get_bounding_client_rect(),
    );
    let (left, top) = flyout_at(
        (outer.left(), outer.top(), outer.right()),
        (size.width(), size.height()),
        window_size(),
        raise,
        MENU_MARGIN,
    );
    let _ = panel.set_attribute("style", &format!("left: {left}px; top: {top}px"));
}

pub(crate) fn hide_menu(state: &State) {
    for id in ["menu", "card-menu", "app-menu", "palette-box"] {
        if let Some(menu) = state.document.get_element_by_id(id) {
            let _ = menu.set_attribute("hidden", "");
        }
    }
}

/// Show the card menu for node `id` with its corner at the pointer.
pub(crate) fn open_card_menu(state: &mut State, id: String, event: &MouseEvent) {
    let Some(menu) = state.document.get_element_by_id("card-menu") else {
        return;
    };
    show_at(
        &menu,
        f64::from(event.client_x()),
        f64::from(event.client_y()),
    );
    state.card_menu_node = Some(id);
}

/// Show `menu` with its corner at (`left`, `top`), moved back inside the window
/// if it would hang out.
fn show_at(menu: &Element, left: f64, top: f64) {
    let _ = menu.set_attribute("style", &format!("left: {left}px; top: {top}px"));
    let _ = menu.remove_attribute("hidden");
    let rect = menu.get_bounding_client_rect();
    let (left, top) = clamp_into(
        (left, top),
        (rect.width(), rect.height()),
        window_size(),
        MENU_MARGIN,
    );
    let _ = menu.set_attribute("style", &format!("left: {left}px; top: {top}px"));
}

/// Open the menu under the button beside Run, or close it if it is open.
pub(crate) fn on_app_menu_button(shared: &Shared) {
    let state = shared.borrow();
    let (Some(button), Some(menu)) = (
        state.document.get_element_by_id("app-menu-button"),
        state.document.get_element_by_id("app-menu"),
    ) else {
        return;
    };
    let was_closed = menu.has_attribute("hidden");
    hide_menu(&state);
    if was_closed {
        let rect = button.get_bounding_client_rect();
        show_at(&menu, rect.left(), rect.bottom() + MENU_MARGIN / 2.0);
    }
}

/// `Try palette`: open the box for a Coolors link, empty and with the cursor in
/// it.
pub(crate) fn on_try_palette(shared: &Shared) {
    let state = shared.borrow();
    hide_menu(&state);
    let (Some(button), Some(form)) = (
        state.document.get_element_by_id("app-menu-button"),
        state.document.get_element_by_id("palette-box"),
    ) else {
        return;
    };
    let rect = button.get_bounding_client_rect();
    show_at(&form, rect.left(), rect.bottom() + MENU_MARGIN / 2.0);
    if let Some(input) = palette_input(&state.document) {
        input.set_value("");
        let _ = input.focus();
    }
}

fn palette_input(document: &Document) -> Option<HtmlInputElement> {
    document
        .get_element_by_id("palette-input")?
        .dyn_into::<HtmlInputElement>()
        .ok()
}

/// The link was submitted: make the palette the page's colours, or say why not.
/// Nothing is sent to Python; a reload brings back the theme Python holds.
pub(crate) fn on_palette_submit(shared: &Shared, event: &web_sys::Event) {
    event.prevent_default();
    let text = palette_input(&shared.borrow().document)
        .map(|input| input.value())
        .unwrap_or_default();
    let tried = with_palette(&shared.borrow().theme, &text);
    match tried {
        Ok(theme) => {
            {
                let mut state = shared.borrow_mut();
                apply_theme(&state.document, &theme);
                state.theme = theme;
                hide_menu(&state);
            }
            redraw(shared);
        }
        Err(reason) => show_error(&shared.borrow().document, &reason),
    }
}

/// Escape in the palette box closes it.
pub(crate) fn on_palette_key(shared: &Shared, event: &KeyboardEvent) {
    if event.key() == "Escape" {
        hide_menu(&shared.borrow());
    }
}

/// A click on a card-menu entry carries out its action on the node.
pub(crate) fn on_card_menu_click(shared: &Shared, event: &MouseEvent) {
    let action = event
        .target()
        .and_then(|target| target.dyn_into::<Element>().ok())
        .and_then(|target| target.get_attribute("data-card-action"))
        .and_then(|index| index.parse::<usize>().ok())
        .and_then(|index| card_actions().get(index).copied());
    let edit = {
        let mut state = shared.borrow_mut();
        hide_menu(&state);
        let id = state.card_menu_node.take();
        let selected_here = matches!(&state.selected, Some(Selected::Node(selected)) if Some(selected) == id.as_ref());
        if selected_here {
            state.selected = None;
        }
        action.zip(id).map(|(action, id)| action.edit(&id))
    };
    match edit {
        Some(edit) => submit(shared, edit),
        None => redraw(shared),
    }
}

pub(crate) fn on_menu_click(shared: &Shared, event: &MouseEvent) {
    let target = event
        .target()
        .and_then(|target| target.dyn_into::<Element>().ok());
    let Some(target) = target else {
        return;
    };
    // A category label and the search box open or filter; they do not close.
    if target.has_attribute("data-category") || target.id() == "menu-search" {
        return;
    }
    match target.get_attribute("data-node-type") {
        Some(name) => add_from_menu(shared, &name),
        None => hide_menu(&shared.borrow()),
    }
}

/// Close the menu and add a node of type `name` where it was opened.
fn add_from_menu(shared: &Shared, name: &str) {
    let edit = {
        let state = shared.borrow();
        hide_menu(&state);
        let Some(replica) = state.replica.as_ref() else {
            return;
        };
        Edit::AddNode {
            id: replica.next_node_id(),
            node_type: name.to_string(),
            x: state.menu_at.0,
            y: state.menu_at.1,
        }
    };
    submit(shared, edit);
}
