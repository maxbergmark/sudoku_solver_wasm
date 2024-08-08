use leptos::{component, ev::MouseEvent, update, use_context, view, IntoView, RwSignal};

use crate::{
    actions::{
        check_all_visible_doubles, check_constraints, check_triples, place_all_hidden_singles,
        place_all_visible_singles, solve_sudoku, toggle_choice_if_selected,
        toggle_digit_if_selected, verify_sudoku,
    },
    state::{DigitMode, GameState, SudokuData},
    util::unwrap_or_panic,
};

#[component]
pub fn DigitDisplay() -> impl IntoView {
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
            <p
                class="leading-none text-white text-4xl"
                style="font-family: 'Source Sans Pro', serif"
            >
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

#[component]
pub fn SudokuDisplay() -> impl IntoView {
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
pub fn KeyboardShortcuts() -> impl IntoView {
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
