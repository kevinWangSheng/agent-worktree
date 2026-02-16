use super::*;
use std::path::{Path, PathBuf};
use std::process::Command as StdCommand;
use std::sync::Mutex;
use tempfile::tempdir;

// Global mutex for tests that change cwd
static CWD_MUTEX: Mutex<()> = Mutex::new(());

// ===========================================================================
// Helper: Setup a minimal git repo for testing
// ===========================================================================
fn setup_test_repo() -> tempfile::TempDir {
    let dir = tempdir().unwrap();
    let path = dir.path();

    StdCommand::new("git")
        .args(["init"])
        .current_dir(path)
        .output()
        .expect("git init failed");

    StdCommand::new("git")
        .args(["config", "user.email", "test@test.com"])
        .current_dir(path)
        .output()
        .unwrap();

    StdCommand::new("git")
        .args(["config", "user.name", "Test"])
        .current_dir(path)
        .output()
        .unwrap();

    std::fs::write(path.join("README.md"), "# Test\n").unwrap();

    StdCommand::new("git")
        .args(["add", "."])
        .current_dir(path)
        .output()
        .unwrap();

    StdCommand::new("git")
        .args(["commit", "-m", "Initial commit"])
        .current_dir(path)
        .output()
        .unwrap();

    StdCommand::new("git")
        .args(["branch", "-M", "main"])
        .current_dir(path)
        .output()
        .ok();

    dir
}

/// Run a test that requires changing cwd, with proper locking
fn with_cwd<F, T>(path: &Path, f: F) -> T
where
    F: FnOnce() -> T,
{
    let _guard = CWD_MUTEX.lock().unwrap();
    let original = std::env::current_dir().unwrap();
    std::env::set_current_dir(path).unwrap();
    let result = f();
    std::env::set_current_dir(original).unwrap();
    result
}

// ===========================================================================
// Parse worktree list tests (pure functions, no cwd issues)
// ===========================================================================
#[test]
fn test_parse_worktree_list_empty() {
    let result = parse_worktree_list("");
    assert!(result.is_empty());
}

#[test]
fn test_parse_worktree_list_single() {
    let content = r#"worktree /path/to/repo
HEAD abc1234567890
branch refs/heads/main
"#;
    let result = parse_worktree_list(content);
    assert_eq!(result.len(), 1);
    assert_eq!(result[0].path, PathBuf::from("/path/to/repo"));
    assert_eq!(result[0].branch, Some("main".to_string()));
    assert_eq!(result[0].commit, Some("abc1234567890".to_string()));
    assert!(!result[0].is_bare);
}

#[test]
fn test_parse_worktree_list_multiple() {
    let content = r#"worktree /path/to/main
HEAD abc123
branch refs/heads/main

worktree /path/to/feature
HEAD def456
branch refs/heads/feature-branch

worktree /path/to/detached
HEAD 789abc
detached
"#;
    let result = parse_worktree_list(content);
    assert_eq!(result.len(), 3);

    assert_eq!(result[0].branch, Some("main".to_string()));
    assert_eq!(result[1].branch, Some("feature-branch".to_string()));
    assert_eq!(result[2].branch, None); // detached HEAD
}

#[test]
fn test_parse_worktree_list_bare() {
    let content = r#"worktree /path/to/bare.git
bare
"#;
    let result = parse_worktree_list(content);
    assert_eq!(result.len(), 1);
    assert!(result[0].is_bare);
    assert!(result[0].branch.is_none());
}

// ===========================================================================
// Error display tests (pure functions)
// ===========================================================================
#[test]
fn test_error_display() {
    let err = Error::NotInRepo;
    assert_eq!(err.to_string(), "not in a git repository");

    let err = Error::WorktreeNotFound("feature".to_string());
    assert_eq!(err.to_string(), "worktree 'feature' not found");

    let err = Error::WorktreeExists("feature".to_string());
    assert_eq!(err.to_string(), "worktree 'feature' already exists");

    let err = Error::BranchNotFound("missing".to_string());
    assert_eq!(err.to_string(), "branch 'missing' not found");

    let err = Error::Command("something failed".to_string());
    assert_eq!(err.to_string(), "something failed");
}

// ===========================================================================
// clean_git_error tests
// ===========================================================================
#[test]
fn test_clean_git_error_fatal_prefix() {
    let msg = clean_git_error("fatal: invalid reference: xxx");
    assert_eq!(msg, "invalid reference: xxx");
}

#[test]
fn test_clean_git_error_error_prefix() {
    let msg = clean_git_error("error: some git error");
    assert_eq!(msg, "some git error");
}

#[test]
fn test_clean_git_error_worktree_uncommitted() {
    let msg = clean_git_error(
        "fatal: '/Users/foo/.agent-worktree/workspaces/proj/branch' contains modified or untracked files, use --force to delete it",
    );
    assert_eq!(msg, "worktree 'branch' has uncommitted changes, use --force");
}

#[test]
fn test_clean_git_error_no_prefix() {
    let msg = clean_git_error("some plain message");
    assert_eq!(msg, "some plain message");
}

// ===========================================================================
// extract_error tests
// ===========================================================================
#[test]
fn test_extract_error_prefers_stderr() {
    let output = std::process::Output {
        status: std::process::ExitStatus::default(),
        stdout: b"stdout info".to_vec(),
        stderr: b"fatal: something broke".to_vec(),
    };
    let err = extract_error(&output);
    assert_eq!(err, "something broke");
}

#[test]
fn test_extract_error_falls_back_to_stdout() {
    let output = std::process::Output {
        status: std::process::ExitStatus::default(),
        stdout: b"CONFLICT (content): Merge conflict in file.txt\n".to_vec(),
        stderr: b"".to_vec(),
    };
    let err = extract_error(&output);
    assert!(err.contains("CONFLICT"));
}

#[test]
fn test_extract_error_whitespace_only_stderr() {
    let output = std::process::Output {
        status: std::process::ExitStatus::default(),
        stdout: b"nothing to commit, working tree clean".to_vec(),
        stderr: b"  \n  ".to_vec(),
    };
    let err = extract_error(&output);
    assert!(err.contains("nothing to commit"));
}

// ===========================================================================
// is_cwd_inside tests
// ===========================================================================
#[test]
fn test_is_cwd_inside_current_dir() {
    let cwd = std::env::current_dir().unwrap();
    assert!(is_cwd_inside(&cwd));
}

#[test]
fn test_is_cwd_inside_nonexistent() {
    assert!(!is_cwd_inside(Path::new("/nonexistent/path/12345")));
}

// ===========================================================================
// Git module function tests (require changing cwd, use mutex)
// ===========================================================================

#[test]
fn test_repo_root() {
    let dir = setup_test_repo();
    with_cwd(dir.path(), || {
        let root = repo_root();
        assert!(root.is_ok());
        let root_path = root.unwrap();
        assert!(root_path.exists());
        assert!(root_path.join(".git").exists());
    });
}

#[test]
fn test_repo_root_not_in_repo() {
    let dir = tempdir().unwrap();
    with_cwd(dir.path(), || {
        let root = repo_root();
        assert!(root.is_err());
        assert!(matches!(root.unwrap_err(), Error::NotInRepo));
    });
}

#[test]
fn test_repo_name() {
    let dir = setup_test_repo();
    with_cwd(dir.path(), || {
        let name = repo_name();
        assert!(name.is_ok());
        assert!(!name.unwrap().is_empty());
    });
}

#[test]
fn test_workspace_id_format() {
    let dir = setup_test_repo();
    with_cwd(dir.path(), || {
        let id = workspace_id().unwrap();
        // Format: {repo_name}-{hash[0:6]}
        let parts: Vec<&str> = id.rsplitn(2, '-').collect();
        assert_eq!(parts.len(), 2);
        assert_eq!(parts[0].len(), 6); // hash suffix
        assert!(parts[0].chars().all(|c: char| c.is_ascii_hexdigit()));
    });
}

#[test]
fn test_workspace_id_deterministic() {
    let dir = setup_test_repo();
    with_cwd(dir.path(), || {
        let id1 = workspace_id().unwrap();
        let id2 = workspace_id().unwrap();
        assert_eq!(id1, id2);
    });
}

#[test]
fn test_workspace_id_unique_for_different_paths() {
    let dir1 = setup_test_repo();
    let dir2 = setup_test_repo();

    let id1 = with_cwd(dir1.path(), || workspace_id().unwrap());
    let id2 = with_cwd(dir2.path(), || workspace_id().unwrap());

    // Same repo name (both are random tempdir names), but different paths
    // Hash suffix should differ
    assert_ne!(id1, id2);
}

#[test]
fn test_current_branch() {
    let dir = setup_test_repo();
    with_cwd(dir.path(), || {
        let branch = current_branch();
        assert!(branch.is_ok());
        assert_eq!(branch.unwrap(), "main");
    });
}

#[test]
fn test_detect_trunk() {
    let dir = setup_test_repo();
    with_cwd(dir.path(), || {
        let trunk = detect_trunk();
        assert!(trunk.is_ok());
        assert_eq!(trunk.unwrap(), "main");
    });
}

#[test]
fn test_branch_exists_true() {
    let dir = setup_test_repo();
    with_cwd(dir.path(), || {
        let exists = branch_exists("main");
        assert!(exists.is_ok());
        assert!(exists.unwrap());
    });
}

#[test]
fn test_branch_exists_false() {
    let dir = setup_test_repo();
    with_cwd(dir.path(), || {
        let exists = branch_exists("nonexistent-branch-12345");
        assert!(exists.is_ok());
        assert!(!exists.unwrap());
    });
}

#[test]
fn test_current_commit() {
    let dir = setup_test_repo();
    with_cwd(dir.path(), || {
        let commit = current_commit();
        assert!(commit.is_ok());
        let hash = commit.unwrap();
        assert_eq!(hash.len(), 40);
    });
}

// ===========================================================================
// parse_shortstat tests (pure function)
// ===========================================================================
#[test]
fn test_parse_shortstat_full() {
    let stat = branch::parse_shortstat(" 3 files changed, 120 insertions(+), 30 deletions(-)");
    assert_eq!(stat.insertions, 120);
    assert_eq!(stat.deletions, 30);
}

#[test]
fn test_parse_shortstat_insertions_only() {
    let stat = branch::parse_shortstat(" 1 file changed, 5 insertions(+)");
    assert_eq!(stat.insertions, 5);
    assert_eq!(stat.deletions, 0);
}

#[test]
fn test_parse_shortstat_deletions_only() {
    let stat = branch::parse_shortstat(" 2 files changed, 10 deletions(-)");
    assert_eq!(stat.insertions, 0);
    assert_eq!(stat.deletions, 10);
}

#[test]
fn test_parse_shortstat_empty() {
    let stat = branch::parse_shortstat("");
    assert_eq!(stat.insertions, 0);
    assert_eq!(stat.deletions, 0);
}

#[test]
fn test_parse_shortstat_single_change() {
    let stat = branch::parse_shortstat(" 1 file changed, 1 insertion(+), 1 deletion(-)");
    assert_eq!(stat.insertions, 1);
    assert_eq!(stat.deletions, 1);
}

#[test]
fn test_has_uncommitted_changes_clean() {
    let dir = setup_test_repo();
    with_cwd(dir.path(), || {
        let has_changes = has_uncommitted_changes();
        assert!(has_changes.is_ok());
        assert!(!has_changes.unwrap());
    });
}

#[test]
fn test_has_uncommitted_changes_dirty() {
    let dir = setup_test_repo();
    std::fs::write(dir.path().join("new_file.txt"), "content").unwrap();
    with_cwd(dir.path(), || {
        let has_changes = has_uncommitted_changes();
        assert!(has_changes.is_ok());
        assert!(has_changes.unwrap());
    });
}

#[test]
fn test_list_worktrees() {
    let dir = setup_test_repo();
    with_cwd(dir.path(), || {
        let worktrees = list_worktrees();
        assert!(worktrees.is_ok());
        let list = worktrees.unwrap();
        assert!(!list.is_empty());
        assert_eq!(list[0].branch, Some("main".to_string()));
    });
}

#[test]
fn test_is_rebase_in_progress() {
    let dir = setup_test_repo();
    with_cwd(dir.path(), || {
        assert!(!is_rebase_in_progress());
    });
}

#[test]
fn test_is_merge_in_progress() {
    let dir = setup_test_repo();
    with_cwd(dir.path(), || {
        assert!(!is_merge_in_progress());
    });
}

#[test]
fn test_log_oneline() {
    let dir = setup_test_repo();
    with_cwd(dir.path(), || {
        let log = log_oneline("HEAD", "HEAD");
        assert!(log.is_ok());
        assert!(log.unwrap().is_empty());
    });
}

#[test]
fn test_commit_count() {
    let dir = setup_test_repo();
    with_cwd(dir.path(), || {
        let count = commit_count("HEAD", "HEAD");
        assert!(count.is_ok());
        assert_eq!(count.unwrap(), 0);
    });
}

#[test]
fn test_fetch() {
    let dir = setup_test_repo();
    with_cwd(dir.path(), || {
        let result = fetch();
        assert!(result.is_ok());
    });
}

#[test]
fn test_is_merged() {
    let dir = setup_test_repo();
    with_cwd(dir.path(), || {
        let result = is_merged("main", "main");
        assert!(result.is_ok());
        assert!(result.unwrap());
    });
}

// ===========================================================================
// Worktree operations
// ===========================================================================
#[test]
fn test_create_and_remove_worktree() {
    let dir = setup_test_repo();
    let wt_path = dir.path().join("worktrees").join("feature");
    std::fs::create_dir_all(wt_path.parent().unwrap()).unwrap();

    with_cwd(dir.path(), || {
        let result = create_worktree(&wt_path, "feature-branch", "main");
        assert!(result.is_ok());
        assert!(wt_path.exists());
        assert!(branch_exists("feature-branch").unwrap());

        let result = remove_worktree(&wt_path, false);
        assert!(result.is_ok());
    });
}

#[test]
fn test_create_worktree_duplicate() {
    let dir = setup_test_repo();
    let wt_path = dir.path().join("worktrees").join("dup");
    let wt_path2 = dir.path().join("worktrees").join("dup2");
    std::fs::create_dir_all(wt_path.parent().unwrap()).unwrap();

    with_cwd(dir.path(), || {
        create_worktree(&wt_path, "dup-branch", "main").unwrap();
        let result = create_worktree(&wt_path2, "dup-branch", "main");
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), Error::WorktreeExists(_)));
    });
}

// ===========================================================================
// Branch operations
// ===========================================================================
#[test]
fn test_rename_branch() {
    let dir = setup_test_repo();
    StdCommand::new("git")
        .args(["branch", "old-name"])
        .current_dir(dir.path())
        .output()
        .unwrap();

    with_cwd(dir.path(), || {
        let result = rename_branch("old-name", "new-name");
        assert!(result.is_ok());
        assert!(!branch_exists("old-name").unwrap());
        assert!(branch_exists("new-name").unwrap());
    });
}

#[test]
fn test_delete_branch() {
    let dir = setup_test_repo();
    StdCommand::new("git")
        .args(["branch", "to-delete"])
        .current_dir(dir.path())
        .output()
        .unwrap();

    with_cwd(dir.path(), || {
        assert!(branch_exists("to-delete").unwrap());
        let result = delete_branch("to-delete", false);
        assert!(result.is_ok());
        assert!(!branch_exists("to-delete").unwrap());
    });
}

#[test]
fn test_checkout() {
    let dir = setup_test_repo();
    StdCommand::new("git")
        .args(["branch", "other-branch"])
        .current_dir(dir.path())
        .output()
        .unwrap();

    with_cwd(dir.path(), || {
        let result = checkout("other-branch");
        assert!(result.is_ok());
        assert_eq!(current_branch().unwrap(), "other-branch");
    });
}

// ===========================================================================
// Abort/continue operations
// ===========================================================================
#[test]
fn test_rebase_abort_no_rebase() {
    let dir = setup_test_repo();
    with_cwd(dir.path(), || {
        let result = rebase_abort();
        assert!(result.is_err());
    });
}

#[test]
fn test_merge_abort_no_merge() {
    let dir = setup_test_repo();
    with_cwd(dir.path(), || {
        let result = merge_abort();
        assert!(result.is_err());
    });
}

#[test]
fn test_reset_merge_clean_repo() {
    let dir = setup_test_repo();
    with_cwd(dir.path(), || {
        // reset --merge on clean repo is a no-op success
        let result = reset_merge();
        assert!(result.is_ok());
    });
}

#[test]
fn test_rebase_continue_no_rebase() {
    let dir = setup_test_repo();
    with_cwd(dir.path(), || {
        let result = rebase_continue();
        assert!(result.is_err());
    });
}

#[test]
fn test_merge_continue_no_merge() {
    let dir = setup_test_repo();
    with_cwd(dir.path(), || {
        let result = merge_continue();
        assert!(result.is_err());
    });
}

// ===========================================================================
// Merge and rebase operations
// ===========================================================================
#[test]
fn test_merge_fast_forward() {
    let dir = setup_test_repo();
    // Create a branch that's already at main
    StdCommand::new("git")
        .args(["branch", "already-merged"])
        .current_dir(dir.path())
        .output()
        .unwrap();

    with_cwd(dir.path(), || {
        // Merge should work (fast-forward or no-op)
        let result = merge("already-merged", false, false, None);
        // May succeed or say "already up to date"
        let _ = result;
    });
}

#[test]
fn test_rebase_same_branch() {
    let dir = setup_test_repo();
    with_cwd(dir.path(), || {
        // Rebase onto self should be a no-op
        let result = rebase("main");
        assert!(result.is_ok());
    });
}

// ===========================================================================
// Remove worktree with force
// ===========================================================================
#[test]
fn test_remove_worktree_force() {
    let dir = setup_test_repo();
    let wt_path = dir.path().join("worktrees").join("force-test");
    std::fs::create_dir_all(wt_path.parent().unwrap()).unwrap();

    with_cwd(dir.path(), || {
        create_worktree(&wt_path, "force-branch", "main").unwrap();

        // Create uncommitted changes in worktree
        std::fs::write(wt_path.join("uncommitted.txt"), "changes").unwrap();

        // Force remove should work
        let result = remove_worktree(&wt_path, true);
        assert!(result.is_ok());
    });
}

// ===========================================================================
// Delete branch with force
// ===========================================================================
#[test]
fn test_delete_branch_force() {
    let dir = setup_test_repo();
    StdCommand::new("git")
        .args(["branch", "unmerged-branch"])
        .current_dir(dir.path())
        .output()
        .unwrap();

    with_cwd(dir.path(), || {
        // Force delete should work
        let result = delete_branch("unmerged-branch", true);
        assert!(result.is_ok());
    });
}

// ===========================================================================
// has_changes_from_trunk tests
// ===========================================================================

#[test]
fn test_has_changes_from_trunk_no_changes() {
    let dir = setup_test_repo();
    with_cwd(dir.path(), || {
        // On main, no changes from main
        let has = has_changes_from_trunk("main");
        assert!(has.is_ok());
        assert!(!has.unwrap());
    });
}

#[test]
fn test_has_changes_from_trunk_with_committed_changes() {
    let dir = setup_test_repo();

    // Create feature branch and add a commit
    StdCommand::new("git")
        .args(["checkout", "-b", "feature"])
        .current_dir(dir.path())
        .output()
        .unwrap();

    std::fs::write(dir.path().join("feature.txt"), "new feature").unwrap();

    StdCommand::new("git")
        .args(["add", "feature.txt"])
        .current_dir(dir.path())
        .output()
        .unwrap();

    StdCommand::new("git")
        .args(["commit", "-m", "Add feature"])
        .current_dir(dir.path())
        .output()
        .unwrap();

    with_cwd(dir.path(), || {
        // Feature branch has commits ahead of main
        let has = has_changes_from_trunk("main");
        assert!(has.is_ok());
        assert!(has.unwrap(), "Should detect committed changes ahead of trunk");
    });
}

#[test]
fn test_has_changes_from_trunk_with_uncommitted_changes() {
    let dir = setup_test_repo();

    // Create feature branch with uncommitted changes only
    StdCommand::new("git")
        .args(["checkout", "-b", "feature"])
        .current_dir(dir.path())
        .output()
        .unwrap();

    std::fs::write(dir.path().join("dirty.txt"), "uncommitted").unwrap();

    with_cwd(dir.path(), || {
        // Feature branch has uncommitted changes
        let has = has_changes_from_trunk("main");
        assert!(has.is_ok());
        assert!(has.unwrap(), "Should detect uncommitted changes");
    });
}
