use std::collections::HashMap;

use anyhow::Result;
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

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct MetaStore {
    #[serde(default = "default_version")]
    pub version: u8,
    #[serde(default)]
    pub worktrees: HashMap<String, WorktreeMeta>,
}

fn default_version() -> u8 {
    1
}

/// Load the metadata store from the repository's common directory.
/// Returns a default empty store if the file does not exist.
pub fn load(_repo: &Repository) -> Result<MetaStore> {
    // TODO: resolve path from repo.commondir() and read JSON
    Ok(MetaStore::default())
}

/// Atomically write the metadata store (tmp file + rename).
pub fn save(_repo: &Repository, _store: &MetaStore) -> Result<()> {
    // TODO: implement atomic write
    Ok(())
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

    fn make_meta() -> WorktreeMeta {
        WorktreeMeta::new(Some("test task".into()), None)
    }

    #[test]
    fn upsert_adds_entry() {
        let store = MetaStore::default();
        let store = upsert_meta(store, "feature/login", make_meta());
        assert!(store.worktrees.contains_key("feature/login"));
    }

    #[test]
    fn remove_deletes_entry() {
        let store = MetaStore::default();
        let store = upsert_meta(store, "feature/login", make_meta());
        let store = remove_meta(store, "feature/login");
        assert!(!store.worktrees.contains_key("feature/login"));
    }

    #[test]
    fn upsert_is_immutable_to_original() {
        let original = MetaStore::default();
        let updated = upsert_meta(MetaStore::default(), "branch", make_meta());
        assert!(original.worktrees.is_empty());
        assert_eq!(updated.worktrees.len(), 1);
    }
}
