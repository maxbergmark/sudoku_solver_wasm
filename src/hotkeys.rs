use leptos::{update, RwSignal};
use leptos_hotkeys::{use_hotkeys, use_hotkeys_scoped};

use crate::{
    actions::{
        check_all_visible_doubles, check_constraints, clear_digit_if_selected, handle_arrow,
        place_all_hidden_singles, place_all_visible_singles, set_digit_if_selected, solve_sudoku,
    },
    state::{GameState, SudokuData},
};

pub fn setup_hotkeys(game_state: RwSignal<GameState>, sudoku: RwSignal<SudokuData>) {
    setup_placement_hotkeys(game_state, sudoku);
    setup_solver_hotkeys(game_state, sudoku);
    setup_movement_hotkeys(game_state);
}

fn setup_placement_hotkeys(game_state: RwSignal<GameState>, sudoku: RwSignal<SudokuData>) {
    for i in 1..=9 {
        setup_digit_hotkey(i, game_state, sudoku);
    }
    use_hotkeys!((format!("escape,backspace")) => move |()| {
        update!(|game_state, sudoku| {
            clear_digit_if_selected(game_state, sudoku);
        });
    });
}

fn setup_solver_hotkeys(game_state: RwSignal<GameState>, sudoku: RwSignal<SudokuData>) {
    use_hotkeys!(("KeyA") => move |()| {
        update!(|game_state, sudoku| {
            game_state.show_result(place_all_visible_singles(sudoku));
        });
    });
    use_hotkeys!(("KeyS") => move |()| {
        update!(|game_state, sudoku| {
            game_state.show_result(place_all_hidden_singles(sudoku));
        });
    });
    use_hotkeys!(("KeyD") => move |()| {
        update!(|game_state, sudoku| {
            game_state.show_result(check_all_visible_doubles(sudoku));
        });
    });
    use_hotkeys!(("KeyF") => move |()| {
        update!(|game_state, sudoku| {
            game_state.show_result(check_constraints(sudoku));
        });
    });
    use_hotkeys!(("KeyG") => move |()| {
        update!(|game_state, sudoku| {
            game_state.show_result(solve_sudoku(sudoku));
        });
    });
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
    use_hotkeys!((format!("digit{i}")) => move |()| {
        update!(|game_state, sudoku| {
            set_digit_if_selected(game_state, sudoku, i as u8);
        });
    });
}
