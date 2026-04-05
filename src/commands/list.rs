use anyhow::Result;
use clap::Args;
use owo_colors::OwoColorize;
use serde_json::json;

use crate::{
    git,
    meta::{self, WorktreeStatus},
};

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

fn colored_status(status: &WorktreeStatus) -> String {
    let label = format!("{:<10}", format!("{:?}", status).to_lowercase());
    match status {
        WorktreeStatus::Active => label.blue().to_string(),
        WorktreeStatus::Paused => label.yellow().to_string(),
        WorktreeStatus::Done => label.green().to_string(),
    }
}

fn print_table(worktrees: &[git::WorktreeInfo], store: &meta::MetaStore) {
    let cwd = std::env::current_dir()
        .ok()
        .and_then(|p| p.canonicalize().ok());

    println!(
        "{:<2} {} {} {} {} {}",
        "",
        format!("{:<20}", "NAME").bold(),
        format!("{:<20}", "BRANCH").bold(),
        format!("{:<10}", "COMMIT").bold(),
        format!("{:<10}", "STATUS").bold(),
        "TASK".bold(),
    );

    for wt in worktrees {
        let is_current = cwd.as_deref().is_some_and(|c| {
            wt.path
                .canonicalize()
                .map(|p| c.starts_with(&p))
                .unwrap_or(false)
        });
        let marker = if is_current {
            format!("{}", "*".green().bold())
        } else {
            " ".to_string()
        };

        let branch = wt.branch.as_deref().unwrap_or("-");
        let commit = if wt.head_commit.is_empty() {
            "-".to_string()
        } else {
            wt.head_commit[..wt.head_commit.len().min(8)].to_string()
        };

        let m = store.worktrees.get(&wt.name);
        let status_col = m
            .map(|m| colored_status(&m.status))
            .unwrap_or_else(|| format!("{:<10}", ""));
        let task = m.and_then(|m| m.task.as_deref()).unwrap_or("");

        let name_col = format!("{:<20}", truncate_str(&wt.name, 20));
        let branch_col = format!("{}", format!("{:<20}", truncate_str(branch, 20)).cyan());
        let commit_col = format!("{:<10}", truncate_str(&commit, 10));

        println!(
            "{:<2} {} {} {} {} {}",
            marker, name_col, branch_col, commit_col, status_col, task,
        );
    }
}

/// Truncate a string to at most `max` characters.
fn truncate_str(s: &str, max: usize) -> &str {
    let mut end = max;
    while end > 0 && !s.is_char_boundary(end) {
        end -= 1;
    }
    &s[..end.min(s.len())]
}
