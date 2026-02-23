// ===========================================================================
// wt exec - Execute a command in each managed worktree
// ===========================================================================

use std::process::Command;

use clap::Args;

use crate::cli::Result;
use crate::config::Config;
use crate::git;

#[derive(Args)]
pub struct ExecArgs {
    /// Command to execute in each worktree
    #[arg(required = true, trailing_var_arg = true)]
    pub command: Vec<String>,

    /// Only run in specified branches (comma-separated)
    #[arg(short, long, value_delimiter = ',')]
    pub branches: Option<Vec<String>>,
}

pub fn run(args: ExecArgs, config: &Config) -> Result<()> {
    let workspace_id = git::workspace_id()?;
    let wt_dir = config.workspaces_dir.join(&workspace_id);

    // Canonicalize to resolve symlinks (e.g. /var -> /private/var on macOS)
    // so path comparisons with git worktree list output match correctly
    let wt_dir = wt_dir.canonicalize().unwrap_or(wt_dir);

    if !wt_dir.exists() {
        eprintln!("No worktrees for this project.");
        return Ok(());
    }

    let worktrees = git::list_worktrees()?;

    let managed: Vec<_> = worktrees
        .iter()
        .filter(|wt| wt.path.starts_with(&wt_dir))
        .filter(|wt| {
            if let Some(ref branches) = args.branches {
                wt.branch
                    .as_deref()
                    .map(|b| branches.iter().any(|f| f == b))
                    .unwrap_or(false)
            } else {
                true
            }
        })
        .collect();

    if managed.is_empty() {
        eprintln!("No worktrees for this project.");
        return Ok(());
    }

    let cmd_str = args.command.join(" ");
    let mut success_count = 0usize;
    let mut failure_count = 0usize;

    for wt in &managed {
        let branch = wt.branch.as_deref().unwrap_or("(detached)");
        println!("==> {} <==", branch);

        let result = if cfg!(target_os = "windows") {
            Command::new("cmd")
                .args(["/C", &cmd_str])
                .current_dir(&wt.path)
                .status()
        } else {
            Command::new("sh")
                .args(["-c", &cmd_str])
                .current_dir(&wt.path)
                .status()
        };

        match result {
            Ok(status) if status.success() => {
                success_count += 1;
            }
            Ok(_) => {
                failure_count += 1;
            }
            Err(e) => {
                eprintln!("Failed to execute command: {}", e);
                failure_count += 1;
            }
        }

        println!();
    }

    println!(
        "Executed in {} worktree(s): {} succeeded, {} failed",
        managed.len(),
        success_count,
        failure_count
    );

    Ok(())
}
