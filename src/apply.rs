//! AWS Athena query execution functionality
//!
//! This module handles building SQL templates and executing them in AWS Athena.
//! It provides functionality to:
//! - Submit queries to Athena
//! - Poll for query completion
//! - Retrieve query results
//! - Extract database context from SQL comments
//!
//! # Database Context
//!
//! You can specify the target database using SQL comments:
//!
//! ```sql
//! -- Database: my_database
//! CREATE TABLE example (id INT);
//! ```
//!
//! or
//!
//! ```sql
//! /* Database: my_database */
//! CREATE TABLE example (id INT);
//! ```

use anyhow::{anyhow, bail, Context, Result};
use aws_sdk_athena::{
    operation::get_query_execution::GetQueryExecutionOutput,
    types::{QueryExecutionContext, QueryExecutionState, ResultConfiguration, ResultSet},
    Client,
};
use devtimer::DevTime;
use log::{error, info};
use once_cell::sync::Lazy;
use regex::Regex;
use std::{collections::HashMap, env, path::PathBuf};
use tokio::time::{sleep, Duration};

use crate::utils::pretty_print;

// Constants
const QUERY_POLL_INTERVAL_SECS: u64 = 5;
const SQL_STATEMENT_SEPARATOR: char = ';';

// Compile regex patterns once and reuse them for extracting database names from SQL
static DATABASE_PATTERNS: Lazy<Vec<Regex>> = Lazy::new(|| {
    vec![
        // Matches: -- Database: db_name
        Regex::new(r"(?i)--\s+Database:\s(.*)").expect("invalid regex pattern"),
        // Matches: /* Database: db_name */
        Regex::new(r"(?i)/*\s+Database:\s([^\s]+)\s\*/").expect("invalid regex pattern"),
    ]
});

#[derive(clap::Args, Debug, Clone)]
pub struct Apply {
    /// Target path to render. If the target path is a directory,
    /// the root folder must contains the index.sql file
    pub file: PathBuf,

    /// Change the context current working dir
    #[arg(long, short)]
    pub context: Option<PathBuf>,

    /// Dry-run
    #[arg(global = true, long, short)]
    pub dry_run: Option<bool>,

    /// AWS Profile
    /// Set this option via environment variable: export AWS_PROFILE=default
    #[arg(global = true, long, short)]
    pub profile: Option<String>,

    /// AWS Region
    #[arg(global = true, long, short)]
    /// Set this option via environment variable: export AWS_DEFAULT_REGION=us-east-1
    pub region: Option<String>,

    /// AWS Athena Workgroup
    /// Set this option via environment variable: export AWS_WORKGROUP=primary
    #[arg(global = true, long, short)]
    pub workgroup: Option<String>,

    /// AWS Athena output location
    /// The location in Amazon S3 where your query results are stored
    /// such as `s3://path/to/query/bucket/`
    /// Set this option via environment variable: export AWS_OUTPUT_LOCATION=s3://bucket/
    #[arg(global = true, long, short)]
    pub output_location: Option<String>,

    /// No pretty print for SQL
    #[arg(long)]
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
        .split(SQL_STATEMENT_SEPARATOR)
        .into_iter()
        .map(|s| s.trim())
        .filter(|s| !s.is_empty())
        .collect::<Vec<_>>();
    info!("Submitting {} queries to Athena", sql.len());

    let mut stats: HashMap<QueryExecutionState, i32> = HashMap::new();

    // Timer
    let mut timer = DevTime::new_simple();
    timer.start();

    for s in sql {
        let state = submit_and_wait(client.clone(), Some(s.to_string()), args.clone()).await?;

        // Update stats
        stats
            .entry(state.clone())
            .and_modify(|c| *c += 1)
            .or_insert(0);
    }

    timer.stop();

    info!("");
    info!("Statistics:");
    info!("  ==> {:?}", stats);
    if let Some(secs) = timer.time_in_secs() {
        info!("  ==> Took: {:?} seconds", secs);
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
    let query = query.as_ref()?;

    let database = get_database_from_sql(query);
    database.as_ref()?;

    let ctx = QueryExecutionContext::builder()
        .set_database(database)
        .build();

    Some(ctx)
}

async fn submit_and_wait(
    client: Client,
    query: Option<String>,
    args: Apply,
) -> Result<QueryExecutionState> {
    if query.clone().is_none() {
        bail!("Empty query");
    }

    // Timer
    let mut timer = DevTime::new_simple();
    timer.start();

    let workgroup = args.workgroup.clone();
    let result_configuration = get_result_configuration(args.clone());
    let query_execution_context = get_query_execution_context(query.clone());
    let query = query.unwrap();

    match &query_execution_context {
        Some(ctx) => match ctx.database() {
            Some(database) => info!("\nSubmitting to database `{}`: ", database),
            _ => info!("\nSubmitting ..."),
        },
        _ => info!("\nSubmitting ..."),
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

    let query_execution_id = resp
        .query_execution_id()
        .ok_or_else(|| anyhow!("query execution id not found in response"))?;
    info!("Query execution id: {}", &query_execution_id);

    let mut state: QueryExecutionState;

    loop {
        let resp = client
            .get_query_execution()
            .set_query_execution_id(Some(query_execution_id.to_string()))
            .send()
            .await?;

        state = status(&resp)
            .ok_or_else(|| anyhow!("could not get query execution status from response"))?
            .clone();

        match state {
            QueryExecutionState::Queued | QueryExecutionState::Running => {
                sleep(Duration::from_secs(QUERY_POLL_INTERVAL_SECS)).await;
                info!(
                    "State: {:?}, sleeping {} secs ...",
                    state, QUERY_POLL_INTERVAL_SECS
                );
            }
            QueryExecutionState::Cancelled | QueryExecutionState::Failed => {
                error!("State: {:?}", state);

                match get_query_result(&client, query_execution_id.to_string()).await {
                    Ok(result) => info!("Result: {:?}", result),
                    Err(e) => error!("Result error: {:?}", e),
                }

                break;
            }
            _ => {
                info!("State: {:?}", state);
                if let Some(millis) = total_execution_time(&resp) {
                    info!("Total execution time: {} millis", millis);
                }

                match get_query_result(&client, query_execution_id.to_string()).await {
                    Ok(result) => info!("Result: {:?}", result),
                    Err(e) => error!("Result error: {:?}", e),
                }

                break;
            }
        }
    }

    timer.stop();
    if let Some(secs) = timer.time_in_secs() {
        info!("Took: {} secs", secs);
    }

    Ok(state.clone())
}

fn status(resp: &GetQueryExecutionOutput) -> Option<&QueryExecutionState> {
    resp.query_execution()
        .and_then(|qe| qe.status())
        .and_then(|s| s.state())
}

fn total_execution_time(resp: &GetQueryExecutionOutput) -> Option<i64> {
    resp.query_execution()
        .and_then(|qe| qe.statistics())
        .and_then(|s| s.total_execution_time_in_millis())
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
    for r in DATABASE_PATTERNS.iter() {
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

        let sql = "/* database: db0 */\n/* database: db1 */";
        assert_eq!(get_database_from_sql(sql).unwrap(), "db0");
    }

    #[test]
    fn test_get_database_from_sql_with_comment() {
        let sql = "-- database: db0\n-- comment\nSELECT * FROM ...;";
        assert_eq!(get_database_from_sql(sql).unwrap(), "db0");

        let sql = "-- database: db0\n-- comment\n-- comment\nSELECT * FROM ...;";
        assert_eq!(get_database_from_sql(sql).unwrap(), "db0");

        let sql = "-- database: db0\n-- comment\n-- comment\n-- comment\nSELECT * FROM ...;";
        assert_eq!(get_database_from_sql(sql).unwrap(), "db0");
    }
}
