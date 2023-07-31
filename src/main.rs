use std::{
    fs::{self, DirEntry},
    path::PathBuf,
    time::Duration,
};

use crossterm::event;
use eyre::Result;

use crate::ui::input::{InputModeResult, InputResult};

mod filesystem;
mod settings;
mod ui;

#[derive(PartialEq, Eq)]
pub enum Mode {
    Basic,
    CreateFile(String),
    RenameFile(PathBuf, String),
    DeleteFile(PathBuf, String),
}

impl Default for Mode {
    fn default() -> Self {
        Self::Basic
    }
}

impl Mode {
    pub fn add_char(&mut self, c: char) {
        match self {
            Self::CreateFile(s) | Self::RenameFile(_, s) | Mode::DeleteFile(_, s) => s.push(c),
            _ => {}
        }
    }
    pub fn remove_char(&mut self) {
        match self {
            Self::CreateFile(s) | Self::RenameFile(_, s) | Mode::DeleteFile(_, s) => {
                s.pop();
            }
            _ => {}
        }
    }
    pub fn get_str(&self) -> Option<&str> {
        match self {
            Self::CreateFile(s) | Self::RenameFile(_, s) | Mode::DeleteFile(_, s) => Some(s),
            _ => None,
        }
    }
}

pub struct App {
    pub ui: ui::UiState,
    pub path: PathBuf,
    pub files: Vec<DirEntry>,
}

impl App {
    pub fn new(path: PathBuf) -> Result<Self> {
        let files = fs::read_dir(path.clone())?.map(|f| f.unwrap()).collect();
        let ui_state = ui::UiState {
            selected: 0,
            max: 0,
            mode: Mode::default(),
        };
        Ok(Self {
            ui: ui_state,
            files,
            path,
        })
    }

    pub async fn run(&mut self) -> Result<()> {
        let mut terminal = ui::make_terminal()?;

        loop {
            self.files = fs::read_dir(&self.path)?.map(|f| f.unwrap()).collect();
            self.ui.max = self.files.len() - 1;
            self.ui.selected = self.ui.selected.clamp(0, self.ui.max);
            terminal.draw(|f| self.ui.draw(f, &self.files))?;

            let event_ready =
                tokio::task::spawn_blocking(|| event::poll(Duration::from_millis(250)));

            if event_ready.await?? {
                match self.ui.input(event::read()?, &self.files).await {
                    InputResult::Quit => {
                        break;
                    }
                    InputResult::MoveUp => {
                        self.ui.selected = self.ui.selected.checked_sub(1).unwrap_or_default();
                    }
                    InputResult::MoveDown => {
                        self.ui.selected = self
                            .ui
                            .selected
                            .checked_add(1)
                            .unwrap()
                            .clamp(0, self.ui.max);
                    }
                    InputResult::Mode(InputModeResult::ModeChange(m)) => {
                        self.ui.mode = m;
                    }
                    InputResult::Mode(InputModeResult::AddChar(c)) => {
                        self.ui.mode.add_char(c);
                    }
                    InputResult::Mode(InputModeResult::RemoveChar) => {
                        self.ui.mode.remove_char();
                    }
                    InputResult::Mode(InputModeResult::Execute) => {
                        let mut mode = Mode::Basic;
                        core::mem::swap(&mut self.ui.mode, &mut mode);
                        match mode {
                            Mode::CreateFile(file) => {
                                tokio::spawn(async move {
                                    filesystem::modify::create_file(&file).await.unwrap()
                                });
                            }
                            Mode::RenameFile(from, new) => {
                                tokio::spawn(async move {
                                    filesystem::modify::rename_file(&from, &new).await.unwrap()
                                });
                            }
                            Mode::DeleteFile(file, confirm) => {
                                if confirm.to_lowercase() == "y" {
                                    tokio::spawn(async move {
                                        filesystem::modify::delete_file(&file).await.unwrap()
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
    App::new(PathBuf::from("./"))?.run().await
}
