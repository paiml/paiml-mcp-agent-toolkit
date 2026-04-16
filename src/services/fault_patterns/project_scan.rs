//! Project-wide scanner (PMAT-613 Phase 4).
//!
//! Walks the project respecting `.gitignore`, runs `scanner::scan_file` on
//! each Rust source file, and writes the aggregated findings to the bug-
//! hunter cache. Called lazily by `enrich_results_with_faults` when no cache
//! is present, so normal queries pay zero cost.

use super::scanner::scan_file;
use super::taxonomy::active_rules;
use super::types::Finding;
use super::writer::write_cache;
use ignore::WalkBuilder;
use std::path::{Path, PathBuf};

/// Scan every Rust file under `project_root` and write a cache file.
///
/// Returns the path to the cache written and the findings count.
pub fn scan_and_cache(project_root: &Path) -> std::io::Result<(PathBuf, usize)> {
    let findings = scan_project(project_root);
    let count = findings.len();
    let path = write_cache(project_root, findings, "pmat-native")?;
    Ok((path, count))
}

/// Collect findings across the project without writing a cache.
pub fn scan_project(project_root: &Path) -> Vec<Finding> {
    let rules = active_rules(pmat_satd_active());
    let mut all: Vec<Finding> = Vec::new();
    let mut finding_counter = 0usize;

    for entry in WalkBuilder::new(project_root)
        .hidden(true)
        .git_ignore(true)
        .git_global(true)
        .build()
        .filter_map(|e| e.ok())
    {
        let path = entry.path();
        if !path.is_file() {
            continue;
        }
        if path.extension().and_then(|e| e.to_str()) != Some("rs") {
            continue;
        }
        let Ok(content) = std::fs::read_to_string(path) else {
            continue;
        };
        let rel = path
            .strip_prefix(project_root)
            .unwrap_or(path)
            .to_string_lossy()
            .into_owned();

        let mut file_findings = scan_file(&rel, &content, &rules);
        // Re-sequence IDs globally so they are unique across the project.
        for f in &mut file_findings {
            finding_counter += 1;
            f.id = reseq_id(&f.id, finding_counter);
        }
        all.extend(file_findings);
    }
    all
}

fn reseq_id(original: &str, counter: usize) -> String {
    if let Some((prefix, _)) = original.rsplit_once('-') {
        format!("{prefix}-{counter:04}")
    } else {
        format!("BH-UNK-{counter:04}")
    }
}

fn pmat_satd_active() -> bool {
    std::env::var("PMAT_SATD_ANALYZER")
        .map(|v| matches!(v.as_str(), "1" | "true" | "on"))
        .unwrap_or(false)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reseq_id_rewrites_suffix() {
        assert_eq!(reseq_id("BH-LOGIC-0001", 7), "BH-LOGIC-0007");
        assert_eq!(reseq_id("BH-MEM-9999", 1), "BH-MEM-0001");
    }

    #[test]
    fn scan_project_on_tempdir_finds_findings() {
        let tmp = tempfile::tempdir().unwrap();
        let rust_file = tmp.path().join("lib.rs");
        std::fs::write(
            &rust_file,
            "fn main() { let x: Result<u32, ()> = Ok(1); let _ = x.unwrap(); }",
        )
        .unwrap();
        let findings = scan_project(tmp.path());
        assert!(
            !findings.is_empty(),
            "expected at least one finding for unwrap"
        );
        assert!(findings.iter().all(|f| !f.id.is_empty()));
    }

    #[test]
    fn scan_and_cache_writes_file() {
        let tmp = tempfile::tempdir().unwrap();
        let rust_file = tmp.path().join("lib.rs");
        std::fs::write(&rust_file, "// just a comment\n").unwrap();
        let (path, _count) = scan_and_cache(tmp.path()).unwrap();
        assert!(path.exists());
        assert!(path.to_string_lossy().contains("pmat-"));
    }
}
