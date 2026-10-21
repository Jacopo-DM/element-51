//! The panel that shows a node's source code. The card menu asks Python for it,
//! and the reply fills a read-only panel that stays until Dismiss or Escape.

use element_51_core::SourceReply;
use element_51_core::source::{source_panel, source_request_text};
use wasm_bindgen::JsCast;
use web_sys::{HtmlElement, KeyboardEvent};

use crate::a11y_ui::announce;
use crate::{Shared, show_error};

/// Ask Python for the source of node `node`. `opener` is the id of the element
/// that had the focus when the request was made, so closing the panel can
/// return to it; a request made with the mouse has none.
pub(crate) fn request_source(shared: &Shared, node: &str, opener: Option<String>) {
    let state = shared.borrow();
    if state
        .socket
        .send_with_str(&source_request_text(node))
        .is_err()
    {
        show_error(
            &state.document,
            "Could not ask for the source: not connected to Python.",
        );
        return;
    }
    if let Some(panel) = state.document.get_element_by_id("source-panel") {
        match opener {
            Some(id) => {
                let _ = panel.set_attribute("data-return", &id);
            }
            None => {
                let _ = panel.remove_attribute("data-return");
            }
        }
    }
}

/// Fill the panel from Python's reply, show it, and put the keyboard focus in
/// the code so the arrow keys scroll it.
pub(crate) fn show_source(shared: &Shared, reply: &SourceReply) {
    let (document, type_name) = {
        let state = shared.borrow();
        let type_name = state.replica.as_ref().and_then(|replica| {
            let node = replica
                .graph()
                .nodes
                .iter()
                .find(|node| node.id == reply.node)?;
            Some(node.node_type.clone())
        });
        (
            state.document.clone(),
            type_name.unwrap_or_else(|| "unknown".to_string()),
        )
    };
    let words = source_panel(reply, &type_name);
    let (Some(panel), Some(title), Some(body)) = (
        document.get_element_by_id("source-panel"),
        document.get_element_by_id("source-title"),
        document.get_element_by_id("source-text"),
    ) else {
        return;
    };
    title.set_text_content(Some(&words.title));
    body.set_text_content(Some(&words.body));
    body.set_scroll_top(0);
    let _ = panel.remove_attribute("hidden");
    if let Ok(body) = body.dyn_into::<HtmlElement>() {
        let _ = body.focus();
    }
    announce(&document, &words.announcement);
}

/// Hide the panel and give the focus back to what had it before, if that is
/// still there.
pub(crate) fn close_source(shared: &Shared) {
    let document = shared.borrow().document.clone();
    let Some(panel) = document.get_element_by_id("source-panel") else {
        return;
    };
    if panel.has_attribute("hidden") {
        return;
    }
    let _ = panel.set_attribute("hidden", "");
    if let Some(opener) = panel
        .get_attribute("data-return")
        .and_then(|id| document.get_element_by_id(&id))
        .and_then(|element| element.dyn_into::<HtmlElement>().ok())
    {
        let _ = opener.focus();
    }
    announce(&document, "Source closed");
}

/// Escape anywhere in the panel closes it.
pub(crate) fn on_source_key(shared: &Shared, event: &KeyboardEvent) {
    if event.key() == "Escape" {
        event.prevent_default();
        event.stop_propagation();
        close_source(shared);
    }
}
