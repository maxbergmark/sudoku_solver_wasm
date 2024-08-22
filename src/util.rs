use std::str::FromStr;

use leptos::{leptos_dom::logging::console_error, Params, RwSignal};
use leptos_router::{Params, ParamsError};
use rust_sudoku_solver::Sudoku;

#[derive(Params, PartialEq, Eq, Debug)]
pub struct SudokuParams {
    sudoku: Option<String>,
}

#[allow(clippy::panic)]
pub fn unwrap_or_panic<T>(signal: Option<RwSignal<T>>) -> RwSignal<T> {
    signal.unwrap_or_else(|| {
        console_error("Component not available");
        panic!("Component not available");
    })
}

pub fn unwrap_params(params: &Result<SudokuParams, ParamsError>) -> Sudoku {
    params
        .as_ref()
        .ok()
        .and_then(|p| p.sudoku.clone())
        .and_then(|s| decompress_string(&s))
        .filter(|s| is_valid_game_str(s))
        .and_then(|s| Sudoku::from_str(&s).ok())
        .unwrap_or_default()
}

fn is_valid_game_str(game_str: &str) -> bool {
    game_str.len() == 81 && game_str.chars().all(|c| c.is_ascii_digit() || c == '.')
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
