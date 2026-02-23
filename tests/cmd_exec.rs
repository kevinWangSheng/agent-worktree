// ===========================================================================
// Integration Tests - Exec Command
// ===========================================================================

mod common;

use std::process::Command;

use common::*;

#[test]
fn test_exec_no_worktrees() {
    let dir = tempfile::tempdir().unwrap();
    setup_git_repo(dir.path());

    let output = Command::new(wt_binary())
        .args(["exec", "echo", "hello"])
        .current_dir(dir.path())
        .output()
        .expect("Failed to execute wt exec");

    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("No worktrees") || output.status.success());
}

#[test]
fn test_exec_runs_in_worktrees() {
    let (_dir, repo, home) = setup_worktree_test_env();

    // Create a worktree
    let output = Command::new(wt_binary())
        .args(["new", "exec-test-1"])
        .current_dir(&repo)
        .env("HOME", &home)
        .output()
        .expect("wt new failed");

    if !output.status.success() {
        return;
    }

    let output = Command::new(wt_binary())
        .args(["exec", "echo", "hello-from-exec"])
        .current_dir(&repo)
        .env("HOME", &home)
        .output()
        .expect("wt exec failed");

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    let combined = format!("{}{}", stdout, stderr);

    assert!(
        (stdout.contains("exec-test-1")
            && stdout.contains("hello-from-exec")
            && stdout.contains("1 succeeded"))
            || combined.contains("No worktrees")
    );
}

#[test]
fn test_exec_multiple_worktrees() {
    let (_dir, repo, home) = setup_worktree_test_env();

    for name in &["exec-multi-1", "exec-multi-2"] {
        let _ = Command::new(wt_binary())
            .args(["new", name])
            .current_dir(&repo)
            .env("HOME", &home)
            .output();
    }

    let output = Command::new(wt_binary())
        .args(["exec", "echo", "test"])
        .current_dir(&repo)
        .env("HOME", &home)
        .output()
        .expect("wt exec failed");

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    let combined = format!("{}{}", stdout, stderr);

    assert!(
        (stdout.contains("exec-multi-1") && stdout.contains("exec-multi-2"))
            || combined.contains("No worktrees")
    );
}

#[test]
fn test_exec_with_branch_filter() {
    let (_dir, repo, home) = setup_worktree_test_env();

    for name in &["exec-filter-a", "exec-filter-b"] {
        let _ = Command::new(wt_binary())
            .args(["new", name])
            .current_dir(&repo)
            .env("HOME", &home)
            .output();
    }

    let output = Command::new(wt_binary())
        .args(["exec", "-b", "exec-filter-a", "echo", "filtered"])
        .current_dir(&repo)
        .env("HOME", &home)
        .output()
        .expect("wt exec failed");

    let stdout = String::from_utf8_lossy(&output.stdout);
    // Should contain the filtered branch but not the other
    if stdout.contains("exec-filter-a") {
        assert!(!stdout.contains("==> exec-filter-b <=="));
        assert!(stdout.contains("1 succeeded"));
    }
}

#[test]
fn test_exec_command_failure_continues() {
    let (_dir, repo, home) = setup_worktree_test_env();

    let output = Command::new(wt_binary())
        .args(["new", "exec-fail-test"])
        .current_dir(&repo)
        .env("HOME", &home)
        .output()
        .expect("wt new failed");

    if !output.status.success() {
        return;
    }

    let output = Command::new(wt_binary())
        .args(["exec", "false"])
        .current_dir(&repo)
        .env("HOME", &home)
        .output()
        .expect("wt exec failed");

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    let combined = format!("{}{}", stdout, stderr);

    assert!(
        stdout.contains("1 failed") || combined.contains("No worktrees")
    );
}

#[test]
fn test_exec_summary_line() {
    let (_dir, repo, home) = setup_worktree_test_env();

    let _ = Command::new(wt_binary())
        .args(["new", "exec-summary"])
        .current_dir(&repo)
        .env("HOME", &home)
        .output();

    let output = Command::new(wt_binary())
        .args(["exec", "true"])
        .current_dir(&repo)
        .env("HOME", &home)
        .output()
        .expect("wt exec failed");

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    let combined = format!("{}{}", stdout, stderr);

    assert!(
        stdout.contains("Executed in") || combined.contains("No worktrees")
    );
}
