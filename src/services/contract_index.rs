//! ContractIndex: O(1) lookup from .pmat/binding-index.json
//!
//! Provides file→bindings and function→contract status for query enrichment.
//! Lazy-loaded on first access, cached for the process lifetime.
//!
//! Spec: commit-level-contract-enforcement.md Phase 6, R-9 remediation.

use std::collections::HashMap;
use std::path::{Path, PathBuf};

/// Contract binding status for a function or file.
#[derive(Debug, Clone)]
pub struct ContractBinding {
    pub binding_names: Vec<String>,
}

/// Index of file→contract bindings, loaded from .pmat/binding-index.json.
#[derive(Debug)]
pub struct ContractIndex {
    /// file_path → binding names
    file_bindings: HashMap<String, Vec<String>>,
    /// Total binding count
    pub total_bindings: usize,
    /// Total files with bindings
    pub total_files: usize,
}

impl ContractIndex {
    /// Load from .pmat/binding-index.json. Returns None if file doesn't exist.
    pub fn load(project_path: &Path) -> Option<Self> {
        debug_assert!(
            project_path.exists(),
            "project_path must exist: {}",
            project_path.display()
        );
        let idx_path = Self::find_index_path(project_path)?;
        let content = std::fs::read_to_string(&idx_path).ok()?;
        let parsed: serde_json::Value = serde_json::from_str(&content).ok()?;
        let obj = parsed.as_object()?;

        let mut file_bindings = HashMap::with_capacity(obj.len());
        let mut total_bindings = 0usize;

        for (file, bindings) in obj {
            if let Some(arr) = bindings.as_array() {
                let names: Vec<String> = arr
                    .iter()
                    .filter_map(|v| {
                        v.as_str().map(|s| s.to_string()).or_else(|| {
                            v.get("name")
                                .and_then(|n| n.as_str())
                                .map(|s| s.to_string())
                        })
                    })
                    .collect();
                total_bindings += names.len();
                file_bindings.insert(file.clone(), names);
            }
        }

        let total_files = file_bindings.len();
        Some(Self {
            file_bindings,
            total_bindings,
            total_files,
        })
    }

    /// Find the binding-index.json path (checks .pmat/ and contracts/).
    fn find_index_path(project_path: &Path) -> Option<PathBuf> {
        let primary = project_path.join(".pmat/binding-index.json");
        if primary.exists() {
            return Some(primary);
        }
        let alt = project_path.join("contracts/binding-index.json");
        if alt.exists() {
            return Some(alt);
        }
        None
    }

    /// Check if a file has contract bindings.
    pub fn has_bindings(&self, file_path: &str) -> bool {
        self.file_bindings.contains_key(file_path)
    }

    /// Get bindings for a file. Returns empty slice if none.
    pub fn get_bindings(&self, file_path: &str) -> &[String] {
        self.file_bindings
            .get(file_path)
            .map(|v| v.as_slice())
            .unwrap_or(&[])
    }

    /// Get all files that have NO contract bindings from a list of file paths.
    /// Used by --contract-gaps to find undercontracted code.
    pub fn find_gaps<'a>(&self, files: &'a [String]) -> Vec<&'a String> {
        files
            .iter()
            .filter(|f| !self.file_bindings.contains_key(f.as_str()))
            .collect()
    }

    /// Get all files WITH bindings.
    pub fn bound_files(&self) -> impl Iterator<Item = (&str, &[String])> {
        self.file_bindings
            .iter()
            .map(|(k, v)| (k.as_str(), v.as_slice()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_load_empty_index() {
        let dir = tempdir().unwrap();
        let pmat = dir.path().join(".pmat");
        std::fs::create_dir(&pmat).unwrap();
        std::fs::write(pmat.join("binding-index.json"), "{}").unwrap();

        let idx = ContractIndex::load(dir.path()).unwrap();
        assert_eq!(idx.total_bindings, 0);
        assert_eq!(idx.total_files, 0);
    }

    #[test]
    fn test_load_with_bindings() {
        let dir = tempdir().unwrap();
        let pmat = dir.path().join(".pmat");
        std::fs::create_dir(&pmat).unwrap();
        std::fs::write(
            pmat.join("binding-index.json"),
            r#"{"src/lib.rs": ["validate_input", "parse_config"], "src/util.rs": ["hash"]}"#,
        )
        .unwrap();

        let idx = ContractIndex::load(dir.path()).unwrap();
        assert_eq!(idx.total_bindings, 3);
        assert_eq!(idx.total_files, 2);
        assert!(idx.has_bindings("src/lib.rs"));
        assert!(!idx.has_bindings("src/main.rs"));
        assert_eq!(idx.get_bindings("src/lib.rs").len(), 2);
    }

    #[test]
    fn test_find_gaps() {
        let dir = tempdir().unwrap();
        let pmat = dir.path().join(".pmat");
        std::fs::create_dir(&pmat).unwrap();
        std::fs::write(
            pmat.join("binding-index.json"),
            r#"{"src/lib.rs": ["validate"]}"#,
        )
        .unwrap();

        let idx = ContractIndex::load(dir.path()).unwrap();
        let files = vec![
            "src/lib.rs".to_string(),
            "src/main.rs".to_string(),
            "src/util.rs".to_string(),
        ];
        let gaps = idx.find_gaps(&files);
        assert_eq!(gaps.len(), 2);
        assert!(gaps.contains(&&"src/main.rs".to_string()));
        assert!(gaps.contains(&&"src/util.rs".to_string()));
    }

    #[test]
    fn test_load_nonexistent() {
        let dir = tempdir().unwrap();
        assert!(ContractIndex::load(dir.path()).is_none());
    }
}
