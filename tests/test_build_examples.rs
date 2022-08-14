use assert_cmd::prelude::*;
use predicates::prelude::*;
use serial_test::serial;
use std::env::set_current_dir;
use std::path::PathBuf;
use std::process::Command;

macro_rules! setup_env {
    () => {
        // Set working dir to examples/
        let dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        let dir = dir.join("examples");
        assert!(set_current_dir(&dir).is_ok());
    };
}

#[test]
#[serial]
fn test_build_example_stg() {
    setup_env!();

    // $ athena build stg
    let mut cmd = Command::cargo_bin("athena").unwrap();
    cmd.arg("build")
        .arg("stg")
        .arg("--no-pretty")
        .arg("true")
        .assert()
        .success()
        .stdout(predicate::str::contains("stg")) // must contains "stg"
        .stdout(predicate::str::contains("prd").count(0)); // must not contains any "prd"
}

#[test]
#[serial]
fn test_build_example_prd() {
    setup_env!();

    // $ athena build stg
    let mut cmd = Command::cargo_bin("athena").unwrap();
    cmd.arg("build")
        .arg("prd")
        .arg("--no-pretty")
        .arg("true")
        .assert()
        .success()
        .stdout(predicate::str::contains("prd")) // must contains "prd"
        .stdout(predicate::str::contains("stg").count(0)); // must not contains any "stg"
}

#[test]
#[serial]
fn test_build_example_single() {
    setup_env!();

    // $ athena build stg
    let mut cmd = Command::cargo_bin("athena").unwrap();
    cmd.arg("build")
        .arg("single/single.sql")
        .arg("--no-pretty")
        .arg("true")
        .assert()
        .success();
}
