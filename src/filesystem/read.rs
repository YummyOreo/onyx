use std::{
    fs::{self, DirEntry},
    path::PathBuf,
};

use eyre::{Context, Report, Result};

pub enum ReadRes {
    FallBack { error: Report, files: Vec<DirEntry> },
    Read(Vec<DirEntry>),
}

pub fn read_with_fallback(path: &PathBuf, fallback: PathBuf) -> Result<ReadRes> {
    match fs::read_dir(path).wrap_err_with(|| {
        format!(
            "Could not read path: \"{}\". Defaulting to \"./\"",
            path.to_str().unwrap()
        )
    }) {
        Ok(t) => Ok(ReadRes::Read(
            t.collect::<Result<Vec<DirEntry>, std::io::Error>>()?,
        )),
        Err(e) => Ok(ReadRes::FallBack {
            error: e,
            files: fs::read_dir(fallback)
                .wrap_err_with(|| format!("Could not read path: \"{}\"", path.to_str().unwrap()))?
                .collect::<Result<Vec<DirEntry>, std::io::Error>>()?,
        }),
    }
}
