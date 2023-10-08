use std::{clone, path::PathBuf, time::Duration};

use crossterm::event;
use eyre::Result;
use filesystem::read::{read_path, read_with_fallback, ReadRes};
use ratatui::widgets::ListState;
use settings::parse_args;
use state::{Files, Info, InfoKind, Mode, SortMode, State};
use ui::input::ModifyMode;

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
            files: Files::new(files),
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
                state.files.files =
                    match read_with_fallback(&state.path, PathBuf::from("./")).await? {
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
                state.files.files = read_path(&state.path).await?;
            }
            state.files.sort();

            state.selected = state
                .selected
                .clamp(0, state.files.files.len().saturating_sub(1));
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
                    .clamp(0, state.files.files.len().saturating_sub(1));
            }
            InputResult::EnterFolder => {
                let folder = &state.files.files[state.selected];
                if folder.is_dir()? {
                    state.path = folder.path.clone();
                    state.selected = 0;
                }
            }
            InputResult::GoBack => {
                state.path.pop();
                state.selected = 0;
            }
            InputResult::ModifyMode(ModifyMode::PopChar) => state.mode.pop(),
            InputResult::ModifyMode(ModifyMode::PushChar(c)) => state.mode.push(c),
            InputResult::ModeChange(m) if matches!(m, Mode::Search(_)) => {
                state.mode = m;
                state.files.sort_mode = SortMode::Fuzzy;
                state.files.input = match &state.mode {
                    Mode::Search(c) => c.clone(),
                    _ => unreachable!(),
                };
            }
            InputResult::ModeChange(Mode::Basic) => {
                state.mode = Mode::Basic;
                state.files.input = Default::default();
                state.files.sort_mode = SortMode::Default;
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
