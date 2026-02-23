// ===========================================================================
// Integration Tests - Merge Dry-Run
// ===========================================================================

mod common;

use std::path::PathBuf;
use std::process::Command;

use common::*;

#[test]
fn test_merge_dry_run_shows_preview() {
    let (dir, repo, home) = setup_worktree_test_env();

    let path_file = create_path_file(dir.path());
    let output = Command::new(wt_binary())
        .args([
            "new",
            "dry-run-merge",
            "--path-file",
            path_file.to_str().unwrap(),
        ])
        .current_dir(&repo)
        .env("HOME", &home)
        .output()
        .expect("wt new failed");

    if !output.status.success() {
        return;
    }

    let wt_path = PathBuf::from(read_path_file(&path_file).trim());

    // Make a commit in the worktree
    std::fs::write(wt_path.join("feature.txt"), "dry run feature").unwrap();
    Command::new("git")
        .args(["add", "."])
        .current_dir(&wt_path)
        .output()
        .unwrap();
    Command::new("git")
        .args(["commit", "-m", "Add dry-run feature"])
        .current_dir(&wt_path)
        .output()
        .unwrap();

    // Run merge --dry-run
    let output = Command::new(wt_binary())
        .args(["merge", "--dry-run"])
        .current_dir(&wt_path)
        .env("HOME", &home)
        .output()
        .expect("wt merge --dry-run failed");

    assert!(output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("[dry-run]"),
        "Expected [dry-run] prefix in output, got: {stderr}"
    );
    assert!(
        stderr.contains("dry-run-merge"),
        "Expected branch name in output, got: {stderr}"
    );
    assert!(
        stderr.contains("main"),
        "Expected trunk name in output, got: {stderr}"
    );
}

#[test]
fn test_merge_dry_run_does_not_merge() {
    let (dir, repo, home) = setup_worktree_test_env();

    let path_file = create_path_file(dir.path());
    let output = Command::new(wt_binary())
        .args([
            "new",
            "dry-run-nomerge",
            "--path-file",
            path_file.to_str().unwrap(),
        ])
        .current_dir(&repo)
        .env("HOME", &home)
        .output()
        .expect("wt new failed");

    if !output.status.success() {
        return;
    }

    let wt_path = PathBuf::from(read_path_file(&path_file).trim());

    // Make a commit
    std::fs::write(wt_path.join("feature.txt"), "should not merge").unwrap();
    Command::new("git")
        .args(["add", "."])
        .current_dir(&wt_path)
        .output()
        .unwrap();
    Command::new("git")
        .args(["commit", "-m", "Should not merge"])
        .current_dir(&wt_path)
        .output()
        .unwrap();

    // Run merge --dry-run
    let output = Command::new(wt_binary())
        .args(["merge", "--dry-run"])
        .current_dir(&wt_path)
        .env("HOME", &home)
        .output()
        .expect("wt merge --dry-run failed");

    assert!(output.status.success());

    // Verify the worktree still exists (was not cleaned up)
    assert!(wt_path.exists(), "Worktree should still exist after dry-run");

    // Verify the branch still exists
    let branch_check = Command::new("git")
        .args(["branch", "--list", "dry-run-nomerge"])
        .current_dir(&repo)
        .output()
        .unwrap();
    let branches = String::from_utf8_lossy(&branch_check.stdout);
    assert!(
        branches.contains("dry-run-nomerge"),
        "Branch should still exist after dry-run"
    );
}

#[test]
fn test_merge_dry_run_with_strategy() {
    let (dir, repo, home) = setup_worktree_test_env();

    let path_file = create_path_file(dir.path());
    let output = Command::new(wt_binary())
        .args([
            "new",
            "dry-run-strategy",
            "--path-file",
            path_file.to_str().unwrap(),
        ])
        .current_dir(&repo)
        .env("HOME", &home)
        .output()
        .expect("wt new failed");

    if !output.status.success() {
        return;
    }

    let wt_path = PathBuf::from(read_path_file(&path_file).trim());

    std::fs::write(wt_path.join("feature.txt"), "strategy test").unwrap();
    Command::new("git")
        .args(["add", "."])
        .current_dir(&wt_path)
        .output()
        .unwrap();
    Command::new("git")
        .args(["commit", "-m", "Strategy test"])
        .current_dir(&wt_path)
        .output()
        .unwrap();

    // Run merge --dry-run with explicit strategy
    let output = Command::new(wt_binary())
        .args(["merge", "--dry-run", "--strategy", "rebase"])
        .current_dir(&wt_path)
        .env("HOME", &home)
        .output()
        .expect("wt merge --dry-run --strategy rebase failed");

    assert!(output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("[dry-run]"),
        "Expected [dry-run] in output, got: {stderr}"
    );
    assert!(
        stderr.contains("Rebase"),
        "Expected strategy name in output, got: {stderr}"
    );
}

#[test]
fn test_merge_dry_run_on_trunk_fails() {
    let dir = tempfile::tempdir().unwrap();
    setup_git_repo(dir.path());

    let output = Command::new(wt_binary())
        .args(["merge", "--dry-run"])
        .current_dir(dir.path())
        .output()
        .expect("wt merge --dry-run failed");

    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("trunk") || stderr.contains("itself"));
}
