//! Keyboard use of the add-node menu and the card menu: arrows, flyouts, focus return.

use crate::Shared;
use crate::a11y_ui::announce;
use crate::menu_ui::add_from_menu;
use crate::menu_ui::fill_card_menu;
use crate::menu_ui::focus_element;
use crate::menu_ui::focus_menu_search;
use crate::menu_ui::hide_menu;
use crate::menu_ui::place_flyout;
use crate::menu_ui::show_add_menu;
use crate::menu_ui::show_at;
use crate::menu_ui::window_size;
use element_51_core::layout::{icon_rect, node_rect};
use element_51_core::menu::step_menu;
use element_51_core::rows::type_of;
use wasm_bindgen::prelude::*;
use web_sys::{Document, Element, KeyboardEvent};

/// The items of a menu list in order: its buttons, and the label button of
/// each submenu (not the buttons inside a submenu's own flyout).
pub(crate) fn menu_items(container: &Element) -> Vec<Element> {
    let children = container.children();
    let mut items = Vec::new();
    for index in 0..children.length() {
        let Some(child) = children.item(index) else {
            continue;
        };
        if child.tag_name() == "BUTTON" {
            items.push(child);
        } else if is_submenu(&child)
            && let Some(label) = child.first_element_child()
        {
            items.push(label);
        }
    }
    items
}

pub(crate) fn is_submenu(element: &Element) -> bool {
    element
        .class_name()
        .split_whitespace()
        .any(|name| name == "submenu")
}

/// The list a menu button belongs to: the tree, the search results, or a
/// flyout panel (a submenu's label button belongs to the list its submenu is in).
pub(crate) fn list_of(button: &Element) -> Option<Element> {
    let parent = button.parent_element()?;
    if is_submenu(&parent) {
        parent.parent_element()
    } else {
        Some(parent)
    }
}

/// The first item of whichever list the add-node menu shows: the search
/// results, or the tree when there is nothing typed.
pub(crate) fn first_visible_item(document: &Document) -> Option<Element> {
    let results = document.get_element_by_id("menu-results")?;
    let list = if results.has_attribute("hidden") {
        document.get_element_by_id("menu-tree")?
    } else {
        results
    };
    menu_items(&list).into_iter().next()
}

/// Close every flyout the keyboard opened.
pub(crate) fn close_all_flyouts(document: &Document) {
    let Ok(open) = document.query_selector_all(".submenu.open") else {
        return;
    };
    for index in 0..open.length() {
        if let Some(submenu) = open.item(index).and_then(|n| n.dyn_into::<Element>().ok()) {
            close_flyout(&submenu);
        }
    }
}

pub(crate) fn close_flyout(submenu: &Element) {
    submenu.set_class_name("submenu");
    if let Some(label) = submenu.first_element_child() {
        let _ = label.set_attribute("aria-expanded", "false");
    }
}

/// Open the flyout of the category button `label`, close the others that do not
/// hold it, and move the focus to the flyout's first item.
pub(crate) fn open_flyout(shared: &Shared, label: &Element) {
    let Some(submenu) = label.parent_element() else {
        return;
    };
    let document = shared.borrow().document.clone();
    if let Ok(open) = document.query_selector_all(".submenu.open") {
        for index in 0..open.length() {
            if let Some(other) = open.item(index).and_then(|n| n.dyn_into::<Element>().ok())
                && !other.contains(Some(&submenu))
            {
                close_flyout(&other);
            }
        }
    }
    submenu.set_class_name("submenu open");
    let _ = label.set_attribute("aria-expanded", "true");
    place_flyout(shared, &submenu);
    if let Ok(Some(panel)) = submenu.query_selector(":scope > .panel")
        && let Some(first) = menu_items(&panel).into_iter().next()
    {
        focus_element(&first);
    }
}

/// Arrow keys, Home, End, Enter and Escape on a button of the add-node menu.
pub(crate) fn on_menu_key(shared: &Shared, event: &KeyboardEvent) {
    let Some(target) = event
        .target()
        .and_then(|target| target.dyn_into::<Element>().ok())
        .filter(|target| target.tag_name() == "BUTTON")
    else {
        return;
    };
    let key = event.key();
    let panel = target.closest(".panel").ok().flatten();
    let is_category = target.has_attribute("data-category");
    match key.as_str() {
        "ArrowDown" | "ArrowUp" | "Home" | "End" => {
            event.prevent_default();
            let Some(list) = list_of(&target) else {
                return;
            };
            let items = menu_items(&list);
            let here = items.iter().position(|item| *item == target);
            if key == "ArrowUp" && here == Some(0) && panel.is_none() {
                let document = shared.borrow().document.clone();
                focus_menu_search(&document);
                return;
            }
            focus_step(&items, here, &key);
        }
        "ArrowRight" if is_category => {
            event.prevent_default();
            open_flyout(shared, &target);
        }
        "Enter" | " " if is_category => {
            event.prevent_default();
            open_flyout(shared, &target);
        }
        "ArrowLeft" if panel.is_some() => {
            event.prevent_default();
            close_enclosing_flyout(panel.as_ref());
        }
        "Escape" => {
            event.prevent_default();
            event.stop_propagation();
            if panel.is_some() {
                close_enclosing_flyout(panel.as_ref());
            } else {
                close_menu_and_return(shared);
            }
        }
        "Tab" => close_menu_and_return(shared),
        _ => {}
    }
}

/// Move the focus within a menu list for ArrowDown, ArrowUp, Home or End;
/// `here` is the index of the item that has it.
pub(crate) fn focus_step(items: &[Element], here: Option<usize>, key: &str) {
    let next = match key {
        "Home" => (!items.is_empty()).then_some(0),
        "End" => items.len().checked_sub(1),
        _ => step_menu(here, items.len(), key == "ArrowDown"),
    };
    if let Some(item) = next.and_then(|index| items.get(index)) {
        focus_element(item);
    }
}

/// Close the flyout whose panel is `panel` and focus its label button.
pub(crate) fn close_enclosing_flyout(panel: Option<&Element>) {
    let Some(submenu) = panel.and_then(|panel| panel.parent_element()) else {
        return;
    };
    close_flyout(&submenu);
    if let Some(label) = submenu.first_element_child() {
        focus_element(&label);
    }
}

/// Arrow keys, Home, End and Escape on a button of the card menu.
pub(crate) fn on_card_menu_key(shared: &Shared, event: &KeyboardEvent) {
    let Some(target) = event
        .target()
        .and_then(|target| target.dyn_into::<Element>().ok())
        .filter(|target| target.tag_name() == "BUTTON")
    else {
        return;
    };
    let key = event.key();
    match key.as_str() {
        "ArrowDown" | "ArrowUp" | "Home" | "End" => {
            event.prevent_default();
            let Some(list) = target.parent_element() else {
                return;
            };
            let items = menu_items(&list);
            let here = items.iter().position(|item| *item == target);
            focus_step(&items, here, &key);
        }
        "Escape" => {
            event.prevent_default();
            event.stop_propagation();
            close_menu_and_return(shared);
        }
        "Tab" => close_menu_and_return(shared),
        _ => {}
    }
}

/// Close every menu and give the focus back to where it was when a keyboard
/// open took it.
pub(crate) fn close_menu_and_return(shared: &Shared) {
    hide_menu(&shared.borrow());
    return_focus(shared);
}

/// Focus the item that was focused when a menu was opened from the keyboard,
/// if it still exists. A menu opened with the mouse leaves nothing to return to.
pub(crate) fn return_focus(shared: &Shared) {
    let document = shared.borrow().document.clone();
    let mut wanted = None;
    for id in ["menu", "card-menu"] {
        if let Some(menu) = document.get_element_by_id(id)
            && let Some(opener) = menu.get_attribute("data-return")
        {
            let _ = menu.remove_attribute("data-return");
            wanted = Some(opener);
        }
    }
    if let Some(opener) = wanted.and_then(|id| document.get_element_by_id(&id)) {
        focus_element(&opener);
    }
}

/// Remember the item that has the focus, so closing the menu can return to it.
pub(crate) fn remember_opener(document: &Document, menu: &Element) {
    if let Some(active) = document.active_element()
        && !active.id().is_empty()
    {
        let _ = menu.set_attribute("data-return", &active.id());
    }
}

/// `N`: open the add-node menu from the keyboard, in the middle of the window;
/// a node added from it is placed there.
pub(crate) fn open_add_menu_from_keyboard(shared: &Shared) {
    let document = {
        let mut state = shared.borrow_mut();
        let size = window_size();
        let centre = (size.0 / 2.0, size.1 / 2.0);
        state.menu_at = state.view.to_graph(centre.0, centre.1);
        hide_menu(&state);
        show_add_menu(&state, centre.0, centre.1 / 2.0);
        if let Some(menu) = state.document.get_element_by_id("menu") {
            remember_opener(&state.document, &menu);
        }
        state.document.clone()
    };
    focus_menu_search(&document);
    announce(
        &document,
        "Add node menu. Type to search, or press Down for the list",
    );
}

/// Shift+F10 or the Menu key on a node: open its card menu beside the node's
/// menu icon.
pub(crate) fn open_card_menu_from_keyboard(shared: &Shared, id: &str) {
    let document = {
        let mut state = shared.borrow_mut();
        let corner = state.replica.as_ref().and_then(|replica| {
            let node = replica.graph().nodes.iter().find(|node| node.id == id)?;
            let node_type = type_of(replica.graph(), replica.node_types(), node)?;
            let rect = node_rect(node, &node_type, &state.theme.metrics);
            let icon = icon_rect(&rect, &state.theme.metrics);
            Some(state.view.to_screen(icon.x, icon.y + icon.h))
        });
        let Some((x, y)) = corner else {
            return;
        };
        let canvas = state.canvas.get_bounding_client_rect();
        hide_menu(&state);
        let Some(menu) = state.document.get_element_by_id("card-menu") else {
            return;
        };
        fill_card_menu(&state, &menu, id);
        show_at(&menu, canvas.left() + x, canvas.top() + y);
        remember_opener(&state.document, &menu);
        state.card_menu_node = Some(id.to_string());
        state.document.clone()
    };
    if let Some(first) = document
        .get_element_by_id("card-menu")
        .and_then(|menu| menu.first_element_child())
    {
        focus_element(&first);
    }
    announce(&document, &format!("Actions for {id}"));
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
        "ArrowDown" => {
            event.prevent_default();
            let document = shared.borrow().document.clone();
            if let Some(first) = first_visible_item(&document) {
                focus_element(&first);
            }
        }
        "Escape" => close_menu_and_return(shared),
        _ => {}
    }
}
