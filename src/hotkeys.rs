use leptos::{leptos_dom::logging::console_error, update, RwSignal, SignalUpdate};
use leptos_hotkeys::{use_hotkeys, use_hotkeys_scoped};
use rust_sudoku_solver::{
    check_all_visible_doubles, place_all_hidden_singles, place_all_visible_singles, solver, Sudoku,
    SudokuError,
};
use std::str::FromStr;
use web_time::Instant;

use crate::state::{Cell, GameState, SudokuData};

pub fn setup_hotkeys(game_state: RwSignal<GameState>, sudoku: RwSignal<SudokuData>) {
    setup_placement_hotkeys(game_state, sudoku);
    setup_solver_hotkeys(sudoku);
    setup_movement_hotkeys(game_state);
}

fn setup_placement_hotkeys(game_state: RwSignal<GameState>, sudoku: RwSignal<SudokuData>) {
    for i in 1..=9 {
        setup_digit_hotkey(i, game_state, sudoku);
    }
    use_hotkeys!((format!("escape,backspace")) => move |_| {
        update!(|game_state, sudoku| {
            clear_digit_if_selected(game_state, sudoku);
        });
    });
}

fn setup_solver_hotkeys(sudoku: RwSignal<SudokuData>) {
    use_hotkeys!(("KeyA") => move |_| {
        sudoku.update(|sudoku| apply_constraint(sudoku, place_all_visible_singles))
    });
    use_hotkeys!(("KeyS") => move |_| {
        sudoku.update(|sudoku| apply_constraint(sudoku, place_all_hidden_singles))
    });
    use_hotkeys!(("KeyD") => move |_| {
        sudoku.update(|sudoku| apply_constraint(sudoku, check_all_visible_doubles))
    });
    use_hotkeys!(("KeyF") => move |_| {
        sudoku.update(|sudoku| apply_constraint(sudoku, solver::check_constraints))
    });
    use_hotkeys!((format!("KeyG")) => move |_| {
        sudoku.update(|sudoku| {
            let _ = Sudoku::from_str(&sudoku.to_string())
            .and_then(solver::solve)
            .inspect(|solution| update_from_sudoku(sudoku, solution, false))
            .map_err(|e| console_error(&format!("{:?}", e)));
        // TODO: Show error message in the UI
        });
    });
}

fn setup_movement_hotkeys(game_state: RwSignal<GameState>) {
    setup_arrow_hotkey("ArrowRight", (0, 1), game_state);
    setup_arrow_hotkey("ArrowLeft", (0, -1), game_state);
    setup_arrow_hotkey("ArrowUp", (-1, 0), game_state);
    setup_arrow_hotkey("ArrowDown", (1, 0), game_state);
}

fn apply_constraint(sudoku: &mut SudokuData, f: impl Fn(&mut Sudoku) -> Result<(), SudokuError>) {
    let _ = Sudoku::from_str(&sudoku.to_string())
        .and_then(|mut sudoku| {
            f(&mut sudoku)?;
            Ok(sudoku)
        })
        .inspect(|solution| update_from_sudoku(sudoku, solution, false))
        .map_err(|e| console_error(&format!("{:?}", e)));
    // TODO: Show error message in the UI
}

fn setup_arrow_hotkey(name: &str, direction: (i32, i32), game_state: RwSignal<GameState>) {
    use_hotkeys!((name) => move |_| {
        handle_arrow(&game_state, direction);
    });
}

fn set_digit_if_selected(game_state: &mut GameState, sudoku: &mut SudokuData, digit: u8) {
    if let Some((row, col)) = game_state.active_cell {
        match sudoku.get(row, col) {
            Cell::Empty { choices } => {
                if choices[(digit - 1) as usize] {
                    sudoku.set(row, col, digit, false);
                }
            }
            Cell::Value { value, choices } => {
                if value != &digit {
                    let is_available = choices[(digit - 1) as usize];
                    sudoku.unset(row, col);
                    if is_available {
                        sudoku.set(row, col, digit, false);
                    }
                }
            }
            Cell::FixedValue { .. } => {}
        }
    }
}

fn clear_digit_if_selected(game_state: &mut GameState, sudoku: &mut SudokuData) {
    if let Some((row, col)) = game_state.active_cell {
        sudoku.unset(row, col);
    }
}

fn to_choices(bitboard: usize) -> [bool; 9] {
    let mut choices = [false; 9];
    for i in 1..=9 {
        choices[i - 1] = (bitboard & (1 << i)) != 0;
    }
    choices
}

pub fn update_from_sudoku(sudoku: &mut SudokuData, solution: &Sudoku, fixed: bool) {
    for i in 0..9 {
        for j in 0..9 {
            let idx = 9 * i + j;
            if solution.digits[idx] == 0 {
                sudoku.rows[i].cells[j] = Cell::Empty {
                    choices: to_choices(solution.bitboard[idx]),
                };
            } else {
                sudoku.set(i, j, (solution.digits[idx]) as u8, fixed);
            }
        }
    }
}

fn setup_digit_hotkey(i: usize, game_state: RwSignal<GameState>, sudoku: RwSignal<SudokuData>) {
    use_hotkeys!((format!("digit{i}")) => move |_| {
        update!(|game_state, sudoku| {
            set_digit_if_selected(game_state, sudoku, i as u8);
        });
    });
}

fn is_valid_cell(row: i32, col: i32) -> bool {
    (0..9).contains(&row) && (0..9).contains(&col)
}

fn handle_arrow(game_state: &RwSignal<GameState>, direction: (i32, i32)) {
    game_state.update(|game_state| {
        if let Some(prev) = game_state.last_key_press {
            let now = Instant::now();
            if now.duration_since(prev).as_millis() < 10 {
                return;
            }
            game_state.last_key_press = Some(now);
        } else {
            game_state.last_key_press = Some(Instant::now());
        }

        if let Some((row, col)) = game_state.active_cell {
            let new_row = row as i32 + direction.0;
            let new_col = col as i32 + direction.1;
            if is_valid_cell(new_row, new_col) {
                game_state.active_cell = Some((new_row as usize, new_col as usize));
            }
        }
    });
}
