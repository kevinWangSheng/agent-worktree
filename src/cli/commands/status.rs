// ===========================================================================
// wt status - Show current worktree status overview
// ===========================================================================

use clap::Args;
use serde::Serialize;

use crate::cli::Result;
use crate::config::Config;
use crate::git;

#[derive(Args)]
pub struct StatusArgs {
    /// Output as JSON
    #[arg(long)]
    pub json: bool,
}

#[derive(Serialize)]
struct StatusInfo {
    branch: String,
    in_worktree: bool,
    trunk: String,
    commits_ahead: usize,
    uncommitted: usize,
    insertions: usize,
    deletions: usize,
    merge_in_progress: bool,
    rebase_in_progress: bool,
}

pub fn run(args: StatusArgs, config: &Config) -> Result<()> {
    let branch = git::current_branch()?;

    let trunk = config
        .trunk
        .clone()
        .unwrap_or_else(|| git::detect_trunk().unwrap_or_else(|_| "main".into()));

    // Determine if we're in a managed worktree
    let workspace_id = git::workspace_id()?;
    let wt_dir = config.workspaces_dir.join(&workspace_id);
    let in_worktree = std::env::current_dir()
        .ok()
        .and_then(|cwd| cwd.canonicalize().ok())
        .and_then(|cwd| wt_dir.canonicalize().ok().map(|p| cwd.starts_with(p)))
        .unwrap_or(false);

    // Gather stats
    let commits_ahead = git::commit_count(&trunk, &branch).unwrap_or(0);
    let uncommitted = git::uncommitted_count_in(&std::env::current_dir().unwrap_or_default())
        .unwrap_or(0);

    let committed_diff = git::diff_shortstat(&trunk, &branch)
        .unwrap_or(git::DiffStat { insertions: 0, deletions: 0 });
    let uncommitted_diff = git::diff_shortstat_in(&std::env::current_dir().unwrap_or_default())
        .unwrap_or(git::DiffStat { insertions: 0, deletions: 0 });

    let total_insertions = committed_diff.insertions + uncommitted_diff.insertions;
    let total_deletions = committed_diff.deletions + uncommitted_diff.deletions;

    let merge_in_progress = git::is_merge_in_progress();
    let rebase_in_progress = git::is_rebase_in_progress();

    if args.json {
        let info = StatusInfo {
            branch,
            in_worktree,
            trunk,
            commits_ahead,
            uncommitted,
            insertions: total_insertions,
            deletions: total_deletions,
            merge_in_progress,
            rebase_in_progress,
        };
        let json = serde_json::to_string_pretty(&info)
            .map_err(|e| crate::cli::Error::Other(format!("JSON serialization failed: {}", e)))?;
        println!("{}", json);
        return Ok(());
    }

    // Print status
    let location = if in_worktree {
        "managed worktree"
    } else {
        "main repository"
    };

    println!("Branch:      {}", branch);
    println!("Location:    {}", location);
    println!("Trunk:       {}", trunk);
    println!("Ahead:       {} commit{}", commits_ahead, if commits_ahead == 1 { "" } else { "s" });
    println!("Uncommitted: {} file{}", uncommitted, if uncommitted == 1 { "" } else { "s" });

    if total_insertions > 0 || total_deletions > 0 {
        println!("Diff:        +{} -{}", total_insertions, total_deletions);
    } else {
        println!("Diff:        -");
    }

    if merge_in_progress {
        println!();
        println!("WARNING: Merge in progress");
    }
    if rebase_in_progress {
        println!();
        println!("WARNING: Rebase in progress");
    }

    Ok(())
}
