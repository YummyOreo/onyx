use crossterm::event::KeyCode;

use crate::state::Mode;

pub fn match_keycode(mode: &Mode, input: KeyCode) -> InputResult {
    match input {
        KeyCode::Char(c) if mode != &Mode::Basic => {
            InputResult::ModifyMode(ModifyMode::PushChar(c))
        }
        KeyCode::Backspace if mode != &Mode::Basic => InputResult::ModifyMode(ModifyMode::PopChar),
        KeyCode::Up | KeyCode::Char('k') => InputResult::MoveUp,
        KeyCode::Down | KeyCode::Char('j') => InputResult::MoveDown,
        KeyCode::Left | KeyCode::Char('h') => InputResult::GoBack,
        KeyCode::Right | KeyCode::Char('l') => InputResult::EnterFolder,
        KeyCode::Char('/') => InputResult::ModeChange(Mode::Search(String::new())),
        KeyCode::Char('q') => InputResult::Quit,
        _ => InputResult::Skip,
    }
}

pub enum ModifyMode {
    PushChar(char),
    PopChar,
}

pub enum InputResult {
    MoveUp,
    MoveDown,

    EnterFolder,
    GoBack,

    ModeChange(Mode),
    ModifyMode(ModifyMode),

    Quit,
    Skip,
}
