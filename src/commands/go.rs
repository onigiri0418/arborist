use anyhow::Result;
use clap::Args;

use crate::git;

#[derive(Args)]
pub struct GoArgs {
    /// Worktree name or branch name (skips interactive selection)
    pub name: Option<String>,
}

pub fn run(args: GoArgs) -> Result<()> {
    let repo = git::open_repo()?;
    let worktrees = git::list_worktrees(&repo)?;

    let wt = if let Some(name) = &args.name {
        git::resolve_worktree(&worktrees, name)?
    } else {
        // Interactive selection
        let names: Vec<String> = worktrees.iter().map(|w| w.name.clone()).collect();
        let selected = inquire::Select::new("Select worktree:", names).prompt()?;
        worktrees
            .iter()
            .find(|w| w.name == selected)
            .ok_or_else(|| anyhow::anyhow!("selected worktree not found"))?
    };

    println!("cd {}", wt.path.display());
    Ok(())
}
