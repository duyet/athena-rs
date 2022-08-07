use anyhow::{bail, Context, Result};
use log::debug;
use std::fs::File;
use std::path::PathBuf;
use tera::Tera;

const ENDTRYPOINT: &str = "index.sql";

#[derive(clap::Args, Debug)]
pub struct Build {
    /// Target path to render. If the target path is a directory,
    /// the root folder must contains the index.sql file
    #[clap(parse(from_os_str))]
    pub file: PathBuf,
    /// Output path. The file will be overwritten if is already exists
    #[clap(long, short, parse(from_os_str))]
    pub out: Option<PathBuf>,
}

pub fn call(args: Build) -> Result<()> {
    debug!("Input path: {:?}", args.file);
    let is_dir = args.file.is_dir();

    let path_str = args
        .file
        .as_path()
        .to_str()
        .expect("failed convert to string");

    // If input path is empty folder, just return
    if is_dir && args.file.read_dir()?.next().is_none() {
        return Ok(());
    }

    // If input path contains no *.sql files, error
    if is_dir
        && !args
            .file
            .read_dir()?
            .filter_map(Result::ok)
            .any(|f| f.path().extension().unwrap_or_default() == "sql")
    {
        let files = args.file.read_dir()?.map(Result::ok).collect::<Vec<_>>();
        bail!("Target path does not contains any .sql files: {:?}", files);
    }

    // Init Tera template and Tera context
    let tera = if is_dir {
        let walk = format!("{}/**/*.sql", path_str.trim_end_matches('/'));

        Tera::new(&walk).with_context(|| format!("Failed to read from {:?}", args.file))?
    } else {
        let mut tera = Tera::default();
        tera.add_template_file(path_str, None)
            .with_context(|| format!("Failed to add {} to template", path_str))?;

        tera
    };

    // TODO: Tera context
    let context = tera::Context::new();

    // Render the index.sql file if the target path is a folder
    let endpoint = if is_dir { ENDTRYPOINT } else { path_str };

    match args.out {
        Some(path) => {
            let out_file = File::create(&path)
                .with_context(|| format!("Could not create output file {}", path_str))?;

            match tera.render_to(endpoint, &context, out_file) {
                Ok(_) => println!("Write to {}", path_str),
                Err(e) => println!("Failed write to {}: {}", path_str, e),
            };
        }
        None => {
            let out = tera
                .render(endpoint, &context)
                .with_context(|| format!("Failed to render from {}", path_str))?;
            print!("{}", out);
        }
    }

    Ok(())
}
