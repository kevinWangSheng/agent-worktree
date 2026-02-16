// ===========================================================================
// snap/start - Snap mode loop (deprecated direct mode)
// ===========================================================================

use std::path::Path;

use crate::cli::{Error, Result};
use crate::config::Config;
use crate::git;
use crate::meta;
use crate::process;
use crate::prompt::{self, SnapExitChoice};

pub fn run_snap_mode(
    cmd: &str,
    wt_path: &Path,
    branch: &str,
    config: &Config,
    trunk: &str,
) -> Result<()> {
    eprintln!("Entering snap mode: {cmd}");
    eprintln!("Worktree: {branch}");
    eprintln!("---");

    loop {
        // Run agent
        let status =
            process::run_interactive(cmd, wt_path).map_err(|e| Error::Other(e.to_string()))?;

        if !status.success() {
            eprintln!("Agent exited abnormally. Worktree preserved.");
            return Ok(());
        }

        // Check change state
        std::env::set_current_dir(wt_path).map_err(|e| Error::Other(e.to_string()))?;
        let has_uncommitted = git::has_uncommitted_changes().unwrap_or(false);
        let has_commits_ahead = git::commit_count(trunk, "HEAD").unwrap_or(0) > 0;

        // No changes at all → cleanup
        if !has_uncommitted && !has_commits_ahead {
            eprintln!("No changes detected. Cleaning up...");
            cleanup_worktree(wt_path, branch, config)?;
            return Ok(());
        }

        // Only committed changes → auto merge (no prompt)
        if !has_uncommitted && has_commits_ahead {
            do_merge(wt_path, branch, trunk, config)?;
            return Ok(());
        }

        // Has uncommitted changes → prompt user
        match prompt::snap_exit_prompt() {
            Ok(SnapExitChoice::Reopen) => {
                eprintln!("Reopening agent...");
                continue;
            }
            Ok(SnapExitChoice::Exit) | Err(_) => {
                eprintln!();
                eprintln!("Exiting snap mode. Worktree preserved.");
                eprintln!();
                eprintln!("Your changes are safe. To continue later:");
                eprintln!("  git add . && git commit -m 'your message'");
                eprintln!("  wt merge    # merge and cleanup");
                eprintln!();
                return Ok(());
            }
        }
    }
}

fn do_merge(wt_path: &Path, branch: &str, trunk: &str, config: &Config) -> Result<()> {
    // Run pre-merge hooks
    if !config.hooks.pre_merge.is_empty() {
        eprintln!("Running pre-merge hooks...");
        process::run_hooks(&config.hooks.pre_merge, wt_path)
            .map_err(|e| Error::Other(e.to_string()))?;
    }

    eprintln!("Merging {} into {}...", branch, trunk);

    let repo_root = git::repo_root()?;
    std::env::set_current_dir(&repo_root).map_err(|e| Error::Other(e.to_string()))?;
    git::checkout(trunk)?;

    let log = git::log_oneline(trunk, branch).unwrap_or_default();
    let msg = super::super::merge::build_merge_message(branch, &log);

    match config.merge_strategy {
        crate::config::MergeStrategy::Squash => {
            git::merge(branch, true, false, None)?;
            git::commit(&msg)?;
        }
        crate::config::MergeStrategy::Merge => {
            git::merge(branch, false, true, Some(&msg))?;
        }
        crate::config::MergeStrategy::Rebase => {
            git::checkout(branch)?;
            git::rebase(trunk)?;
            git::checkout(trunk)?;
            git::merge(branch, false, false, None)?;
        }
    }

    eprintln!("Merged {} into {}", branch, trunk);

    // Run post-merge hooks
    if !config.hooks.post_merge.is_empty() {
        eprintln!("Running post-merge hooks...");
        process::run_hooks(&config.hooks.post_merge, &repo_root)
            .map_err(|e| Error::Other(e.to_string()))?;
    }

    cleanup_worktree(wt_path, branch, config)?;
    Ok(())
}

fn cleanup_worktree(wt_path: &Path, branch: &str, config: &Config) -> Result<()> {
    // Move back to main repo first
    git::remove_worktree(wt_path, true)?;
    git::delete_branch(branch, true).ok();

    // Remove metadata
    if let Ok(workspace_id) = git::workspace_id() {
        let wt_dir = config.workspaces_dir.join(&workspace_id);
        meta::remove_meta(&wt_dir, branch);
    }

    Ok(())
}
