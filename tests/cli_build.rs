use assert_cmd::prelude::*; // Add methods on commands
use indoc::indoc;
use predicates::prelude::*;
use serial_test::serial;
use std::env::set_current_dir;
use std::fs::File;
use std::io::Write;
use std::process::Command; // Run programs
use tempfile::tempdir;

#[test]
#[serial]
fn missing_arguments() {
    let mut cmd = Command::cargo_bin("athena").unwrap();

    // OPTIONS:
    //     -d, --dry-run <DRY_RUN>    Dry-run
    //     -h, --help                 Print help information
    //     -V, --version              Print version information
    //
    // SUBCOMMANDS:
    //     apply    Apply SQL to Athena
    //     build    Build SQL from template
    //     help     Print this message or the help of the given subcommand(s)

    cmd.assert()
        .failure()
        .stderr(predicate::str::contains("apply"))
        .stderr(predicate::str::contains("build"))
        .stderr(predicate::str::contains("help"));
}

#[test]
#[serial]
fn build_missing_file() {
    let mut cmd = Command::cargo_bin("athena").unwrap();
    cmd.arg("build").assert().failure();

    // error: The following required arguments were not provided:
    //     <FILE>
    //
    // USAGE:
    //     athena build [OPTIONS] <FILE>

    cmd.assert()
        .failure()
        .stderr(predicate::str::contains(
            "The following required arguments were not provided",
        ))
        .stderr(predicate::str::contains("<FILE>"));
}

/// Create an empty folder.
/// $ athena build .
/// stderr and stdout should empty
#[test]
#[serial]
fn build_on_empty_dir() {
    // create a temporary directory
    let dir = tempdir().unwrap();

    // Set working dir to tempdir
    assert!(set_current_dir(&dir).is_ok());

    let mut cmd = Command::cargo_bin("athena").unwrap();
    cmd.arg("build")
        .arg(".")
        .assert()
        .success()
        .stdout(predicate::str::is_empty())
        .stderr(predicate::str::is_empty());

    // cleanup
    dir.close().unwrap();
}

/// Create an empty folder <temp>.
/// Create a index.sql file with content: SELECT 1
/// $ athena build .
/// stdout should be: SELECT 1
#[test]
#[serial]
fn test_render_simple_file_should_works() {
    // create a temporary directory
    let dir = tempdir().unwrap();

    // Create a file inside dir
    let file_path = dir.path().join("index.sql");
    let mut file = File::create(file_path).expect("could not create temp file");
    writeln!(file, "SELECT 1").expect("could not write to temp file");

    // Set working dir to tempdir
    assert!(set_current_dir(&dir).is_ok());

    // $ athena build <path>
    let mut cmd = Command::cargo_bin("athena").unwrap();
    cmd.arg("build")
        .arg(".")
        .arg("--no-pretty")
        .arg("true")
        .assert()
        .success()
        .stdout(predicate::str::contains("SELECT 1"));

    // cleanup
    dir.close().unwrap();
}

/// Create an empty folder.
/// Create a index.sql file with content: SELECT 1
/// $ athena build ./////
/// stdout should be: SELECT 1
#[test]
#[serial]
fn should_works_with_trailing_slashs() {
    // create a temporary directory
    let dir = tempdir().unwrap();

    // Create a file inside tempdir
    let file_path = dir.path().join("index.sql");
    let mut file = File::create(file_path).expect("could not create temp file");
    writeln!(file, "SELECT 1").expect("could not write to temp file");

    // Set working dir to tempdir
    assert!(set_current_dir(&dir).is_ok());

    // $ athena build <path>////
    let mut cmd = Command::cargo_bin("athena").unwrap();
    cmd.arg("build")
        .arg("./////")
        .arg("--no-pretty")
        .arg("true")
        .assert()
        .success()
        .stdout(predicate::str::contains("SELECT 1"));

    // cleanup
    dir.close().unwrap();
}

/// Create an empty folder.
/// Create a file with content: SELECT 1
/// $ athena build .
/// stdout should be: SELECT 1
#[test]
#[serial]
fn does_not_contains_index_file() {
    // create a temporary directory
    let dir = tempdir().unwrap();

    // Create a file inside dir
    let file_path = dir.path().join("not_index.sql");
    let mut file = File::create(file_path).expect("could not create temp file");
    writeln!(file, "SELECT 1").expect("could not write to temp file");

    // Set working dir to tempdir
    assert!(set_current_dir(&dir).is_ok());

    // $ athena build <path>
    // but the <path> doesn't contain index.sql file
    let mut cmd = Command::cargo_bin("athena").unwrap();
    cmd.arg("build")
        .arg(dir.path())
        .arg("--no-pretty")
        .arg("true")
        .assert()
        .failure()
        .stderr(predicate::str::contains("'index.sql' not found"));

    // cleanup
    dir.close().unwrap();
}

/// Create a random sql file with content: SELECT 1
/// $ athena build <path>
/// stdout should be: SELECT 1
#[test]
#[serial]
fn should_works_with_a_file_in_cwd() {
    // create a temporary directory
    let dir = tempdir().unwrap();

    // Create a file inside tempdir
    let file_path = dir.path().join("index.sql");
    let mut file = File::create(file_path).expect("could not create temp file");
    writeln!(file, "SELECT 1").expect("could not write to temp file");

    // Set working dir to tempdir
    assert!(set_current_dir(&dir).is_ok());

    let full_file_path = format!("{}/index.sql", dir.path().display());

    // $ athena build <file>
    let mut cmd = Command::cargo_bin("athena").unwrap();
    cmd.arg("build")
        .arg(full_file_path)
        .arg("--no-pretty")
        .arg("true")
        .assert()
        .success()
        .stdout(predicate::str::contains("SELECT 1"));

    // cleanup
    dir.close().unwrap();
}

/// Create a random sql file with content: SELECT 1
/// $ athena build <path>
/// stdout should be: SELECT 1
#[test]
#[serial]
fn should_works_with_a_file_at_any_cwd() {
    // create a temporary directory
    let dir = tempdir().unwrap();

    // Create a file inside tempdir
    let file_path = dir.path().join("a.sql");
    let mut file = File::create(file_path).expect("could not create temp file");
    writeln!(file, "SELECT 1").expect("could not write to temp file");

    // Set working dir to tempdir
    assert!(set_current_dir(&dir).is_ok());

    let full_file_path = format!("{}/a.sql", dir.path().display());

    // $ athena build <file>
    let mut cmd = Command::cargo_bin("athena").unwrap();
    cmd.arg("build")
        .arg(full_file_path)
        .arg("--no-pretty")
        .arg("true")
        .assert()
        .success()
        .stdout(predicate::str::contains("SELECT 1"));

    // cleanup
    dir.close().expect("could not clean up tempdir");
}

#[test]
#[serial]
fn test_render_variables() {
    let template = indoc! { r#"
        {% set env = "production" %}

        THIS IS {{ env }}
    "# };
    let expected = "THIS IS production";

    // create a temporary directory
    let dir = tempdir().unwrap();

    // Create a file inside tempdir
    let file_path = dir.path().join("index.sql");
    let mut file = File::create(file_path).expect("could not create temp file");

    writeln!(file, "{}", &template).expect("could not write to temp file");

    // Set working dir to tempdir
    assert!(set_current_dir(&dir).is_ok());

    let full_file_path = format!("{}/index.sql", dir.path().display());

    // $ athena build <file>
    let mut cmd = Command::cargo_bin("athena").unwrap();
    cmd.arg("build")
        .arg(full_file_path)
        .arg("--no-pretty")
        .arg("true")
        .assert()
        .success()
        .stdout(predicate::str::contains(expected));

    // cleanup
    dir.close().unwrap();
}

#[test]
#[serial]
fn test_render_include_sub_template_file() {
    let template_table = indoc! { r#"
        SELECT * FROM {{ env }}
    "# };
    let template_index = indoc! { r#"
        {% set env = "production" %}
        {% include "table.sql" %}
    "# };
    let expected = "SELECT * FROM production";

    // create a temporary directory
    let dir = tempdir().unwrap();

    // Create a table.sql file
    let file_path = dir.path().join("table.sql");
    let mut file = File::create(file_path).expect("could not create temp file");
    writeln!(file, "{}", &template_table).expect("could not write to temp file");

    // Create a index.sql file
    let file_path = dir.path().join("index.sql");
    let mut file = File::create(file_path).expect("could not create temp file");
    writeln!(file, "{}", &template_index).expect("could not write to temp file");

    // Set working dir to tempdir
    assert!(set_current_dir(&dir).is_ok());

    let file_path = format!("{}", dir.path().display());

    // $ athena build <file>
    let mut cmd = Command::cargo_bin("athena").unwrap();
    cmd.arg("build")
        .arg(file_path)
        .arg("--no-pretty")
        .arg("true")
        .assert()
        .success()
        .stdout(predicate::str::contains(expected));

    // cleanup
    dir.close().unwrap();
}

#[test]
#[serial]
fn test_render_date_range() {
    let template_table = indoc! { r#"
        {% for date in date_range(start = start_date, end = end_date)  %}
        - date: {{ date }} => Y: {{ date | date(format = "%Y") -}}
        {% endfor %}
    "# };
    let template_index = indoc! { r#"
        {% set start_date = "2022-01-01" %}
        {% set end_date = "2022-01-05" %}
        {% include "table.sql" %}
    "# };

    let expected = indoc! { r#"
        - date: 2022-01-01 => Y: 2022
        - date: 2022-01-02 => Y: 2022
        - date: 2022-01-03 => Y: 2022
        - date: 2022-01-04 => Y: 2022"# };

    // create a temporary directory
    let dir = tempdir().unwrap();

    // Create a table.sql file
    let file_path = dir.path().join("table.sql");
    let mut file = File::create(file_path).expect("could not create temp file");
    writeln!(file, "{}", &template_table).expect("could not write to temp file");

    // Create a index.sql file
    let file_path = dir.path().join("index.sql");
    let mut file = File::create(file_path).expect("could not create temp file");
    writeln!(file, "{}", &template_index).expect("could not write to temp file");

    // Set working dir to tempdir
    assert!(set_current_dir(&dir).is_ok());

    let file_path = format!("{}", dir.path().display());

    // $ athena build <file>
    let mut cmd = Command::cargo_bin("athena").unwrap();
    cmd.arg("build")
        .arg(file_path)
        .arg("--no-pretty")
        .arg("true")
        .assert()
        .success()
        .stdout(predicate::str::contains(expected));

    // cleanup
    dir.close().unwrap();
}
