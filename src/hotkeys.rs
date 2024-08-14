use leptos::{create_rw_signal, provide_context, update, RwSignal, SignalUpdate};
use leptos_hotkeys::{use_hotkeys, use_hotkeys_context, use_hotkeys_scoped, HotkeysContext};

use crate::{
    actions::{
        check_all_visible_doubles, check_constraints, check_triples, clear_digit_if_selected,
        handle_arrow, place_all_hidden_singles, place_all_visible_singles, solve_sudoku,
        toggle_choice_if_selected, toggle_digit_if_selected, verify_sudoku,
    },
    state::{DigitMode, GameState, SudokuData},
};

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

fn setup_solver_hotkeys(game_state: RwSignal<GameState>, sudoku: RwSignal<SudokuData>) {
    let apply_and_show = move |f: fn(&mut SudokuData) -> crate::Result<String>| {
        move |()| {
            update!(|game_state, sudoku| {
                game_state.show_result(f(sudoku));
            });
        }
    };

    use_hotkeys!(("KeyA") => apply_and_show(place_all_visible_singles));
    use_hotkeys!(("KeyS") => apply_and_show(place_all_hidden_singles));
    use_hotkeys!(("KeyD") => apply_and_show(check_all_visible_doubles));
    use_hotkeys!(("KeyF") => apply_and_show(check_triples));
    use_hotkeys!(("KeyG") => apply_and_show(check_constraints));
    use_hotkeys!(("KeyH") => apply_and_show(solve_sudoku));
    use_hotkeys!(("KeyJ") => apply_and_show(verify_sudoku));
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
    use_hotkeys!((format!("digit{i}"), "place_digits") => move |()| {
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
    use_hotkeys!((format!("digit{i}"), "toggle_choices") => move |()| {
        update!(|game_state, sudoku| {
            toggle_choice_if_selected(game_state, sudoku, i as u8);
        });
    });
}
