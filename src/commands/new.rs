use std::path::PathBuf;

use anyhow::{Result, anyhow};
use clap::Args;
use git2::Repository;

use crate::{git, meta};

#[derive(Args)]
pub struct NewArgs {
    /// Branch name for the new worktree
    pub branch: String,

    /// Task name to attach as metadata
    #[arg(long)]
    pub task: Option<String>,

    /// Directory to create the worktree in (default: ../repo-branch)
    #[arg(long)]
    pub path: Option<PathBuf>,

    /// Base branch or commit to branch from (default: HEAD)
    #[arg(long)]
    pub base: Option<String>,
}

pub fn run(args: NewArgs) -> Result<()> {
    let repo = git::open_repo()?;

    let path = match args.path {
        Some(p) => p,
        None => default_path(&repo, &args.branch)?,
    };

    git::create_worktree(&repo, &args.branch, &path, args.base.as_deref())?;

    if let Some(task) = args.task {
        let wt_name = path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or(&args.branch)
            .to_string();
        let store = meta::load(&repo)?;
        let m = meta::WorktreeMeta::new(Some(task), None);
        let store = meta::upsert_meta(store, &wt_name, m);
        meta::save(&repo, &store)?;
    }

    println!("{}", path.display());
    Ok(())
}

fn default_path(repo: &Repository, branch: &str) -> Result<PathBuf> {
    let workdir = repo
        .workdir()
        .ok_or_else(|| anyhow!("bare repository not supported"))?;
    let repo_name = workdir
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("repo");
    let sanitized = git::sanitize_branch_name(branch);
    let dir_name = format!("{repo_name}-{sanitized}");
    let parent = workdir.parent().unwrap_or(workdir);
    Ok(parent.join(dir_name))
}
