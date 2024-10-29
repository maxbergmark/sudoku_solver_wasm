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

use codee::string::JsonSerdeCodec;
use hotkeys::setup_hotkeys;
use leptos::create_rw_signal;
use leptos::leptos_dom::logging::console_log;
use leptos::provide_context;
use leptos::SignalGetUntracked;
use leptos::SignalSet;
use leptos_hotkeys::{provide_hotkeys_context, scopes, HotkeysContext};
use leptos_use::use_cookie;
use state::GameState;
use state::SudokuData;
use sudoku::SudokuGame;

use leptos::{component, create_node_ref, html, mount_to_body, view, IntoView};
use leptos_meta::provide_meta_context;
use leptos_router::{Route, Router, Routes, TrailingSlash};
use rstest as _;
use serde_json as _;

mod actions;
mod error;
mod generator;
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
    let (sudoku_data_cookie, set_sudoku_data_cookie) =
        use_cookie::<SudokuData, JsonSerdeCodec>("sudoku_data");
    let (game_state_cookie, _set_game_state_cookie) =
        use_cookie::<GameState, JsonSerdeCodec>("game_state");

    console_log(format!("cookie: {:?}", sudoku_data_cookie.get_untracked()).as_str());
    console_log(format!("cookie: {:?}", game_state_cookie.get_untracked()).as_str());
    let sudoku_data = create_rw_signal(sudoku_data_cookie.get_untracked().unwrap_or_default());
    let game_state = create_rw_signal(game_state_cookie.get_untracked().unwrap_or_default());
    // create derived signals without the Option
    console_log(format!("signal: {:}", sudoku_data.get_untracked()).as_str());

    // let reset_sudoku = move || set_sudoku_data_cookie.set(Some(SudokuData::default()));
    // let reset_state = move || set_game_state_cookie.set(Some(GameState::default()));

    // if sudoku_data_cookie.get_untracked().is_none() {
    //     reset_sudoku();
    // }

    // if game_state_cookie.get_untracked().is_none() {
    //     reset_state();
    // }

    provide_context(game_state);
    // provide_context(set_game_state);
    provide_context(sudoku_data);
    // provide_context(set_sudoku_data);
    setup_hotkeys(game_state, sudoku_data);
    // set_sudoku_data_cookie.set(Some(sudoku_data.get()));
    // console_log(format!("{:?}", game_state()).as_str());
    // set_game_state_cookie.set(Some(game_state()));

    view! {
        {move || {
            console_log("Updating cookies");
            console_log(sudoku_data().to_string().as_str());
            set_sudoku_data_cookie.set(Some(sudoku_data()));
            console_log(
                format!("cookie after update: {:?}", sudoku_data_cookie.get_untracked()).as_str(),
            );
        }}
        <div class=move || game_state().dark_mode.class() on:click=move |_| {}>
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
