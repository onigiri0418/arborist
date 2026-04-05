use git2::{Repository, Signature};
use tempfile::TempDir;

/// Create an empty git repository (no commits).
pub fn init_repo() -> (TempDir, Repository) {
    let dir = tempfile::tempdir().unwrap();
    let repo = Repository::init(dir.path()).unwrap();
    (dir, repo)
}

/// Create a git repository with one initial commit.
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

/// Return an `assert_cmd::Command` for the arborist binary,
/// with the current directory set to `dir`.
pub fn arborist_in(dir: &std::path::Path) -> assert_cmd::Command {
    let mut cmd = assert_cmd::Command::cargo_bin("arborist").unwrap();
    cmd.current_dir(dir);
    cmd
}
