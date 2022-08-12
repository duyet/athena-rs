use anyhow::{anyhow, bail, Context, Result};
use aws_sdk_athena::{
    model::{
        QueryExecutionContext,
        QueryExecutionState::{self, *},
        ResultConfiguration, ResultSet,
    },
    output::GetQueryExecutionOutput,
    Client,
};
use log::{error, info};
use regex::Regex;
use std::{env, path::PathBuf};
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
    /// Set this option via environment variable: export AWS_PROFILE=default
    #[clap(global = true, long, short)]
    pub profile: Option<String>,

    /// AWS Region
    #[clap(global = true, long, short)]
    /// Set this option via environment variable: export AWS_DEFAULT_REGION=us-east-1
    pub region: Option<String>,

    /// AWS Athena Workgroup
    /// Set this option via environment variable: export AWS_WORKGROUP=primary
    #[clap(global = true, long, short)]
    pub workgroup: Option<String>,

    /// AWS Athena output location
    /// The location in Amazon S3 where your query results are stored
    /// such as `s3://path/to/query/bucket/`
    /// Set this option via environment variable: export AWS_OUTPUT_LOCATION=s3://bucket/
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
    if args.no_pretty.unwrap_or_default() {
        print!("{}", sql);
    } else {
        pretty_print(sql.as_bytes());
    }

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
    let output_location = args
        .output_location
        .or_else(|| env::var("AWS_OUTPUT_LOCATION").ok());

    ResultConfiguration::builder()
        .set_output_location(output_location)
        .build()
}

fn get_query_execution_context(query: Option<String>) -> Option<QueryExecutionContext> {
    query.as_ref()?;

    let database = get_database_from_sql(query.unwrap());
    database.as_ref()?;

    let ctx = QueryExecutionContext::builder()
        .set_database(database)
        .build();

    Some(ctx)
}

async fn submit_and_wait(client: Client, query: Option<String>, args: Apply) -> Result<()> {
    if query.clone().is_none() {
        bail!("Empty query");
    }

    let workgroup = args.workgroup.clone();
    let result_configuration = get_result_configuration(args.clone());
    let query_execution_context = get_query_execution_context(query.clone());
    let query = query.unwrap();

    match &query_execution_context {
        Some(ctx) => match ctx.database() {
            Some(database) => info!("Submitting to database `{}`: ", database),
            _ => info!("Submitting ..."),
        },
        _ => info!("Submitting ..."),
    }

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
        .set_query_execution_context(query_execution_context)
        .send()
        .await?;

    let query_execution_id = resp.query_execution_id().unwrap_or_default();
    info!("Query execution id: {}", &query_execution_id);

    loop {
        let resp = client
            .get_query_execution()
            .set_query_execution_id(Some(query_execution_id.to_string()))
            .send()
            .await?;

        let status = status(&resp).unwrap();

        match status {
            Queued | Running => {
                sleep(Duration::from_secs(5)).await;
                info!("State: {:?}, sleep 5 secs ...", status);
            }
            Cancelled | Failed => {
                error!("State: {:?}", status);

                match get_query_result(&client, query_execution_id.to_string()).await {
                    Ok(result) => info!("Result: {:?}", result),
                    Err(e) => error!("Result error: {:?}", e),
                }

                break;
            }
            _ => {
                let millis = total_execution_time(&resp).unwrap();
                info!("State: {:?}", status);
                info!("Total execution time: {} millis", millis);

                match get_query_result(&client, query_execution_id.to_string()).await {
                    Ok(result) => info!("Result: {:?}", result),
                    Err(e) => error!("Result error: {:?}", e),
                }

                break;
            }
        }
    }

    Ok(())
}

fn status(resp: &GetQueryExecutionOutput) -> Option<&QueryExecutionState> {
    resp.query_execution().unwrap().status().unwrap().state()
}

fn total_execution_time(resp: &GetQueryExecutionOutput) -> Option<i64> {
    resp.query_execution()
        .unwrap()
        .statistics()
        .unwrap()
        .total_execution_time_in_millis()
}

async fn get_query_result(client: &Client, query_execution_id: String) -> Result<ResultSet> {
    let resp = client
        .get_query_results()
        .set_query_execution_id(Some(query_execution_id.clone()))
        .send()
        .await
        .with_context(|| {
            format!(
                "could not get query results for query id {}",
                query_execution_id
            )
        })?;

    Ok(resp
        .result_set()
        .ok_or_else(|| anyhow!("could not get query result"))?
        .clone())
}

fn get_database_from_sql<S: AsRef<str>>(sql: S) -> Option<String> {
    let re = vec![
        Regex::new(r"(?i)--\s+Database:\s(.*)").unwrap(),
        Regex::new(r"(?i)/*\s+Database:\s([^\s]+)\s\*/").unwrap(),
    ];

    for r in re.iter() {
        if let Some(caps) = r.captures(sql.as_ref()) {
            let name = caps.get(1).map_or("", |m| m.as_str());
            return Some(name.trim().to_string());
        }
    }

    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_database_from_sql() {
        let sql = "-- database: db0";
        assert_eq!(get_database_from_sql(sql).unwrap(), "db0");

        let sql = "-- database: db1\nSELECT * FROM ...;";
        assert_eq!(get_database_from_sql(sql).unwrap(), "db1");

        let sql = "-- Database: db2\nSELECT * FROM ...;";
        assert_eq!(get_database_from_sql(sql).unwrap(), "db2");

        let sql = "-- Database: db3 \nSELECT * FROM ...;";
        assert_eq!(get_database_from_sql(sql).unwrap(), "db3");

        let sql = "-- Database: db4    \nSELECT * FROM ...;";
        assert_eq!(get_database_from_sql(sql).unwrap(), "db4");

        let sql = "--   Database: db4    \nSELECT * FROM ...;";
        assert_eq!(get_database_from_sql(sql).unwrap(), "db4");

        let sql = "/* Database: db5 */\nSELECT * FROM ...;";
        assert_eq!(get_database_from_sql(sql).unwrap(), "db5");

        let sql = "/* database: db6 */\nSELECT * FROM ...;";
        assert_eq!(get_database_from_sql(sql).unwrap(), "db6");

        let sql = "/*        database: db7 */\nSELECT * FROM ...;";
        assert_eq!(get_database_from_sql(sql).unwrap(), "db7");

        let sql = "SELECT * FROM ...;";
        assert!(get_database_from_sql(sql).is_none());

        let sql = "-- database: db0 \n-- database: db1";
        assert_eq!(get_database_from_sql(sql).unwrap(), "db0");
    }
}
