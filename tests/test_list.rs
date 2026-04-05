mod common;

use predicates::prelude::*;

#[test]
fn list_shows_main_worktree() {
    let (dir, _repo) = common::init_repo_with_commit();
    let repo_name = dir
        .path()
        .file_name()
        .unwrap()
        .to_str()
        .unwrap()
        .to_string();
    common::arborist_in(dir.path())
        .args(["list"])
        .assert()
        .success()
        .stdout(predicate::str::contains(repo_name));
}

#[test]
fn list_with_linked_worktree() {
    let (dir, repo) = common::init_repo_with_commit();
    let wt_dir = tempfile::tempdir().unwrap();
    let wt_path = wt_dir.path().join("feature-x");
    common::add_worktree(&repo, "feature-x", &wt_path);

    common::arborist_in(dir.path())
        .args(["list"])
        .assert()
        .success()
        .stdout(predicate::str::contains("feature-x"));
}

#[test]
fn list_json_output_is_valid_json_array() {
    let (dir, repo) = common::init_repo_with_commit();
    let wt_dir = tempfile::tempdir().unwrap();
    let wt_path = wt_dir.path().join("my-wt");
    common::add_worktree(&repo, "my-branch", &wt_path);

    let output = common::arborist_in(dir.path())
        .args(["list", "--json"])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let text = String::from_utf8(output).unwrap();
    let parsed: serde_json::Value = serde_json::from_str(&text).unwrap();
    assert!(parsed.is_array());
    assert_eq!(parsed.as_array().unwrap().len(), 2);
    // First entry should have expected fields
    let first = &parsed[0];
    assert!(first.get("name").is_some());
    assert!(first.get("path").is_some());
    assert!(first.get("is_main").is_some());
}

#[test]
fn list_short_output_shows_paths() {
    let (dir, _repo) = common::init_repo_with_commit();
    let output = common::arborist_in(dir.path())
        .args(["list", "--short"])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let text = String::from_utf8(output).unwrap();
    // Each line should look like a path (contains the repo dir)
    assert!(text.trim().contains(dir.path().to_str().unwrap()));
}

#[test]
fn list_outside_repo_fails() {
    let dir = tempfile::tempdir().unwrap();
    common::arborist_in(dir.path())
        .args(["list"])
        .assert()
        .failure();
}
