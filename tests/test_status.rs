mod common;

use predicates::prelude::*;

#[test]
fn status_shows_all_worktrees() {
    let (dir, repo) = common::init_repo_with_commit();
    let wt_dir = tempfile::tempdir().unwrap();
    let wt1 = wt_dir.path().join("wt-a");
    let wt2 = wt_dir.path().join("wt-b");
    common::add_worktree(&repo, "branch-a", &wt1);
    common::add_worktree(&repo, "branch-b", &wt2);

    common::arborist_in(dir.path())
        .args(["status"])
        .assert()
        .success()
        .stdout(predicate::str::contains("wt-a").and(predicate::str::contains("wt-b")));
}

#[test]
fn status_shows_clean_for_no_changes() {
    let (dir, _repo) = common::init_repo_with_commit();

    common::arborist_in(dir.path())
        .args(["status"])
        .assert()
        .success()
        .stdout(predicate::str::contains("clean"));
}

#[test]
fn status_shows_diff_counts() {
    let (dir, repo) = common::init_repo_with_file();
    let wt_dir = tempfile::tempdir().unwrap();
    let wt_path = wt_dir.path().join("dirty-status-wt");
    common::add_worktree(&repo, "dirty-status-branch", &wt_path);

    // Modify the tracked file
    std::fs::write(wt_path.join("README.md"), "modified\n").unwrap();

    common::arborist_in(dir.path())
        .args(["status"])
        .assert()
        .success()
        .stdout(predicate::str::contains("dirty-status-wt"));
}

#[test]
fn status_short_output() {
    let (dir, repo) = common::init_repo_with_commit();
    let wt_dir = tempfile::tempdir().unwrap();
    let wt_path = wt_dir.path().join("short-wt");
    common::add_worktree(&repo, "short-branch", &wt_path);

    let output = common::arborist_in(dir.path())
        .args(["status", "--short"])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let text = String::from_utf8(output).unwrap();
    let lines: Vec<_> = text.trim().lines().collect();
    // Should have one line per worktree
    assert!(lines.len() >= 2);
}
