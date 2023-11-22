use std::{fs, path::{PathBuf, Path}};

use eyre::Result;

pub fn create_file(dir: &Path, name: &str) -> Result<()> {
    let mut path = dir.to_path_buf();
    path.push(PathBuf::from(name));
    fs::write(path, "")?;
    Ok(())
}

pub fn delete_file(dir: &Path, name: &str) -> Result<()> {
    let mut path = dir.to_path_buf();
    path.push(PathBuf::from(name));
    fs::remove_file(path)?;
    Ok(())
}

pub fn rename_file(dir: &Path, name: &str, new_name: &str) -> Result<()> {
    let mut path = dir.to_path_buf();
    path.push(PathBuf::from(name));
    let mut new_path = dir.to_path_buf();
    new_path.push(new_name);
    fs::rename(path, new_path)?;
    Ok(())
}
