use anyhow::Result;
use clap::Args;

#[derive(Args)]
pub struct GoArgs {
    /// Worktree name or branch name (skips interactive selection)
    pub name: Option<String>,
}

pub fn run(_args: GoArgs) -> Result<()> {
    // TODO: implement
    // 1. open_repo()
    // 2. list_worktrees()
    // 3. if name given: resolve by prefix match
    //    else: inquire::Select for fuzzy interactive selection
    // 4. print "cd <path>" to stdout (caller uses eval "$(arborist go)")
    println!("go: not yet implemented");
    Ok(())
}
