#[cfg_attr(coverage_nightly, coverage(off))]
#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[tokio::test]
    async fn test_incremental_coverage() {
        let temp_dir = TempDir::new().unwrap();
        let db_path = temp_dir.path().join("coverage.db");

        let analyzer = IncrementalCoverageAnalyzer::new(&db_path).unwrap();

        let test_file = temp_dir.path().join("test.rs");
        tokio::fs::write(
            &test_file,
            r#"
fn main() {
    println!("Hello, world!");
}

fn add(a: i32, b: i32) -> i32 {
    a + b
}
"#,
        )
        .await
        .unwrap();

        let hash = analyzer.compute_file_hash(&test_file).await.unwrap();
        let file_id = FileId {
            path: test_file.clone(),
            hash,
        };

        let changeset = ChangeSet {
            modified_files: vec![file_id],
            added_files: vec![],
            deleted_files: vec![],
        };

        let update = analyzer.analyze_changes(&changeset).await.unwrap();

        assert!(!update.file_coverage.is_empty());
        assert!(update.aggregate_coverage.line_percentage > 0.0);
    }
}

