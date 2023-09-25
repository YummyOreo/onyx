use std::{path::PathBuf, time::Duration};

use crossterm::event;
use eyre::Result;
use filesystem::read::{read_path, read_with_fallback, ReadRes};
use ratatui::widgets::ListState;
use settings::parse_args;
use state::{Info, InfoKind, State};

use crate::ui::input::InputResult;

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
                if folder.is_dir()? {
                    state.path = folder.path.clone();
                    state.selected = 0;
                }
            }
            InputResult::GoBack => {
                state.path.pop();
                state.selected = 0;
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
