use std::str::FromStr;

use crate::actions::{
    check_all_visible_doubles, check_constraints, check_triples, place_all_hidden_singles,
    place_all_visible_singles, solve_sudoku, toggle_choice_if_selected, toggle_digit_if_selected,
    update_from_sudoku, verify_sudoku,
};
use crate::hotkeys::setup_hotkeys;
use crate::state::{decompress_string, Cell, DigitMode, GameState, SudokuData};
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

#[component]
fn DigitDisplay() -> impl IntoView {
    view! {
        <div class="p-2 flex flex-col rounded-2xl bg-slate-100 shadow-2xl justify-between">
            <DigitModeDisplay />
            <div class="flex flex-col space-y-2">
                <DigitButtonRow start_digit=1 />
                <DigitButtonRow start_digit=4 />
                <DigitButtonRow start_digit=7 />
            </div>
        </div>
    }
}

#[component]
fn DigitButtonRow(start_digit: u8) -> impl IntoView {
    view! {
        <div class="flex space-x-2 float-right">
            <DigitButton digit=start_digit />
            <DigitButton digit=start_digit + 1 />
            <DigitButton digit=start_digit + 2 />
        </div>
    }
}

#[component]
fn DigitButton(digit: u8) -> impl IntoView {
    let game_state = unwrap_or_panic(use_context::<RwSignal<GameState>>());
    let sudoku_data = unwrap_or_panic(use_context::<RwSignal<SudokuData>>());
    let current_scope = unwrap_or_panic(use_context::<RwSignal<DigitMode>>());
    let on_click = move |_| {
        update!(|game_state, sudoku_data| {
            match current_scope() {
                DigitMode::Value => toggle_digit_if_selected(game_state, sudoku_data, digit),
                DigitMode::Choice => toggle_choice_if_selected(game_state, sudoku_data, digit),
            }
        });
    };
    view! {
        <div
            class="w-20 h-20 flex outline outline-1 rounded-lg items-center justify-center bg-slate-400 hover:bg-blue-400 select-none"
            on:click=on_click
        >
            <p class="leading-none text-white text-4xl" style="font-family: 'Source Sans Pro'">
                {digit}
            </p>
        </div>
    }
}

#[component]
fn DigitModeDisplay() -> impl IntoView {
    let current_scope = unwrap_or_panic(use_context::<RwSignal<DigitMode>>());
    view! {
        <div
            class="w-full h-14 p-2 pl-2 flex bg-slate-400 outline outline-1 rounded-lg leading-none items-center space-x-2 select-none"
            on:click=move |_| {
                update!(
                    |current_scope| {
                    *current_scope = match *current_scope {
                        DigitMode::Value => DigitMode::Choice,
                        DigitMode::Choice => DigitMode::Value,
                    };
                }
                );
            }
        >
            <div class="basis-8">
                <KeyButton key="↹" />
            </div>
            <div class="max-w-full h-full flex relative basis-full">
                // this just makes sure that optional tailwind classes are compiled
                <div class="bg-slate-300 left-10 translate-x-full text-slate-300" />
                <div class="w-1/2 inline-flex font-mono h-full items-center justify-center">
                    <p
                        class:text-slate-300=move || current_scope() == DigitMode::Choice
                        class:text-white=move || current_scope() == DigitMode::Value
                        class="z-10 transition-all"
                    >
                        Digits
                    </p>
                </div>
                <div class="w-1/2 inline-flex font-mono h-full items-center justify-center">
                    <p
                        class:text-slate-300=move || current_scope() == DigitMode::Value
                        class:text-white=move || current_scope() == DigitMode::Choice
                        class="z-10 transition-all"
                    >
                        Choices
                    </p>
                </div>
                <div
                    class:translate-x-full=move || current_scope() == DigitMode::Choice
                    class="w-1/2 h-full bg-slate-500 absolute transition-all rounded-lg outline outline-1"
                />
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
    view! {
        <p class="font-mono">{move || game_state().message.unwrap_or_else(|| "\u{200b}".into())}</p>
    }
}

#[component]
fn KeyboardShortcut(
    key: &'static str,
    action: &'static str,
    on_click: impl FnMut(MouseEvent) + 'static,
) -> impl IntoView {
    let class = "p-2 space-x-2 flex outline outline-1 shadow-lg rounded-lg items-center bg-slate-400 hover:bg-blue-400 select-none";
    view! {
        <div class=class on:click=on_click>
            <KeyButton key=key />
            <p class="min-h-0 leading-none text-sm font-mono text-white">{action}</p>
        </div>
    }
}

#[component]
fn KeyButton(key: &'static str) -> impl IntoView {
    view! {
        <div class="flex h-6 w-6 justify-center rounded-lg outline outline-1 items-center bg-slate-500">
            <p class="min-h-0 leading-none font-mono font-thin text-lg text-white">{key}</p>
        </div>
    }
}

fn apply_solution(
    game_state: RwSignal<GameState>,
    sudoku: RwSignal<SudokuData>,
    f: impl Fn(&mut SudokuData) -> crate::Result<String>,
) -> impl Fn(MouseEvent) {
    move |_| {
        update!(|game_state, sudoku| {
            game_state.show_result(f(sudoku));
        });
    }
}

#[component]
fn KeyboardShortcuts() -> impl IntoView {
    let sudoku = unwrap_or_panic(use_context::<RwSignal<SudokuData>>());
    let game_state = unwrap_or_panic(use_context::<RwSignal<GameState>>());

    let with_signals = move |f: fn(&mut SudokuData) -> crate::Result<String>| {
        apply_solution(game_state, sudoku, f)
    };

    view! {
        <div class="flex space-y-2 p-2 bg-slate-100 rounded-2xl flex-col">
            <KeyboardShortcut
                key="A"
                action="Singles"
                on_click=with_signals(place_all_visible_singles)
            />
            <KeyboardShortcut
                key="S"
                action="Hidden"
                on_click=with_signals(place_all_hidden_singles)
            />
            <KeyboardShortcut
                key="D"
                action="Doubles"
                on_click=with_signals(check_all_visible_doubles)
            />
            <KeyboardShortcut key="F" action="Triples" on_click=with_signals(check_triples) />
            <KeyboardShortcut
                key="G"
                action="Constraints"
                on_click=with_signals(check_constraints)
            />
            <KeyboardShortcut key="H" action="Solve" on_click=with_signals(solve_sudoku) />
            <KeyboardShortcut key="J" action="Verify" on_click=with_signals(verify_sudoku) />

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
