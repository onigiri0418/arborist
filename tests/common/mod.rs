use git2::{Repository, Signature};
use tempfile::TempDir;

/// Create an empty git repository (no commits).
pub fn init_repo() -> (TempDir, Repository) {
    let dir = tempfile::tempdir().unwrap();
    let repo = Repository::init(dir.path()).unwrap();
    (dir, repo)
}

/// Create a git repository with one initial commit (empty tree).
/// A minimum of one commit is required to use worktrees.
pub fn init_repo_with_commit() -> (TempDir, Repository) {
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

/// Create a git repository with one initial commit that includes a README.md.
/// Useful for testing dirty-worktree scenarios (modify the tracked file).
pub fn init_repo_with_file() -> (TempDir, Repository) {
    let (dir, repo) = init_repo();
    {
        let sig = Signature::now("Test", "test@example.com").unwrap();
        std::fs::write(dir.path().join("README.md"), "# Test\n").unwrap();
        let tree_id = {
            let mut index = repo.index().unwrap();
            index.add_path(std::path::Path::new("README.md")).unwrap();
            index.write().unwrap();
            index.write_tree().unwrap()
        };
        let tree = repo.find_tree(tree_id).unwrap();
        repo.commit(Some("HEAD"), &sig, &sig, "initial", &tree, &[])
            .unwrap();
    }
    (dir, repo)
}

/// Create a linked worktree using git2 directly (for test setup).
/// The worktree name is derived from `path.file_name()`.
pub fn add_worktree(repo: &Repository, branch: &str, path: &std::path::Path) {
    let commit = repo.head().unwrap().peel_to_commit().unwrap();
    if repo.find_branch(branch, git2::BranchType::Local).is_err() {
        repo.branch(branch, &commit, false).unwrap();
    }
    let refname = format!("refs/heads/{branch}");
    let reference = repo.find_reference(&refname).unwrap();
    let name = path.file_name().and_then(|n| n.to_str()).unwrap_or(branch);
    let mut opts = git2::WorktreeAddOptions::new();
    opts.reference(Some(&reference));
    repo.worktree(name, path, Some(&opts)).unwrap();
}

/// Add an extra commit to the worktree at `path` (makes the branch "ahead" of its base).
pub fn commit_in_worktree(path: &std::path::Path) {
    let repo = Repository::open(path).unwrap();
    let sig = Signature::now("Test", "test@example.com").unwrap();
    std::fs::write(path.join("extra.txt"), "extra").unwrap();
    let tree_id = {
        let mut index = repo.index().unwrap();
        index.add_path(std::path::Path::new("extra.txt")).unwrap();
        index.write().unwrap();
        index.write_tree().unwrap()
    };
    let tree = repo.find_tree(tree_id).unwrap();
    let parent = repo.head().unwrap().peel_to_commit().unwrap();
    repo.commit(Some("HEAD"), &sig, &sig, "extra commit", &tree, &[&parent])
        .unwrap();
}

/// Return an `assert_cmd::Command` for the arborist binary,
/// with the current directory set to `dir`.
pub fn arborist_in(dir: &std::path::Path) -> assert_cmd::Command {
    let mut cmd = assert_cmd::Command::cargo_bin("arborist").unwrap();
    cmd.current_dir(dir);
    cmd
}
