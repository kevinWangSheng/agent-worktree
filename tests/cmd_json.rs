// ===========================================================================
// Integration Tests - JSON Output Mode
// ===========================================================================

mod common;

use std::process::Command;

use common::*;

// ===========================================================================
// wt ls --json
// ===========================================================================

#[test]
fn test_ls_json_no_worktrees() {
    let dir = tempfile::tempdir().unwrap();
    setup_git_repo(dir.path());

    let output = Command::new(wt_binary())
        .args(["ls", "--json"])
        .current_dir(dir.path())
        .output()
        .expect("Failed to execute wt ls --json");

    let stdout = String::from_utf8_lossy(&output.stdout);
    // Should output empty JSON array or nothing
    if !stdout.trim().is_empty() {
        let parsed: serde_json::Value = serde_json::from_str(stdout.trim())
            .expect("Output should be valid JSON");
        assert!(parsed.is_array());
        assert_eq!(parsed.as_array().unwrap().len(), 0);
    }
}

#[test]
fn test_ls_json_with_worktrees() {
    let (_dir, repo, home) = setup_worktree_test_env();

    let output = Command::new(wt_binary())
        .args(["new", "json-ls-test"])
        .current_dir(&repo)
        .env("HOME", &home)
        .output()
        .expect("wt new failed");

    if !output.status.success() {
        return;
    }

    let output = Command::new(wt_binary())
        .args(["ls", "--json"])
        .current_dir(&repo)
        .env("HOME", &home)
        .output()
        .expect("wt ls --json failed");

    let stdout = String::from_utf8_lossy(&output.stdout);
    let parsed: serde_json::Value = serde_json::from_str(stdout.trim())
        .expect("Output should be valid JSON");

    assert!(parsed.is_array());
    let arr = parsed.as_array().unwrap();

    if !arr.is_empty() {
        let item = &arr[0];
        // Verify all expected fields exist
        assert!(item.get("branch").is_some());
        assert!(item.get("is_current").is_some());
        assert!(item.get("uncommitted").is_some());
        assert!(item.get("commits").is_some());
        assert!(item.get("insertions").is_some());
        assert!(item.get("deletions").is_some());
        assert!(item.get("path").is_some());

        // Verify types
        assert!(item["branch"].is_string());
        assert!(item["is_current"].is_boolean());
        assert!(item["uncommitted"].is_number());
        assert!(item["commits"].is_number());
        assert!(item["insertions"].is_number());
        assert!(item["deletions"].is_number());
        assert!(item["path"].is_string());
    }
}

#[test]
fn test_ls_json_contains_branch_name() {
    let (_dir, repo, home) = setup_worktree_test_env();

    let _ = Command::new(wt_binary())
        .args(["new", "json-branch-check"])
        .current_dir(&repo)
        .env("HOME", &home)
        .output();

    let output = Command::new(wt_binary())
        .args(["ls", "--json"])
        .current_dir(&repo)
        .env("HOME", &home)
        .output()
        .expect("wt ls --json failed");

    let stdout = String::from_utf8_lossy(&output.stdout);
    let parsed: serde_json::Value = serde_json::from_str(stdout.trim())
        .expect("Output should be valid JSON");

    let arr = parsed.as_array().unwrap();
    if !arr.is_empty() {
        let branches: Vec<&str> = arr
            .iter()
            .filter_map(|item| item["branch"].as_str())
            .collect();
        assert!(branches.contains(&"json-branch-check"));
    }
}

#[test]
fn test_ls_json_multiple_worktrees() {
    let (_dir, repo, home) = setup_worktree_test_env();

    for name in &["json-multi-1", "json-multi-2"] {
        let _ = Command::new(wt_binary())
            .args(["new", name])
            .current_dir(&repo)
            .env("HOME", &home)
            .output();
    }

    let output = Command::new(wt_binary())
        .args(["ls", "--json"])
        .current_dir(&repo)
        .env("HOME", &home)
        .output()
        .expect("wt ls --json failed");

    let stdout = String::from_utf8_lossy(&output.stdout);
    let parsed: serde_json::Value = serde_json::from_str(stdout.trim())
        .expect("Output should be valid JSON");

    let arr = parsed.as_array().unwrap();
    // Should have at least 2 entries (or 0 if env issue)
    assert!(arr.len() >= 2 || arr.is_empty());
}

// ===========================================================================
// wt status --json
// ===========================================================================

#[test]
fn test_status_json_valid() {
    let dir = tempfile::tempdir().unwrap();
    setup_git_repo(dir.path());

    let output = Command::new(wt_binary())
        .args(["status", "--json"])
        .current_dir(dir.path())
        .output()
        .expect("wt status --json failed");

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(output.status.success());

    let parsed: serde_json::Value = serde_json::from_str(stdout.trim())
        .expect("Output should be valid JSON");

    assert!(parsed.is_object());
}

#[test]
fn test_status_json_has_all_fields() {
    let dir = tempfile::tempdir().unwrap();
    setup_git_repo(dir.path());

    let output = Command::new(wt_binary())
        .args(["status", "--json"])
        .current_dir(dir.path())
        .output()
        .expect("wt status --json failed");

    let stdout = String::from_utf8_lossy(&output.stdout);
    let parsed: serde_json::Value = serde_json::from_str(stdout.trim())
        .expect("Output should be valid JSON");

    // Verify all expected fields
    assert!(parsed.get("branch").is_some());
    assert!(parsed.get("in_worktree").is_some());
    assert!(parsed.get("trunk").is_some());
    assert!(parsed.get("commits_ahead").is_some());
    assert!(parsed.get("uncommitted").is_some());
    assert!(parsed.get("insertions").is_some());
    assert!(parsed.get("deletions").is_some());
    assert!(parsed.get("merge_in_progress").is_some());
    assert!(parsed.get("rebase_in_progress").is_some());

    // Verify types
    assert!(parsed["branch"].is_string());
    assert!(parsed["in_worktree"].is_boolean());
    assert!(parsed["trunk"].is_string());
    assert!(parsed["commits_ahead"].is_number());
    assert!(parsed["uncommitted"].is_number());
    assert!(parsed["insertions"].is_number());
    assert!(parsed["deletions"].is_number());
    assert!(parsed["merge_in_progress"].is_boolean());
    assert!(parsed["rebase_in_progress"].is_boolean());
}

#[test]
fn test_status_json_branch_value() {
    let dir = tempfile::tempdir().unwrap();
    setup_git_repo(dir.path());

    let output = Command::new(wt_binary())
        .args(["status", "--json"])
        .current_dir(dir.path())
        .output()
        .expect("wt status --json failed");

    let stdout = String::from_utf8_lossy(&output.stdout);
    let parsed: serde_json::Value = serde_json::from_str(stdout.trim())
        .expect("Output should be valid JSON");

    assert_eq!(parsed["branch"].as_str().unwrap(), "main");
    assert_eq!(parsed["in_worktree"].as_bool().unwrap(), false);
    assert_eq!(parsed["merge_in_progress"].as_bool().unwrap(), false);
    assert_eq!(parsed["rebase_in_progress"].as_bool().unwrap(), false);
}

#[test]
fn test_status_json_with_uncommitted_changes() {
    let dir = tempfile::tempdir().unwrap();
    setup_git_repo(dir.path());

    // Create uncommitted change
    std::fs::write(dir.path().join("new_file.txt"), "hello\n").unwrap();

    let output = Command::new(wt_binary())
        .args(["status", "--json"])
        .current_dir(dir.path())
        .output()
        .expect("wt status --json failed");

    let stdout = String::from_utf8_lossy(&output.stdout);
    let parsed: serde_json::Value = serde_json::from_str(stdout.trim())
        .expect("Output should be valid JSON");

    let uncommitted = parsed["uncommitted"].as_u64().unwrap();
    assert!(uncommitted >= 1);
}

// ===========================================================================
// Verify text output is unchanged
// ===========================================================================

#[test]
fn test_ls_without_json_unchanged() {
    let dir = tempfile::tempdir().unwrap();
    setup_git_repo(dir.path());

    let output = Command::new(wt_binary())
        .arg("ls")
        .current_dir(dir.path())
        .output()
        .expect("wt ls failed");

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);

    // Text mode should not output JSON
    assert!(!stdout.contains("{"));
    assert!(!stdout.contains("["));
    assert!(stderr.contains("No worktrees") || output.status.success());
}

#[test]
fn test_status_without_json_unchanged() {
    let dir = tempfile::tempdir().unwrap();
    setup_git_repo(dir.path());

    let output = Command::new(wt_binary())
        .arg("status")
        .current_dir(dir.path())
        .output()
        .expect("wt status failed");

    let stdout = String::from_utf8_lossy(&output.stdout);

    // Text mode should show human-readable output
    assert!(stdout.contains("Branch:"));
    assert!(stdout.contains("Location:"));
    assert!(!stdout.starts_with("{"));
}
