use crate::Result;
use serde::{Deserialize, Serialize};
use std::{fmt::Display, str::FromStr};

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

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
#[allow(clippy::module_name_repetitions)]
pub struct GameState {
    pub active_cell: Option<(usize, usize)>,
    pub message: Option<String>,
    pub dark_mode: DarkMode,
}

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
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

impl FromStr for GameState {
    type Err = crate::Error;

    fn from_str(s: &str) -> Result<Self> {
        let mut game_state = Self::default();
        let mut chars = s.chars();
        if let Some(c) = chars.next() {
            match c {
                'l' => game_state.dark_mode = DarkMode::Light,
                'd' => game_state.dark_mode = DarkMode::Dark,
                _ => return Err(crate::Error::GenerateSudoku),
            }
        }
        Ok(game_state)
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

impl Display for GameState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.dark_mode {
            DarkMode::Light => write!(f, "l"),
            DarkMode::Dark => write!(f, "d"),
        }
    }
}
