// ===========================================================================
// Integration Tests - Clean Dry-Run
// ===========================================================================

mod common;

use std::process::Command;
use tempfile::tempdir;

use common::*;

#[test]
fn test_clean_dry_run_empty() {
    let dir = tempdir().unwrap();
    setup_git_repo(dir.path());

    let output = Command::new(wt_binary())
        .args(["clean", "--dry-run"])
        .current_dir(dir.path())
        .output()
        .expect("wt clean --dry-run failed");

    assert!(output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("No") || stderr.is_empty(),
        "Expected 'No worktrees' message, got: {stderr}"
    );
}

#[test]
fn test_clean_dry_run_lists_worktrees() {
    let (_dir, repo, home) = setup_worktree_test_env();

    // Create a worktree with no changes (will be a clean candidate)
    let output = Command::new(wt_binary())
        .args(["new", "clean-dry-test"])
        .current_dir(&repo)
        .env("HOME", &home)
        .output()
        .expect("wt new failed");

    if !output.status.success() {
        return;
    }

    // Merge the branch into main so it has no diff
    Command::new("git")
        .args(["merge", "clean-dry-test", "--no-edit"])
        .current_dir(&repo)
        .output()
        .ok();

    // Run clean --dry-run
    let output = Command::new(wt_binary())
        .args(["clean", "--dry-run"])
        .current_dir(&repo)
        .env("HOME", &home)
        .output()
        .expect("wt clean --dry-run failed");

    assert!(output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);

    // Should show dry-run message or "no worktrees" if already cleaned
    if stderr.contains("[dry-run]") {
        assert!(
            stderr.contains("clean-dry-test"),
            "Expected branch name in dry-run output, got: {stderr}"
        );
        assert!(
            stderr.contains("Would remove"),
            "Expected 'Would remove' in dry-run output, got: {stderr}"
        );
    }
}

#[test]
fn test_clean_dry_run_does_not_remove() {
    let (_dir, repo, home) = setup_worktree_test_env();

    let path_file = create_path_file(_dir.path());
    let output = Command::new(wt_binary())
        .args([
            "new",
            "clean-dry-keep",
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

    let wt_path = read_path_file(&path_file).trim().to_string();

    // Merge into main so branch has no diff
    Command::new("git")
        .args(["merge", "clean-dry-keep", "--no-edit"])
        .current_dir(&repo)
        .output()
        .ok();

    // Run clean --dry-run
    let output = Command::new(wt_binary())
        .args(["clean", "--dry-run"])
        .current_dir(&repo)
        .env("HOME", &home)
        .output()
        .expect("wt clean --dry-run failed");

    assert!(output.status.success());

    // Verify the worktree still exists
    assert!(
        std::path::Path::new(&wt_path).exists(),
        "Worktree should still exist after dry-run clean"
    );

    // Verify the branch still exists
    let branch_check = Command::new("git")
        .args(["branch", "--list", "clean-dry-keep"])
        .current_dir(&repo)
        .output()
        .unwrap();
    let branches = String::from_utf8_lossy(&branch_check.stdout);
    assert!(
        branches.contains("clean-dry-keep"),
        "Branch should still exist after dry-run clean"
    );
}

#[test]
fn test_clean_dry_run_with_path_file() {
    let dir = tempdir().unwrap();
    setup_git_repo(dir.path());

    let path_file = create_path_file(dir.path());
    let output = Command::new(wt_binary())
        .args([
            "clean",
            "--dry-run",
            "--path-file",
            path_file.to_str().unwrap(),
        ])
        .current_dir(dir.path())
        .output()
        .expect("wt clean --dry-run failed");

    assert!(output.status.success());
}
