use crate::{
    actions::{to_choices, update_from_sudoku},
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

#[derive(Debug, Clone, Copy)]
pub enum Cell {
    Empty { choices: [bool; 9] },
    Value { value: u8, choices: [bool; 9] },
    FixedValue { value: u8 },
    Error { value: u8, choices: [bool; 9] },
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
            Cell::Value { .. } | Cell::FixedValue { .. } | Cell::Error { .. } => {}
        }
    }

    pub fn unset(&mut self, row: usize, col: usize) {
        match self.rows[row].cells[col] {
            Cell::Empty { .. } | Cell::FixedValue { .. } => {}
            Cell::Value { choices, .. } | Cell::Error { choices, .. } => {
                self.rows[row].cells[col] = Cell::Empty { choices };
                let sudoku = Sudoku::from(&*self);
                update_from_sudoku(self, &sudoku, false);
            }
        }
    }

    pub const fn get(&self, row: usize, col: usize) -> &Cell {
        &self.rows[row].cells[col]
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
                    Cell::Empty { .. } | Cell::Value { .. } | Cell::Error { .. } => {}
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
        match &mut self.rows[row].cells[col] {
            Cell::Empty { choices } => {
                choices[(value - 1) as usize] = false;
            }
            Cell::Value { .. } | Cell::FixedValue { .. } | Cell::Error { .. } => {}
        }
    }

    #[allow(unused)]
    fn add_choice(&mut self, row: usize, col: usize, value: u8) {
        match &mut self.rows[row].cells[col] {
            Cell::Empty { choices } => {
                choices[(value - 1) as usize] = true;
            }
            Cell::Value { .. } | Cell::FixedValue { .. } | Cell::Error { .. } => {}
        }
    }

    pub fn to_compressed(&self) -> String {
        compress_string(&self.to_string()).unwrap_or_default()
    }
}

pub fn compress_string(s: &str) -> Option<String> {
    let mut compressed = String::new();
    let mut count = 0;
    let Some(mut last_char) = s.chars().next() else {
        return Some(compressed);
    };

    for c in s.chars() {
        if c == last_char {
            count = push_if_full(&mut compressed, c, count)?;
        } else {
            push_repeated(&mut compressed, last_char, count)?;
            last_char = c;
            count = 1;
        }
    }
    push_repeated(&mut compressed, last_char, count)?;
    Some(compressed)
}

fn push_if_full(compressed: &mut String, c: char, count: usize) -> Option<usize> {
    if count == 52 {
        compressed.push_str(&format!("{}{}", c, get_letter(count - 1)?));
        Some(1)
    } else {
        Some(count + 1)
    }
}

fn push_repeated(compressed: &mut String, c: char, count: usize) -> Option<()> {
    if count > 1 {
        compressed.push_str(&format!("{}{}", c, get_letter(count - 1)?));
    } else {
        compressed.push(c);
    }
    Some(())
}

pub fn decompress_string(s: &str) -> Option<String> {
    let mut decompressed = String::new();
    for c in s.chars() {
        if let Some(idx) = get_count(c) {
            let letter = decompressed.pop()?;
            decompressed.push_str(&letter.to_string().repeat(idx + 1));
        } else {
            decompressed.push(c);
        }
    }
    Some(decompressed)
}

const fn get_count(c: char) -> Option<usize> {
    match c {
        'a'..='z' => Some(c as usize - 'a' as usize),
        'A'..='Z' => Some(c as usize - 'A' as usize + 26),
        _ => None,
    }
}

const fn get_letter(idx: usize) -> Option<char> {
    match idx {
        // start with lowercase, then uppercase
        0..=25 => Some(('a' as usize + idx) as u8 as char),
        26..=52 => Some(('A' as usize + idx - 26) as u8 as char),
        _ => None,
    }
}

impl Display for SudokuData {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for row in &self.rows {
            for cell in &row.cells {
                match cell {
                    Cell::Empty { .. } => write!(f, ".")?,
                    Cell::Value { value, .. }
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

#[cfg(test)]
mod tests {
    use rstest::rstest;

    use super::*;

    #[rstest]
    #[case("", Some(""))]
    #[case("1", Some("1"))]
    #[case("11", Some("1b"))]
    #[case("111111111", Some("1i"))]
    #[case("111111112", Some("1h2"))]
    #[case("111111122", Some("1g2b"))]
    #[case("111111222", Some("1f2c"))]
    #[case("111112222", Some("1e2d"))]
    #[case("111122222", Some("1d2e"))]
    #[case("111222222", Some("1c2f"))]
    #[case("112222222", Some("1b2g"))]
    #[case("122222222", Some("12h"))]
    #[case("222222222", Some("2i"))]
    #[case("1....2", Some("1.d2"))]
    fn test_compress_string(#[case] input: &str, #[case] expected: Option<&str>) {
        assert_eq!(compress_string(input), expected.map(ToString::to_string));
    }

    #[rstest]
    #[case("", Some(""))]
    #[case("1", Some("1"))]
    #[case("1b", Some("11"))]
    #[case("1i", Some("111111111"))]
    #[case("1h2", Some("111111112"))]
    #[case("1g2b", Some("111111122"))]
    #[case("1f2c", Some("111111222"))]
    #[case("1e2d", Some("111112222"))]
    #[case("1d2e", Some("111122222"))]
    #[case("1c2f", Some("111222222"))]
    #[case("1b2g", Some("112222222"))]
    #[case("12h", Some("122222222"))]
    #[case("2i", Some("222222222"))]
    #[case("1.d2", Some("1....2"))]
    #[case("d", None)]
    fn test_decompress_string(#[case] input: &str, #[case] expected: Option<&str>) {
        assert_eq!(decompress_string(input), expected.map(ToString::to_string));
    }
}
