use std::path::PathBuf;

#[derive(thiserror::Error, Debug)]
#[allow(dead_code)]
pub enum ArboristError {
    #[error("Not inside a git repository")]
    NotARepo,

    #[error("Worktree '{0}' not found")]
    WorktreeNotFound(String),

    #[error("Ambiguous name '{0}': matches {1:?}")]
    AmbiguousName(String, Vec<String>),

    #[error("Worktree '{0}' has uncommitted changes. Use --force to override.")]
    DirtyWorktree(String),

    #[error("Path '{0}' already exists")]
    PathAlreadyExists(PathBuf),

    #[error("Cannot remove the current worktree")]
    CannotRemoveCurrent,
}
