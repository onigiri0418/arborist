use anyhow::Result;
use clap::Args;

#[derive(Args)]
pub struct StatusArgs {
    /// Compact one-line-per-worktree output
    #[arg(long)]
    pub short: bool,
}

pub fn run(_args: StatusArgs) -> Result<()> {
    // TODO: implement
    // 1. open_repo()
    // 2. list_worktrees()
    // 3. diff_stat() for each worktree (parallel)
    // 4. render table or short format
    println!("status: not yet implemented");
    Ok(())
}
