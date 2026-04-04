use anyhow::Result;
use clap::Args;

use crate::meta::WorktreeStatus;

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

pub fn run(_args: TagArgs) -> Result<()> {
    // TODO: implement
    // 1. open_repo()
    // 2. resolve worktree by name
    // 3. meta::load()
    // 4. if no flags: display current meta and exit
    // 5. upsert_meta() with updated fields
    // 6. meta::save()
    // 7. display updated meta
    println!("tag: not yet implemented");
    Ok(())
}
