use std::path::PathBuf;

use app::App;
use eyre::Result;

mod app;
mod settings;
mod ui;

#[tokio::main]
async fn main() -> Result<()> {
    App::new(PathBuf::from("./"))?.run().await
}
