use anyhow::Result;
use std::path::PathBuf;

#[derive(clap::Args, Debug)]
pub struct Apply {
    /// Target path to render. If the target path is a directory,
    /// the root folder must contains the index.sql file
    #[clap(parse(from_os_str))]
    pub file: PathBuf,

    /// Change the context current working dir
    #[clap(long, short, parse(from_os_str))]
    pub context: Option<PathBuf>,

    /// Dry-run
    #[clap(global = true, long, short)]
    pub dry_run: Option<bool>,

    /// AWS Profile
    #[clap(global = true, long, short)]
    pub profile: Option<String>,
}

pub fn call(args: Apply) -> Result<()> {
    let build_args = crate::build::Build {
        file: args.file.clone(),
        out: None,
        context: args.context.clone(),
    };

    let sql = crate::build::build(build_args)?;
    println!("SQL: {}", sql);

    // Set AWS_PROFILE
    if let Some(profile) = args.profile {
        std::env::set_var("AWS_PROFILE", profile);
    }

    Ok(())
}
