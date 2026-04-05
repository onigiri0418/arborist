use anyhow::Result;
use clap::Args;
use serde_json::json;

use crate::{git, meta};

#[derive(Args)]
pub struct ListArgs {
    /// Output as JSON array
    #[arg(long)]
    pub json: bool,

    /// Output paths only (plain text)
    #[arg(long)]
    pub short: bool,
}

pub fn run(args: ListArgs) -> Result<()> {
    let repo = git::open_repo()?;
    let worktrees = git::list_worktrees(&repo)?;
    let store = meta::load(&repo)?;

    if args.json {
        let arr: Vec<_> = worktrees
            .iter()
            .map(|wt| {
                let m = store.worktrees.get(&wt.name);
                json!({
                    "name": wt.name,
                    "path": wt.path,
                    "branch": wt.branch,
                    "head_commit": wt.head_commit,
                    "head_summary": wt.head_summary,
                    "is_locked": wt.is_locked,
                    "is_main": wt.is_main,
                    "task": m.and_then(|m| m.task.as_deref()),
                    "memo": m.and_then(|m| m.memo.as_deref()),
                    "status": m.map(|m| format!("{:?}", m.status).to_lowercase()),
                })
            })
            .collect();
        println!("{}", serde_json::to_string_pretty(&arr)?);
    } else if args.short {
        for wt in &worktrees {
            println!("{}", wt.path.display());
        }
    } else {
        print_table(&worktrees, &store);
    }

    Ok(())
}

fn print_table(
    worktrees: &[git::WorktreeInfo],
    store: &meta::MetaStore,
) {
    let cwd = std::env::current_dir()
        .ok()
        .and_then(|p| p.canonicalize().ok());

    println!("{:<2} {:<20} {:<20} {:<10} {:<10} {}",
        "", "NAME", "BRANCH", "COMMIT", "STATUS", "TASK");

    for wt in worktrees {
        let is_current = cwd.as_deref().map_or(false, |c| {
            wt.path
                .canonicalize()
                .map(|p| c.starts_with(&p))
                .unwrap_or(false)
        });
        let marker = if is_current { "*" } else { " " };
        let branch = wt.branch.as_deref().unwrap_or("-");
        let commit = if wt.head_commit.is_empty() {
            "-".to_string()
        } else {
            wt.head_commit[..wt.head_commit.len().min(8)].to_string()
        };
        let m = store.worktrees.get(&wt.name);
        let status = m.map(|m| format!("{:?}", m.status).to_lowercase()).unwrap_or_default();
        let task = m.and_then(|m| m.task.as_deref()).unwrap_or("");

        println!("{:<2} {:<20} {:<20} {:<10} {:<10} {}",
            marker,
            truncate(&wt.name, 20),
            truncate(branch, 20),
            truncate(&commit, 10),
            truncate(&status, 10),
            task,
        );
    }
}

fn truncate(s: &str, max: usize) -> &str {
    if s.len() <= max {
        s
    } else {
        &s[..max]
    }
}
