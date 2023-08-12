use std::path::PathBuf;

use clap::Parser;
use eyre::Result;

#[derive(Parser, Debug)]
#[command(name = "onyx", bin_name = "onyx", author, version, about = "A command line terminal", long_about = None)]
pub struct Settings {
    // the dir that should be opened
    #[arg(short, long)]
    pub dir: Option<PathBuf>,
}

pub fn parse_args() -> Result<Settings> {
    Ok(Settings::try_parse()?)
}
