use std::path::PathBuf;

use clap::Parser;

const VERSION: &str = env!("CARGO_PKG_VERSION");
static HELP_TEMPLATE: &str = "\
{before-help}{name} {version}
{author}
{about}

{usage-heading}
  {usage}

{all-args}{after-help}";

#[derive(Parser)]
#[command(name = "onyx", bin_name = "ox", author = "YummyOreo", version = VERSION, about = "A command line terminal", help_template(HELP_TEMPLATE),)]
pub struct Settings {
    // the dir that should be opened
    #[arg(default_value = "./")]
    pub dir: PathBuf,
}

pub fn parse_args() -> Settings {
    Settings::parse()
}
