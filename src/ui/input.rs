use std::path::PathBuf;

use crossterm::event::KeyCode;

use crate::Mode;

pub fn match_keycode(mode: &Mode, current_file: Option<PathBuf>, input: KeyCode) -> InputResult {
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
        KeyCode::Left | KeyCode::Char('h') => InputResult::GoBack,
        KeyCode::Right | KeyCode::Char('l') => InputResult::EnterFolder,
        KeyCode::Char('q') => InputResult::Quit,
        KeyCode::Char('c') => {
            InputResult::Mode(InputModeResult::ModeChange(Mode::CreateFile(String::new())))
        }
        KeyCode::Char('r') | KeyCode::Char('d') if current_file.is_none() => InputResult::Skip,
        KeyCode::Char('r') => {
            let current_file = current_file.expect("should be there");
            InputResult::Mode(InputModeResult::ModeChange(Mode::RenameFile(
                current_file.clone(),
                current_file.to_string_lossy().to_string(),
            )))
        }
        KeyCode::Char('d') => InputResult::Mode(InputModeResult::ModeChange(Mode::DeleteFile(
            current_file.expect("should be there"),
            String::new(),
        ))),
        _ => InputResult::Skip,
    }
}

pub enum InputResult {
    MoveUp,
    MoveDown,

    Mode(InputModeResult),
    EnterFolder,
    GoBack,

    Quit,
    Skip,
}

pub enum InputModeResult {
    ModeChange(Mode),
    AddChar(char),
    RemoveChar,
    Execute,
}
