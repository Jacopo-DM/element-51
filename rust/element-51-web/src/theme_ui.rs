//! The app menu's theme entries and the palette box.

use crate::menu_ui::MENU_MARGIN;
use crate::menu_ui::hide_menu;
use crate::menu_ui::show_at;
use crate::{Shared, apply_theme, redraw, show_error};
use element_51_core::palette::with_palette;
use element_51_core::theme::Theme;
use wasm_bindgen::prelude::*;
use web_sys::{Document, Element, HtmlInputElement, KeyboardEvent, MouseEvent};

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

pub(crate) fn palette_input(document: &Document) -> Option<HtmlInputElement> {
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
