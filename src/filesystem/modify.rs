use std::path::{Path, PathBuf};

use eyre::Result;
use tokio::fs;

use super::utils;

pub async fn create_file(file: &str, current_path: &Path) -> Result<()> {
    match utils::get_type_by_name(file) {
        utils::FileType::Folder => {
            fs::create_dir_all(current_path.join(file)).await?;
        }
        utils::FileType::File(_) => {
            fs::File::create(current_path.join(file)).await?;
        }
    }
    Ok(())
}

pub async fn rename_file(original: &Path, new: &str) -> Result<()> {
    fs::rename(original, PathBuf::from(new)).await?;
    Ok(())
}

pub async fn delete_file(file: &Path) -> Result<()> {
    match utils::get_type_by_path(file) {
        utils::FileType::Folder => {
            fs::remove_dir_all(file).await?;
        }
        utils::FileType::File(_) => {
            fs::remove_file(file).await?;
        }
    }
    Ok(())
}
