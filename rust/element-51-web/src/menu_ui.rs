//! The add-node menu and the card menu: the DOM parts around the canvas.

use element_51_core::layout::{icon_rect, node_rect};
use element_51_core::menu::{
    Entry, build_menu, card_actions, clamp_into, flyout_at, search as search_node_types, step_menu,
};
use element_51_core::palette::with_palette;
use element_51_core::theme::Theme;
use element_51_core::{Edit, Snapshot};
use wasm_bindgen::prelude::*;
use web_sys::{Document, Element, HtmlElement, HtmlInputElement, KeyboardEvent, MouseEvent};

use crate::a11y_ui::announce;
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
fn show_add_menu(state: &State, left: f64, top: f64) {
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
fn focus_menu_search(document: &Document) {
    if let Some(input) = document.get_element_by_id("menu-search") {
        focus_element(&input);
    }
}

fn focus_element(element: &Element) {
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
    place_flyout(shared, &entry);
}

/// Put the flyout panel of the submenu `entry` beside its entry.
fn place_flyout(shared: &Shared, entry: &Element) {
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

/// `Select theme`: open the list of built-in themes under the menu button.
pub(crate) fn on_select_theme(shared: &Shared) {
    let state = shared.borrow();
    hide_menu(&state);
    let (Some(button), Some(list)) = (
        state.document.get_element_by_id("app-menu-button"),
        state.document.get_element_by_id("theme-menu"),
    ) else {
        return;
    };
    let rect = button.get_bounding_client_rect();
    show_at(&list, rect.left(), rect.bottom() + MENU_MARGIN / 2.0);
}

/// A click in the theme list: the page takes that built-in theme at once.
/// Nothing is sent to Python; a reload brings back the theme Python holds.
pub(crate) fn on_theme_click(shared: &Shared, event: &MouseEvent) {
    let name = event
        .target()
        .and_then(|target| target.dyn_into::<Element>().ok())
        .and_then(|target| target.get_attribute("data-theme"));
    let Some(theme) = name.and_then(|name| Theme::builtin(&name)) else {
        return;
    };
    {
        let mut state = shared.borrow_mut();
        apply_theme(&state.document, &theme);
        state.theme = theme;
        hide_menu(&state);
    }
    redraw(shared);
}

/// `Load palette from Coolors link`: open the box for a Coolors link, empty and with the cursor in
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
    return_focus(shared);
}

/// The items of a menu list in order: its buttons, and the label button of
/// each submenu (not the buttons inside a submenu's own flyout).
fn menu_items(container: &Element) -> Vec<Element> {
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

fn is_submenu(element: &Element) -> bool {
    element
        .class_name()
        .split_whitespace()
        .any(|name| name == "submenu")
}

/// The list a menu button belongs to: the tree, the search results, or a
/// flyout panel (a submenu's label button belongs to the list its submenu is in).
fn list_of(button: &Element) -> Option<Element> {
    let parent = button.parent_element()?;
    if is_submenu(&parent) {
        parent.parent_element()
    } else {
        Some(parent)
    }
}

/// The first item of whichever list the add-node menu shows: the search
/// results, or the tree when there is nothing typed.
fn first_visible_item(document: &Document) -> Option<Element> {
    let results = document.get_element_by_id("menu-results")?;
    let list = if results.has_attribute("hidden") {
        document.get_element_by_id("menu-tree")?
    } else {
        results
    };
    menu_items(&list).into_iter().next()
}

/// Close every flyout the keyboard opened.
fn close_all_flyouts(document: &Document) {
    let Ok(open) = document.query_selector_all(".submenu.open") else {
        return;
    };
    for index in 0..open.length() {
        if let Some(submenu) = open.item(index).and_then(|n| n.dyn_into::<Element>().ok()) {
            close_flyout(&submenu);
        }
    }
}

fn close_flyout(submenu: &Element) {
    submenu.set_class_name("submenu");
    if let Some(label) = submenu.first_element_child() {
        let _ = label.set_attribute("aria-expanded", "false");
    }
}

/// Open the flyout of the category button `label`, close the others that do not
/// hold it, and move the focus to the flyout's first item.
fn open_flyout(shared: &Shared, label: &Element) {
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
            let next = match key.as_str() {
                "Home" => (!items.is_empty()).then_some(0),
                "End" => items.len().checked_sub(1),
                _ => step_menu(here, items.len(), key == "ArrowDown"),
            };
            if let Some(item) = next.and_then(|index| items.get(index)) {
                focus_element(item);
            }
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

/// Close the flyout whose panel is `panel` and focus its label button.
fn close_enclosing_flyout(panel: Option<&Element>) {
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
            let next = match key.as_str() {
                "Home" => (!items.is_empty()).then_some(0),
                "End" => items.len().checked_sub(1),
                _ => step_menu(here, items.len(), key == "ArrowDown"),
            };
            if let Some(item) = next.and_then(|index| items.get(index)) {
                focus_element(item);
            }
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
fn close_menu_and_return(shared: &Shared) {
    hide_menu(&shared.borrow());
    return_focus(shared);
}

/// Focus the item that was focused when a menu was opened from the keyboard,
/// if it still exists. A menu opened with the mouse leaves nothing to return to.
fn return_focus(shared: &Shared) {
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
fn remember_opener(document: &Document, menu: &Element) {
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
            let node_type = replica
                .node_types()
                .iter()
                .find(|known| known.name == node.node_type)?;
            let rect = node_rect(node, node_type, &state.theme.metrics);
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
