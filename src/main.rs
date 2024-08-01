use leptos_hotkeys::{provide_hotkeys_context, scopes, HotkeysContext};
use sudoku::SudokuGame;

use leptos::*;
use leptos_meta::provide_meta_context;
use leptos_router::{Route, Router, Routes, TrailingSlash};

mod actions;
mod error;
mod hotkeys;
mod state;
mod sudoku;

pub use error::Error;
pub use error::Result;

#[component]
fn App() -> impl IntoView {
    provide_meta_context();

    let main_ref = create_node_ref::<html::Main>();
    let HotkeysContext { .. } = provide_hotkeys_context(main_ref, false, scopes!());

    view! {
        <Router>
            <main _ref=main_ref>
                <Routes>
                    <Route path="/" view=SudokuGame />
                    <Route
                        path="/sudoku_solver_wasm/"
                        trailing_slash=TrailingSlash::Exact
                        view=SudokuGame
                    />
                    <Route path="/*any" view=move || view! { <p>"Page not found"</p> } />
                </Routes>
            </main>
        </Router>
    }
}

fn main() {
    console_error_panic_hook::set_once();
    mount_to_body(|| view! { <App /> })
}
