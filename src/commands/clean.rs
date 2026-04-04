use anyhow::Result;
use clap::Args;

#[derive(Args)]
pub struct CleanArgs {
    /// Show candidates without deleting
    #[arg(long)]
    pub dry_run: bool,

    /// Delete all candidates without confirmation prompt
    #[arg(long)]
    pub all: bool,
}

pub fn run(_args: CleanArgs) -> Result<()> {
    // TODO: implement
    // 1. open_repo()
    // 2. list_worktrees() → filter merged or status==Done
    // 3. if --dry-run: print and exit
    // 4. if !--all: inquire checkbox selection
    // 5. remove selected worktrees and update meta
    println!("clean: not yet implemented");
    Ok(())
}
