mod common;

use predicates::prelude::*;

#[test]
fn clean_nothing_to_clean() {
    let (dir, repo) = common::init_repo_with_commit();
    let wt_dir = tempfile::tempdir().unwrap();
    let wt_path = wt_dir.path().join("active-wt");
    common::add_worktree(&repo, "active-branch", &wt_path);
    // Add extra commit to make it "not merged"
    common::commit_in_worktree(&wt_path);

    common::arborist_in(dir.path())
        .args(["clean", "--dry-run"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Nothing"));
}

#[test]
fn clean_dry_run_shows_candidates() {
    let (dir, repo) = common::init_repo_with_commit();
    let wt_dir = tempfile::tempdir().unwrap();
    let wt_path = wt_dir.path().join("done-dry-wt");
    common::add_worktree(&repo, "done-dry-branch", &wt_path);

    // Tag as done
    common::arborist_in(dir.path())
        .args(["tag", "done-dry-wt", "--status", "done"])
        .assert()
        .success();

    // Dry-run should show candidate without removing
    common::arborist_in(dir.path())
        .args(["clean", "--dry-run"])
        .assert()
        .success()
        .stdout(predicate::str::contains("done-dry-wt"));

    assert!(wt_path.exists());
}

#[test]
fn clean_removes_done_status_worktrees() {
    let (dir, repo) = common::init_repo_with_commit();
    let wt_dir = tempfile::tempdir().unwrap();
    let wt_path = wt_dir.path().join("done-wt");
    common::add_worktree(&repo, "done-branch", &wt_path);

    common::arborist_in(dir.path())
        .args(["tag", "done-wt", "--status", "done"])
        .assert()
        .success();

    common::arborist_in(dir.path())
        .args(["clean", "--all"])
        .assert()
        .success();

    assert!(!wt_path.exists());
}

#[test]
fn clean_removes_merged_worktrees() {
    let (dir, repo) = common::init_repo_with_commit();

    // Ensure a "main" branch exists for is_branch_merged
    let commit = repo.head().unwrap().peel_to_commit().unwrap();
    if repo.find_branch("main", git2::BranchType::Local).is_err() {
        repo.branch("main", &commit, false).unwrap();
    }

    let wt_dir = tempfile::tempdir().unwrap();
    let wt_path = wt_dir.path().join("merged-wt");
    // Branch created at same commit as main → it's "merged"
    common::add_worktree(&repo, "merged-branch", &wt_path);

    common::arborist_in(dir.path())
        .args(["clean", "--all"])
        .assert()
        .success();

    assert!(!wt_path.exists());
}

#[test]
fn clean_skips_main_worktree() {
    let (dir, _repo) = common::init_repo_with_commit();

    // Even with --all, main worktree should never be removed
    common::arborist_in(dir.path())
        .args(["clean", "--all"])
        .assert()
        .success();

    assert!(dir.path().exists());
}

#[test]
fn clean_skips_unmerged_active() {
    let (dir, repo) = common::init_repo_with_commit();

    // Ensure main branch exists
    let commit = repo.head().unwrap().peel_to_commit().unwrap();
    if repo.find_branch("main", git2::BranchType::Local).is_err() {
        repo.branch("main", &commit, false).unwrap();
    }

    let wt_dir = tempfile::tempdir().unwrap();
    let wt_path = wt_dir.path().join("unmerged-wt");
    common::add_worktree(&repo, "unmerged-branch", &wt_path);
    // Extra commit makes it "not merged" into main
    common::commit_in_worktree(&wt_path);

    // Status is active (default) and branch is unmerged → should not be cleaned
    common::arborist_in(dir.path())
        .args(["clean", "--all"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Nothing"));

    assert!(wt_path.exists());
}
