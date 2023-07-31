use std::path::PathBuf;

use eyre::Result;
use tokio::fs;

use super::utils;

pub async fn create_file(file: &str) -> Result<()> {
    match utils::get_type_by_name(file) {
        utils::FileType::Folder => {
            fs::create_dir_all(PathBuf::from(file)).await?;
        }
        utils::FileType::File(_) => {
            fs::File::create(PathBuf::from(file)).await?;
        }
    }
    Ok(())
}

pub async fn rename_file(original: &PathBuf, new: &str) -> Result<()> {
    fs::rename(original, PathBuf::from(new)).await?;
    Ok(())
}
