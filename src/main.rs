use hotkeys::setup_hotkeys;
use leptos_hotkeys::{provide_hotkeys_context, scopes, HotkeysContext};
use state::{Cell, GameState, SudokuData};

use leptos::*;
use leptos_meta::provide_meta_context;

mod hotkeys;
mod state;

#[component]
fn CellChoice(idx: usize, show: bool) -> impl IntoView {
    view! {
        <div class="w-1/3 basis-1/3 flex items-center justify-center">
            <p class="min-h-0 leading-none">
                {if show { (idx + 1).to_string() } else { "".to_string() }}
            </p>
        </div>
    }
}

fn render_choices(choices: &[bool; 9]) -> leptos::HtmlElement<leptos::html::Div> {
    view! {
        <div
            style="font-size: min(1vw, 1vh);"
            class="flex flex-col w-full h-full font-serif text-slate-500"
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

#[component]
fn CellInside(row: usize, col: usize) -> impl IntoView {
    let sudoku = use_context::<RwSignal<SudokuData>>().unwrap();
    view! {
        {move || {
            sudoku
                .with(|sudoku| match sudoku.get(row, col) {
                    Cell::Empty { choices } => render_choices(choices),
                    Cell::Value(v) => {
                        view! {
                            <div>
                                <p class="min-h-0 leading-none">{*v}</p>
                            </div>
                        }
                    }
                })
        }}
    }
}

fn get_cell_classes(is_selected: bool) -> &'static str {
    if is_selected {
        "border-gray-800 border hover:border-2 hover:bg-gray-300 bg-gray-300 flex justify-center items-center basis-1/3 select-none font-serif"
    } else {
        "border-gray-800 border hover:border-2 hover:bg-gray-100 flex justify-center items-center basis-1/3 select-none font-serif"
    }
}

#[component]
fn SudokuCell(row: usize, col: usize) -> impl IntoView {
    let game_state = use_context::<RwSignal<GameState>>().unwrap();

    view! {
        <div
            style="font-size: min(4vw, 4vh);"
            class=move || {
                if let Some((r, c)) = game_state.with(|game_state| game_state.active_cell) {
                    get_cell_classes(r == row && c == col)
                } else {
                    get_cell_classes(false)
                }
            }
            on:click=move |_| {
                game_state
                    .update(|game_state| {
                        game_state.active_cell = Some((row, col));
                    });
            }
        >
            <CellInside row=row col=col />
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
fn SudokuBox(idx: usize) -> impl IntoView {
    view! {
        <div class="border-gray-800 border-2 flex flex-col basis-1/3">
            <SudokuBoxRow row=idx / 3 * 3 idx=idx % 3 />
            <SudokuBoxRow row=idx / 3 * 3 + 1 idx=idx % 3 />
            <SudokuBoxRow row=idx / 3 * 3 + 2 idx=idx % 3 />
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
fn SudokuGrid() -> impl IntoView {
    view! {
        <div
            style="width: min(50vw, 50vh);height: min(50vw, 50vh);"
            class="bg-white border-gray-800 border-4 shadow-lg flex flex-col m-auto"
        >
            <SudokuRow idx=0 />
            <SudokuRow idx=1 />
            <SudokuRow idx=2 />
        </div>
    }
}

#[component]
fn SudokuDisplay() -> impl IntoView {
    let sudoku = use_context::<RwSignal<SudokuData>>().unwrap();

    view! {
        <div class="bg-slate-100 rounded-3xl p-4">
            <p class="font-mono">{move || sudoku().to_string()}</p>
        </div>
    }
}

#[component]
fn SudokuGame() -> impl IntoView {
    let game_state = create_rw_signal(GameState::default());
    let sudoku = create_rw_signal(SudokuData::default());
    provide_context(game_state);
    provide_context(sudoku);
    setup_hotkeys(game_state, sudoku);

    view! {
        <div class="p-1 h-screen max-h w-full bg-sky-100">
            <div class="m-20 p-20 space-y-10 bg-slate-300 flex flex-col text-center items-center justify-center shadow-lg rounded-3xl">
                <h1 class="text-6xl font-serif">"Sudoku"</h1>
                <SudokuGrid />
                <SudokuDisplay />
            </div>
        </div>
    }
}

#[component]
fn App() -> impl IntoView {
    provide_meta_context();

    let main_ref = create_node_ref::<html::Main>();
    let HotkeysContext { .. } = provide_hotkeys_context(main_ref, false, scopes!());

    view! {
        <main _ref=main_ref>
            <SudokuGame />
        </main>
    }
}

fn main() {
    console_error_panic_hook::set_once();
    mount_to_body(|| view! { <App /> })
}
