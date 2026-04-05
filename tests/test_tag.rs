mod common;

use predicates::prelude::*;

fn setup() -> (tempfile::TempDir, git2::Repository, std::path::PathBuf, tempfile::TempDir) {
    let (dir, repo) = common::init_repo_with_commit();
    let wt_dir = tempfile::tempdir().unwrap();
    let wt_path = wt_dir.path().join("my-wt");
    common::add_worktree(&repo, "my-branch", &wt_path);
    (dir, repo, wt_path, wt_dir)
}

#[test]
fn tag_sets_task() {
    let (dir, _repo, _wt_path, _wt_dir) = setup();

    common::arborist_in(dir.path())
        .args(["tag", "my-wt", "--task", "Build feature"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Build feature"));
}

#[test]
fn tag_sets_memo() {
    let (dir, _repo, _wt_path, _wt_dir) = setup();

    common::arborist_in(dir.path())
        .args(["tag", "my-wt", "--memo", "Remember this"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Remember this"));
}

#[test]
fn tag_sets_status() {
    let (dir, _repo, _wt_path, _wt_dir) = setup();

    common::arborist_in(dir.path())
        .args(["tag", "my-wt", "--status", "done"])
        .assert()
        .success()
        .stdout(predicate::str::contains("done"));
}

#[test]
fn tag_display_existing_meta() {
    let (dir, _repo, _wt_path, _wt_dir) = setup();

    // Set meta first
    common::arborist_in(dir.path())
        .args(["tag", "my-wt", "--task", "My task", "--memo", "Some memo"])
        .assert()
        .success();

    // Display without flags
    common::arborist_in(dir.path())
        .args(["tag", "my-wt"])
        .assert()
        .success()
        .stdout(
            predicate::str::contains("My task")
                .and(predicate::str::contains("Some memo")),
        );
}

#[test]
fn tag_partial_update_preserves_other_fields() {
    let (dir, _repo, _wt_path, _wt_dir) = setup();

    // Set task and memo
    common::arborist_in(dir.path())
        .args(["tag", "my-wt", "--task", "Original task", "--memo", "Original memo"])
        .assert()
        .success();

    // Update only status
    common::arborist_in(dir.path())
        .args(["tag", "my-wt", "--status", "paused"])
        .assert()
        .success();

    // Display — task and memo should still be there
    common::arborist_in(dir.path())
        .args(["tag", "my-wt"])
        .assert()
        .success()
        .stdout(
            predicate::str::contains("Original task")
                .and(predicate::str::contains("Original memo"))
                .and(predicate::str::contains("paused")),
        );
}

#[test]
fn tag_not_found() {
    let (dir, _repo) = common::init_repo_with_commit();

    common::arborist_in(dir.path())
        .args(["tag", "nonexistent"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("not found"));
}

#[test]
fn tag_updated_at_changes() {
    let (dir, _repo, _wt_path, _wt_dir) = setup();

    common::arborist_in(dir.path())
        .args(["tag", "my-wt", "--task", "First task"])
        .assert()
        .success();

    // Small delay to ensure timestamp differs
    std::thread::sleep(std::time::Duration::from_millis(10));

    common::arborist_in(dir.path())
        .args(["tag", "my-wt", "--task", "Second task"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Second task"));

    // Verify current display shows updated task
    common::arborist_in(dir.path())
        .args(["tag", "my-wt"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Second task"));
}
