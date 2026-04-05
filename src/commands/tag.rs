use anyhow::Result;
use chrono::Utc;
use clap::Args;
use owo_colors::OwoColorize;

use crate::{
    git,
    meta::{self, WorktreeMeta, WorktreeStatus},
};

#[derive(Args)]
pub struct TagArgs {
    /// Worktree name or branch name (prefix match)
    pub name: String,

    /// Set task name
    #[arg(long)]
    pub task: Option<String>,

    /// Set memo text
    #[arg(long)]
    pub memo: Option<String>,

    /// Set status
    #[arg(long, value_enum)]
    pub status: Option<WorktreeStatus>,
}

pub fn run(args: TagArgs) -> Result<()> {
    let repo = git::open_repo()?;
    let worktrees = git::list_worktrees(&repo)?;
    let wt = git::resolve_worktree(&worktrees, &args.name)?;
    let store = meta::load(&repo)?;

    let has_updates = args.task.is_some() || args.memo.is_some() || args.status.is_some();

    if !has_updates {
        if let Some(m) = store.worktrees.get(&wt.name) {
            print_meta(&wt.name, m);
        } else {
            println!("No metadata for '{}'", wt.name.yellow());
        }
        return Ok(());
    }

    let now = Utc::now();
    let existing = store.worktrees.get(&wt.name).cloned();
    let updated = match existing {
        Some(mut m) => {
            if let Some(task) = args.task {
                m.task = Some(task);
            }
            if let Some(memo) = args.memo {
                m.memo = Some(memo);
            }
            if let Some(status) = args.status {
                m.status = status;
            }
            m.updated_at = now;
            m
        }
        None => WorktreeMeta {
            task: args.task,
            memo: args.memo,
            status: args.status.unwrap_or_default(),
            created_at: now,
            updated_at: now,
        },
    };

    let name = wt.name.clone();
    let store = meta::upsert_meta(store, &name, updated);
    meta::save(&repo, &store)?;

    if let Some(m) = store.worktrees.get(&name) {
        print_meta(&name, m);
    }
    Ok(())
}

fn colored_status_label(status: &WorktreeStatus) -> String {
    let label = format!("{:?}", status).to_lowercase();
    match status {
        WorktreeStatus::Active => label.blue().to_string(),
        WorktreeStatus::Paused => label.yellow().to_string(),
        WorktreeStatus::Done => label.green().to_string(),
    }
}

fn print_meta(name: &str, m: &meta::WorktreeMeta) {
    println!("{} {}", "Worktree:".bold(), name.bold());
    println!(
        "  {}  {}",
        "task:      ".dimmed(),
        m.task.as_deref().unwrap_or("--")
    );
    println!(
        "  {}  {}",
        "memo:      ".dimmed(),
        m.memo.as_deref().unwrap_or("--")
    );
    println!(
        "  {}  {}",
        "status:    ".dimmed(),
        colored_status_label(&m.status)
    );
    println!(
        "  {}  {}",
        "created_at:".dimmed(),
        m.created_at.format("%Y-%m-%d %H:%M UTC").dimmed()
    );
    println!(
        "  {}  {}",
        "updated_at:".dimmed(),
        m.updated_at.format("%Y-%m-%d %H:%M UTC").dimmed()
    );
}
