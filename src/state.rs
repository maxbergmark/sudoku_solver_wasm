use std::fmt::Display;
use web_time::Instant;

#[derive(Debug, Default, Clone)]
pub struct SudokuData {
    pub rows: [SudokuRow; 9],
}

#[derive(Debug, Default, Clone)]
pub struct SudokuRow {
    pub cells: [Cell; 9],
}

#[derive(Debug, Clone)]
pub enum Cell {
    Empty { choices: [bool; 9] },
    Value(u8),
}

impl Default for Cell {
    fn default() -> Self {
        Cell::Empty { choices: [true; 9] }
    }
}

#[derive(Debug, Default, Clone)]
pub struct GameState {
    pub active_cell: Option<(usize, usize)>,
    pub last_key_press: Option<Instant>,
}

impl SudokuData {
    pub fn set(&mut self, row: usize, col: usize, value: u8) {
        self.rows[row].cells[col] = Cell::Value(value);
        for i in 0..9 {
            self.remove_choice(row, i, value);
            self.remove_choice(i, col, value);
        }
        for (r, c) in Self::get_box_positions(row, col) {
            self.remove_choice(r, c, value);
        }
    }

    pub fn unset(&mut self, row: usize, col: usize) {
        match self.rows[row].cells[col] {
            Cell::Empty { .. } => {}
            Cell::Value(v) => {
                self.rows[row].cells[col] = Cell::Empty { choices: [true; 9] };
                for i in 0..9 {
                    self.add_choice(row, i, v);
                    self.add_choice(i, col, v);
                }
                for (r, c) in Self::get_box_positions(row, col) {
                    self.add_choice(r, c, v);
                }
            }
        }
    }

    pub fn get(&self, row: usize, col: usize) -> &Cell {
        &self.rows[row].cells[col]
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
                if let Some(v) = Self::get_only_choice(choices) {
                    self.set(row, col, v);
                }
            }
            Cell::Value(_) => {}
        }
    }

    fn add_choice(&mut self, row: usize, col: usize, value: u8) {
        match &mut self.rows[row].cells[col] {
            Cell::Empty { choices } => {
                choices[(value - 1) as usize] = true;
            }
            Cell::Value(_) => {}
        }
    }
}

impl Display for SudokuData {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for row in self.rows.iter() {
            for cell in row.cells.iter() {
                match cell {
                    Cell::Empty { .. } => write!(f, ".")?,
                    Cell::Value(v) => write!(f, "{v}")?,
                };
            }
        }
        Ok(())
    }
}
