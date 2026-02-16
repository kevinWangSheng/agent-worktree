// ===========================================================================
// Integration Tests - Main Command
// ===========================================================================

mod common;

use std::process::Command;
use tempfile::tempdir;

use common::*;

#[test]
fn test_main_returns_repo_root() {
    let dir = tempdir().unwrap();
    setup_git_repo(dir.path());

    let path_file = create_path_file(dir.path());
    let output = Command::new(wt_binary())
        .args(["main", "--path-file", path_file.to_str().unwrap()])
        .current_dir(dir.path())
        .output()
        .expect("Failed to execute wt main");

    assert!(output.status.success());

    let path = read_path_file(&path_file);
    let path = path.trim();
    assert!(!path.is_empty());
    assert!(std::path::Path::new(path).exists());
}

#[test]
fn test_main_without_print_path() {
    let dir = tempdir().unwrap();
    setup_git_repo(dir.path());

    let output = Command::new(wt_binary())
        .arg("main")
        .current_dir(dir.path())
        .output()
        .expect("wt main failed");

    assert!(output.status.success());
}

#[test]
fn test_main_from_subdirectory() {
    let dir = tempdir().unwrap();
    setup_git_repo(dir.path());

    let sub = dir.path().join("nested").join("deep");
    std::fs::create_dir_all(&sub).unwrap();

    let path_file = create_path_file(dir.path());
    let output = Command::new(wt_binary())
        .args(["main", "--path-file", path_file.to_str().unwrap()])
        .current_dir(&sub)
        .output()
        .expect("wt main failed");

    assert!(output.status.success());
    let path = read_path_file(&path_file);
    assert!(!path.trim().is_empty());
}
