use std::{
    path::PathBuf,
    time::{Duration, Instant},
};

use eyre::Report;
use fuzzy_matcher::{skim::SkimMatcherV2, FuzzyMatcher};

use crate::filesystem::read::File;

type GetScoreFn = dyn Fn(&str, &File) -> Option<(i64, Vec<usize>)>;

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

pub enum SortMode {
    Default,
    Fuzzy,
}

impl Default for SortMode {
    fn default() -> Self {
        Self::Default
    }
}

impl SortMode {
    pub fn get_score_fn(&self) -> Option<Box<GetScoreFn>> {
        match self {
            Self::Default => None,
            Self::Fuzzy => {
                let matcher = SkimMatcherV2::default();
                Some(Box::new(move |pattern, file| {
                    matcher.fuzzy_indices(&file.name.to_string_lossy(), pattern)
                }))
            }
        }
    }
}

#[derive(Default)]
pub struct Files<'a> {
    pub files: Vec<File>,
    pub sort_mode: SortMode,
    pub input: &'a str,
}

impl<'a> Files<'a> {
    pub fn new(files: Vec<File>) -> Self {
        Self {
            files,
            ..Default::default()
        }
    }
    pub fn sort(&mut self) -> Option<()> {
        let f = self.sort_mode.get_score_fn()?;
        self.files.sort_by(|a, b| {
            f(self.input, a)
                .unwrap()
                .0
                .cmp(&f(self.input, b).unwrap().0)
        });
        Some(())
    }
}

#[derive(Default)]
pub struct State<'a> {
    pub path: PathBuf,
    pub last_path: PathBuf,
    pub files: Files<'a>,
    pub selected: usize,
    pub info: Vec<Info>,
    pub mode: Mode,
}

impl<'a> State<'a> {
    pub async fn purge_info(infos: &mut Vec<Info>, d: Duration) {
        infos.retain(|i| i.time.elapsed() < d);
    }
}
