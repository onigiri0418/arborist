use anyhow::Result;
use clap::Args;

use crate::{error::ArboristError, git, meta};

#[derive(Args)]
pub struct RmArgs {
    /// Worktree name or branch name (prefix match)
    pub name: String,

    /// Remove even if there are uncommitted changes
    #[arg(long)]
    pub force: bool,
}

pub fn run(args: RmArgs) -> Result<()> {
    let repo = git::open_repo()?;
    let worktrees = git::list_worktrees(&repo)?;

    // Only allow removing linked worktrees (not main)
    let linked: Vec<_> = worktrees.iter().filter(|w| !w.is_main).cloned().collect();
    let wt = git::resolve_worktree(&linked, &args.name)?;

    // Refuse to remove the worktree we're currently inside
    if let Ok(cwd) = std::env::current_dir() {
        if let (Ok(cwd_c), Ok(wt_c)) = (cwd.canonicalize(), wt.path.canonicalize()) {
            if cwd_c.starts_with(&wt_c) {
                return Err(ArboristError::CannotRemoveCurrent.into());
            }
        }
    }

    let name = wt.name.clone();
    git::remove_worktree(&repo, &name, args.force)?;

    let store = meta::load(&repo)?;
    let store = meta::remove_meta(store, &name);
    meta::save(&repo, &store)?;

    println!("Removed worktree '{name}'");
    Ok(())
}
