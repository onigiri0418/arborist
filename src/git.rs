use std::path::{Path, PathBuf};
use std::time::SystemTime;

use anyhow::Result;
use git2::Repository;

/// A snapshot of a single worktree's state.
#[derive(Debug, Clone)]
pub struct WorktreeInfo {
    pub name: String,
    pub path: PathBuf,
    pub branch: Option<String>,
    pub head_commit: String,
    pub head_summary: String,
    pub is_bare: bool,
    pub is_locked: bool,
    pub last_modified: SystemTime,
}

/// Summary of working-tree changes (equivalent to `git diff --stat`).
#[derive(Debug, Clone, Default)]
pub struct DiffStat {
    pub files_changed: usize,
    pub insertions: usize,
    pub deletions: usize,
}

impl DiffStat {
    pub fn is_clean(&self) -> bool {
        self.files_changed == 0
    }
}

/// Open the git repository from the current directory.
/// Works correctly when called from inside a linked worktree.
pub fn open_repo() -> Result<Repository> {
    let repo = Repository::discover(".")?;
    Ok(repo)
}

/// List all worktrees attached to the repository.
pub fn list_worktrees(_repo: &Repository) -> Result<Vec<WorktreeInfo>> {
    // TODO: implement using git2 worktree API
    Ok(vec![])
}

/// Create a new linked worktree at `path` on `branch`.
/// If `branch` does not exist it will be created from HEAD.
pub fn create_worktree(_repo: &Repository, _branch: &str, _path: &Path) -> Result<()> {
    // TODO: implement
    Ok(())
}

/// Remove a linked worktree by name.
pub fn remove_worktree(_repo: &Repository, _name: &str, _force: bool) -> Result<()> {
    // TODO: implement
    Ok(())
}

/// Return true if `branch` has been fully merged into main/master.
pub fn is_branch_merged(_repo: &Repository, _branch: &str) -> Result<bool> {
    // TODO: implement
    Ok(false)
}

/// Return diff statistics for the worktree at `path`.
pub fn diff_stat(_repo: &Repository, _path: &Path) -> Result<DiffStat> {
    // TODO: implement
    Ok(DiffStat::default())
}

/// Sanitize a branch name for use as a directory name component.
/// Invalid characters are replaced with `-`; consecutive dashes are collapsed.
/// e.g. `feature/my-thing` → `feature-my-thing`
/// e.g. `feat: my thing!`  → `feat-my-thing`
pub fn sanitize_branch_name(branch: &str) -> String {
    let replaced: String = branch
        .chars()
        .map(|c| {
            if c.is_ascii_alphanumeric() || matches!(c, '-' | '_' | '.') {
                c
            } else {
                '-'
            }
        })
        .collect();

    let mut result = String::with_capacity(replaced.len());
    let mut prev_dash = false;
    for c in replaced.chars() {
        if c == '-' {
            if !prev_dash && !result.is_empty() {
                result.push('-');
            }
            prev_dash = true;
        } else {
            result.push(c);
            prev_dash = false;
        }
    }
    result.trim_end_matches('-').to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sanitize_replaces_slashes() {
        assert_eq!(sanitize_branch_name("feature/login"), "feature-login");
    }

    #[test]
    fn sanitize_strips_invalid_chars() {
        assert_eq!(sanitize_branch_name("feat: my thing!"), "feat-my-thing");
    }
}
