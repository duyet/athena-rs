use anyhow::{Context, Result};
use std::fs::canonicalize;
use std::path::{Path, PathBuf};
use tera::Tera;
use walkdir::WalkDir;

/// Get current working dir
pub fn get_current_working_dir(working_dir: Option<PathBuf>) -> Result<PathBuf> {
    let current_dir = std::env::current_dir()?;
    let dir = working_dir.unwrap_or(current_dir);

    canonicalize(&dir).with_context(|| format!("could not get full path: {:?}", dir))
}

/// Get abs path and convert to string
pub fn get_full_path_str(path: &Path) -> Result<String> {
    let path = canonicalize(path).with_context(|| format!("couldn't get full path {:?}", &path))?;

    path.to_str()
        .map(|t| t.trim_end_matches('/').to_string())
        .with_context(|| "could not convert to string".to_string())
}

/// Get Tera template, load the template from working dir
pub fn get_tera(_target_dir: PathBuf, working_dir: PathBuf) -> Result<Tera> {
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

    Ok(tera)
}

#[cfg(test)]
mod tests {
    use super::*;
    use predicates::prelude::*;
    use std::path::Path;
    use tempfile::tempdir;

    #[test]
    fn test_get_current_working_dir() {
        // No default working dir
        assert_eq!(
            get_current_working_dir(None).unwrap(),
            std::env::current_dir().unwrap()
        );

        let dir = tempdir().expect("could not create temp dir");
        let path_1 = dir.path();
        let path_2 = dir.path();
        let actual = get_current_working_dir(Some(path_1.to_path_buf())).unwrap();
        let predicate_fn = predicate::str::contains(format!("{}", path_2.display()));
        assert!(predicate_fn.eval(actual.to_str().unwrap()));
    }

    #[test]
    fn test_get_full_path_str() {
        let dir = tempdir().unwrap();
        let dir_str = dir.path().to_str().unwrap();

        // Create tempdir <temp>/dir for tests
        let path = dir.path().join("dir");
        std::fs::create_dir_all(&path).expect("could not create dir");

        let test_path = dir.path().join("dir");
        let predicate_fn = predicate::str::contains(format!("{}/dir", dir_str));
        assert!(predicate_fn.eval(&get_full_path_str(&test_path).unwrap()));

        let test_path = dir.path().join("dir/");
        let predicate_fn = predicate::str::contains(format!("{}/dir", dir_str));
        assert!(predicate_fn.eval(&get_full_path_str(&test_path).unwrap()));

        let test_path = dir.path().join("dir///");
        let predicate_fn = predicate::str::contains(format!("{}/dir", dir_str));
        assert!(predicate_fn.eval(&get_full_path_str(&test_path).unwrap()));

        // Folder is not found
        let test_path = dir.path().join("/not/exists/dir///");
        assert!(get_full_path_str(&test_path).is_err());

        // Could not work with a file
        let path = Path::new("/tmp/dir/a.sql");
        assert!(get_full_path_str(path).is_err());

        dir.close().expect("could not close the tempdir");
    }
}
