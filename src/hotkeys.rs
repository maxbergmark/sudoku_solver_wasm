use leptos::{create_rw_signal, provide_context, update, RwSignal, SignalUpdate};
use leptos_hotkeys::{use_hotkeys, use_hotkeys_context, use_hotkeys_scoped, HotkeysContext};

use crate::{
    actions::{
        check_all_visible_doubles, check_constraints, check_triples, clear_digit_if_selected,
        handle_arrow, load_random_sudoku, place_all_hidden_singles, place_all_visible_singles,
        solve_sudoku, toggle_choice_if_selected, toggle_digit_if_selected, verify_sudoku,
    },
    generator::Difficulty,
    state::{DigitMode, GameState},
    sudoku_data::SudokuData,
    Result,
};

pub struct Hotkey {
    pub key: &'static str,
    pub action: &'static str,
    pub on_click: fn(&mut SudokuData) -> Result<String>,
}

pub struct GeneratorHotkey {
    pub key: &'static str,
    pub action: &'static str,
    pub difficulty: Difficulty,
}

#[allow(clippy::module_name_repetitions)]
pub fn setup_hotkeys(game_state: RwSignal<GameState>, sudoku: RwSignal<SudokuData>) {
    setup_placement_hotkeys(game_state, sudoku);
    setup_solver_hotkeys(game_state, sudoku);
    setup_movement_hotkeys(game_state);

    let HotkeysContext {
        toggle_scope,
        enable_scope,
        ..
    } = use_hotkeys_context();
    let digit_mode = create_rw_signal(DigitMode::Value);
    provide_context(digit_mode);
    enable_scope("place_digits".into());

    use_hotkeys!(("Tab") => move |()| {
        toggle_scope("toggle_choices".into());
        toggle_scope("place_digits".into());
        digit_mode.update(DigitMode::toggle);
    });
}

fn setup_placement_hotkeys(game_state: RwSignal<GameState>, sudoku: RwSignal<SudokuData>) {
    for i in 1..=9 {
        setup_digit_hotkey(i, game_state, sudoku);
        setup_digit_choice_hotkey(i, game_state, sudoku);
    }
    use_hotkeys!((format!("Escape,Backspace")) => move |()| {
        update!(|game_state, sudoku| {
            clear_digit_if_selected(game_state, sudoku);
        });
    });
}

#[allow(clippy::module_name_repetitions)]
pub fn get_solver_hotkeys() -> Vec<Hotkey> {
    vec![
        Hotkey {
            key: "A",
            action: "SINGLES",
            on_click: place_all_visible_singles,
        },
        Hotkey {
            key: "S",
            action: "HIDDEN",
            on_click: place_all_hidden_singles,
        },
        Hotkey {
            key: "D",
            action: "DOUBLES",
            on_click: check_all_visible_doubles,
        },
        Hotkey {
            key: "F",
            action: "TRIPLES",
            on_click: check_triples,
        },
        Hotkey {
            key: "G",
            action: "CONSTRAINTS",
            on_click: check_constraints,
        },
        Hotkey {
            key: "H",
            action: "SOLVE",
            on_click: solve_sudoku,
        },
        Hotkey {
            key: "J",
            action: "VERIFY",
            on_click: verify_sudoku,
        },
    ]
}

#[allow(clippy::module_name_repetitions)]
pub fn get_generator_hotkeys() -> Vec<GeneratorHotkey> {
    vec![
        GeneratorHotkey {
            key: "B",
            action: "HARD",
            difficulty: Difficulty::Hard,
        },
        GeneratorHotkey {
            key: "N",
            action: "17 CLUE",
            difficulty: Difficulty::Clue17,
        },
        GeneratorHotkey {
            key: "M",
            action: "EXTREME",
            difficulty: Difficulty::Extreme,
        },
    ]
}

fn setup_solver_hotkeys(game_state: RwSignal<GameState>, sudoku: RwSignal<SudokuData>) {
    let apply_and_show = move |f: fn(&mut SudokuData) -> crate::Result<String>| {
        move |()| {
            update!(|game_state, sudoku| {
                game_state.show_result(f(sudoku));
            });
        }
    };

    for shortcut in get_solver_hotkeys() {
        use_hotkeys!((shortcut.key) => apply_and_show(shortcut.on_click));
    }
    for shortcut in get_generator_hotkeys() {
        use_hotkeys!((shortcut.key) => move |()| load_random_sudoku(shortcut.difficulty));
    }
}

fn setup_movement_hotkeys(game_state: RwSignal<GameState>) {
    setup_arrow_hotkey("ArrowRight", (0, 1), game_state);
    setup_arrow_hotkey("ArrowLeft", (0, -1), game_state);
    setup_arrow_hotkey("ArrowUp", (-1, 0), game_state);
    setup_arrow_hotkey("ArrowDown", (1, 0), game_state);
}

fn setup_arrow_hotkey(name: &str, direction: (i32, i32), game_state: RwSignal<GameState>) {
    use_hotkeys!((name) => move |()| {
        handle_arrow(&game_state, direction);
    });
}

fn setup_digit_hotkey(i: usize, game_state: RwSignal<GameState>, sudoku: RwSignal<SudokuData>) {
    use_hotkeys!((format!("{i}"), "place_digits") => move |()| {
        update!(|game_state, sudoku| {
            toggle_digit_if_selected(game_state, sudoku, i as u8);
        });
    });
}

fn setup_digit_choice_hotkey(
    i: usize,
    game_state: RwSignal<GameState>,
    sudoku: RwSignal<SudokuData>,
) {
    use_hotkeys!((format!("{i}"), "toggle_choices") => move |()| {
        update!(|game_state, sudoku| {
            toggle_choice_if_selected(game_state, sudoku, i as u8);
        });
    });
}
