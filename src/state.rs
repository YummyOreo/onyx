use std::{
    borrow::BorrowMut,
    cell::RefCell,
    path::PathBuf,
    rc::Rc,
    time::{Duration, Instant},
};

use eyre::Report;
use fuzzy_matcher::{skim::SkimMatcherV2, FuzzyMatcher};

use crate::filesystem::read::File;

type GetScoreFn = dyn Fn(&str, &File) -> Option<(i64, Vec<usize>)>;

#[derive(PartialEq, Eq, PartialOrd, Ord)]
pub enum Mode {
    Basic,
    EscapedSearch,
    Search(Rc<RefCell<String>>),
}

impl Default for Mode {
    fn default() -> Self {
        Self::Basic
    }
}

impl Mode {
    pub fn get(&self) -> Option<String> {
        match &self {
            Self::Search(s) => Some(s.borrow().clone()),
            _ => None,
        }
    }
    pub fn push(&mut self, c: char) {
        if let Self::Search(s) = self {
            s.borrow_mut().replace_with(|s| {
                s.push(c);
                s.to_string()
            });
        }
    }
    pub fn pop(&mut self) {
        if let Self::Search(s) = self {
            s.borrow_mut().replace_with(|s| {
                s.pop();
                s.to_string()
            });
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
pub struct Files {
    pub files: Vec<File>,
    pub sort_mode: SortMode,
    pub input: Rc<RefCell<String>>,
}

impl Files {
    pub fn new(files: Vec<File>) -> Self {
        Self {
            files,
            ..Default::default()
        }
    }
    pub fn sort(&mut self) -> Option<()> {
        let f = self.sort_mode.get_score_fn()?;
        self.files.sort_by(|a, b| {
            f(&self.input.borrow(), a)
                .unwrap_or_default()
                .0
                .cmp(&f(&self.input.borrow(), b).unwrap_or_default().0)
        });
        self.files.reverse();
        Some(())
    }
}

#[derive(Default)]
pub struct State {
    pub path: PathBuf,
    pub last_path: PathBuf,
    pub files: Files,
    pub selected: usize,
    pub info: Vec<Info>,
    pub mode: Mode,
}

impl State {
    pub async fn purge_info(infos: &mut Vec<Info>, d: Duration) {
        infos.retain(|i| i.time.elapsed() < d);
    }

    pub fn change_sort_mode(&mut self, mode: Mode, search_mode: SortMode) {
        self.mode = mode;
        match &self.mode {
            Mode::Basic => {
                self.files.input = Default::default();
                self.files.sort_mode = SortMode::Default;
            }
            Mode::EscapedSearch => {}
            Mode::Search(s) => {
                self.files.sort_mode = search_mode;
                self.files.input = s.clone();
            }
        }
    }
}
