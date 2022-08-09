mod apply;
mod build;
mod cli;
mod utils;

use anyhow::Result;

fn main() -> Result<()> {
    let args = cli::parse();

    match args.cmd {
        cli::Command::Build(args) => build::call(args),
        cli::Command::Apply(args) => apply::call(args),
    }
}
