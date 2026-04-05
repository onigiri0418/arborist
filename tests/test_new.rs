mod common;

use predicates::prelude::*;

#[test]
fn new_creates_worktree() {
    let (dir, _repo) = common::init_repo_with_commit();
    let wt_dir = tempfile::tempdir().unwrap();
    let wt_path = wt_dir.path().join("new-wt");

    common::arborist_in(dir.path())
        .args(["new", "new-branch", "--path", wt_path.to_str().unwrap()])
        .assert()
        .success()
        .stdout(predicate::str::contains(wt_path.to_str().unwrap()));

    assert!(wt_path.exists());
}

#[test]
fn new_with_task_saves_meta() {
    let (dir, _repo) = common::init_repo_with_commit();
    let wt_dir = tempfile::tempdir().unwrap();
    let wt_path = wt_dir.path().join("task-wt");

    common::arborist_in(dir.path())
        .args([
            "new",
            "task-branch",
            "--path",
            wt_path.to_str().unwrap(),
            "--task",
            "Build login page",
        ])
        .assert()
        .success();

    // Verify meta was saved
    common::arborist_in(dir.path())
        .args(["tag", "task-wt"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Build login page"));
}

#[test]
fn new_with_explicit_path() {
    let (dir, _repo) = common::init_repo_with_commit();
    let wt_dir = tempfile::tempdir().unwrap();
    let wt_path = wt_dir.path().join("custom-wt");

    common::arborist_in(dir.path())
        .args(["new", "custom-branch", "--path", wt_path.to_str().unwrap()])
        .assert()
        .success();

    assert!(wt_path.exists());
}

#[test]
fn new_fails_if_path_exists() {
    let (dir, _repo) = common::init_repo_with_commit();
    let wt_dir = tempfile::tempdir().unwrap();
    let wt_path = wt_dir.path().join("existing");
    std::fs::create_dir(&wt_path).unwrap();

    common::arborist_in(dir.path())
        .args(["new", "some-branch", "--path", wt_path.to_str().unwrap()])
        .assert()
        .failure()
        .stderr(predicate::str::contains("already exists"));
}

#[test]
fn new_creates_branch_from_base() {
    let (dir, _repo) = common::init_repo_with_commit();
    let wt_dir = tempfile::tempdir().unwrap();
    let wt_path = wt_dir.path().join("based-wt");

    common::arborist_in(dir.path())
        .args([
            "new",
            "based-branch",
            "--path",
            wt_path.to_str().unwrap(),
            "--base",
            "HEAD",
        ])
        .assert()
        .success();

    assert!(wt_path.exists());
}

#[test]
fn new_sanitizes_branch_in_default_path() {
    let (dir, _repo) = common::init_repo_with_commit();

    common::arborist_in(dir.path())
        .args(["new", "feature/x"])
        .assert()
        .success()
        .stdout(predicate::str::contains("feature-x"));
}
