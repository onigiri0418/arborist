mod common;

use predicates::prelude::*;

#[test]
fn rm_removes_worktree() {
    let (dir, repo) = common::init_repo_with_commit();
    let wt_dir = tempfile::tempdir().unwrap();
    let wt_path = wt_dir.path().join("to-remove");
    common::add_worktree(&repo, "to-remove", &wt_path);
    assert!(wt_path.exists());

    common::arborist_in(dir.path())
        .args(["rm", "to-remove"])
        .assert()
        .success();

    assert!(!wt_path.exists());
}

#[test]
fn rm_removes_meta_entry() {
    let (dir, repo) = common::init_repo_with_commit();
    let wt_dir = tempfile::tempdir().unwrap();
    let wt_path = wt_dir.path().join("tagged-wt");
    common::add_worktree(&repo, "tagged-branch", &wt_path);

    // Tag it first
    common::arborist_in(dir.path())
        .args(["tag", "tagged-wt", "--task", "Some task"])
        .assert()
        .success();

    // Remove it
    common::arborist_in(dir.path())
        .args(["rm", "tagged-wt"])
        .assert()
        .success();

    // Meta should be gone (list should not mention it)
    common::arborist_in(dir.path())
        .args(["list"])
        .assert()
        .success()
        .stdout(predicate::str::contains("tagged-wt").not());
}

#[test]
fn rm_not_found() {
    let (dir, _repo) = common::init_repo_with_commit();

    common::arborist_in(dir.path())
        .args(["rm", "nonexistent"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("not found"));
}

#[test]
fn rm_ambiguous_name() {
    let (dir, repo) = common::init_repo_with_commit();
    let wt_dir = tempfile::tempdir().unwrap();
    let wt1 = wt_dir.path().join("feature-login");
    let wt2 = wt_dir.path().join("feature-signup");
    common::add_worktree(&repo, "feature-login", &wt1);
    common::add_worktree(&repo, "feature-signup", &wt2);

    common::arborist_in(dir.path())
        .args(["rm", "feature"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("Ambiguous").or(predicate::str::contains("ambiguous")));
}

#[test]
fn rm_dirty_without_force() {
    let (dir, repo) = common::init_repo_with_file();
    let wt_dir = tempfile::tempdir().unwrap();
    let wt_path = wt_dir.path().join("dirty-wt");
    common::add_worktree(&repo, "dirty-branch", &wt_path);

    // Modify the tracked file (README.md was committed in init_repo_with_file)
    std::fs::write(wt_path.join("README.md"), "modified content").unwrap();

    common::arborist_in(dir.path())
        .args(["rm", "dirty-wt"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("uncommitted changes"));

    assert!(wt_path.exists());
}

#[test]
fn rm_dirty_with_force() {
    let (dir, repo) = common::init_repo_with_file();
    let wt_dir = tempfile::tempdir().unwrap();
    let wt_path = wt_dir.path().join("dirty-force-wt");
    common::add_worktree(&repo, "dirty-force-branch", &wt_path);

    std::fs::write(wt_path.join("README.md"), "modified content").unwrap();

    common::arborist_in(dir.path())
        .args(["rm", "--force", "dirty-force-wt"])
        .assert()
        .success();

    assert!(!wt_path.exists());
}

#[test]
fn rm_current_worktree() {
    let (dir, repo) = common::init_repo_with_commit();
    let wt_dir = tempfile::tempdir().unwrap();
    let wt_path = wt_dir.path().join("current-wt");
    common::add_worktree(&repo, "current-branch", &wt_path);
    let _ = dir; // keep dir alive

    // Run arborist from inside the worktree
    common::arborist_in(&wt_path)
        .args(["rm", "current-wt"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("current worktree"));
}
