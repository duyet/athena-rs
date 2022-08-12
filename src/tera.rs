use chrono::{Duration, NaiveDate};
use std::collections::HashMap;
use std::path::PathBuf;
use tera::{from_value, to_value, Error, Tera, Value};
use walkdir::WalkDir;

use crate::utils::is_dir;

const DATE_FORMAT: &str = "%Y-%m-%d";

/// Get Tera template, load the template from working dir
pub fn get_tera(target_path: PathBuf, working_dir: PathBuf) -> anyhow::Result<Tera> {
    let is_dir = is_dir(&target_path);
    let working_dir_str = working_dir.to_str().expect("could not get working dir str");
    let prefix = format!("{}/", working_dir_str);

    let mut tera = Tera::default();

    // Scan working_dir and adding .sql file as template
    for entry in WalkDir::new(&working_dir)
        .into_iter()
        .filter_map(|e| e.ok())
    {
        if let Some(ext) = entry.path().extension() {
            if ext == "sql" {
                let template_path = entry.path().display().to_string();
                let template_name = template_path.clone();
                let template_name = template_name.trim_start_matches(&prefix);
                tera.add_template_file(template_path, Some(template_name))?;
            }
        }
    }

    if !is_dir {
        let template_path = target_path.display().to_string();
        let template_name = target_path.file_name().expect("could not get file name");
        tera.add_template_file(template_path, template_name.to_str())?;
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

    let input = input.unwrap();

    match from_value::<String>(input.clone()) {
        Ok(val) => match NaiveDate::parse_from_str(&val, DATE_FORMAT) {
            Ok(v) => Ok(v),
            Err(_) => {
                return Err(Error::msg(format!(
                    "Function `date_range` received {key}={input} but `{key}` is invalid format ({DATE_FORMAT})",
                )))
            }
        },
        Err(_) => Err(Error::msg("Function `date_range` received {key}={input} but it is not a string")),
    }
}

pub fn date_range(args: &HashMap<String, Value>) -> tera::Result<Value> {
    let start = get_native_date("start", args.get("start")).unwrap();
    let end = get_native_date("end", args.get("end")).unwrap();

    let mut items = vec![];
    let mut cursor = start;
    while cursor < end {
        items.push(cursor.format(DATE_FORMAT).to_string());
        cursor += Duration::days(1);
    }

    Ok(to_value(items).unwrap())
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
