//! Cache writer (PMAT-613 Phase 3).
//!
//! Emits `.pmat/bug-hunter-cache/pmat-<hash>.json` matching the batuta
//! `BugHunterCache` JSON shape. Downstream readers (enrichment.rs,
//! git_history_annotations) pick up the newest cache by mtime, so a pmat-
//! produced cache supersedes stale batuta output naturally.

use super::types::{BugHunterCache, Finding};
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use std::path::{Path, PathBuf};

/// Stable 64-bit hash of the canonicalized project root, rendered as 16 hex chars.
pub fn project_hash(project_root: &Path) -> String {
    let canonical = project_root
        .canonicalize()
        .unwrap_or_else(|_| project_root.to_path_buf());
    let mut hasher = DefaultHasher::new();
    canonical.hash(&mut hasher);
    format!("{:016x}", hasher.finish())
}

/// Destination path for a pmat-produced cache file.
pub fn cache_path(project_root: &Path) -> PathBuf {
    project_root
        .join(".pmat")
        .join("bug-hunter-cache")
        .join(format!("pmat-{}.json", project_hash(project_root)))
}

/// Write a cache file. Creates the directory if missing.
pub fn write_cache(
    project_root: &Path,
    findings: Vec<Finding>,
    mode: &str,
) -> std::io::Result<PathBuf> {
    let path = cache_path(project_root);
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let cache = BugHunterCache {
        findings,
        mode: mode.to_string(),
        config_hash: project_hash(project_root),
    };
    let json = serde_json::to_string(&cache)
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;
    std::fs::write(&path, json)?;
    Ok(path)
}

#[cfg(test)]
mod tests {
    use super::super::types::{DefectCategory, FindingSeverity};
    use super::*;

    #[test]
    fn project_hash_is_stable() {
        let h1 = project_hash(Path::new("."));
        let h2 = project_hash(Path::new("."));
        assert_eq!(h1, h2);
        assert_eq!(h1.len(), 16);
    }

    #[test]
    fn cache_path_uses_pmat_prefix() {
        let root = Path::new("/tmp/some-project");
        let p = cache_path(root);
        let name = p.file_name().unwrap().to_string_lossy().into_owned();
        assert!(name.starts_with("pmat-"), "got {name}");
        assert!(name.ends_with(".json"));
    }

    #[test]
    fn write_cache_roundtrips() {
        let tmp = tempfile::tempdir().unwrap();
        let finding = Finding {
            id: "BH-LOGIC-0001".to_string(),
            file: "src/foo.rs".to_string(),
            line: 42,
            column: None,
            title: "test".to_string(),
            description: "test".to_string(),
            severity: FindingSeverity::Medium,
            category: DefectCategory::LogicErrors,
            suspiciousness: 0.5,
            discovered_by: "Pattern".to_string(),
        };
        let path = write_cache(tmp.path(), vec![finding.clone()], "pmat-native").unwrap();
        assert!(path.exists());
        let read: BugHunterCache =
            serde_json::from_str(&std::fs::read_to_string(&path).unwrap()).unwrap();
        assert_eq!(read.findings.len(), 1);
        assert_eq!(read.findings[0].id, "BH-LOGIC-0001");
        assert_eq!(read.mode, "pmat-native");
    }
}
