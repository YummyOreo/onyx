use std::path::PathBuf;

use crossterm::event::KeyCode;

use crate::Mode;

pub fn match_keycode(mode: &Mode, current_file: PathBuf, input: KeyCode) -> InputResult {
    match input {
        KeyCode::Char(c) if mode != &Mode::Basic => InputResult::Mode(InputModeResult::AddChar(c)),
        KeyCode::Backspace if mode != &Mode::Basic => {
            InputResult::Mode(InputModeResult::RemoveChar)
        }
        KeyCode::Enter if mode != &Mode::Basic => InputResult::Mode(InputModeResult::Execute),
        KeyCode::Esc if mode != &Mode::Basic => {
            InputResult::Mode(InputModeResult::ModeChange(Mode::Basic))
        }
        KeyCode::Up | KeyCode::Char('k') => InputResult::MoveUp,
        KeyCode::Down | KeyCode::Char('j') => InputResult::MoveDown,
        KeyCode::Char('q') => InputResult::Quit,
        KeyCode::Char('c') => {
            InputResult::Mode(InputModeResult::ModeChange(Mode::CreateFile(String::new())))
        }
        KeyCode::Char('r') => InputResult::Mode(InputModeResult::ModeChange(Mode::RenameFile(
            current_file.clone(),
            current_file.to_string_lossy().to_string(),
        ))),
        KeyCode::Char('d') => InputResult::Mode(InputModeResult::ModeChange(Mode::DeleteFile(
            current_file,
            String::new(),
        ))),
        _ => InputResult::Skip,
    }
}

pub enum InputResult {
    MoveUp,
    MoveDown,

    Mode(InputModeResult),

    Quit,
    Skip,
}

pub enum InputModeResult {
    ModeChange(Mode),
    AddChar(char),
    RemoveChar,
    Execute,
}
