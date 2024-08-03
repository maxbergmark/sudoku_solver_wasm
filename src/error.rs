use std::fmt::{Display, Formatter};

use derive_more::From;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, From)]
pub enum Error {
    #[from]
    Sudoku(rust_sudoku_solver::Error),
}

impl Display for Error {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        match self {
            Self::Sudoku(e) => write!(f, "{e}"),
        }
    }
}
