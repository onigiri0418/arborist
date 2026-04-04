use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
use git2::Repository;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, clap::ValueEnum)]
#[serde(rename_all = "lowercase")]
pub enum WorktreeStatus {
    Active,
    Paused,
    Done,
}

impl Default for WorktreeStatus {
    fn default() -> Self {
        Self::Active
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct WorktreeMeta {
    pub task: Option<String>,
    pub memo: Option<String>,
    #[serde(default)]
    pub status: WorktreeStatus,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl WorktreeMeta {
    pub fn new(task: Option<String>, memo: Option<String>) -> Self {
        let now = Utc::now();
        Self {
            task,
            memo,
            status: WorktreeStatus::Active,
            created_at: now,
            updated_at: now,
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct MetaStore {
    #[serde(default = "default_version")]
    pub version: u8,
    #[serde(default)]
    pub worktrees: HashMap<String, WorktreeMeta>,
}

impl Default for MetaStore {
    fn default() -> Self {
        Self {
            version: 1,
            worktrees: HashMap::new(),
        }
    }
}

fn default_version() -> u8 {
    1
}

/// Load the metadata store from the repository's common directory.
/// Returns a default empty store if the file does not exist.
pub fn load(repo: &Repository) -> Result<MetaStore> {
    let path = meta_path(repo);
    if !path.exists() {
        return Ok(MetaStore::default());
    }
    let content = fs::read_to_string(&path)
        .with_context(|| format!("failed to read {}", path.display()))?;
    serde_json::from_str(&content)
        .with_context(|| format!("failed to parse {}", path.display()))
}

/// Atomically write the metadata store (tmp file + rename).
pub fn save(repo: &Repository, store: &MetaStore) -> Result<()> {
    let path = meta_path(repo);
    let tmp = path.with_extension("json.tmp");
    let content = serde_json::to_string_pretty(store)?;
    fs::write(&tmp, &content)
        .with_context(|| format!("failed to write {}", tmp.display()))?;
    fs::rename(&tmp, &path)
        .with_context(|| format!("failed to rename to {}", path.display()))?;
    Ok(())
}

fn meta_path(repo: &Repository) -> PathBuf {
    commondir(repo).join("arborist-meta.json")
}

/// Return the common git directory (shared across all worktrees).
/// For linked worktrees, reads the `commondir` file inside the gitdir.
/// For normal repos, returns `repo.path()` directly.
fn commondir(repo: &Repository) -> PathBuf {
    let git_dir = repo.path();
    let commondir_file = git_dir.join("commondir");
    if commondir_file.exists() {
        if let Ok(content) = fs::read_to_string(&commondir_file) {
            let relative = content.trim();
            let resolved = git_dir.join(relative);
            if let Ok(canonical) = resolved.canonicalize() {
                return canonical;
            }
        }
    }
    git_dir.to_path_buf()
}

/// Return a new MetaStore with the entry for `name` inserted or updated.
pub fn upsert_meta(mut store: MetaStore, name: &str, meta: WorktreeMeta) -> MetaStore {
    store.worktrees.insert(name.to_string(), meta);
    store
}

/// Return a new MetaStore with the entry for `name` removed.
pub fn remove_meta(mut store: MetaStore, name: &str) -> MetaStore {
    store.worktrees.remove(name);
    store
}

#[cfg(test)]
mod tests {
    use super::*;
    use git2::Repository;
    use tempfile::TempDir;

    fn make_meta() -> WorktreeMeta {
        WorktreeMeta::new(Some("test task".into()), None)
    }

    fn init_repo() -> (TempDir, Repository) {
        let dir = tempfile::tempdir().unwrap();
        let repo = Repository::init(dir.path()).unwrap();
        (dir, repo)
    }

    // --- load / save ---

    #[test]
    fn load_returns_default_when_file_missing() {
        let (_dir, repo) = init_repo();
        let store = load(&repo).unwrap();
        assert!(store.worktrees.is_empty());
        assert_eq!(store.version, 1);
    }

    #[test]
    fn load_parses_valid_json() {
        let (_dir, repo) = init_repo();
        let path = commondir(&repo).join("arborist-meta.json");
        let json = r#"{"version":1,"worktrees":{"feat":{"task":"do it","memo":null,"status":"active","created_at":"2024-01-01T00:00:00Z","updated_at":"2024-01-01T00:00:00Z"}}}"#;
        fs::write(&path, json).unwrap();

        let store = load(&repo).unwrap();
        assert!(store.worktrees.contains_key("feat"));
        assert_eq!(store.worktrees["feat"].task.as_deref(), Some("do it"));
    }

    #[test]
    fn load_returns_error_on_invalid_json() {
        let (_dir, repo) = init_repo();
        let path = commondir(&repo).join("arborist-meta.json");
        fs::write(&path, b"not valid json").unwrap();

        assert!(load(&repo).is_err());
    }

    #[test]
    fn save_creates_file() {
        let (_dir, repo) = init_repo();
        let store = MetaStore::default();
        save(&repo, &store).unwrap();

        let path = commondir(&repo).join("arborist-meta.json");
        assert!(path.exists());
    }

    #[test]
    fn save_and_load_roundtrip() {
        let (_dir, repo) = init_repo();
        let store = upsert_meta(MetaStore::default(), "my-branch", make_meta());
        save(&repo, &store).unwrap();

        let loaded = load(&repo).unwrap();
        assert!(loaded.worktrees.contains_key("my-branch"));
        assert_eq!(
            loaded.worktrees["my-branch"].task.as_deref(),
            Some("test task")
        );
    }

    #[test]
    fn save_overwrites_existing() {
        let (_dir, repo) = init_repo();
        let store1 = upsert_meta(MetaStore::default(), "old", make_meta());
        save(&repo, &store1).unwrap();

        let store2 = upsert_meta(MetaStore::default(), "new", make_meta());
        save(&repo, &store2).unwrap();

        let loaded = load(&repo).unwrap();
        assert!(!loaded.worktrees.contains_key("old"));
        assert!(loaded.worktrees.contains_key("new"));
    }

    // --- upsert / remove ---

    #[test]
    fn upsert_adds_new_entry() {
        let store = MetaStore::default();
        let store = upsert_meta(store, "feature/login", make_meta());
        assert!(store.worktrees.contains_key("feature/login"));
    }

    #[test]
    fn upsert_updates_existing_entry() {
        let store = upsert_meta(MetaStore::default(), "branch", make_meta());
        let updated_meta = WorktreeMeta::new(Some("new task".into()), Some("memo".into()));
        let store = upsert_meta(store, "branch", updated_meta);
        assert_eq!(store.worktrees["branch"].task.as_deref(), Some("new task"));
        assert_eq!(store.worktrees["branch"].memo.as_deref(), Some("memo"));
        assert_eq!(store.worktrees.len(), 1);
    }

    #[test]
    fn upsert_does_not_mutate_original() {
        let original = MetaStore::default();
        let updated = upsert_meta(MetaStore::default(), "branch", make_meta());
        assert!(original.worktrees.is_empty());
        assert_eq!(updated.worktrees.len(), 1);
    }

    #[test]
    fn remove_deletes_entry() {
        let store = upsert_meta(MetaStore::default(), "feature/login", make_meta());
        let store = remove_meta(store, "feature/login");
        assert!(!store.worktrees.contains_key("feature/login"));
    }

    #[test]
    fn remove_noop_when_missing() {
        let store = MetaStore::default();
        let store = remove_meta(store, "nonexistent");
        assert!(store.worktrees.is_empty());
    }
}
