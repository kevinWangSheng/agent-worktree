// ===========================================================================
// wt diff - Show diff between current branch and trunk
// ===========================================================================

use std::process::Command;

use clap::Args;

use crate::cli::{Error, Result};
use crate::config::Config;
use crate::git;

#[derive(Args)]
pub struct DiffArgs {
    /// Show only diffstat (file list with +/- counts)
    #[arg(long)]
    pub stat: bool,

    /// Compare with specific branch (default: trunk)
    #[arg(long)]
    pub target: Option<String>,
}

/// Run a git command and return its stdout as a String.
fn git_output(args: &[&str]) -> Result<String> {
    let output = Command::new("git")
        .args(args)
        .output()
        .map_err(|e| Error::Other(format!("failed to run git: {e}")))?;

    Ok(String::from_utf8_lossy(&output.stdout).to_string())
}

pub fn run(args: DiffArgs, config: &Config) -> Result<()> {
    let current = git::current_branch()?;

    let target = args.target.unwrap_or_else(|| {
        config
            .trunk
            .clone()
            .unwrap_or_else(|| git::detect_trunk().unwrap_or_else(|_| "main".into()))
    });

    let range = format!("{target}...HEAD");

    if args.stat {
        print_section(
            &format!("Committed changes ({current} vs {target}):"),
            &git_output(&["diff", "--stat", &range])?,
        );
        print_section(
            "Uncommitted changes:",
            &git_output(&["diff", "--stat"])?,
        );
        print_section(
            "Staged changes:",
            &git_output(&["diff", "--stat", "--cached"])?,
        );
    } else {
        print_section(
            &format!("Committed changes ({current} vs {target}):"),
            &git_output(&["diff", &range])?,
        );
        print_section(
            "Uncommitted changes:",
            &git_output(&["diff"])?,
        );
        print_section(
            "Staged changes:",
            &git_output(&["diff", "--cached"])?,
        );
    }

    Ok(())
}

fn print_section(header: &str, content: &str) {
    if !content.is_empty() {
        println!("{header}");
        print!("{content}");
    }
}
