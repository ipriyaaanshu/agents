use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

/// A single entry in the registry index (JSONL format).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IndexEntry {
    pub name: String,
    #[serde(rename = "vers")]
    pub version: String,
    #[serde(rename = "cksum")]
    pub checksum: String,
    #[serde(default)]
    pub yanked: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub keywords: Vec<String>,
}

/// Git-based registry index, modeled after crates.io.
pub struct RegistryIndex {
    index_path: PathBuf,
}

impl RegistryIndex {
    /// Open an existing local index or clone from remote.
    pub fn open(path: impl Into<PathBuf>) -> Self {
        Self {
            index_path: path.into(),
        }
    }

    /// Get the path prefix for a skill name (2-char prefix dirs like crates.io).
    fn name_prefix(name: &str) -> PathBuf {
        match name.len() {
            1 => PathBuf::from("1"),
            2 => PathBuf::from("2"),
            3 => PathBuf::from("3").join(&name[..1]),
            _ => PathBuf::from(&name[..2]).join(&name[2..4.min(name.len())]),
        }
    }

    /// Get path to the index file for a skill.
    fn skill_index_path(&self, name: &str) -> PathBuf {
        self.index_path.join(Self::name_prefix(name)).join(name)
    }

    /// Look up all versions of a skill.
    pub fn lookup(&self, name: &str) -> Result<Vec<IndexEntry>> {
        let path = self.skill_index_path(name);
        if !path.exists() {
            return Ok(Vec::new());
        }

        let content = std::fs::read_to_string(&path)?;
        let entries: Vec<IndexEntry> = content
            .lines()
            .filter(|l| !l.is_empty())
            .map(serde_json::from_str)
            .collect::<std::result::Result<Vec<_>, _>>()?;

        Ok(entries)
    }

    /// Look up a specific version.
    pub fn lookup_version(&self, name: &str, version: &str) -> Result<Option<IndexEntry>> {
        let entries = self.lookup(name)?;
        Ok(entries.into_iter().find(|e| e.version == version))
    }

    /// Get the latest non-yanked version.
    pub fn latest(&self, name: &str) -> Result<Option<IndexEntry>> {
        let entries = self.lookup(name)?;
        Ok(entries.into_iter().rev().find(|e| !e.yanked))
    }

    /// Search entries by query (name or keyword match).
    pub fn search(&self, query: &str, limit: usize) -> Result<Vec<IndexEntry>> {
        let mut results = Vec::new();
        let query_lower = query.to_lowercase();

        // Walk the index directory
        Self::walk_index(&self.index_path, &mut |entry: IndexEntry| {
            if results.len() >= limit {
                return;
            }
            if entry.name.contains(&query_lower)
                || entry
                    .keywords
                    .iter()
                    .any(|k| k.to_lowercase().contains(&query_lower))
                || entry
                    .description
                    .as_ref()
                    .map(|d| d.to_lowercase().contains(&query_lower))
                    .unwrap_or(false)
            {
                results.push(entry);
            }
        })?;

        Ok(results)
    }

    /// Add an entry to the index.
    pub fn add_entry(&self, entry: &IndexEntry) -> Result<()> {
        let path = self.skill_index_path(&entry.name);
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }

        let mut file = std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(&path)?;

        use std::io::Write;
        writeln!(file, "{}", serde_json::to_string(entry)?)?;
        Ok(())
    }

    fn walk_index(dir: &Path, callback: &mut dyn FnMut(IndexEntry)) -> Result<()> {
        if !dir.exists() {
            return Ok(());
        }

        for entry in std::fs::read_dir(dir)? {
            let entry = entry?;
            let path = entry.path();
            if path.is_dir() {
                // Skip hidden dirs (.git, etc.)
                if path
                    .file_name()
                    .map(|n| n.to_string_lossy().starts_with('.'))
                    .unwrap_or(false)
                {
                    continue;
                }
                Self::walk_index(&path, callback)?;
            } else if path.is_file() {
                if let Ok(content) = std::fs::read_to_string(&path) {
                    for line in content.lines() {
                        if let Ok(idx_entry) = serde_json::from_str::<IndexEntry>(line) {
                            callback(idx_entry);
                        }
                    }
                }
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_name_prefix() {
        assert_eq!(RegistryIndex::name_prefix("a"), PathBuf::from("1"));
        assert_eq!(RegistryIndex::name_prefix("ab"), PathBuf::from("2"));
        assert_eq!(
            RegistryIndex::name_prefix("abc"),
            PathBuf::from("3").join("a")
        );
        assert_eq!(
            RegistryIndex::name_prefix("file-ops"),
            PathBuf::from("fi").join("le")
        );
    }
}
