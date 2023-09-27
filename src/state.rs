use std::{
    path::PathBuf,
    time::{Duration, Instant},
};

use eyre::Report;

use crate::filesystem::read::File;

#[derive(PartialEq, Eq, PartialOrd, Ord)]
pub enum Mode {
    Basic,
    Search(String),
}

impl Default for Mode {
    fn default() -> Self {
        Self::Basic
    }
}

impl Mode {
    pub fn get(&self) -> Option<&str> {
        match &self {
            Self::Search(s) => Some(s),
            _ => None,
        }
    }
    pub fn push(&mut self, c: char) {
        if let Self::Search(s) = self {
            s.push(c);
        }
    }
    pub fn pop(&mut self) {
        if let Self::Search(s) = self {
            s.pop();
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
    pub last_path: PathBuf,
    pub files: Vec<File>,
    pub selected: usize,
    pub info: Vec<Info>,
    pub mode: Mode,
}

impl State {
    pub async fn purge_info(infos: &mut Vec<Info>, d: Duration) {
        infos.retain(|i| i.time.elapsed() < d);
    }
}
