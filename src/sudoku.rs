use std::str::FromStr;

use crate::actions::update_from_sudoku;
use crate::hotkeys::setup_hotkeys;
use crate::state::{decompress_string, Cell, GameState, SudokuData};
use crate::ui::{DigitDisplay, KeyboardShortcuts, SudokuDisplay};
use crate::util::unwrap_or_panic;
use rust_sudoku_solver::Sudoku;

use leptos::{
    component, create_rw_signal, provide_context, use_context, view, IntoView, Params, RwSignal,
    SignalUpdate, SignalWith,
};
use leptos_router::{use_query, Params, ParamsError};

#[derive(Params, PartialEq, Debug)]
struct SudokuParams {
    sudoku: Option<String>,
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
    let update = move |data: &mut SudokuData| update_from_sudoku(data, &sudoku(), true);
    view! {
        {move || sudoku_data.update(update)}
        <div class="p-1 h-full min-h-screen w-full bg-sky-100">
            <div class="m-10 p-10 pt-20 space-y-6 bg-slate-300 flex flex-col text-center items-center justify-center shadow-lg rounded-3xl">
                <SudokuGrid />
                <div class="flex space-x-10">
                    <DigitDisplay />
                    <KeyboardShortcuts />
                </div>
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

fn is_valid_game_str(game_str: &str) -> bool {
    game_str.len() == 81 && game_str.chars().all(|c| c.is_ascii_digit() || c == '.')
}

#[component]
fn SudokuGrid() -> impl IntoView {
    view! {
        <div
            style="width: min(60vw, 60vh);height: min(60vw, 60vh);font-family: 'Source Sans Pro', serif"
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

#[component]
fn SudokuCell(row: usize, col: usize) -> impl IntoView {
    let game_state = unwrap_or_panic(use_context::<RwSignal<GameState>>());
    let on_click = move |_| {
        game_state.update(|state| {
            state.active_cell = Some((row, col));
        });
    };
    let class = move || get_cell_classes(is_active_cell(row, col));
    view! {
        <div style="font-size: min(5vw, 5vh);" class=class on:click=on_click>
            <CellInside row=row col=col />
        </div>
    }
}

#[component]
fn CellChoiceRow(idx: usize, choices: [bool; 9]) -> impl IntoView {
    view! {
        <div class="flex flex-row basis-1/3">
            <CellChoice idx=3 * idx show=choices[3 * idx] />
            <CellChoice idx=3 * idx + 1 show=choices[3 * idx + 1] />
            <CellChoice idx=3 * idx + 2 show=choices[3 * idx + 2] />
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

fn render_choices(choices: &[bool; 9]) -> leptos::HtmlElement<leptos::html::Div> {
    if choices.iter().all(|&b| b) {
        view! { <div class="flex flex-col w-full h-full" /> }
    } else {
        view! {
            <div
                style="font-size: min(1.5vw, 1.5vh);"
                class="flex flex-col w-full h-full text-slate-500"
            >
                <CellChoiceRow idx=0 choices=*choices />
                <CellChoiceRow idx=1 choices=*choices />
                <CellChoiceRow idx=2 choices=*choices />
            </div>
        }
    }
}

#[component]
fn CellInside(row: usize, col: usize) -> impl IntoView {
    let sudoku_data = unwrap_or_panic(use_context::<RwSignal<SudokuData>>());
    move || sudoku_data.with(|sudoku| render_cell(sudoku, row, col))
}

fn render_cell(
    sudoku: &SudokuData,
    row: usize,
    col: usize,
) -> leptos::HtmlElement<leptos::html::Div> {
    match sudoku.get(row, col) {
        Cell::Empty { choices } => render_choices(choices),
        Cell::Value { value, .. } => render_value(&ValueType::Value(*value)),
        Cell::FixedValue { value } => render_value(&ValueType::FixedValue(*value)),
        Cell::Error { value, .. } => render_value(&ValueType::Error(*value)),
    }
}

enum ValueType {
    Value(u8),
    FixedValue(u8),
    Error(u8),
}

fn render_value(value: &ValueType) -> leptos::HtmlElement<leptos::html::Div> {
    let class = match value {
        ValueType::Value(_) => "min-h-0 leading-none",
        ValueType::FixedValue(_) => "min-h-0 leading-none text-sky-700",
        ValueType::Error(_) => "min-h-0 leading-none text-red-700",
    };
    let v = match value {
        ValueType::Value(v) | ValueType::FixedValue(v) | ValueType::Error(v) => *v,
    };
    view! {
        <div>
            <p class=class>{v}</p>
        </div>
    }
}
