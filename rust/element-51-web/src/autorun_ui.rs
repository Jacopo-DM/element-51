//! `Run automatically`: the box, the timer, and what an edit or a result does
//! to it. The rule itself is `element-51-core::autorun`.

use crate::Shared;
use crate::a11y_ui::announce;
use crate::run_ui::{auto_run_ticked, start_run};
use element_51_core::announce::queued_run_text;
use element_51_core::autorun::Reaction;
use wasm_bindgen::JsCast;
use wasm_bindgen::prelude::{Closure, wasm_bindgen};

#[wasm_bindgen]
extern "C" {
    /// `Date.now()`: milliseconds, the clock the rule is given.
    #[wasm_bindgen(js_namespace = Date, js_name = now)]
    fn now() -> f64;
}

/// The box was ticked or unticked.
pub(crate) fn on_toggle(shared: &Shared) {
    let mut state = shared.borrow_mut();
    let on = auto_run_ticked(&state.document);
    state.auto.set_ticked(on);
    let Some(button) = state.document.get_element_by_id("run") else {
        return;
    };
    if on {
        let _ = button.set_attribute("aria-disabled", "true");
        // Ticking runs the graph once, so the screen is current from the start.
        if state.auto.tick_run() {
            start_run(&state);
        }
    } else if button.get_attribute("aria-busy").as_deref() != Some("true") {
        let _ = button.remove_attribute("aria-disabled");
    }
}

/// A change that alters a result happened (an edit other than a move, a
/// refused edit that was rolled back, or a graph pushed from Python).
pub(crate) fn changed(shared: &Shared) {
    let reaction = shared.borrow_mut().auto.change(now());
    match reaction {
        Reaction::Ignore => {}
        Reaction::Wait(milliseconds) => wait(shared, milliseconds),
        Reaction::Queued => {
            let state = shared.borrow();
            announce(&state.document, queued_run_text());
            if let Some(button) = state.document.get_element_by_id("run") {
                button.set_text_content(Some("Running, will run again"));
            }
        }
    }
}

/// Call `due` once `milliseconds` have passed.
fn wait(shared: &Shared, milliseconds: f64) {
    let Some(window) = web_sys::window() else {
        return;
    };
    let shared = shared.clone();
    let timer = Closure::once_into_js(move || due(&shared));
    let _ = window.set_timeout_with_callback_and_timeout_and_arguments_0(
        timer.unchecked_ref(),
        milliseconds as i32,
    );
}

fn due(shared: &Shared) {
    let mut state = shared.borrow_mut();
    if state.auto.due(now()) {
        start_run(&state);
    }
}
