use std::path::PathBuf;

use crossterm::event::KeyCode;

pub fn match_keycode(current_file: Option<PathBuf>, input: KeyCode) -> InputResult {
    match input {
        KeyCode::Up | KeyCode::Char('k') => InputResult::MoveUp,
        KeyCode::Down | KeyCode::Char('j') => InputResult::MoveDown,
        KeyCode::Left | KeyCode::Char('h') => InputResult::GoBack,
        KeyCode::Right | KeyCode::Char('l') => InputResult::EnterFolder,
        KeyCode::Char('q') => InputResult::Quit,
        _ => InputResult::Skip,
    }
}

pub enum InputResult {
    MoveUp,
    MoveDown,

    EnterFolder,
    GoBack,

    Quit,
    Skip,
}
