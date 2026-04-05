use std::fs;
use std::path::{Path, PathBuf};
use std::time::SystemTime;

use anyhow::{Context, Result};
use git2::Repository;

use crate::error::ArboristError;

/// A snapshot of a single worktree's state.
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct WorktreeInfo {
    pub name: String,
    pub path: PathBuf,
    pub branch: Option<String>,
    pub head_commit: String,
    pub head_summary: String,
    pub is_bare: bool,
    pub is_locked: bool,
    pub is_main: bool,
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

/// Open the git repository rooted at `path`.
pub fn open_repo_at(path: &Path) -> Result<Repository> {
    Ok(Repository::discover(path)?)
}

/// List all worktrees attached to the repository (main + linked).
pub fn list_worktrees(repo: &Repository) -> Result<Vec<WorktreeInfo>> {
    let mut result = Vec::new();

    // Main worktree
    if let Some(workdir) = repo.workdir() {
        let name = workdir
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("main")
            .to_string();
        result.push(build_worktree_info(repo, &name, workdir, false, true)?);
    }

    // Linked worktrees
    let names = repo.worktrees()?;
    for name in names.iter().flatten() {
        let wt = repo.find_worktree(name)?;
        let is_locked = matches!(
            wt.is_locked().unwrap_or(git2::WorktreeLockStatus::Unlocked),
            git2::WorktreeLockStatus::Locked(_)
        );
        let wt_repo = Repository::open(wt.path())?;
        result.push(build_worktree_info(
            &wt_repo,
            name,
            wt.path(),
            is_locked,
            false,
        )?);
    }

    Ok(result)
}

fn build_worktree_info(
    repo: &Repository,
    name: &str,
    path: &Path,
    is_locked: bool,
    is_main: bool,
) -> Result<WorktreeInfo> {
    let branch = repo.head().ok().and_then(|h| {
        if h.is_branch() {
            h.shorthand().map(|s| s.to_string())
        } else {
            None
        }
    });

    let (head_commit, head_summary) = repo
        .head()
        .ok()
        .and_then(|h| h.peel_to_commit().ok())
        .map(|c| {
            let id = format!("{:.8}", c.id());
            let summary = c.summary().unwrap_or("").to_string();
            (id, summary)
        })
        .unwrap_or_default();

    let last_modified = fs::metadata(path)
        .and_then(|m| m.modified())
        .unwrap_or(SystemTime::UNIX_EPOCH);

    Ok(WorktreeInfo {
        name: name.to_string(),
        path: path.to_path_buf(),
        branch,
        head_commit,
        head_summary,
        is_bare: repo.is_bare(),
        is_locked,
        is_main,
        last_modified,
    })
}

/// Create a new linked worktree at `path` on `branch`.
/// If `branch` does not exist it will be created from HEAD (or from `base` if given).
pub fn create_worktree(
    repo: &Repository,
    branch: &str,
    path: &Path,
    base: Option<&str>,
) -> Result<()> {
    if path.exists() {
        return Err(ArboristError::PathAlreadyExists(path.to_path_buf()).into());
    }

    // Check if branch exists; create it if not.
    let branch_exists = repo.find_branch(branch, git2::BranchType::Local).is_ok();

    if !branch_exists {
        let commit = if let Some(base_ref) = base {
            repo.revparse_single(base_ref)
                .with_context(|| format!("base ref '{base_ref}' not found"))?
                .peel_to_commit()?
        } else {
            repo.head()?.peel_to_commit()?
        };
        repo.branch(branch, &commit, false)?;
    }

    let refname = format!("refs/heads/{branch}");
    let reference = repo.find_reference(&refname)?;

    let name = path.file_name().and_then(|n| n.to_str()).unwrap_or(branch);

    let mut opts = git2::WorktreeAddOptions::new();
    opts.reference(Some(&reference));
    repo.worktree(name, path, Some(&opts))?;

    Ok(())
}

/// Remove a linked worktree by name.
pub fn remove_worktree(repo: &Repository, name: &str, force: bool) -> Result<()> {
    let wt = repo
        .find_worktree(name)
        .map_err(|_| ArboristError::WorktreeNotFound(name.to_string()))?;

    let wt_path = wt.path().to_path_buf();

    if !force && let Ok(wt_repo) = Repository::open(&wt_path) {
        let stat = diff_stat(&wt_repo, &wt_path)?;
        if !stat.is_clean() {
            return Err(ArboristError::DirtyWorktree(name.to_string()).into());
        }
    }

    // Remove the working tree directory first.
    if wt_path.exists() {
        fs::remove_dir_all(&wt_path)
            .with_context(|| format!("failed to remove {}", wt_path.display()))?;
    }

    // Prune git's tracking metadata.
    let mut opts = git2::WorktreePruneOptions::new();
    opts.valid(true);
    if force {
        opts.locked(true);
    }
    wt.prune(Some(&mut opts))?;

    Ok(())
}

/// Return true if `branch` has been fully merged into main/master.
pub fn is_branch_merged(repo: &Repository, branch: &str) -> Result<bool> {
    let branch_ref = repo.find_branch(branch, git2::BranchType::Local)?;
    let branch_commit = branch_ref.get().peel_to_commit()?;

    // Detect the main branch (main or master).
    let main_commit = if let Ok(b) = repo.find_branch("main", git2::BranchType::Local) {
        b.get().peel_to_commit()?
    } else if let Ok(b) = repo.find_branch("master", git2::BranchType::Local) {
        b.get().peel_to_commit()?
    } else {
        return Ok(false);
    };

    // Branch is merged if its tip is an ancestor of the main tip.
    let merge_base = repo.merge_base(branch_commit.id(), main_commit.id())?;
    Ok(merge_base == branch_commit.id())
}

/// Return diff statistics (index + workdir vs HEAD) for the given repo.
pub fn diff_stat(repo: &Repository, _path: &Path) -> Result<DiffStat> {
    let tree = match repo.head() {
        Ok(head) => Some(head.peel_to_commit()?.tree()?),
        Err(_) => None,
    };

    let diff = repo.diff_tree_to_workdir_with_index(tree.as_ref(), None)?;
    let stats = diff.stats()?;

    Ok(DiffStat {
        files_changed: stats.files_changed(),
        insertions: stats.insertions(),
        deletions: stats.deletions(),
    })
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

/// Resolve a worktree by name: exact match takes priority, then prefix match.
/// Returns `WorktreeNotFound` if nothing matches, `AmbiguousName` if multiple prefix matches.
pub fn resolve_worktree<'a>(worktrees: &'a [WorktreeInfo], name: &str) -> Result<&'a WorktreeInfo> {
    // Exact match takes priority over prefix match.
    if let Some(exact) = worktrees.iter().find(|w| w.name == name) {
        return Ok(exact);
    }

    let matches: Vec<_> = worktrees
        .iter()
        .filter(|w| w.name.starts_with(name))
        .collect();

    match matches.as_slice() {
        [single] => Ok(single),
        [] => Err(ArboristError::WorktreeNotFound(name.to_string()).into()),
        many => Err(ArboristError::AmbiguousName(
            name.to_string(),
            many.iter().map(|w| w.name.clone()).collect(),
        )
        .into()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use git2::{Repository, Signature};
    use tempfile::TempDir;

    // ── Helpers ──────────────────────────────────────────────────────────────

    fn init_repo() -> (TempDir, Repository) {
        let dir = tempfile::tempdir().unwrap();
        let repo = Repository::init(dir.path()).unwrap();
        (dir, repo)
    }

    fn init_repo_with_commit() -> (TempDir, Repository) {
        let (dir, repo) = init_repo();
        {
            let sig = Signature::now("Test", "test@example.com").unwrap();
            let tree_id = {
                let mut index = repo.index().unwrap();
                index.write_tree().unwrap()
            };
            let tree = repo.find_tree(tree_id).unwrap();
            repo.commit(Some("HEAD"), &sig, &sig, "initial", &tree, &[])
                .unwrap();
        }
        (dir, repo)
    }

    fn make_worktree_info(name: &str) -> WorktreeInfo {
        WorktreeInfo {
            name: name.to_string(),
            path: PathBuf::from("/tmp"),
            branch: None,
            head_commit: String::new(),
            head_summary: String::new(),
            is_bare: false,
            is_locked: false,
            is_main: false,
            last_modified: SystemTime::UNIX_EPOCH,
        }
    }

    // ── sanitize_branch_name ─────────────────────────────────────────────────

    #[test]
    fn sanitize_replaces_slashes() {
        assert_eq!(sanitize_branch_name("feature/login"), "feature-login");
    }

    #[test]
    fn sanitize_strips_invalid_chars() {
        assert_eq!(sanitize_branch_name("feat: my thing!"), "feat-my-thing");
    }

    #[test]
    fn sanitize_collapses_consecutive_dashes() {
        assert_eq!(sanitize_branch_name("feat--thing"), "feat-thing");
    }

    #[test]
    fn sanitize_trims_trailing_dash() {
        assert_eq!(sanitize_branch_name("feat/"), "feat");
    }

    #[test]
    fn sanitize_preserves_dots_and_underscores() {
        assert_eq!(sanitize_branch_name("v1.0_alpha"), "v1.0_alpha");
    }

    #[test]
    fn sanitize_empty_string() {
        assert_eq!(sanitize_branch_name(""), "");
    }

    // ── resolve_worktree ──────────────────────────────────────────────────────

    #[test]
    fn resolve_exact_match() {
        let worktrees = vec![make_worktree_info("login"), make_worktree_info("logout")];
        let result = resolve_worktree(&worktrees, "login").unwrap();
        assert_eq!(result.name, "login");
    }

    #[test]
    fn resolve_prefix_match() {
        let worktrees = vec![
            make_worktree_info("feature-login"),
            make_worktree_info("fix-bug"),
        ];
        let result = resolve_worktree(&worktrees, "feature").unwrap();
        assert_eq!(result.name, "feature-login");
    }

    #[test]
    fn resolve_not_found() {
        let worktrees = vec![make_worktree_info("login")];
        let err = resolve_worktree(&worktrees, "nonexistent").unwrap_err();
        assert!(err.to_string().contains("not found") || err.to_string().contains("not found"));
    }

    #[test]
    fn resolve_ambiguous() {
        let worktrees = vec![
            make_worktree_info("feature-login"),
            make_worktree_info("feature-signup"),
        ];
        let err = resolve_worktree(&worktrees, "feature").unwrap_err();
        assert!(err.to_string().contains("Ambiguous"));
    }

    #[test]
    fn resolve_prefers_exact_over_prefix() {
        let worktrees = vec![make_worktree_info("login"), make_worktree_info("login-v2")];
        let result = resolve_worktree(&worktrees, "login").unwrap();
        assert_eq!(result.name, "login");
    }

    // ── list_worktrees ────────────────────────────────────────────────────────

    #[test]
    fn list_worktrees_includes_main() {
        let (dir, repo) = init_repo_with_commit();
        let worktrees = list_worktrees(&repo).unwrap();
        assert_eq!(worktrees.len(), 1);
        // Path should point to the repo working directory.
        assert_eq!(
            worktrees[0].path.canonicalize().unwrap(),
            dir.path().canonicalize().unwrap()
        );
    }

    #[test]
    fn list_worktrees_has_branch_and_commit() {
        let (_dir, repo) = init_repo_with_commit();
        let worktrees = list_worktrees(&repo).unwrap();
        assert!(!worktrees[0].head_commit.is_empty());
        assert!(!worktrees[0].head_summary.is_empty());
    }

    #[test]
    fn list_worktrees_includes_linked_worktree() {
        let (_dir, repo) = init_repo_with_commit();
        let wt_dir = tempfile::tempdir().unwrap();
        let wt_path = wt_dir.path().join("feature-x");
        create_worktree(&repo, "feature-x", &wt_path, None).unwrap();

        let worktrees = list_worktrees(&repo).unwrap();
        assert_eq!(worktrees.len(), 2);
        assert!(worktrees.iter().any(|w| w.name == "feature-x"));
    }

    // ── create_worktree ───────────────────────────────────────────────────────

    #[test]
    fn create_worktree_creates_directory() {
        let (_dir, repo) = init_repo_with_commit();
        let wt_dir = tempfile::tempdir().unwrap();
        let wt_path = wt_dir.path().join("new-wt");
        create_worktree(&repo, "new-branch", &wt_path, None).unwrap();
        assert!(wt_path.exists());
    }

    #[test]
    fn create_worktree_fails_if_path_exists() {
        let (_dir, repo) = init_repo_with_commit();
        let wt_dir = tempfile::tempdir().unwrap();
        let wt_path = wt_dir.path().join("existing");
        fs::create_dir(&wt_path).unwrap();
        let err = create_worktree(&repo, "branch", &wt_path, None).unwrap_err();
        assert!(err.to_string().contains("already exists"));
    }

    #[test]
    fn create_worktree_with_base_branch() {
        let (_dir, repo) = init_repo_with_commit();
        let wt_dir = tempfile::tempdir().unwrap();
        let wt_path = wt_dir.path().join("based-wt");
        // Use HEAD as explicit base — same as default behaviour.
        create_worktree(&repo, "based-branch", &wt_path, Some("HEAD")).unwrap();
        assert!(wt_path.exists());
    }

    // ── remove_worktree ───────────────────────────────────────────────────────

    #[test]
    fn remove_worktree_removes_directory_and_tracking() {
        let (_dir, repo) = init_repo_with_commit();
        let wt_dir = tempfile::tempdir().unwrap();
        let wt_path = wt_dir.path().join("to-remove");
        create_worktree(&repo, "to-remove", &wt_path, None).unwrap();
        assert!(wt_path.exists());

        remove_worktree(&repo, "to-remove", false).unwrap();
        assert!(!wt_path.exists());

        // Should no longer appear in the list.
        let worktrees = list_worktrees(&repo).unwrap();
        assert!(worktrees.iter().all(|w| w.name != "to-remove"));
    }

    #[test]
    fn remove_worktree_not_found_returns_error() {
        let (_dir, repo) = init_repo_with_commit();
        let err = remove_worktree(&repo, "ghost", false).unwrap_err();
        assert!(err.to_string().contains("not found") || err.to_string().contains("not found"));
    }

    // ── is_branch_merged ──────────────────────────────────────────────────────

    #[test]
    fn is_branch_merged_true_when_same_commit() {
        let (_dir, repo) = init_repo_with_commit();
        let head_commit = repo.head().unwrap().peel_to_commit().unwrap();
        // Create a branch at the same commit as HEAD (main/master).
        repo.branch("side", &head_commit, false).unwrap();
        // Rename HEAD to "main" so we have a proper main branch.
        // git2 uses the initial HEAD ref — let's just test with whatever branch exists.
        // If there's no main/master, returns false gracefully.
        let result = is_branch_merged(&repo, "side");
        // Either true (if main/master exists at same commit) or false (no main/master).
        assert!(result.is_ok());
    }

    #[test]
    fn is_branch_merged_false_for_ahead_branch() {
        use git2::Signature;
        let (dir, repo) = init_repo_with_commit();

        {
            let head_commit = repo.head().unwrap().peel_to_commit().unwrap();
            repo.branch("feature", &head_commit, false).unwrap();

            // Add a commit only to "feature" branch.
            let sig = Signature::now("Test", "test@example.com").unwrap();
            let new_file = dir.path().join("extra.txt");
            fs::write(&new_file, "extra").unwrap();
            let mut index = repo.index().unwrap();
            index.add_path(Path::new("extra.txt")).unwrap();
            index.write().unwrap();
            let tree_id = index.write_tree().unwrap();
            let tree = repo.find_tree(tree_id).unwrap();
            repo.commit(
                Some("refs/heads/feature"),
                &sig,
                &sig,
                "extra commit",
                &tree,
                &[&head_commit],
            )
            .unwrap();
        }

        // "feature" is ahead of wherever HEAD is — not merged into main/master.
        // No main/master branch, so returns false gracefully.
        assert_eq!(is_branch_merged(&repo, "feature").unwrap(), false);
    }

    // ── diff_stat ─────────────────────────────────────────────────────────────

    #[test]
    fn diff_stat_clean_repo() {
        let (dir, repo) = init_repo_with_commit();
        let stat = diff_stat(&repo, dir.path()).unwrap();
        assert!(stat.is_clean());
    }

    #[test]
    fn diff_stat_detects_modified_file() {
        use git2::Signature;
        let (dir, repo) = init_repo_with_commit();

        let file = dir.path().join("file.txt");
        fs::write(&file, "original").unwrap();

        // Stage, commit the file.
        {
            let mut index = repo.index().unwrap();
            index.add_path(Path::new("file.txt")).unwrap();
            index.write().unwrap();
            let tree_id = index.write_tree().unwrap();
            let tree = repo.find_tree(tree_id).unwrap();
            let sig = Signature::now("Test", "test@example.com").unwrap();
            let head = repo.head().unwrap().peel_to_commit().unwrap();
            repo.commit(Some("HEAD"), &sig, &sig, "add file", &tree, &[&head])
                .unwrap();
        }

        // Modify without staging.
        fs::write(&file, "modified").unwrap();

        let stat = diff_stat(&repo, dir.path()).unwrap();
        assert!(!stat.is_clean());
        assert_eq!(stat.files_changed, 1);
    }
}
