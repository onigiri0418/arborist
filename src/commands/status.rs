use anyhow::Result;
use clap::Args;

use crate::{git, meta};

#[derive(Args)]
pub struct StatusArgs {
    /// Compact one-line-per-worktree output
    #[arg(long)]
    pub short: bool,
}

pub fn run(args: StatusArgs) -> Result<()> {
    let repo = git::open_repo()?;
    let worktrees = git::list_worktrees(&repo)?;
    let store = meta::load(&repo)?;

    let cwd = std::env::current_dir()
        .ok()
        .and_then(|p| p.canonicalize().ok());

    if !args.short {
        println!("{:<2} {:<20} {:<8} {:<14} {}",
            "", "NAME", "CHANGES", "DIFF", "TASK");
    }

    for wt in &worktrees {
        let stat = git::open_repo_at(&wt.path)
            .and_then(|r| git::diff_stat(&r, &wt.path))
            .unwrap_or_default();

        let is_current = cwd.as_deref().map_or(false, |c| {
            wt.path
                .canonicalize()
                .map(|p| c.starts_with(&p))
                .unwrap_or(false)
        });
        let marker = if is_current { "*" } else { " " };
        let task = store
            .worktrees
            .get(&wt.name)
            .and_then(|m| m.task.as_deref())
            .unwrap_or("");

        if args.short {
            if stat.is_clean() {
                println!("{}{} clean", marker, wt.name);
            } else {
                println!("{}{} +{} -{}", marker, wt.name, stat.insertions, stat.deletions);
            }
        } else {
            let changes = if stat.is_clean() {
                "clean".to_string()
            } else {
                "dirty".to_string()
            };
            let diff = if stat.is_clean() {
                String::new()
            } else {
                format!("+{} -{}", stat.insertions, stat.deletions)
            };
            println!("{:<2} {:<20} {:<8} {:<14} {}",
                marker,
                truncate(&wt.name, 20),
                changes,
                diff,
                task,
            );
        }
    }

    Ok(())
}

fn truncate(s: &str, max: usize) -> &str {
    if s.len() <= max { s } else { &s[..max] }
}
