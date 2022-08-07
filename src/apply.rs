use anyhow::Result;
use log::debug;
use std::path::PathBuf;

#[derive(clap::Args, Debug)]
pub struct Apply {
    /// File path
    #[clap(parse(from_os_str))]
    pub file: PathBuf,
    /// Dry-run
    #[clap(global = true, long, short)]
    pub dry_run: Option<bool>,
}

pub fn call(args: Apply) -> Result<()> {
    debug!("Args: {:?}", args);

    Ok(())
}
