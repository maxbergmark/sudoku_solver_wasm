use crate::actions::update_from_sudoku;
use crate::state::{Cell, GameState, SudokuData};
use crate::ui::{
    DarkModeToggle, DigitDisplay, GeneratorShortcuts, KeyboardShortcuts, SudokuDisplay,
};
use crate::util::{unwrap_or_panic, unwrap_params, SudokuParams};

use leptos::{
    component, create_memo, use_context, view, IntoView, RwSignal, SignalUpdate, SignalWith,
};
use leptos_router::use_query;

#[component]
pub fn SudokuGame() -> impl IntoView {
    let sudoku_data = unwrap_or_panic(use_context::<RwSignal<SudokuData>>());
    let params = use_query::<SudokuParams>();
    let sudoku = move || params.with(unwrap_params);
    let update = move |data: &mut SudokuData| {
        data.clear();
        update_from_sudoku(data, &sudoku(), true);
    };
    view! {
        {move || sudoku_data.update(update)}
        <div class="p-1 h-full min-h-screen w-full bg-sky-100 dark:bg-black fade-dark">
            <div class="m-10 p-10 pt-20 space-y-6 bg-slate-300 dark:bg-zinc-950 outline outline-1 outline-slate-300 dark:outline-zinc-900 flex flex-col text-center items-center justify-center shadow-lg rounded-3xl fade-dark">
                <div class="absolute top-0 right-0 p-4 m-10">
                    <DarkModeToggle />
                </div>
                <SudokuGrid />
                <div class="flex space-x-10">
                    <DigitDisplay />
                    <KeyboardShortcuts />
                    <GeneratorShortcuts />
                </div>
                <SudokuDisplay />
            </div>
        </div>
    }
}

#[component]
fn SudokuGrid() -> impl IntoView {
    view! {
        <div
            style="width: min(60vw, 60vh);height: min(60vw, 60vh);font-family: 'Source Sans Pro', serif"
            class="bg-white border-gray-800 dark:bg-black border-4 shadow-lg flex flex-col m-auto lining-nums fade-dark"
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
        <div class="border-gray-800 border-2 z-10 flex flex-col basis-1/3">
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
    let class = move || {
        if game_state().is_active_cell(row, col) {
            "sudoku-cell hover:bg-cerulean-blue-300 dark:hover:bg-zinc-800 bg-gray-300 dark:bg-zinc-900"
        } else {
            "sudoku-cell hover:bg-cerulean-blue-100 dark:hover:bg-zinc-900"
        }
    };
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
    let effect = create_memo(move |_| sudoku_data.with(|sudoku| sudoku.get(row, col)));
    move || effect.with(render_cell)
}

fn render_cell(cell: &Cell) -> leptos::HtmlElement<leptos::html::Div> {
    match cell {
        Cell::Empty { choices } => render_choices(choices),
        Cell::Value { value, .. } => render_value(&ValueType::Value(*value)),
        Cell::AnimatedValue {
            value,
            fade_delay_ms,
            animation,
            ..
        } => render_value(&ValueType::FadeInValue {
            value: *value,
            fade_delay_ms: *fade_delay_ms,
            animation,
        }),
        Cell::FixedValue { value } => render_value(&ValueType::FixedValue(*value)),
        Cell::Error { value, .. } => render_value(&ValueType::Error(*value)),
    }
}

enum ValueType {
    Value(u8),
    FadeInValue {
        value: u8,
        fade_delay_ms: i32,
        animation: &'static str,
    },
    FixedValue(u8),
    Error(u8),
}

fn render_value(value: &ValueType) -> leptos::HtmlElement<leptos::html::Div> {
    let (style, class) = match value {
        ValueType::Value(_) => (
            String::default(),
            "min-h-0 leading-none dark:text-gray-500 fade-dark".to_string(),
        ),
        ValueType::FadeInValue {
            fade_delay_ms,
            animation,
            ..
        } => (
            format!("animation-delay: {fade_delay_ms}ms;"),
            format!("min-h-0 leading-none dark:text-gray-500 {animation} fade-dark"),
        ),
        ValueType::FixedValue(_) => (
            String::default(),
            "min-h-0 leading-none text-cerulean-blue-700".to_string(),
        ),
        ValueType::Error(_) => (
            String::default(),
            "min-h-0 leading-none text-red-700".to_string(),
        ),
    };
    let v = match value {
        ValueType::Value(v)
        | ValueType::FixedValue(v)
        | ValueType::Error(v)
        | ValueType::FadeInValue { value: v, .. } => *v,
    };
    view! {
        <div>
            <p class=class style=style>
                {v}
            </p>
        </div>
    }
}
