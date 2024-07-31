use leptos::{leptos_dom::logging::console_log, update, RwSignal, SignalUpdate};
use leptos_hotkeys::{use_hotkeys, use_hotkeys_scoped};
use rust_sudoku_solver::{check_all_visible_doubles, place_all_visible_singles, solver, Sudoku};
use std::str::FromStr;
use web_time::Instant;

use crate::state::{Cell, GameState, SudokuData};

pub fn setup_hotkeys(game_state: RwSignal<GameState>, sudoku: RwSignal<SudokuData>) {
    for i in 1..=9 {
        setup_digit_hotkey(i, game_state, sudoku);
    }
    use_hotkeys!((format!("escape,backspace")) => move |_| {
        update!(|game_state, sudoku| {
            clear_digit_if_selected(game_state, sudoku);
        });
    });
    use_hotkeys!((format!("enter")) => move |_| {
        sudoku.update(|sudoku| {
            let s = sudoku.to_string();
            let input = Sudoku::from_str(&s).unwrap();
            let solution = solver::solve(input).unwrap();
            update_from_sudoku(sudoku, &solution);
        });
    });

    use_hotkeys!((format!("KeyS")) => move |_| {
        // let s = game_state().sudoku.to_string();
        sudoku.update(|sudoku| {
            let s = sudoku.to_string();
            let mut partial_solution = Sudoku::from_str(&s).unwrap();
            place_all_visible_singles(&mut partial_solution).unwrap();
            console_log(format!("singles: {partial_solution}").as_str());
            update_from_sudoku(sudoku, &partial_solution);
        });
    });
    use_hotkeys!((format!("KeyD")) => move |_| {
        sudoku.update(|sudoku| {
            let s = sudoku.to_string();
            let mut partial_solution = Sudoku::from_str(&s).unwrap();
            check_all_visible_doubles(&mut partial_solution).unwrap();
            console_log(format!("doubles: {partial_solution}").as_str());
            update_from_sudoku(sudoku, &partial_solution);
        });
    });
    use_hotkeys!((format!("KeyC")) => move |_| {
        sudoku.update(|sudoku| {
            let s = sudoku.to_string();
            let mut partial_solution = Sudoku::from_str(&s).unwrap();
            solver::check_constraints(&mut partial_solution).unwrap();
            console_log(&partial_solution.to_string());
            update_from_sudoku(sudoku, &partial_solution);
        });
    });

    use_hotkeys!((format!("ArrowRight")) => move |_| {
        handle_arrow(&game_state, (0, 1));
    });
    use_hotkeys!((format!("ArrowLeft")) => move |_| {
        handle_arrow(&game_state, (0, -1));
    });
    use_hotkeys!((format!("ArrowUp")) => move |_| {
        handle_arrow(&game_state, (-1, 0));
    });
    use_hotkeys!((format!("ArrowDown")) => move |_| {
        handle_arrow(&game_state, (1, 0));
    });
}

fn set_digit_if_selected(game_state: &mut GameState, sudoku: &mut SudokuData, digit: u8) {
    if let Some((row, col)) = game_state.active_cell {
        match sudoku.get(row, col) {
            Cell::Empty { choices } => {
                if choices[(digit - 1) as usize] {
                    sudoku.set(row, col, digit);
                }
            }
            Cell::Value(v) => {
                if v != &digit {
                    sudoku.unset(row, col);
                    sudoku.set(row, col, digit);
                }
            }
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

fn update_from_sudoku(sudoku: &mut SudokuData, solution: &Sudoku) {
    for i in 0..9 {
        for j in 0..9 {
            let idx = 9 * i + j;
            if solution.digits[idx] == 0 {
                sudoku.rows[i].cells[j] = Cell::Empty {
                    choices: to_choices(solution.bitboard[idx]),
                };
            } else {
                sudoku.set(i, j, (solution.digits[idx]) as u8);
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
