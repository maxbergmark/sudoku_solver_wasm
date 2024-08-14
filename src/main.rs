#![warn(
    // missing_docs,
    // unreachable_pub,
    keyword_idents,
    unexpected_cfgs,
    missing_copy_implementations,
    missing_debug_implementations,
    non_ascii_idents,
    noop_method_call,
    unused_crate_dependencies,
    unused_extern_crates,
    unused_import_braces,
    future_incompatible,
    nonstandard_style,
    bad_style,
    dead_code,
    improper_ctypes,
    non_shorthand_field_patterns,
    no_mangle_generic_items,
    overflowing_literals,
    path_statements,
    patterns_in_fns_without_body,
    unconditional_recursion,
    unused,
    unused_allocation,
    unused_comparisons,
    unused_parens,
    while_true,
)]

use hotkeys::setup_hotkeys;
use leptos::create_rw_signal;
use leptos::provide_context;
use leptos_hotkeys::{provide_hotkeys_context, scopes, HotkeysContext};
use state::GameState;
use state::SudokuData;
use sudoku::SudokuGame;

use leptos::{component, create_node_ref, html, mount_to_body, view, IntoView};
use leptos_meta::provide_meta_context;
use leptos_router::{Route, Router, Routes, TrailingSlash};
use rstest as _;

mod actions;
mod error;
mod hotkeys;
mod state;
mod sudoku;
mod ui;
mod util;

pub use error::Error;
pub use error::Result;

#[component]
fn App() -> impl IntoView {
    provide_meta_context();

    let main_ref = create_node_ref::<html::Main>();
    let HotkeysContext { .. } = provide_hotkeys_context(main_ref, false, scopes!());
    let sudoku_data = create_rw_signal(SudokuData::default());
    let game_state = create_rw_signal(GameState::default());

    provide_context(game_state);
    provide_context(sudoku_data);
    setup_hotkeys(game_state, sudoku_data);

    view! {
        <div class=move || game_state().dark_mode.class()>
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
        </div>
    }
}

fn main() {
    console_error_panic_hook::set_once();
    mount_to_body(|| view! { <App /> });
}
