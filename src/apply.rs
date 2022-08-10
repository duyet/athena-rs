use anyhow::{bail, Result};
use aws_sdk_athena::{model::ResultConfiguration, Client};
use log::{error, info};
use std::path::PathBuf;
use tokio::time::{sleep, Duration};

use crate::utils::pretty_print;

#[derive(clap::Args, Debug, Clone)]
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

    /// AWS Region
    #[clap(global = true, long, short)]
    pub region: Option<String>,

    /// AWS Athena Workgroup
    #[clap(global = true, long, short)]
    pub workgroup: Option<String>,

    /// AWS Athena output location
    /// The location in Amazon S3 where your query results are stored
    /// such as `s3://path/to/query/bucket/`
    #[clap(global = true, long, short)]
    pub output_location: Option<String>,

    /// No pretty print for SQL
    #[clap(long)]
    pub no_pretty: Option<bool>,
}

pub async fn call(args: Apply) -> Result<()> {
    let build_args = crate::build::Build {
        file: args.file.clone(),
        out: None,
        context: args.context.clone(),
        no_pretty: None,
    };

    let sql = crate::build::build(build_args)?;
    println!("SQL: {}", sql);

    // Set AWS_PROFILE
    if let Some(ref profile) = args.profile {
        std::env::set_var("AWS_PROFILE", profile);
    }

    // Set AWS_DEFAULT_REGION
    if let Some(ref region) = args.region {
        std::env::set_var("AWS_DEFAULT_REGION", region);
    }

    let shared_config = aws_config::load_from_env().await;
    let client = Client::new(&shared_config);

    // Healthcheck
    submit_and_wait(client.clone(), Some("SELECT 1".to_string()), args.clone()).await?;

    // Submit SQL
    let sql = sql
        .split(';')
        .into_iter()
        .map(|s| s.trim())
        .filter(|s| !s.is_empty())
        .collect::<Vec<_>>();
    info!("Submitting {} queries to Athena", sql.len());

    for s in sql {
        submit_and_wait(client.clone(), Some(s.to_string()), args.clone()).await?;
    }

    Ok(())
}

fn get_result_configuration(args: Apply) -> ResultConfiguration {
    ResultConfiguration::builder()
        .set_output_location(args.output_location)
        .build()
}

async fn submit_and_wait(client: Client, query: Option<String>, args: Apply) -> Result<()> {
    let workgroup = args.workgroup.clone();
    let result_configuration = get_result_configuration(args.clone());

    if query.is_none() {
        bail!("Empty query");
    }

    let query = query.unwrap();

    info!("Submitting: ");
    if args.no_pretty.unwrap_or_default() {
        print!("{}", query);
    } else {
        pretty_print(query.as_bytes());
    }

    let resp = client
        .start_query_execution()
        .set_query_string(Some(query))
        .set_work_group(workgroup)
        .set_result_configuration(Some(result_configuration.clone()))
        .send()
        .await?;

    let query_execution_id = resp.query_execution_id().unwrap_or_default();

    loop {
        let resp = client
            .get_query_execution()
            .set_query_execution_id(Some(query_execution_id.to_string()))
            .send()
            .await?;

        let status = resp
            .query_execution()
            .unwrap()
            .status()
            .unwrap()
            .state()
            .unwrap();

        use aws_sdk_athena::model::QueryExecutionState::*;
        match status {
            Queued | Running => {
                sleep(Duration::from_secs(5)).await;
                info!("State: {:?}, sleep 5 secs ...", status);
            }
            Cancelled | Failed => {
                error!("State: {:?}", status);
                break;
            }
            _ => {
                let total_execution_time_in_millis = resp
                    .query_execution()
                    .unwrap()
                    .statistics()
                    .unwrap()
                    .total_execution_time_in_millis()
                    .unwrap();

                info!("State: {:?}", status);
                info!(
                    "Total execution time: {} millis",
                    total_execution_time_in_millis
                );
                break;
            }
        }
    }

    Ok(())
}
