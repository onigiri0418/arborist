use anyhow::Result;
use clap::Args;

#[derive(Args)]
pub struct RmArgs {
    /// Worktree name or branch name (prefix match)
    pub name: String,

    /// Remove even if there are uncommitted changes
    #[arg(long)]
    pub force: bool,
}

pub fn run(_args: RmArgs) -> Result<()> {
    // TODO: implement
    // 1. open_repo()
    // 2. resolve worktree by name (exact then prefix match)
    // 3. if !force: diff_stat() → prompt if dirty
    // 4. git::remove_worktree()
    // 5. meta::load() → remove_meta() → save()
    println!("rm: not yet implemented");
    Ok(())
}
