use clap::{Parser, Subcommand};

use crate::commands::{
    clean::CleanArgs, go::GoArgs, list::ListArgs, new::NewArgs, rm::RmArgs, status::StatusArgs,
    tag::TagArgs,
};

#[derive(Parser)]
#[command(
    name = "arborist",
    version,
    about = "Git worktree manager for AI-driven development",
    long_about = None,
)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    /// List all worktrees with rich display
    List(ListArgs),
    /// Create a new worktree
    New(NewArgs),
    /// Remove a worktree
    Rm(RmArgs),
    /// Remove merged or stale worktrees
    Clean(CleanArgs),
    /// Switch to a worktree (use with: eval "$(arborist go)")
    Go(GoArgs),
    /// Show change summary for all worktrees
    Status(StatusArgs),
    /// Attach task metadata to a worktree
    Tag(TagArgs),
}
