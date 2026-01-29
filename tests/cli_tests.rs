use assert_cmd::Command;
use predicates::prelude::*;
use std::fs::{self, File};
use tempfile::tempdir;

#[test]
fn test_cli_basic_flow() -> Result<(), Box<dyn std::error::Error>> {
    let dir = tempdir()?;
    let root = dir.path();

    File::create(root.join("main.rs"))?;
    fs::create_dir(root.join("src"))?;
    File::create(root.join("src/lib.rs"))?;

    let mut cmd = Command::new(assert_cmd::cargo::cargo_bin!("context"));

    cmd.arg(root).arg("--format").arg("markdown").arg("-v");

    cmd.assert()
        .success()
        .stdout(predicate::str::contains("# Project Context Report"))
        .stdout(predicate::str::contains("main.rs"))
        .stderr(predicate::str::contains("Found 2 files"));

    Ok(())
}

#[test]
fn test_cli_filtering_flow() -> Result<(), Box<dyn std::error::Error>> {
    let dir = tempdir()?;
    let root = dir.path();

    File::create(root.join("keep.rs"))?;
    File::create(root.join("ignore.py"))?;

    let mut cmd = Command::new(assert_cmd::cargo::cargo_bin!("context"));

    cmd.arg(root).arg("-e").arg("rs").arg("-v");

    cmd.assert()
        .success()
        .stdout(predicate::str::contains("keep.rs"))
        .stdout(predicate::str::contains("ignore.py").not());

    Ok(())
}
