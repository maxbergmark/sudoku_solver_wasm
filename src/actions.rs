use derive_more::From;

use leptos::leptos_dom::logging::console_log;
use leptos::{set_timeout, RwSignal, SignalUpdate};
use rand::seq::SliceRandom;
use rand::thread_rng;
use rust_sudoku_solver::{solver, Sudoku};
use web_time::Instant;

use crate::state::{Cell, GameState, SudokuData};
use crate::Result;

#[derive(Debug, From)]
pub struct Duration(pub web_time::Duration);

#[allow(clippy::use_debug)]
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

pub fn verify_sudoku(sudoku_data: &mut SudokuData) -> Result<String> {
    Ok(sudoku_data)
        .and_then_timed(compare_with_solution)
        .map(|((), elapsed)| format!("Sudoku verified in {elapsed}"))
}

pub fn solve_sudoku(sudoku_data: &mut SudokuData) -> Result<String> {
    Ok(Sudoku::from(&*sudoku_data))
        .and_then_timed(solver::solve)
        .map(|(solution, elapsed)| {
            update_from_sudoku(sudoku_data, &solution, false);
            elapsed
        })
        .map_err(|err| {
            let r = compare_with_solution(sudoku_data);
            match r {
                Ok(()) => err.into(),
                Err(e) => e,
            }
        })
        .map(|elapsed| format!("Sudoku solved in {elapsed}"))
}

pub fn animate_solve(sudoku: RwSignal<SudokuData>) {
    // let sudoku = sudoku.clone();
    let solution = Ok(Sudoku::from(&sudoku())).and_then(solver::solve); //solver::solve(sudoku().fixed_sudoku());
    match solution {
        Ok(solution) => {
            animate_from_sudoku(sudoku, &solution, false);
        }
        Err(_) => {
            console_log("No solution found");
        }
    }
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

pub fn check_triples(sudoku: &mut SudokuData) -> Result<String> {
    apply_constraint(sudoku, rust_sudoku_solver::check_triples)
        .map(|elapsed| format!("Triples checked in {elapsed}"))
}

pub fn check_constraints(sudoku: &mut SudokuData) -> Result<String> {
    apply_constraint(sudoku, rust_sudoku_solver::check_constraints)
        .map(|elapsed| format!("Constraints checked in {elapsed}"))
}

pub fn toggle_digit_if_selected(game_state: &GameState, sudoku: &mut SudokuData, digit: u8) {
    if let Some((row, col)) = game_state.active_cell {
        let cell = *sudoku.get(row, col);
        match cell {
            Cell::Empty { choices } => {
                if choices[(digit - 1) as usize] {
                    sudoku.set(row, col, digit, false);
                }
            }
            Cell::Value { value, choices } | Cell::Error { value, choices } => {
                toggle_if_available(value, digit, &choices, sudoku, row, col);
            }
            Cell::FixedValue { .. } => {}
        }
    }
}

// TODO: make this work by clicking the choice instead
pub fn toggle_choice_if_selected(game_state: &GameState, sudoku: &mut SudokuData, digit: u8) {
    if let Some((row, col)) = game_state.active_cell {
        match sudoku.get_mut(row, col) {
            Cell::Empty { choices } => {
                choices[(digit - 1) as usize] = !choices[(digit - 1) as usize];
            }
            Cell::Value { .. } | Cell::Error { .. } | Cell::FixedValue { .. } => {}
        }
    }
}

fn toggle_if_available(
    value: u8,
    digit: u8,
    choices: &[bool; 9],
    sudoku: &mut SudokuData,
    row: usize,
    col: usize,
) {
    if value == digit {
        sudoku.unset(row, col);
    } else {
        let is_available = choices[(digit - 1) as usize];
        sudoku.unset(row, col);
        if is_available {
            sudoku.set(row, col, digit, false);
        }
    }
}

pub fn clear_digit_if_selected(game_state: &GameState, sudoku: &mut SudokuData) {
    if let Some((row, col)) = game_state.active_cell {
        sudoku.unset(row, col);
    }
}

pub fn to_choices(bitboard: usize) -> [bool; 9] {
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

// this "works" but is not very performant. the better way to do it would be to set the set_timeout
// for each cell in the sudoku, and then update the cell in the sudoku. this would allow the
// animation to be more smooth and not have to wait for the entire sudoku to be updated at once.
pub fn animate_from_sudoku(sudoku: RwSignal<SudokuData>, solution: &Sudoku, fixed: bool) {
    console_log("Animating solution");
    let mut d = std::time::Duration::from_millis(0);
    let mut vec: Vec<usize> = (0..81).collect();
    vec.shuffle(&mut thread_rng());
    for &idx in &vec {
        let i = idx / 9;
        let j = idx % 9;
        let solution = solution.clone();
        if solution.digits[idx] == 0 {
            set_timeout(
                move || {
                    sudoku.update(|s| {
                        s.rows[i].cells[j] = Cell::Empty {
                            choices: to_choices(solution.bitboard[idx]),
                        };
                    });
                },
                d,
            );
        } else {
            set_timeout(
                move || {
                    sudoku.update(|s| {
                        s.set(i, j, (solution.digits[idx]) as u8, fixed);
                    });
                },
                d,
            );
        }
        if sudoku().rows[i].cells[j].is_empty() {
            d += std::time::Duration::from_millis(50);
        }
    }
}

pub fn compare_with_solution(sudoku: &mut SudokuData) -> Result<()> {
    let solution = solver::solve(sudoku.fixed_sudoku())?;

    for i in 0..9 {
        for j in 0..9 {
            let idx = 9 * i + j;
            let cell = sudoku.rows[i].cells[j];
            if let Cell::Value { value, .. } = cell {
                if value != solution.digits[idx] as u8 {
                    sudoku.rows[i].cells[j] = Cell::Error {
                        value,
                        choices: to_choices(solution.bitboard[idx]),
                    };
                }
            }
        }
    }
    Ok(())
}

fn is_valid_cell(row: i32, col: i32) -> bool {
    (0..9).contains(&row) && (0..9).contains(&col)
}

pub fn handle_arrow(game_state: &RwSignal<GameState>, direction: (i32, i32)) {
    game_state.update(|state| {
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
    f: impl Fn(&mut Sudoku) -> rust_sudoku_solver::Result<()>,
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
