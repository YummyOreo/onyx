use std::{path::PathBuf, time::Duration};

use crossterm::event;
use eyre::Result;
use filesystem::read::{read_path, read_with_fallback, File, ReadRes};
use ratatui::widgets::ListState;
use settings::parse_args;
use state::{Info, InfoKind, Mode, State};

use crate::ui::input::{InputModeResult, InputResult};

mod filesystem;
mod settings;
mod state;
mod ui;

pub struct App {
    pub ui: ui::UiState,
    pub state: State,
}

impl App {
    pub fn new(path: PathBuf) -> Result<Self> {
        let files = Vec::default();
        let ui_state = ui::UiState {
            scroll_state: ListState::default(),
        };

        let state = State {
            files,
            info: Vec::default(),
            path,
            last_path: PathBuf::new(),
            ..Default::default()
        };
        Ok(Self {
            ui: ui_state,
            state,
        })
    }

    pub async fn run(&mut self) -> Result<()> {
        let mut terminal = ui::make_terminal()?;

        loop {
            let state = &mut self.state;
            if state.last_path != state.path {
                state.files = match read_with_fallback(&state.path, PathBuf::from("./")).await? {
                    ReadRes::Read(files) => files,
                    ReadRes::FallBack { error, files } => {
                        state.path = PathBuf::from("./");
                        state.info.push(Info::new(InfoKind::Error(error)));
                        files
                    }
                };
                if !state.path.is_absolute() {
                    state.path = state.path.canonicalize()?;
                }
                state.last_path = state.path.clone()
            } else {
                state.files = read_path(&state.path).await?;
            }

            state.selected = state.selected.clamp(0, state.files.len().saturating_sub(1));
            terminal.draw(|f| self.ui.draw(f, state))?;
            State::purge_info(&mut state.info, Duration::from_secs(4)).await;

            let event_ready =
                tokio::task::spawn_blocking(|| event::poll(Duration::from_millis(250)));

            if event_ready.await??
                && App::handle_input(self.ui.input(event::read()?, state).await, state).await?
            {
                break;
            }
        }

        // restore terminal
        ui::restore_terminal(terminal)
    }

    async fn handle_input(input: InputResult, state: &mut State) -> Result<bool> {
        match input {
            InputResult::Quit => {
                return Ok(true);
            }
            InputResult::MoveUp => {
                state.selected = state.selected.checked_sub(1).unwrap_or_default();
            }
            InputResult::MoveDown => {
                state.selected = state
                    .selected
                    .checked_add(1)
                    .unwrap()
                    .clamp(0, state.files.len().saturating_sub(1));
            }
            InputResult::EnterFolder => {
                let folder = &state.files[state.selected];
                if folder.file_type.is_dir()
                    | (folder.file_type.is_symlink() && folder.path.canonicalize()?.is_dir())
                {
                    state.path = folder.path.clone();
                    state.selected = 0;
                }
            }
            InputResult::GoBack => {
                state.path.pop();
                state.selected = 0;
            }
            InputResult::Mode(InputModeResult::ModeChange(m)) => {
                state.mode = m;
            }
            InputResult::Mode(InputModeResult::AddChar(c)) => {
                state.mode.add_char(c);
            }
            InputResult::Mode(InputModeResult::RemoveChar) => {
                state.mode.remove_char();
            }
            InputResult::Mode(InputModeResult::Execute) => {
                let mut mode = Mode::Basic;
                core::mem::swap(&mut state.mode, &mut mode);
                match mode {
                    Mode::CreateFile(file) => {
                        if let Err(e) = filesystem::modify::create_file(&file, &state.path).await {
                            state.info.push(Info::new(InfoKind::Error(e)));
                        }
                    }
                    Mode::RenameFile(from, new) => {
                        if let Err(e) = filesystem::modify::rename_file(&from, &new).await {
                            state.info.push(Info::new(InfoKind::Error(e)));
                        }
                    }
                    Mode::DeleteFile(file, confirm) if confirm.to_lowercase() == "y" => {
                        if let Err(e) = filesystem::modify::delete_file(&file).await {
                            state.info.push(Info::new(InfoKind::Error(e)));
                        }
                    }
                    _ => {}
                };
            }
            _ => {}
        }
        Ok(false)
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    let settings = parse_args();
    App::new(PathBuf::from(&settings.dir))?.run().await
}
