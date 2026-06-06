//! Command-line interface definitions and argument parsing
//!
//! This module defines the CLI structure using `clap` with derive macros.
//! It provides the main CLI entry point and command definitions for `build` and `apply`.

use clap::Parser;

use crate::{apply::Apply, build::Build};

/// Managing AWS Athena Schemas
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
#[clap(arg_required_else_help(true))]
#[clap(color(clap::ColorChoice::Auto))]
pub struct Cli {
    #[command(subcommand)]
    pub cmd: Command,
}

#[derive(clap::Subcommand, Debug)]
pub enum Command {
    /// Build SQL from template path
    Build(Build),
    /// Build and execute SQL to Athena
    Apply(Apply),
}

// Parse the command line arguments
pub fn parse() -> Cli {
    Cli::parse()
}
