// ===========================================================================
// git/ops - Git 执行操作
// ===========================================================================

use std::path::Path;
use std::process::Command;

use super::{run, run_extract, Result};

/// Run git merge
pub fn merge(branch: &str, squash: bool, no_ff: bool, message: Option<&str>) -> Result<()> {
    let mut args = vec!["merge"];
    if squash {
        args.push("--squash");
    }
    if no_ff {
        args.push("--no-ff");
    }
    if let Some(msg) = message {
        args.push("-m");
        args.push(msg);
    }
    args.push(branch);
    run_extract(&args)
}

/// Run git rebase
pub fn rebase(onto: &str) -> Result<()> {
    run(&["rebase", onto])
}

/// Checkout a branch
pub fn checkout(branch: &str) -> Result<()> {
    run(&["checkout", branch])
}

/// Commit staged changes
pub fn commit(message: &str) -> Result<()> {
    run_extract(&["commit", "-m", message])
}

/// Fetch updates from remote
pub fn fetch() -> Result<()> {
    let output = Command::new("git").args(["fetch", "--quiet"]).output()?;

    if !output.status.success() {
        // Fetch failing is often not critical, just warn
    }

    Ok(())
}

/// Abort an in-progress rebase
pub fn rebase_abort() -> Result<()> {
    run(&["rebase", "--abort"])
}

/// Continue an in-progress rebase
pub fn rebase_continue() -> Result<()> {
    run(&["rebase", "--continue"])
}

/// Abort an in-progress merge
pub fn merge_abort() -> Result<()> {
    run(&["merge", "--abort"])
}

/// Reset index to HEAD, clearing any merge/squash conflict state.
///
/// Unlike `merge --abort`, this also works for `--squash` conflicts
/// which don't create MERGE_HEAD.
pub fn reset_merge() -> Result<()> {
    run(&["reset", "--merge"])
}

/// Continue an in-progress merge (after conflict resolution)
pub fn merge_continue() -> Result<()> {
    run_extract(&["commit", "--no-edit"])
}

/// Check if a rebase is in progress
pub fn is_rebase_in_progress() -> bool {
    let git_dir = Command::new("git")
        .args(["rev-parse", "--git-dir"])
        .output()
        .ok()
        .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string());

    if let Some(dir) = git_dir {
        let rebase_merge = Path::new(&dir).join("rebase-merge");
        let rebase_apply = Path::new(&dir).join("rebase-apply");
        return rebase_merge.exists() || rebase_apply.exists();
    }

    false
}

/// Check if a merge is in progress
pub fn is_merge_in_progress() -> bool {
    let git_dir = Command::new("git")
        .args(["rev-parse", "--git-dir"])
        .output()
        .ok()
        .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string());

    if let Some(dir) = git_dir {
        let merge_head = Path::new(&dir).join("MERGE_HEAD");
        return merge_head.exists();
    }

    false
}
