use crate::{
    actions::{to_choices, update_from_sudoku},
    util::compress_string,
    Result,
};
use rust_sudoku_solver::Sudoku;
use std::fmt::Display;

#[derive(Debug, Default, Clone)]
pub struct SudokuData {
    pub rows: [SudokuRow; 9],
}

#[derive(Debug, Default, Clone, Copy)]
pub struct SudokuRow {
    pub cells: [Cell; 9],
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Cell {
    Empty {
        choices: [bool; 9],
    },
    Value {
        value: u8,
        choices: [bool; 9],
    },
    FixedValue {
        value: u8,
    },
    AnimatedValue {
        value: u8,
        choices: [bool; 9],
        fade_delay_ms: i32,
        animation: &'static str,
    },
    Error {
        value: u8,
        choices: [bool; 9],
    },
}

impl Default for Cell {
    fn default() -> Self {
        Self::Empty { choices: [true; 9] }
    }
}

impl Cell {
    pub const fn is_empty(&self) -> bool {
        matches!(self, Self::Empty { .. })
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DigitMode {
    Value,
    Choice,
}

impl DigitMode {
    pub fn toggle(&mut self) {
        *self = match self {
            Self::Value => Self::Choice,
            Self::Choice => Self::Value,
        }
    }
}

#[derive(Debug, Default, Clone)]
#[allow(clippy::module_name_repetitions)]
pub struct GameState {
    pub active_cell: Option<(usize, usize)>,
    pub message: Option<String>,
    pub dark_mode: DarkMode,
}

#[derive(Debug, Default, Clone)]
pub enum DarkMode {
    Light,
    #[default]
    Dark,
}

impl DarkMode {
    pub const fn class(&self) -> &'static str {
        match self {
            Self::Light => "light",
            Self::Dark => "dark",
        }
    }

    pub fn toggle(&mut self) {
        *self = match self {
            Self::Light => Self::Dark,
            Self::Dark => Self::Light,
        }
    }

    pub const fn active(&self) -> bool {
        matches!(self, Self::Dark)
    }
}

impl GameState {
    pub fn show_result(&mut self, result: Result<impl Display>) {
        match result {
            Ok(v) => self.message = Some(v.to_string()),
            Err(e) => self.message = Some(e.to_string()),
        }
    }

    pub fn is_active_cell(&self, row: usize, col: usize) -> bool {
        self.active_cell.is_some() && self.active_cell == Some((row, col))
    }
}

impl From<&SudokuData> for Sudoku {
    fn from(data: &SudokuData) -> Self {
        let mut sudoku = Self::default();
        for (i, row) in data.rows.iter().enumerate() {
            for (j, cell) in row.cells.iter().enumerate() {
                let idx = i * 9 + j;
                match cell {
                    Cell::Empty { .. } => {}
                    Cell::Value { value, .. }
                    | Cell::FixedValue { value }
                    | Cell::AnimatedValue { value, .. }
                    | Cell::Error { value, .. } => {
                        sudoku.place(idx, *value as usize);
                    }
                }
            }
        }
        sudoku
    }
}

impl From<&Sudoku> for SudokuData {
    fn from(sudoku: &Sudoku) -> Self {
        let mut data = Self::default();
        for i in 0..9 {
            for j in 0..9 {
                let idx = 9 * i + j;
                if sudoku.digits[idx] == 0 {
                    data.rows[i].cells[j] = Cell::Empty {
                        choices: to_choices(sudoku.bitboard[idx]),
                    };
                } else {
                    data.rows[i].cells[j] = Cell::FixedValue {
                        value: sudoku.digits[idx] as u8,
                    };
                }
            }
        }
        data
    }
}

impl SudokuData {
    pub fn set(&mut self, row: usize, col: usize, value: u8, fixed: bool) {
        match self.rows[row].cells[col] {
            Cell::Empty { choices } => {
                if fixed {
                    self.rows[row].cells[col] = Cell::FixedValue { value };
                } else {
                    self.rows[row].cells[col] = Cell::Value { value, choices };
                }

                for i in 0..9 {
                    self.remove_choice(row, i, value);
                    self.remove_choice(i, col, value);
                }
                for (r, c) in Self::get_box_positions(row, col) {
                    self.remove_choice(r, c, value);
                }
            }
            Cell::Value { .. }
            | Cell::FixedValue { .. }
            | Cell::Error { .. }
            | Cell::AnimatedValue { .. } => {}
        }
    }

    pub fn set_fade(&mut self, row: usize, col: usize, value: u8, fade_delay_ms: i32) {
        match self.rows[row].cells[col] {
            Cell::Empty { .. } => {
                self.rows[row].cells[col] = Cell::AnimatedValue {
                    value,
                    choices: [false; 9],
                    fade_delay_ms,
                    animation: "fade-in",
                };
            }
            Cell::Value { .. }
            | Cell::FixedValue { .. }
            | Cell::Error { .. }
            | Cell::AnimatedValue { .. } => {}
        }
    }

    pub fn unset(&mut self, row: usize, col: usize) {
        match self.rows[row].cells[col] {
            Cell::Empty { .. } | Cell::FixedValue { .. } => {}
            Cell::Value { choices, .. }
            | Cell::Error { choices, .. }
            | Cell::AnimatedValue { choices, .. } => {
                self.rows[row].cells[col] = Cell::Empty { choices };
                let sudoku = Sudoku::from(&*self);
                update_from_sudoku(self, &sudoku, false);
            }
        }
    }

    pub fn clear(&mut self) {
        for i in 0..9 {
            for j in 0..9 {
                self.rows[i].cells[j] = Cell::Empty { choices: [true; 9] };
            }
        }
    }

    pub const fn get(&self, row: usize, col: usize) -> Cell {
        self.rows[row].cells[col]
    }

    pub fn get_mut(&mut self, row: usize, col: usize) -> &mut Cell {
        &mut self.rows[row].cells[col]
    }

    pub fn fixed_sudoku(&self) -> Sudoku {
        let mut sudoku = Sudoku::default();
        for (i, row) in self.rows.iter().enumerate() {
            for (j, cell) in row.cells.iter().enumerate() {
                let idx = i * 9 + j;
                match cell {
                    Cell::Empty { .. }
                    | Cell::Value { .. }
                    | Cell::Error { .. }
                    | Cell::AnimatedValue { .. } => {}
                    Cell::FixedValue { value } => {
                        sudoku.place(idx, *value as usize);
                    }
                }
            }
        }
        sudoku
    }

    fn get_box_positions(row: usize, col: usize) -> Vec<(usize, usize)> {
        let mut positions = Vec::new();
        let box_row = row / 3;
        let box_col = col / 3;
        for i in 0..3 {
            for j in 0..3 {
                positions.push((3 * box_row + i, 3 * box_col + j));
            }
        }
        positions
    }

    #[allow(dead_code)]
    fn get_only_choice(choices: &[bool; 9]) -> Option<u8> {
        let mut count = 0;
        let mut value = 0;
        for (i, &choice) in choices.iter().enumerate() {
            if choice {
                count += 1;
                value = i + 1;
            }
        }
        if count == 1 {
            Some(value as u8)
        } else {
            None
        }
    }

    fn remove_choice(&mut self, row: usize, col: usize, value: u8) {
        if let Cell::Empty { choices } = &mut self.rows[row].cells[col] {
            choices[(value - 1) as usize] = false;
        }
    }

    #[allow(dead_code)]
    fn add_choice(&mut self, row: usize, col: usize, value: u8) {
        if let Cell::Empty { choices } = &mut self.rows[row].cells[col] {
            choices[(value - 1) as usize] = true;
        }
    }

    pub fn to_compressed(&self) -> String {
        compress_string(&self.to_string()).unwrap_or_default()
    }
}

impl Display for SudokuData {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for row in &self.rows {
            for cell in &row.cells {
                match cell {
                    Cell::Empty { .. } => write!(f, ".")?,
                    Cell::Value { value, .. }
                    | Cell::AnimatedValue { value, .. }
                    | Cell::FixedValue { value, .. }
                    | Cell::Error { value, .. } => {
                        write!(f, "{value}")?;
                    }
                };
            }
        }
        Ok(())
    }
}
