use anyhow::{bail, Context, Result};
use log::debug;
use std::{fs::File, io::Write, path::PathBuf};

use crate::tera::get_tera;
use crate::utils::{get_current_working_dir, get_full_path_str, is_dir, pretty_print};

#[derive(clap::Args, Debug, Clone)]
pub struct Build {
    /// Target path to render. If the target path is a directory,
    /// the root folder must contains the index.sql file
    #[clap(parse(from_os_str))]
    pub file: PathBuf,

    /// Output path. The file will be overwritten if is already exists
    #[clap(long, short, parse(from_os_str))]
    pub out: Option<PathBuf>,

    /// Change the context current working dir
    #[clap(long, short, parse(from_os_str))]
    pub context: Option<PathBuf>,

    /// No pretty print for SQL
    #[clap(long, short)]
    pub no_pretty: Option<bool>,
}

pub async fn call(args: Build) -> Result<()> {
    let (_, path_str) = get_dirs(args.clone())?;

    // Render SQL
    let sql = build(args.clone())?;

    // Print to stdout or write to file?
    match args.out {
        Some(path) => {
            let mut file = File::create(&path)
                .with_context(|| format!("could not create output file {}", path_str))?;

            match file.write_all(sql.as_bytes()) {
                Ok(_) => println!("write to {}", path_str),
                Err(e) => println!("failed write to {}: {}", path_str, e),
            }
        }
        None => {
            if args.no_pretty.unwrap_or_default() {
                print!("{}", sql);
            } else {
                pretty_print(sql.as_bytes());
            }
        }
    }

    Ok(())
}

pub fn build(args: Build) -> Result<String> {
    let path = &args.file;

    let is_dir = is_dir(path);

    // If input path is empty folder, just return empty
    if is_dir && path.read_dir()?.next().is_none() {
        return Ok("".to_string());
    }

    let (working_dir, path_str) = get_dirs(args.clone())?;

    // If input path contains no *.sql files, error
    if is_dir
        && !path
            .read_dir()?
            .filter_map(Result::ok)
            .any(|f| f.path().extension().unwrap_or_default() == "sql")
    {
        let files = path
            .read_dir()?
            .map(Result::ok)
            .into_iter()
            .flatten()
            .map(|f| f.path())
            .collect::<Vec<_>>();

        bail!("top-level doesn't contains any index.sql file: {:?}", files);
    }

    // Init Tera template
    let tera = get_tera(args.file.clone(), working_dir)?;

    // For debug
    let loaded_template: Vec<_> = tera.get_template_names().collect();
    debug!("loaded templates: {:?}", loaded_template);

    // TODO: Tera context
    let context = tera::Context::new();

    // Render the index.sql file if the target path is a folder
    let endpoint = if is_dir {
        format!("{}/index.sql", path_str)
            .trim_start_matches('/')
            .to_string()
    } else {
        "index.sql".to_string()
    };

    let out = tera
        .render(&endpoint, &context)
        .with_context(|| format!("failed to render from {}", path_str))?;

    Ok(out.trim().to_string())
}

fn get_dirs(args: Build) -> Result<(PathBuf, String)> {
    let path = &args.file;

    // Working directory (context directory)
    let working_dir = get_current_working_dir(args.context)?;
    debug!("Working dir: {}", &working_dir.display());

    // Get path_str (without context directory prefix)
    let path_str = get_full_path_str(path)?;
    let working_dir_str = working_dir.to_str().expect("could not get working dir str");
    let path_str = path_str
        .trim_start_matches(working_dir_str)
        .trim_start_matches('/')
        .to_string();

    Ok((working_dir, path_str))
}
