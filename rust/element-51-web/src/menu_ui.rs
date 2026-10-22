//! The add-node menu and the card menu: the DOM parts around the canvas.

use element_51_core::menu::{
    CardAction, Entry, build_menu, card_actions, clamp_into, flyout_at, search as search_node_types,
};
use element_51_core::rows::type_of;
use element_51_core::{CardDisplay, Edit, Snapshot};
use wasm_bindgen::prelude::*;
use web_sys::{Document, Element, HtmlElement, HtmlInputElement, MouseEvent};

use crate::draw::Selected;
use crate::menu_keys::{close_all_flyouts, return_focus};
use crate::save_csv::save_csv;
use crate::save_png::save_png;
use crate::source_ui::request_source;
use crate::{Shared, State, redraw, submit};

/// The least gap between the right-click menu and the window edge, in pixels.
pub(crate) const MENU_MARGIN: f64 = 8.0;

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
                let _ = label.set_attribute("aria-haspopup", "true");
                let _ = label.set_attribute("aria-expanded", "false");
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

pub(crate) fn on_context_menu(shared: &Shared, event: &MouseEvent) {
    event.prevent_default();
    let document = {
        let mut state = shared.borrow_mut();
        state.menu_at = state
            .view
            .to_graph(f64::from(event.offset_x()), f64::from(event.offset_y()));
        let (left, top) = (f64::from(event.client_x()), f64::from(event.client_y()));
        show_add_menu(&state, left, top);
        state.document.clone()
    };
    focus_menu_search(&document);
}

/// Show the add-node menu with its corner at (`left`, `top`), moved back inside
/// the window if it hangs out. The caller puts the focus in the search box once
/// it holds no borrow of the state (moving focus out of the hidden tree runs
/// its handlers, which borrow it).
pub(crate) fn show_add_menu(state: &State, left: f64, top: f64) {
    let Some(menu) = state.document.get_element_by_id("menu") else {
        return;
    };
    let _ = menu.remove_attribute("data-return");
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

/// Put the cursor in the add-node menu's search box.
pub(crate) fn focus_menu_search(document: &Document) {
    if let Some(input) = document.get_element_by_id("menu-search") {
        focus_element(&input);
    }
}

pub(crate) fn focus_element(element: &Element) {
    if let Ok(html) = element.clone().dyn_into::<HtmlElement>() {
        let _ = html.focus();
    }
}

/// Empty the search box, show the tree, and put the cursor in the box.
fn reset_menu_search(document: &Document) {
    if let Some(input) = document
        .get_element_by_id("menu-search")
        .and_then(|element| element.dyn_into::<HtmlInputElement>().ok())
    {
        input.set_value("");
    }
    close_all_flyouts(document);
    if let Some(tree) = document.get_element_by_id("menu-tree") {
        let _ = tree.remove_attribute("hidden");
        tree.set_scroll_top(0);
    }
    if let Some(results) = document.get_element_by_id("menu-results") {
        let _ = results.set_attribute("hidden", "");
    }
}

/// The size of the browser window, in pixels.
pub(crate) fn window_size() -> (f64, f64) {
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
    place_flyout(shared, &entry);
}

/// Put the flyout panel of the submenu `entry` beside its entry.
pub(crate) fn place_flyout(shared: &Shared, entry: &Element) {
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
    for id in ["menu", "card-menu", "app-menu", "theme-menu", "palette-box"] {
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
    let _ = menu.remove_attribute("data-return");
    fill_card_menu(state, &menu, &id);
    show_at(
        &menu,
        f64::from(event.client_x()),
        f64::from(event.client_y()),
    );
    state.card_menu_node = Some(id);
}

/// Give the card menu the entries node `id` is offered: Save as PNG only where
/// the card shows a picture or a chart, Save as CSV only where it shows a chart. A button's `data-card-action` is its
/// place in `card_actions()`, so it does not change with what is left out.
pub(crate) fn fill_card_menu(state: &State, menu: &Element, id: &str) {
    let display = state
        .replica
        .as_ref()
        .and_then(|replica| {
            let node = replica.graph().nodes.iter().find(|node| node.id == id)?;
            Some(type_of(replica.graph(), replica.node_types(), node)?.display)
        })
        .unwrap_or(CardDisplay::None);
    menu.set_text_content(None);
    for (index, action) in card_actions().into_iter().enumerate() {
        if !action.offered(display) {
            continue;
        }
        if let Ok(button) = state.document.create_element("button") {
            button.set_text_content(Some(action.label()));
            let _ = button.set_attribute("data-card-action", &index.to_string());
            let _ = menu.append_child(&button);
        }
    }
}

/// Show `menu` with its corner at (`left`, `top`), moved back inside the window
/// if it would hang out.
pub(crate) fn show_at(menu: &Element, left: f64, top: f64) {
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

/// A click on a card-menu entry carries out its action on the node.
pub(crate) fn on_card_menu_click(shared: &Shared, event: &MouseEvent) {
    let action = event
        .target()
        .and_then(|target| target.dyn_into::<Element>().ok())
        .and_then(|target| target.get_attribute("data-card-action"))
        .and_then(|index| index.parse::<usize>().ok())
        .and_then(|index| card_actions().get(index).copied());
    if action == Some(CardAction::ViewSource) {
        let (id, opener) = {
            let mut state = shared.borrow_mut();
            let opener = state
                .document
                .get_element_by_id("card-menu")
                .and_then(|menu| menu.get_attribute("data-return"));
            hide_menu(&state);
            (state.card_menu_node.take(), opener)
        };
        if let Some(id) = id {
            request_source(shared, &id, opener);
        }
        return_focus(shared);
        return;
    }
    if action == Some(CardAction::SavePng) {
        let id = {
            let mut state = shared.borrow_mut();
            hide_menu(&state);
            state.card_menu_node.take()
        };
        if let Some(id) = id {
            save_png(shared, &id);
        }
        return_focus(shared);
        return;
    }
    if action == Some(CardAction::SaveCsv) {
        let id = {
            let mut state = shared.borrow_mut();
            hide_menu(&state);
            state.card_menu_node.take()
        };
        if let Some(id) = id {
            save_csv(shared, &id);
        }
        return_focus(shared);
        return;
    }
    let edit = {
        let mut state = shared.borrow_mut();
        hide_menu(&state);
        let id = state.card_menu_node.take();
        let selected_here = matches!(&state.selected, Some(Selected::Node(selected)) if Some(selected) == id.as_ref());
        if selected_here {
            state.selected = None;
        }
        action.zip(id).and_then(|(action, id)| action.edit(&id))
    };
    match edit {
        Some(edit) => submit(shared, edit),
        None => redraw(shared),
    }
    return_focus(shared);
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
pub(crate) fn add_from_menu(shared: &Shared, name: &str) {
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
    return_focus(shared);
}
