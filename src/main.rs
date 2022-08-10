mod apply;
mod build;
mod cli;
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
