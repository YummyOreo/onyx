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
}

pub enum ReadRes {
    FallBack { error: Report, files: Vec<File> },
    Read(Vec<File>),
}

pub async fn read_with_fallback(path: &PathBuf, fallback: PathBuf) -> Result<ReadRes> {
    match fs::read_dir(path).await.wrap_err_with(|| {
        format!(
            "Could not read path: \"{}\". Defaulting to \"./\"",
            path.to_str().unwrap()
        )
    }) {
        Ok(mut t) => {
            let mut files = vec![];
            while let Some(d) = t.next_entry().await? {
                files.push(File::new(d).await?);
            }
            Ok(ReadRes::Read(files))
        }
        Err(e) => {
            let mut files = vec![];
            while let Some(d) = fs::read_dir(&fallback)
                .await
                .wrap_err_with(|| format!("Could not read path: \"{}\"", path.to_str().unwrap()))?
                .next_entry()
                .await?
            {
                files.push(File::new(d).await?);
            }
            Ok(ReadRes::FallBack { error: e, files })
        }
    }
}
