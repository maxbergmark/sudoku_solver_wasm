use leptos::{leptos_dom::logging::console_error, RwSignal};

#[allow(clippy::panic)]
pub fn unwrap_or_panic<T>(signal: Option<RwSignal<T>>) -> RwSignal<T> {
    signal.unwrap_or_else(|| {
        console_error("Component not available");
        panic!("Component not available");
    })
}
