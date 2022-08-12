use anyhow::{Context, Result};
use std::{
    fs::canonicalize,
    path::{Path, PathBuf},
};

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

pub fn pretty_print(input: &[u8]) {
    bat::PrettyPrinter::new()
        .header(true)
        .grid(true)
        .line_numbers(true)
        .language("sql")
        .input_from_bytes(input)
        .print()
        .unwrap();
}

/// Check if a directory
/// Not sure why the .is_dir() is not works
/// https://doc.rust-lang.org/std/path/struct.PathBuf.html#method.is_dir
pub fn is_dir(path: &Path) -> bool {
    path.read_dir().is_ok()
}

#[cfg(test)]
mod tests {
    use super::*;
    use predicates::prelude::*;
    use std::env::current_dir;
    use std::fs::create_dir_all;
    use std::path::Path;
    use tempfile::tempdir;

    // No default working dir
    #[test]
    fn test_get_current_working_dir_without_param() {
        assert_eq!(
            get_current_working_dir(None).unwrap(),
            current_dir().unwrap()
        );
    }

    // With default working dir
    #[test]
    fn test_get_current_working_dir_with_param() {
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
        create_dir_all(&path).expect("could not create dir");

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
