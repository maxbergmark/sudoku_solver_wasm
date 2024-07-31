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
    Value { value: u8, choices: [bool; 9] },
    FixedValue { value: u8 },
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
            Cell::Value { .. } | Cell::FixedValue { .. } => {}
        }
    }

    pub fn unset(&mut self, row: usize, col: usize) {
        match self.rows[row].cells[col] {
            Cell::Empty { .. } | Cell::FixedValue { .. } => {}
            Cell::Value { value, choices } => {
                self.rows[row].cells[col] = Cell::Empty { choices };
                for i in 0..9 {
                    self.add_choice(row, i, value);
                    self.add_choice(i, col, value);
                }
                for (r, c) in Self::get_box_positions(row, col) {
                    self.add_choice(r, c, value);
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
                // if let Some(v) = Self::get_only_choice(choices) {
                // self.set(row, col, v, false);
                // }
            }
            Cell::Value { .. } | Cell::FixedValue { .. } => {}
        }
    }

    fn add_choice(&mut self, row: usize, col: usize, value: u8) {
        match &mut self.rows[row].cells[col] {
            Cell::Empty { choices } => {
                choices[(value - 1) as usize] = true;
            }
            Cell::Value { .. } | Cell::FixedValue { .. } => {}
        }
    }

    pub fn to_compressed(&self) -> String {
        compress_string(&self.to_string())
    }
}

pub fn compress_string(s: &str) -> String {
    let mut compressed = String::new();
    let mut count = 0;
    let mut last_char = s.chars().next().unwrap();
    for c in s.chars() {
        if c == last_char {
            if count == 52 {
                compressed.push_str(&format!("{}{}", last_char, get_letter(count - 1)));
                last_char = c;
                count = 1;
            } else {
                count += 1;
            }
        } else {
            if count > 1 {
                compressed.push_str(&format!("{}{}", last_char, get_letter(count - 1)));
            } else {
                compressed.push(last_char);
            }
            last_char = c;
            count = 1;
        }
    }
    if count > 1 {
        compressed.push_str(&format!("{}{}", last_char, get_letter(count - 1)));
    } else {
        compressed.push(last_char);
    }
    compressed
}

pub fn decompress_string(s: &str) -> String {
    let mut decompressed = String::new();
    for c in s.chars() {
        if let Some(idx) = get_count(c) {
            let letter = decompressed.pop().unwrap();
            decompressed.push_str(&letter.to_string().repeat(idx + 1));
        } else {
            decompressed.push(c);
        }
    }
    decompressed
}

fn get_count(c: char) -> Option<usize> {
    match c {
        'a'..='z' => Some(c as usize - 'a' as usize),
        'A'..='Z' => Some(c as usize - 'A' as usize + 26),
        _ => None,
    }
}

fn get_letter(idx: usize) -> char {
    match idx {
        // start with lowercase, then uppercase
        0..=25 => ('a' as usize + idx) as u8 as char,
        26..=52 => ('A' as usize + idx - 26) as u8 as char,
        _ => panic!("Invalid index: {}", idx),
    }
}

impl Display for SudokuData {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for row in self.rows.iter() {
            for cell in row.cells.iter() {
                match cell {
                    Cell::Empty { .. } => write!(f, ".")?,
                    Cell::Value { value, .. } | Cell::FixedValue { value, .. } => {
                        write!(f, "{value}")?
                    }
                };
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_compress_string() {
        assert_eq!(compress_string("1"), "1");
        assert_eq!(compress_string("11"), "1b");
        assert_eq!(compress_string("111111111"), "1i");
        assert_eq!(compress_string("111111112"), "1h2");
        assert_eq!(compress_string("111111122"), "1g2b");
        assert_eq!(compress_string("111111222"), "1f2c");
        assert_eq!(compress_string("111112222"), "1e2d");
        assert_eq!(compress_string("111122222"), "1d2e");
        assert_eq!(compress_string("111222222"), "1c2f");
        assert_eq!(compress_string("112222222"), "1b2g");
        assert_eq!(compress_string("122222222"), "12h");
        assert_eq!(compress_string("222222222"), "2i");
        assert_eq!(compress_string("1....2"), "1.d2");
    }

    #[test]
    fn test_decompress_string() {
        assert_eq!(decompress_string("1"), "1");
        assert_eq!(decompress_string("1b"), "11");
        assert_eq!(decompress_string("1i"), "111111111");
        assert_eq!(decompress_string("1h2"), "111111112");
        assert_eq!(decompress_string("1g2b"), "111111122");
        assert_eq!(decompress_string("1f2c"), "111111222");
        assert_eq!(decompress_string("1e2d"), "111112222");
        assert_eq!(decompress_string("1d2e"), "111122222");
        assert_eq!(decompress_string("1c2f"), "111222222");
        assert_eq!(decompress_string("1b2g"), "112222222");
        assert_eq!(decompress_string("12h"), "122222222");
        assert_eq!(decompress_string("2i"), "222222222");
        assert_eq!(decompress_string("1.d2"), "1....2");
    }
}
