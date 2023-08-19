use std::{
    fs::DirEntry,
    path::PathBuf,
    time::{Duration, Instant},
};

use eyre::Report;

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

pub enum InfoKind {
    Error(Report),
    Message(String),
}

pub struct Info {
    pub kind: InfoKind,
    pub time: Instant,
}

impl Info {
    pub fn new(k: InfoKind) -> Self {
        Self {
            kind: k,
            time: Instant::now(),
        }
    }
}

#[derive(Default)]
pub struct State {
    pub path: PathBuf,
    pub files: Vec<DirEntry>,
    pub selected: usize,
    pub mode: Mode,
    pub info: Vec<Info>,
}

impl State {
    pub async fn purge_info(infos: &mut Vec<Info>, d: Duration) {
        infos.retain(|i| i.time.elapsed() < d);
    }
}
