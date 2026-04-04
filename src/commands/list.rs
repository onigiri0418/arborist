use anyhow::Result;
use clap::Args;

#[derive(Args)]
pub struct ListArgs {
    /// Output as JSON array
    #[arg(long)]
    pub json: bool,

    /// Output paths only (plain text)
    #[arg(long)]
    pub short: bool,
}

pub fn run(_args: ListArgs) -> Result<()> {
    // TODO: implement
    // 1. open_repo()
    // 2. git::list_worktrees()
    // 3. meta::load() and join on name
    // 4. render based on --json / --short / default
    println!("list: not yet implemented");
    Ok(())
}
