use std::{
    path::PathBuf,
    time::{Duration, Instant},
};

use eyre::Report;

use crate::filesystem::read::File;

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
}

impl State {
    pub async fn purge_info(infos: &mut Vec<Info>, d: Duration) {
        infos.retain(|i| i.time.elapsed() < d);
    }
}
