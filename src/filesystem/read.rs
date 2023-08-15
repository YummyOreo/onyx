use std::{
    fs::{self, DirEntry},
    path::PathBuf,
};

use eyre::{Report, Result};

pub enum ReadRes {
    FallBack { error: Report, files: Vec<DirEntry> },
    Read(Vec<DirEntry>),
}

pub fn read_with_fallback(path: &PathBuf, fallback: PathBuf) -> Result<ReadRes> {
    match fs::read_dir(path) {
        Ok(t) => Ok(ReadRes::Read(
            t.collect::<Result<Vec<DirEntry>, std::io::Error>>()?,
        )),
        Err(e) => Ok(ReadRes::FallBack {
            error: e.into(),
            files: fs::read_dir(&fallback)?.collect::<Result<Vec<DirEntry>, std::io::Error>>()?,
        }),
    }
}
