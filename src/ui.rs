use leptos::{
    component, ev::MouseEvent, update, use_context, view, CollectView, IntoView, RwSignal,
    SignalUpdate,
};

use crate::{
    actions::{
        apply_solution, load_random_sudoku, toggle_choice_if_selected, toggle_digit_if_selected,
    },
    generator::Difficulty,
    hotkeys::{get_generator_hotkeys, get_solver_hotkeys},
    state::{DigitMode, GameState},
    sudoku_data::SudokuData,
    util::unwrap_or_panic,
};

#[component]
pub fn DigitDisplay() -> impl IntoView {
    view! {
        <div class="p-2 flex flex-col rounded-2xl bg-slate-100 dark:bg-zinc-900 outline outline-1 outline-slate-100 dark:outline-zinc-800 shadow-2xl justify-between fade-dark">
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
        <div class="btn-primary w-20 h-20 flex items-center justify-center" on:click=on_click>
            <p
                class="leading-none text-white text-5xl"
                style="font-family: 'Source Sans Pro', serif"
            >
                {digit}
            </p>
        </div>
    }
}

#[component]
fn DigitModeDisplay() -> impl IntoView {
    let digit_mode = unwrap_or_panic(use_context::<RwSignal<DigitMode>>());
    view! {
        <div
            class="btn-primary w-full h-14 p-2 pl-2 flex leading-none items-center space-x-2"
            on:click=move |_| {
                digit_mode.update(DigitMode::toggle);
            }
        >
            <div class="basis-8">
                <KeyButton key="↹" />
            </div>
            <div class="max-w-full h-full flex relative basis-full font-sans font-bold">
                // this just makes sure that optional tailwind classes are compiled
                // bg-slate-300 left-10 translate-x-full text-slate-300
                <div class="w-1/2 inline-flex h-full items-center justify-center">
                    <p
                        class:text-slate-300=move || digit_mode() == DigitMode::Choice
                        class:text-white=move || digit_mode() == DigitMode::Value
                        class="z-10 transition-all"
                    >
                        DIGITS
                    </p>
                </div>
                <div class="w-1/2 inline-flex h-full items-center justify-center">
                    <p
                        class:text-slate-300=move || digit_mode() == DigitMode::Value
                        class:text-white=move || digit_mode() == DigitMode::Choice
                        class="z-10 transition-all"
                    >
                        CHOICES
                    </p>
                </div>
                <div
                    class:translate-x-full=move || digit_mode() == DigitMode::Choice
                    class="w-1/2 h-full bg-slate-500 dark:bg-cerulean-blue-500 outline outline-1 outline-slate-500 dark:outline-cerulean-blue-400 opacity-50 absolute transition-all rounded-lg fade-dark"
                />
            </div>
        </div>
    }
}

#[component]
pub fn SudokuDisplay() -> impl IntoView {
    let sudoku_data = unwrap_or_panic(use_context::<RwSignal<SudokuData>>());

    view! {
        <div class="bg-slate-100 dark:bg-zinc-900 outline outline-1 outline-slate-100 dark:outline-zinc-800 rounded-3xl p-4 shadow-lg text-xs fade-dark">
            <p class="font-mono dark:text-white fade-dark">{move || sudoku_data().to_string()}</p>
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
        <p class="font-mono dark:text-white fade-dark">
            {move || game_state().message.unwrap_or_else(|| "\u{200b}".into())}
        </p>
    }
}

#[component]
fn KeyboardShortcut(
    key: &'static str,
    action: &'static str,
    on_click: impl FnMut(MouseEvent) + 'static,
) -> impl IntoView {
    let class = "btn-primary pr-4 p-2 space-x-2 flex items-center";
    view! {
        <div class=class on:click=on_click>
            <KeyButton key=key />
            <p class="min-h-0 leading-none font-sans font-bold text-white">{action}</p>
        </div>
    }
}

#[component]
fn KeyButton(key: &'static str) -> impl IntoView {
    view! {
        <div class="flex h-6 w-6 justify-center rounded-lg items-center bg-slate-500 dark:bg-cerulean-blue-600 outline outline-1 outline-slate-500 dark:outline-cerulean-blue-500 fade-dark">
            <p class="min-h-0 leading-none font-sans font-bold text-white">{key}</p>
        </div>
    }
}

#[component]
pub fn KeyboardShortcuts() -> impl IntoView {
    let set_sudoku = unwrap_or_panic(use_context::<RwSignal<SudokuData>>());
    let set_game_state = unwrap_or_panic(use_context::<RwSignal<GameState>>());

    let with_signals = move |f: fn(&mut SudokuData) -> crate::Result<String>| {
        apply_solution(set_game_state, set_sudoku, f)
    };

    view! {
        <div class="flex space-y-2 p-2 bg-slate-100 dark:bg-zinc-900 outline outline-1 outline-slate-100 dark:outline-zinc-800 rounded-2xl flex-col fade-dark">
            {get_solver_hotkeys()
                .into_iter()
                .map(|shortcut| {
                    view! {
                        <KeyboardShortcut
                            key=shortcut.key
                            action=shortcut.action
                            on_click=with_signals(shortcut.on_click)
                        />
                    }
                })
                .collect_view()}
        </div>
    }
}

#[component]
pub fn GeneratorShortcuts() -> impl IntoView {
    view! {
        <div class="flex space-y-2 p-2 bg-slate-100 justify-between dark:bg-zinc-900 outline outline-1 outline-slate-100 dark:outline-zinc-800 rounded-2xl flex-col fade-dark">
            {get_generator_hotkeys()
                .into_iter()
                .map(|shortcut| {
                    view! {
                        <GenerateSudokuButton
                            key=shortcut.key
                            text=shortcut.action
                            difficulty=shortcut.difficulty
                        />
                    }
                })
                .collect_view()}
        </div>
    }
}

#[component]
fn GenerateSudokuButton(
    key: &'static str,
    text: &'static str,
    difficulty: Difficulty,
) -> impl IntoView {
    view! {
        <div
            class="btn-primary pr-4 p-2 space-x-2 flex items-center"
            on:click=move |_| load_random_sudoku(difficulty)
        >
            <KeyButton key=key />
            <p class="min-h-0 leading-none font-sans font-bold text-white">{text}</p>
        </div>
    }
}

#[component]
pub fn DarkModeToggle() -> impl IntoView {
    let game_state = unwrap_or_panic(use_context::<RwSignal<GameState>>());
    let on_click = move |_| {
        update!(|game_state| {
            game_state.dark_mode.toggle();
        });
    };
    view! {
        <div
            class="btn-primary p-1 w-14 h-8 flex rounded-full items-center fade-dark"
            on:click=on_click
        >
            <div class="opacity-0" />
            <div
                class:translate-x-full=move || game_state().dark_mode.active()
                class="w-6 h-6 rounded-full bg-slate-500 dark:bg-cerulean-blue-600 shadow-sm transition-all"
            >
                <img
                    src="https://cdn2.iconfinder.com/data/icons/ui-minimalist-0-1-1/16/UI_Web_Moon_Night_Night_Mode_Dark-512.png"
                    alt="Dark mode"
                    class="w-6 h-6 absolute transition-all invert"
                    class:opacity-0=move || !game_state().dark_mode.active()
                />
                <img
                    src="https://static.thenounproject.com/png/4808961-200.png"
                    alt="Light mode"
                    class="w-6 h-6 absolute transition-all invert"
                    class:opacity-0=move || game_state().dark_mode.active()
                />
            </div>
        </div>
    }
}
