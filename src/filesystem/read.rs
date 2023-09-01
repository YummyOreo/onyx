use std::{
    ffi::OsString,
    fs::{FileType, Metadata},
    path::PathBuf,
};

use eyre::{Context, Report, Result};
use tokio::fs::{self, DirEntry};

pub struct File {
    pub path: PathBuf,
    pub file_type: FileType,
    pub name: OsString,
    pub metadata: Metadata,
}

impl File {
    pub async fn new(d: DirEntry) -> Result<Self> {
        Ok(Self {
            path: d.path(),
            file_type: d.file_type().await?,
            name: d.file_name(),
            metadata: d.metadata().await?,
        })
    }

    pub fn is_dir(&self) -> Result<bool> {
        Ok(self.file_type.is_dir()
            | (self.file_type.is_symlink() && self.path.canonicalize()?.is_dir()))
    }
    pub fn is_file(&self) -> Result<bool> {
        Ok(self.file_type.is_file()
            | (self.file_type.is_symlink() && self.path.canonicalize()?.is_file()))
    }
}

pub enum ReadRes {
    FallBack { error: Report, files: Vec<File> },
    Read(Vec<File>),
}

pub async fn read_with_fallback(path: &PathBuf, fallback: PathBuf) -> Result<ReadRes> {
    if path.exists() {
        Ok(ReadRes::Read(read_path(path).await?))
    } else {
        let r = eyre::eyre!(
            "Could not read path: \"{}\". Defaulting to \"./\"",
            path.to_str().unwrap()
        );
        Ok(ReadRes::FallBack {
            error: r,
            files: read_path(&fallback).await?,
        })
    }
}

pub async fn read_path(path: &PathBuf) -> Result<Vec<File>> {
    let mut t = fs::read_dir(path)
        .await
        .wrap_err_with(|| format!("Could not read path: \"{}\".", path.to_str().unwrap()))?;

    let mut files = vec![];
    while let Some(d) = t.next_entry().await? {
        files.push(File::new(d).await?);
    }
    Ok(files)
}
