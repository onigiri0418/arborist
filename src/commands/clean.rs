use anyhow::Result;
use clap::Args;

use crate::{git, meta, meta::WorktreeStatus};

#[derive(Args)]
pub struct CleanArgs {
    /// Show candidates without deleting
    #[arg(long)]
    pub dry_run: bool,

    /// Delete all candidates without confirmation prompt
    #[arg(long)]
    pub all: bool,
}

pub fn run(args: CleanArgs) -> Result<()> {
    let repo = git::open_repo()?;
    let worktrees = git::list_worktrees(&repo)?;
    let store = meta::load(&repo)?;

    // Candidates: linked worktrees that are merged OR have status Done
    let candidates: Vec<_> = worktrees
        .iter()
        .filter(|wt| {
            if wt.is_main {
                return false;
            }
            let is_done = store
                .worktrees
                .get(&wt.name)
                .map(|m| m.status == WorktreeStatus::Done)
                .unwrap_or(false);
            let is_merged = wt
                .branch
                .as_deref()
                .map(|b| git::is_branch_merged(&repo, b).unwrap_or(false))
                .unwrap_or(false);
            is_done || is_merged
        })
        .collect();

    if candidates.is_empty() {
        println!("Nothing to clean.");
        return Ok(());
    }

    if args.dry_run {
        println!("Would remove:");
        for wt in &candidates {
            println!("  {} ({})", wt.name, wt.path.display());
        }
        return Ok(());
    }

    if args.all {
        for wt in &candidates {
            git::remove_worktree(&repo, &wt.name, false)?;
            let s = meta::load(&repo)?;
            let s = meta::remove_meta(s, &wt.name);
            meta::save(&repo, &s)?;
            println!("Removed '{}'", wt.name);
        }
    } else {
        // Interactive selection via inquire
        let items: Vec<String> = candidates
            .iter()
            .map(|wt| format!("{} ({})", wt.name, wt.path.display()))
            .collect();
        let selected = inquire::MultiSelect::new("Select worktrees to remove:", items)
            .prompt()?;
        for label in &selected {
            // Extract name (before the space)
            let name = label.split(' ').next().unwrap_or("");
            git::remove_worktree(&repo, name, false)?;
            let s = meta::load(&repo)?;
            let s = meta::remove_meta(s, name);
            meta::save(&repo, &s)?;
            println!("Removed '{name}'");
        }
    }

    Ok(())
}
