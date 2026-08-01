// Tests for RustBorrowChecker
// Included by rust_borrow_checker.rs - no `use` imports or `#!` attributes

#[cfg_attr(coverage_nightly, coverage(off))]
#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[tokio::test]
    async fn test_rust_borrow_checker_creation() {
        let checker = RustBorrowChecker::new();
        assert!(checker.is_ok());
    }

    #[tokio::test]
    async fn test_rust_borrow_checker_collect() {
        let temp_dir = TempDir::new().unwrap();
        let rust_file = temp_dir.path().join("test.rs");

        // Create a simple Rust file
        std::fs::write(
            &rust_file,
            r#"
            fn safe_function() {
                let x = 42;
                println!("{}", x);
            }
        "#,
        )
        .unwrap();

        let checker = RustBorrowChecker::default();
        let cache = Arc::new(RwLock::new(ProofCache::new()));
        let symbol_table = Arc::new(SymbolTable::new());

        let result = checker
            .collect(temp_dir.path(), &cache, &symbol_table)
            .await;
        assert!(result.is_ok());

        let collection_result = result.unwrap();
        assert_eq!(collection_result.metrics.files_processed, 1);
        assert!(!collection_result.annotations.is_empty());
    }

    /// A location to seed the content-derived `annotation_id` with.
    fn seed_location() -> Location {
        Location::new(std::path::PathBuf::from("src/seed.rs"), 0, 100)
    }

    #[test]
    fn test_memory_safety_annotation() {
        let checker = RustBorrowChecker::default();
        let annotation = checker.memory_safety_annotation(&seed_location());

        assert_eq!(annotation.property_proven, PropertyType::MemorySafety);
        assert_eq!(annotation.method, VerificationMethod::BorrowChecker);
        assert_eq!(annotation.confidence_level, ConfidenceLevel::High);

        // This used to require tool_name == "rustc-stable", pinning a false
        // provenance claim: pmat never invokes rustc, it parses with syn and
        // reasons about the result. Attributing a proof annotation to a
        // compiler that never ran -- on a version nobody queried ("1.70.0
        // (unknown)") -- is fabricated evidence on the one artifact type where
        // provenance is the whole point. The test enforced it; it now forbids it.
        assert_eq!(annotation.tool_name, "pmat-syn-static-analysis");
        assert!(
            !annotation.tool_name.contains("rustc"),
            "must not attribute findings to a compiler pmat never ran"
        );
    }

    #[test]
    fn test_thread_safety_annotation() {
        let checker = RustBorrowChecker::default();
        let annotation = checker.create_thread_safety_annotation(&seed_location());

        assert_eq!(annotation.property_proven, PropertyType::ThreadSafety);
        assert_eq!(annotation.method, VerificationMethod::BorrowChecker);
        assert_eq!(annotation.confidence_level, ConfidenceLevel::High);
    }

    /// DETERMINISM regression (round-3 sweep): `annotation_id` was
    /// `Uuid::new_v4()`, so `analyze proof-annotations --format json` handed
    /// the SAME annotation about the SAME unchanged line a different
    /// `annotationId` on every invocation — 1298 annotations, all new
    /// identities each run, so no two runs could ever be diffed.
    ///
    /// Same site + same claim => same id; different site or different claim =>
    /// different id.
    #[test]
    fn annotation_ids_are_derived_from_content_not_random() {
        let checker = RustBorrowChecker::default();
        let here = seed_location();
        let elsewhere = Location::new(std::path::PathBuf::from("src/other.rs"), 0, 100);

        let a = checker.memory_safety_annotation(&here);
        for _ in 0..5 {
            assert_eq!(
                checker.memory_safety_annotation(&here).annotation_id,
                a.annotation_id,
                "identical input must produce an identical annotationId"
            );
        }

        assert_ne!(
            checker.memory_safety_annotation(&elsewhere).annotation_id,
            a.annotation_id,
            "a different file must not collide"
        );
        assert_ne!(
            checker
                .create_thread_safety_annotation(&here)
                .annotation_id,
            a.annotation_id,
            "a different property at the same site must not collide"
        );
        assert_ne!(
            checker.const_fn_termination(&here).annotation_id,
            a.annotation_id,
            "a different property at the same site must not collide"
        );
    }
}

