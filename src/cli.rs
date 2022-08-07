use clap::Parser;

use crate::{apply::Apply, build::Build};

/// AWS Athena Management
#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
#[clap(arg_required_else_help(true))]
#[clap(color(clap::ColorChoice::Auto))]
pub struct Cli {
    #[clap(subcommand)]
    pub cmd: Command,
}

#[derive(clap::Subcommand, Debug)]
pub enum Command {
    /// Build SQL from template
    Build(Build),
    /// Apply SQL to Athena
    Apply(Apply),
}

// Parse the command line arguments
pub fn parse() -> Cli {
    Cli::from_args()
}
