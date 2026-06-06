//! athena-rs: A CLI tool for managing AWS Athena schemas using templated SQL
//!
//! This application provides two main commands:
//! - `build`: Render SQL from template files using the Tera template engine
//! - `apply`: Build and execute SQL statements in AWS Athena
//!
//! # Examples
//!
//! Build SQL from templates:
//! ```bash
//! athena build ./templates
//! ```
//!
//! Apply SQL to Athena:
//! ```bash
//! athena apply --output_location=s3://my-bucket/ ./templates
//! ```

mod apply;
mod build;
mod cli;
mod tera;
mod utils;

use anyhow::Result;
use env_logger::Env;

#[tokio::main]
async fn main() -> Result<()> {
    let env = Env::new().default_filter_or("info,aws_config=error,aws_smithy_http_tower=warn");
    env_logger::init_from_env(env);

    let args = cli::parse();

    match args.cmd {
        cli::Command::Build(args) => build::call(args).await,
        cli::Command::Apply(args) => apply::call(args).await,
    }
}
