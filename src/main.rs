use std::{fs, path::PathBuf, sync::Arc, time::Duration};

use crossterm::event;
use eyre::Result;
use ratatui::widgets::ListState;
use settings::parse_args;
use state::{InfoKind, Mode, State};
use tokio::sync::Mutex;

use crate::ui::input::{InputModeResult, InputResult};

mod filesystem;
mod settings;
mod state;
mod ui;

pub struct App {
    pub ui: ui::UiState,
    pub state: Arc<Mutex<State>>,
}

impl App {
    pub fn new(path: PathBuf) -> Result<Self> {
        let files = fs::read_dir(path.clone())?.map(|f| f.unwrap()).collect();
        let ui_state = ui::UiState {
            scroll_state: ListState::default(),
        };

        let state = State {
            files,
            info: vec![InfoKind::Error(eyre::eyre!("Could not find file: x"))],
            path,
            ..Default::default()
        };
        Ok(Self {
            ui: ui_state,
            state: Arc::new(Mutex::new(state)),
        })
    }

    pub async fn run(&mut self) -> Result<()> {
        let mut terminal = ui::make_terminal()?;

        loop {
            let mut state = self.state.lock().await;
            state.files = fs::read_dir(&state.path)?.map(|f| f.unwrap()).collect();
            state.selected = state.selected.clamp(0, state.files.len() - 1);
            terminal.draw(|f| self.ui.draw(f, &state))?;

            let event_ready =
                tokio::task::spawn_blocking(|| event::poll(Duration::from_millis(250)));

            if event_ready.await?? {
                match self.ui.input(event::read()?, &state).await {
                    InputResult::Quit => {
                        break;
                    }
                    InputResult::MoveUp => {
                        state.selected = state.selected.checked_sub(1).unwrap_or_default();
                    }
                    InputResult::MoveDown => {
                        state.selected = state
                            .selected
                            .checked_add(1)
                            .unwrap()
                            .clamp(0, state.files.len() - 1);
                    }
                    InputResult::EnterFolder => {
                        let folder = &state.files[state.selected];
                        if folder.file_type().unwrap().is_dir() {
                            state.path = folder.path().canonicalize()?
                        }
                        state.selected = 0;
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
                                let state = self.state.clone();
                                tokio::spawn(async move {
                                    match filesystem::modify::create_file(&file).await {
                                        Ok(_) => {}
                                        Err(e) => state.lock().await.info.push(InfoKind::Error(e)),
                                    }
                                });
                            }
                            Mode::RenameFile(from, new) => {
                                let state = self.state.clone();
                                tokio::spawn(async move {
                                    match filesystem::modify::rename_file(&from, &new).await {
                                        Ok(_) => {}
                                        Err(e) => state.lock().await.info.push(InfoKind::Error(e)),
                                    }
                                });
                            }
                            Mode::DeleteFile(file, confirm) => {
                                if confirm.to_lowercase() == "y" {
                                    let state = self.state.clone();
                                    tokio::spawn(async move {
                                        match filesystem::modify::delete_file(&file).await {
                                            Ok(_) => {}
                                            Err(e) => {
                                                state.lock().await.info.push(InfoKind::Error(e))
                                            }
                                        }
                                    });
                                }
                            }
                            _ => {}
                        };
                    }
                    _ => {}
                }
            }
        }

        // restore terminal
        ui::restore_terminal(terminal)
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    let settings = parse_args();
    App::new(PathBuf::from(&settings.dir).canonicalize()?)?
        .run()
        .await
}
