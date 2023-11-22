use std::{cell::RefCell, rc::Rc};

use crossterm::event::KeyCode;

use crate::state::Mode;

pub fn match_keycode(mode: &Mode, input: KeyCode) -> InputResult {
    match input {
        KeyCode::Char(c) if matches!(mode, &Mode::Search(_) | &Mode::Command(_)) => {
            InputResult::ModifyMode(ModifyMode::PushChar(c))
        }
        KeyCode::Backspace if matches!(mode, &Mode::Search(_) | &Mode::Command(_)) => {
            InputResult::ModifyMode(ModifyMode::PopChar)
        }
        KeyCode::Enter if matches!(mode, &Mode::Command(_)) => InputResult::ExecuteMode,
        KeyCode::Up | KeyCode::Char('k') => InputResult::MoveUp,
        KeyCode::Down | KeyCode::Char('j') => InputResult::MoveDown,
        KeyCode::Left | KeyCode::Char('h') | KeyCode::Enter => InputResult::GoBack,
        KeyCode::Right | KeyCode::Char('l') => InputResult::EnterFolder,
        KeyCode::Char('/') => {
            InputResult::ModeChange(Mode::Search(Rc::new(RefCell::new(String::new()))))
        }
        KeyCode::Char(':') => {
            InputResult::ModeChange(Mode::Command(Rc::new(RefCell::new(String::new()))))
        }
        KeyCode::Esc if matches!(mode, &Mode::Search(_)) => {
            InputResult::ModeChange(Mode::EscapedSearch)
        }
        KeyCode::Esc if matches!(mode, &Mode::EscapedSearch | &Mode::Command(_)) => {
            InputResult::ModeChange(Mode::Basic)
        }
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
    ExecuteMode,

    Quit,
    Skip,
}
