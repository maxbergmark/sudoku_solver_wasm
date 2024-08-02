use std::str::FromStr;

use crate::actions::{
    check_all_visible_doubles, check_constraints, place_all_hidden_singles,
    place_all_visible_singles, solve_sudoku, update_from_sudoku,
};
use crate::hotkeys::setup_hotkeys;
use crate::state::{decompress_string, Cell, GameState, SudokuData};
use ev::MouseEvent;
use leptos_dom::logging::console_error;
use rust_sudoku_solver::Sudoku;

use leptos::{
    component, create_rw_signal, ev, leptos_dom, provide_context, update, use_context, view,
    IntoView, Params, RwSignal, SignalUpdate, SignalWith,
};
use leptos_router::{use_query, Params, ParamsError};

#[derive(Params, PartialEq, Debug)]
struct SudokuParams {
    sudoku: Option<String>,
}

fn is_valid_game_str(game_str: &str) -> bool {
    game_str.len() == 81 && game_str.chars().all(|c| c.is_ascii_digit() || c == '.')
}

#[component]
pub fn SudokuGame() -> impl IntoView {
    let sudoku_data = create_rw_signal(SudokuData::default());
    let game_state = create_rw_signal(GameState::default());

    provide_context(game_state);
    provide_context(sudoku_data);
    setup_hotkeys(game_state, sudoku_data);

    let params = use_query::<SudokuParams>();
    let sudoku = move || params.with(unwrap_params);

    view! {
        {move || {
            sudoku_data
                .update(|data| {
                    update_from_sudoku(data, &sudoku(), true);
                });
        }}
        <div class="p-1 h-full min-h-screen w-full bg-sky-100">
            <div class="m-10 p-10 pt-20 space-y-6 bg-slate-300 flex flex-col text-center items-center justify-center shadow-lg rounded-3xl">
                <SudokuGrid />
                <KeyboardShortcuts />
                <SudokuDisplay />
            </div>
        </div>
    }
}

fn unwrap_params(params: &Result<SudokuParams, ParamsError>) -> Sudoku {
    params
        .as_ref()
        .ok()
        .and_then(|p| p.sudoku.clone())
        .and_then(|s| decompress_string(&s))
        .filter(|s| is_valid_game_str(s))
        .and_then(|s| Sudoku::from_str(&s).ok())
        .unwrap_or_default()
}

#[component]
fn SudokuGrid() -> impl IntoView {
    view! {
        <div
            style="width: min(60vw, 60vh);height: min(60vw, 60vh);font-family: 'Source Sans Pro'"
            class="bg-white border-gray-800 border-4 shadow-lg flex flex-col m-auto lining-nums"
        >
            <SudokuRow idx=0 />
            <SudokuRow idx=1 />
            <SudokuRow idx=2 />
        </div>
    }
}

#[component]
fn SudokuRow(idx: usize) -> impl IntoView {
    view! {
        <div class="basis-1/3 flex w-full">
            <SudokuBox idx=3 * idx />
            <SudokuBox idx=3 * idx + 1 />
            <SudokuBox idx=3 * idx + 2 />
        </div>
    }
}

#[component]
fn SudokuBox(idx: usize) -> impl IntoView {
    view! {
        <div class="border-gray-800 border-1 outline-gray-800 outline outline-2 flex flex-col basis-1/3">
            <SudokuBoxRow row=idx / 3 * 3 idx=idx % 3 />
            <SudokuBoxRow row=idx / 3 * 3 + 1 idx=idx % 3 />
            <SudokuBoxRow row=idx / 3 * 3 + 2 idx=idx % 3 />
        </div>
    }
}

#[component]
fn SudokuBoxRow(row: usize, idx: usize) -> impl IntoView {
    view! {
        <div class="flex basis-1/3">
            <SudokuCell row=row col=3 * idx />
            <SudokuCell row=row col=3 * idx + 1 />
            <SudokuCell row=row col=3 * idx + 2 />
        </div>
    }
}

const fn get_cell_classes(is_selected: bool) -> &'static str {
    if is_selected {
        "border-gray-600 border border-1 hover:bg-blue-300 bg-gray-300 flex justify-center items-center basis-1/3 select-none"
    } else {
        "border-gray-600 border border-1 hover:bg-blue-100 flex justify-center items-center basis-1/3 select-none"
    }
}

fn is_active_cell(row: usize, col: usize) -> bool {
    let game_state = unwrap_or_panic(use_context::<RwSignal<GameState>>());
    game_state.with(|state| state.active_cell.is_some_and(|(r, c)| r == row && c == col))
}

#[component]
fn SudokuCell(row: usize, col: usize) -> impl IntoView {
    let game_state = unwrap_or_panic(use_context::<RwSignal<GameState>>());
    view! {
        <div
            style="font-size: min(5vw, 5vh);"
            class=move || {
                let is_selected = is_active_cell(row, col);
                get_cell_classes(is_selected)
            }
            on:click=move |_| {
                game_state
                    .update(|state| {
                        state.active_cell = Some((row, col));
                    });
            }
        >
            <CellInside row=row col=col />
        </div>
    }
}

#[component]
fn CellChoice(idx: usize, show: bool) -> impl IntoView {
    view! {
        <div class="w-1/3 basis-1/3 flex items-center justify-center">
            <p class="min-h-0 leading-none">
                {if show { (idx + 1).to_string() } else { String::new() }}
            </p>
        </div>
    }
}

fn render_choices(choices: &[bool; 9]) -> leptos::HtmlElement<leptos::html::Div> {
    if choices.iter().all(|&b| b) {
        view! { <div class="flex flex-col w-full h-full" /> }
    } else {
        view! {
            <div
                style="font-size: min(1.5vw, 1.5vh);"
                class="flex flex-col w-full h-full text-slate-500"
            >
                <div class="flex flex-row basis-1/3">
                    <CellChoice idx=0 show=choices[0] />
                    <CellChoice idx=1 show=choices[1] />
                    <CellChoice idx=2 show=choices[2] />
                </div>
                <div class="flex flex-row basis-1/3">
                    <CellChoice idx=3 show=choices[3] />
                    <CellChoice idx=4 show=choices[4] />
                    <CellChoice idx=5 show=choices[5] />
                </div>
                <div class="flex flex-row basis-1/3">
                    <CellChoice idx=6 show=choices[6] />
                    <CellChoice idx=7 show=choices[7] />
                    <CellChoice idx=8 show=choices[8] />
                </div>

            </div>
        }
    }
}

#[component]
fn CellInside(row: usize, col: usize) -> impl IntoView {
    let sudoku_data = unwrap_or_panic(use_context::<RwSignal<SudokuData>>());

    view! {
        {move || {
            sudoku_data
                .with(|sudoku| match sudoku.get(row, col) {
                    Cell::Empty { choices } => render_choices(choices),
                    Cell::Value { value, .. } => {
                        view! {
                            <div>
                                <p class="min-h-0 leading-none">{*value}</p>
                            </div>
                        }
                    }
                    Cell::FixedValue { value } => {
                        view! {
                            <div>
                                <p class="min-h-0 leading-none text-sky-700">{*value}</p>
                            </div>
                        }
                    }
                })
        }}
    }
}

#[component]
fn SudokuDisplay() -> impl IntoView {
    let sudoku_data = unwrap_or_panic(use_context::<RwSignal<SudokuData>>());

    view! {
        <div class="bg-slate-100 rounded-3xl p-4 shadow-lg text-xs">
            <p class="font-mono">{move || sudoku_data().to_string()}</p>
            <p class="font-mono text-slate-400">{move || sudoku_data().to_compressed()}</p>
            <Message />
        </div>
    }
}

#[component]
fn Message() -> impl IntoView {
    let game_state = unwrap_or_panic(use_context::<RwSignal<GameState>>());

    // If no message is available, use a zero-width space to keep the layout stable
    view! { <p class="font-mono">{move || game_state().message.unwrap_or_else(|| "\u{200b}".into())}</p> }
}

#[component]
fn KeyboardShortcut(
    key: &'static str,
    action: &'static str,
    f: impl FnMut(MouseEvent) + 'static,
) -> impl IntoView {
    view! {
        <div
            class="p-2 space-x-2 flex outline outline-1 shadow-lg rounded-lg items-center bg-slate-400 hover:bg-blue-400 select-none"
            on:click=f
        >
            <div class="flex h-6 w-6 justify-center rounded-lg outline outline-1 items-center bg-slate-500">
                <p class="min-h-0 leading-none font-mono font-thin text-lg text-white">{key}</p>
            </div>
            <p class="min-h-0 leading-none text-sm font-mono text-white">{action}</p>
        </div>
    }
}

#[component]
fn KeyboardShortcuts() -> impl IntoView {
    let sudoku = unwrap_or_panic(use_context::<RwSignal<SudokuData>>());
    let game_state = unwrap_or_panic(use_context::<RwSignal<GameState>>());

    view! {
        <div class="flex space-x-2 p-2">
            <KeyboardShortcut
                key="A"
                action="Singles"
                f=move |_| {
                    update!(
                        |game_state, sudoku| {
                            game_state.show_result(place_all_visible_singles(sudoku));
                        }
                    );
                }
            />
            <KeyboardShortcut
                key="S"
                action="Hidden"
                f=move |_| {
                    update!(
                        |game_state, sudoku| {
                            game_state.show_result(place_all_hidden_singles(sudoku));
                        }
                    );
                }
            />
            <KeyboardShortcut
                key="D"
                action="Doubles"
                f=move |_| {
                    update!(
                        |game_state, sudoku| {
                            game_state.show_result(check_all_visible_doubles(sudoku));
                        }
                    );
                }
            />
            <KeyboardShortcut
                key="F"
                action="Constraints"
                f=move |_| {
                    update!(
                        |game_state, sudoku| {
                            game_state.show_result(check_constraints(sudoku));
                        }
                    );
                }
            />
            <KeyboardShortcut
                key="G"
                action="Solve"
                f=move |_| {
                    update!(
                        |game_state, sudoku| {
                            game_state.show_result(solve_sudoku(sudoku));
                        }
                    );
                }
            />
        </div>
    }
}

#[allow(clippy::panic)]
fn unwrap_or_panic<T>(signal: Option<RwSignal<T>>) -> RwSignal<T> {
    signal.unwrap_or_else(|| {
        console_error("Component not available");
        panic!("Component not available");
    })
}
