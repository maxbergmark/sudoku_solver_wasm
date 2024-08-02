use derive_more::From;

use leptos::{RwSignal, SignalUpdate};
use rust_sudoku_solver::{solver, Sudoku, SudokuError};
use web_time::Instant;

use crate::state::{Cell, GameState, SudokuData};
use crate::Result;

#[derive(Debug, From)]
pub struct Duration(pub web_time::Duration);

impl std::fmt::Display for Duration {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{:?}", self.0)
    }
}

trait TimedAction<T, E> {
    #[allow(unused)]
    fn map_timed<F, U>(self, f: F) -> std::result::Result<(U, Duration), E>
    where
        F: FnOnce(T) -> U;
    fn and_then_timed<F, U>(self, f: F) -> std::result::Result<(U, Duration), E>
    where
        F: FnOnce(T) -> std::result::Result<U, E>;
}

impl<T, E> TimedAction<T, E> for std::result::Result<T, E> {
    fn map_timed<F, U>(self, f: F) -> std::result::Result<(U, Duration), E>
    where
        F: FnOnce(T) -> U,
    {
        self.map(|t| {
            let now = Instant::now();
            (f(t), now.elapsed().into())
        })
    }

    fn and_then_timed<F, U>(self, f: F) -> std::result::Result<(U, Duration), E>
    where
        F: FnOnce(T) -> std::result::Result<U, E>,
    {
        self.and_then(|t| {
            let now = Instant::now();
            f(t).map(|u| (u, now.elapsed().into()))
        })
    }
}

pub fn solve_sudoku(sudoku_data: &mut SudokuData) -> Result<String> {
    let res = Ok(Sudoku::from(&*sudoku_data))
        .and_then_timed(solver::solve)
        .map(|(solution, elapsed)| {
            update_from_sudoku(sudoku_data, &solution, false);
            elapsed
        })
        .map(|elapsed| format!("Sudoku solved in {elapsed}"));
    Ok(res?)
}

pub fn place_all_visible_singles(sudoku: &mut SudokuData) -> Result<String> {
    apply_constraint(sudoku, rust_sudoku_solver::place_all_visible_singles)
        .map(|elapsed| format!("Visible singles placed in {elapsed}"))
}

pub fn place_all_hidden_singles(sudoku: &mut SudokuData) -> Result<String> {
    apply_constraint(sudoku, rust_sudoku_solver::place_all_hidden_singles)
        .map(|elapsed| format!("Hidden singles placed in {elapsed}"))
}

pub fn check_all_visible_doubles(sudoku: &mut SudokuData) -> Result<String> {
    apply_constraint(sudoku, rust_sudoku_solver::check_all_visible_doubles)
        .map(|elapsed| format!("Doubles checked in {elapsed}"))
}

pub fn check_constraints(sudoku: &mut SudokuData) -> Result<String> {
    apply_constraint(sudoku, rust_sudoku_solver::check_constraints)
        .map(|elapsed| format!("Constraints checked in {elapsed}"))
}

pub fn set_digit_if_selected(game_state: &GameState, sudoku: &mut SudokuData, digit: u8) {
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

pub fn clear_digit_if_selected(game_state: &GameState, sudoku: &mut SudokuData) {
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

fn is_valid_cell(row: i32, col: i32) -> bool {
    (0..9).contains(&row) && (0..9).contains(&col)
}

pub fn handle_arrow(game_state: &RwSignal<GameState>, direction: (i32, i32)) {
    game_state.update(|state| {
        if let Some(prev) = state.last_key_press {
            let now = Instant::now();
            if now.duration_since(prev).as_millis() < 10 {
                return;
            }
            state.last_key_press = Some(now);
        } else {
            state.last_key_press = Some(Instant::now());
        }

        if let Some((row, col)) = state.active_cell {
            let new_row = row as i32 + direction.0;
            let new_col = col as i32 + direction.1;
            if is_valid_cell(new_row, new_col) {
                state.active_cell = Some((new_row as usize, new_col as usize));
            }
        }
    });
}

fn apply_constraint(
    sudoku_data: &mut SudokuData,
    f: impl Fn(&mut Sudoku) -> std::result::Result<(), SudokuError>,
) -> Result<Duration> {
    Ok(Sudoku::from(&*sudoku_data))
        .and_then_timed(|mut sudoku| {
            f(&mut sudoku)?;
            Ok(sudoku)
        })
        .map(|(solution, elapsed)| {
            update_from_sudoku(sudoku_data, &solution, false);
            elapsed
        })
}
