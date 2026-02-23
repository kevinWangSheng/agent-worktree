// ===========================================================================
// Integration Tests - Diff Command
// ===========================================================================

mod common;

use std::process::Command;
use tempfile::tempdir;

use common::*;

#[test]
fn test_diff_no_changes() {
    let dir = tempdir().unwrap();
    setup_git_repo(dir.path());

    let output = Command::new(wt_binary())
        .arg("diff")
        .current_dir(dir.path())
        .output()
        .expect("Failed to execute wt diff");

    assert!(output.status.success());
    // No changes from trunk, so stdout should be empty
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.is_empty() || !stdout.contains("fatal"));
}

#[test]
fn test_diff_with_committed_changes() {
    let dir = tempdir().unwrap();
    setup_git_repo(dir.path());

    // Create a branch with a committed change
    Command::new("git")
        .args(["checkout", "-b", "test-diff-branch"])
        .current_dir(dir.path())
        .output()
        .unwrap();

    std::fs::write(dir.path().join("new_file.txt"), "hello\n").unwrap();

    Command::new("git")
        .args(["add", "new_file.txt"])
        .current_dir(dir.path())
        .output()
        .unwrap();

    Command::new("git")
        .args(["commit", "-m", "Add new file"])
        .current_dir(dir.path())
        .output()
        .unwrap();

    let output = Command::new(wt_binary())
        .arg("diff")
        .current_dir(dir.path())
        .output()
        .expect("Failed to execute wt diff");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Committed changes"));
    assert!(stdout.contains("new_file.txt"));
}

#[test]
fn test_diff_with_uncommitted_changes() {
    let dir = tempdir().unwrap();
    setup_git_repo(dir.path());

    // Create a branch and make an uncommitted change
    Command::new("git")
        .args(["checkout", "-b", "test-diff-uncommitted"])
        .current_dir(dir.path())
        .output()
        .unwrap();

    // Modify an existing tracked file (uncommitted, unstaged)
    std::fs::write(dir.path().join("README.md"), "# Modified\n").unwrap();

    let output = Command::new(wt_binary())
        .arg("diff")
        .current_dir(dir.path())
        .output()
        .expect("Failed to execute wt diff");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Uncommitted changes"));
    assert!(stdout.contains("README.md"));
}

#[test]
fn test_diff_stat_mode() {
    let dir = tempdir().unwrap();
    setup_git_repo(dir.path());

    Command::new("git")
        .args(["checkout", "-b", "test-diff-stat"])
        .current_dir(dir.path())
        .output()
        .unwrap();

    std::fs::write(dir.path().join("stat_file.txt"), "content\n").unwrap();

    Command::new("git")
        .args(["add", "stat_file.txt"])
        .current_dir(dir.path())
        .output()
        .unwrap();

    Command::new("git")
        .args(["commit", "-m", "Add stat file"])
        .current_dir(dir.path())
        .output()
        .unwrap();

    let output = Command::new(wt_binary())
        .args(["diff", "--stat"])
        .current_dir(dir.path())
        .output()
        .expect("Failed to execute wt diff --stat");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Committed changes"));
    assert!(stdout.contains("stat_file.txt"));
    // --stat output contains insertion/deletion counts
    assert!(stdout.contains("insertion"));
}

#[test]
fn test_diff_target_branch() {
    let dir = tempdir().unwrap();
    setup_git_repo(dir.path());

    // Create a target branch with a file
    Command::new("git")
        .args(["checkout", "-b", "target-branch"])
        .current_dir(dir.path())
        .output()
        .unwrap();

    std::fs::write(dir.path().join("target.txt"), "target content\n").unwrap();

    Command::new("git")
        .args(["add", "target.txt"])
        .current_dir(dir.path())
        .output()
        .unwrap();

    Command::new("git")
        .args(["commit", "-m", "Add target file"])
        .current_dir(dir.path())
        .output()
        .unwrap();

    // Create a feature branch from main with different content
    Command::new("git")
        .args(["checkout", "main"])
        .current_dir(dir.path())
        .output()
        .unwrap();

    Command::new("git")
        .args(["checkout", "-b", "feature-branch"])
        .current_dir(dir.path())
        .output()
        .unwrap();

    std::fs::write(dir.path().join("feature.txt"), "feature content\n").unwrap();

    Command::new("git")
        .args(["add", "feature.txt"])
        .current_dir(dir.path())
        .output()
        .unwrap();

    Command::new("git")
        .args(["commit", "-m", "Add feature file"])
        .current_dir(dir.path())
        .output()
        .unwrap();

    // Compare against target-branch
    let output = Command::new(wt_binary())
        .args(["diff", "--target", "target-branch"])
        .current_dir(dir.path())
        .output()
        .expect("Failed to execute wt diff --target");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Committed changes"));
    assert!(stdout.contains("target-branch"));
}

#[test]
fn test_diff_stat_no_changes() {
    let dir = tempdir().unwrap();
    setup_git_repo(dir.path());

    let output = Command::new(wt_binary())
        .args(["diff", "--stat"])
        .current_dir(dir.path())
        .output()
        .expect("Failed to execute wt diff --stat");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    // No changes means no output
    assert!(!stdout.contains("fatal"));
}

#[test]
fn test_diff_with_staged_changes() {
    let dir = tempdir().unwrap();
    setup_git_repo(dir.path());

    Command::new("git")
        .args(["checkout", "-b", "test-diff-staged"])
        .current_dir(dir.path())
        .output()
        .unwrap();

    std::fs::write(dir.path().join("staged.txt"), "staged content\n").unwrap();

    Command::new("git")
        .args(["add", "staged.txt"])
        .current_dir(dir.path())
        .output()
        .unwrap();

    // Don't commit -- the file is staged but not committed
    let output = Command::new(wt_binary())
        .arg("diff")
        .current_dir(dir.path())
        .output()
        .expect("Failed to execute wt diff");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Staged changes"));
    assert!(stdout.contains("staged.txt"));
}
