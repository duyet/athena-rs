//! Tera template engine setup and custom functions
//!
//! This module configures the Tera template engine for rendering SQL templates.
//! It provides:
//! - Template loading from the working directory
//! - Custom Tera functions (e.g., `date_range`)
//!
//! # Custom Functions
//!
//! ## `date_range`
//!
//! Generates a list of dates between start and end dates (exclusive of end).
//!
//! ```sql
//! {% set start_date = "2022-01-01" %}
//! {% set end_date = "2022-01-05" %}
//! {% for date in date_range(start=start_date, end=end_date) %}
//!   PARTITION (date='{{ date }}')
//! {% endfor %}
//! ```
//!
//! This generates:
//! ```text
//! PARTITION (date='2022-01-01')
//! PARTITION (date='2022-01-02')
//! PARTITION (date='2022-01-03')
//! PARTITION (date='2022-01-04')
//! ```

use chrono::{Duration, NaiveDate};
use log::debug;
use std::collections::HashMap;
use std::ffi::OsStr;
use std::path::PathBuf;
use tera::{from_value, to_value, Error, Tera, Value};
use walkdir::WalkDir;

use crate::utils::is_dir;

// Constants
const DATE_FORMAT: &str = "%Y-%m-%d";
const SQL_FILE_EXTENSION: &str = "sql";

/// Get Tera template, load the template from working dir
pub fn get_tera(target_path: PathBuf, working_dir: PathBuf) -> anyhow::Result<Tera> {
    let is_dir = is_dir(&target_path);
    let working_dir_str = working_dir
        .to_str()
        .ok_or_else(|| anyhow::anyhow!("working directory path is not valid UTF-8"))?;
    let prefix = format!("{}/", working_dir_str);

    let mut tera = Tera::default();

    // Scan working_dir and adding .sql file as template
    let templates = WalkDir::new(&working_dir)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter_map(|e| {
            if e.path().extension() == Some(OsStr::new(SQL_FILE_EXTENSION)) {
                Some(e)
            } else {
                None
            }
        })
        .map(|e| {
            let template_path = e.path().display().to_string();
            let template_name = template_path.trim_start_matches(&prefix).to_string();
            (template_path, Some(template_name))
        })
        .collect::<Vec<_>>();

    debug!("Loaded: {:?}", templates);
    tera.add_template_files(templates)?;

    if !is_dir {
        let template_path = target_path.display().to_string();
        let template_name = target_path
            .file_name()
            .and_then(|n| n.to_str())
            .ok_or_else(|| anyhow::anyhow!("could not get file name from target path"))?;
        tera.add_template_file(template_path, Some(template_name))?;
    }

    // Register functions
    tera.register_function("date_range", date_range);

    Ok(tera)
}

fn get_native_date(key: &str, input: Option<&Value>) -> tera::Result<NaiveDate> {
    if input.is_none() {
        return Err(Error::msg(format!(
            "Function `date_range` was called without a `{key}` argument",
        )));
    }

    // Safety: We just checked that input is Some above
    let input = input.unwrap();

    match from_value::<String>(input.clone()) {
        Ok(val) => match NaiveDate::parse_from_str(&val, DATE_FORMAT) {
            Ok(v) => Ok(v),
            Err(_) => {
                Err(Error::msg(format!(
                    "Function `date_range` received {key}={input} but `{key}` is invalid format ({DATE_FORMAT})",
                )))
            }
        },
        Err(_) => Err(Error::msg(format!("Function `date_range` received {key}={input} but it is not a string"))),
    }
}

pub fn date_range(args: &HashMap<String, Value>) -> tera::Result<Value> {
    let start = get_native_date("start", args.get("start"))?;
    let end = get_native_date("end", args.get("end"))?;

    let mut items = vec![];
    let mut cursor = start;
    while cursor < end {
        items.push(cursor.format(DATE_FORMAT).to_string());
        cursor += Duration::days(1);
    }

    to_value(items).map_err(|e| Error::msg(format!("failed to convert date range to value: {}", e)))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_date_range() {
        let mut args = HashMap::new();
        args.insert("start".to_string(), to_value("2022-01-29").unwrap());
        args.insert("end".to_string(), to_value("2022-02-02").unwrap());

        let res = date_range(&args).unwrap();
        assert_eq!(
            res,
            to_value(vec!["2022-01-29", "2022-01-30", "2022-01-31", "2022-02-01"]).unwrap()
        );
    }
}
