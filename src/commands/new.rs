use std::path::PathBuf;

use anyhow::Result;
use clap::Args;

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

pub fn run(_args: NewArgs) -> Result<()> {
    // TODO: implement
    // 1. open_repo()
    // 2. resolve target path (--path or default)
    // 3. git::create_worktree()
    // 4. if --task: meta::load() → upsert → save()
    // 5. print created path
    println!("new: not yet implemented");
    Ok(())
}
