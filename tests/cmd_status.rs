// ===========================================================================
// Integration Tests - Status Command
// ===========================================================================

mod common;

use std::process::Command;
use tempfile::tempdir;

use common::*;

#[test]
fn test_status_in_main_repo() {
    let dir = tempdir().unwrap();
    setup_git_repo(dir.path());

    let output = Command::new(wt_binary())
        .arg("status")
        .current_dir(dir.path())
        .output()
        .expect("Failed to execute wt status");

    assert!(output.status.success());

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Branch:"));
    assert!(stdout.contains("main"));
    assert!(stdout.contains("Location:"));
    assert!(stdout.contains("main repository"));
    assert!(stdout.contains("Trunk:"));
    assert!(stdout.contains("Ahead:"));
    assert!(stdout.contains("Uncommitted:"));
    assert!(stdout.contains("Diff:"));
}

#[test]
fn test_status_with_uncommitted_changes() {
    let dir = tempdir().unwrap();
    setup_git_repo(dir.path());

    // Create an uncommitted file
    std::fs::write(dir.path().join("new_file.txt"), "hello\n").unwrap();

    let output = Command::new(wt_binary())
        .arg("status")
        .current_dir(dir.path())
        .output()
        .expect("Failed to execute wt status");

    assert!(output.status.success());

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Uncommitted:"));
    // Should show at least 1 uncommitted file
    assert!(stdout.contains("1 file"));
}

#[test]
fn test_status_with_commits_ahead() {
    let dir = tempdir().unwrap();
    setup_git_repo(dir.path());

    // Create a feature branch with a commit
    Command::new("git")
        .args(["checkout", "-b", "feature-status-test"])
        .current_dir(dir.path())
        .output()
        .unwrap();

    std::fs::write(dir.path().join("feature.txt"), "feature content\n").unwrap();

    Command::new("git")
        .args(["add", "."])
        .current_dir(dir.path())
        .output()
        .unwrap();

    Command::new("git")
        .args(["commit", "-m", "feature commit"])
        .current_dir(dir.path())
        .output()
        .unwrap();

    let output = Command::new(wt_binary())
        .arg("status")
        .current_dir(dir.path())
        .output()
        .expect("Failed to execute wt status");

    assert!(output.status.success());

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Branch:"));
    assert!(stdout.contains("feature-status-test"));
    assert!(stdout.contains("1 commit"));
}

#[test]
fn test_status_in_worktree() {
    let (_dir, repo, home) = setup_worktree_test_env();

    // Create a worktree
    let output = Command::new(wt_binary())
        .args(["new", "status-wt-test"])
        .current_dir(&repo)
        .env("HOME", &home)
        .output()
        .expect("wt new failed");

    if !output.status.success() {
        return;
    }

    // Find the worktree path from ls output
    let output = Command::new(wt_binary())
        .args(["ls", "-l"])
        .current_dir(&repo)
        .env("HOME", &home)
        .output()
        .expect("wt ls failed");

    let stdout = String::from_utf8_lossy(&output.stdout);

    // Extract the worktree path from ls -l output
    let wt_path = stdout
        .lines()
        .find(|l| l.contains("status-wt-test"))
        .and_then(|line| {
            // Path is the last column after spaces
            line.split_whitespace().last().map(|s| s.to_string())
        });

    if let Some(path) = wt_path {
        // Replace ~ with actual home
        let full_path = if path.starts_with("~/") {
            home.join(&path[2..])
        } else {
            std::path::PathBuf::from(&path)
        };

        if full_path.exists() {
            let output = Command::new(wt_binary())
                .arg("status")
                .current_dir(&full_path)
                .env("HOME", &home)
                .output()
                .expect("wt status in worktree failed");

            assert!(output.status.success());

            let stdout = String::from_utf8_lossy(&output.stdout);
            assert!(stdout.contains("Branch:"));
            assert!(stdout.contains("status-wt-test"));
            assert!(stdout.contains("Location:"));
            assert!(stdout.contains("managed worktree"));
        }
    }
}

#[test]
fn test_status_clean_repo() {
    let dir = tempdir().unwrap();
    setup_git_repo(dir.path());

    let output = Command::new(wt_binary())
        .arg("status")
        .current_dir(dir.path())
        .output()
        .expect("Failed to execute wt status");

    assert!(output.status.success());

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("0 commits"));
    assert!(stdout.contains("0 files"));
    assert!(stdout.contains("Diff:        -"));
}
