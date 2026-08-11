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

    /// #712: every annotation used to carry the literal span `0..100`
    /// ("Would need proper span handling"), so all 2964 annotations this
    /// command produced over `src/` shared ONE location. Because the collector
    /// deduplicates on `(location, property)`, that constant also silently
    /// DISCARDED every function after the first in each file.
    #[cfg(feature = "rust-ast")]
    #[tokio::test]
    async fn test_annotation_spans_are_measured_not_the_constant_0_to_100() {
        let temp_dir = TempDir::new().unwrap();
        let rust_file = temp_dir.path().join("spans.rs");
        let source = concat!(
            "fn alpha() {\n",
            "    let x = 1;\n",
            "}\n",
            "\n",
            "fn beta() {\n",
            "    let y = 2;\n",
            "}\n",
        );
        std::fs::write(&rust_file, source).unwrap();

        let checker = RustBorrowChecker::default();
        let cache = Arc::new(RwLock::new(ProofCache::new()));
        let symbol_table = Arc::new(SymbolTable::new());
        let result = checker
            .collect(temp_dir.path(), &cache, &symbol_table)
            .await
            .unwrap();

        let spans: std::collections::BTreeSet<(u32, u32)> = result
            .annotations
            .iter()
            .map(|(loc, _)| (loc.span.start.0, loc.span.end.0))
            .collect();

        assert!(
            !spans.contains(&(0, 100)),
            "the placeholder span 0..100 must not survive; got {spans:?}"
        );
        assert_eq!(
            spans.len(),
            2,
            "two functions must produce two distinct spans, got {spans:?}"
        );

        // Each span must actually bracket its function in the source text.
        for (start, end) in spans {
            let slice = &source[start as usize..end as usize];
            assert!(
                slice.starts_with("fn "),
                "span {start}..{end} does not start at a fn: {slice:?}"
            );
            assert!(
                slice.ends_with('}'),
                "span {start}..{end} does not end at the closing brace: {slice:?}"
            );
        }
    }

    /// #712 companion: distinct locations mean distinct functions survive the
    /// collector's `(location, property)` deduplication.
    #[cfg(feature = "rust-ast")]
    #[tokio::test]
    async fn test_each_safe_function_yields_its_own_annotation() {
        let temp_dir = TempDir::new().unwrap();
        std::fs::write(
            temp_dir.path().join("many.rs"),
            "fn a() {}\nfn b() {}\nfn c() {}\nfn d() {}\n",
        )
        .unwrap();

        let checker = RustBorrowChecker::default();
        let cache = Arc::new(RwLock::new(ProofCache::new()));
        let symbol_table = Arc::new(SymbolTable::new());
        let result = checker
            .collect(temp_dir.path(), &cache, &symbol_table)
            .await
            .unwrap();

        let distinct: std::collections::BTreeSet<(u32, u32)> = result
            .annotations
            .iter()
            .map(|(loc, _)| (loc.span.start.0, loc.span.end.0))
            .collect();

        assert_eq!(
            distinct.len(),
            4,
            "four functions must have four distinct locations, got {distinct:?}"
        );
    }

    #[test]
    fn test_memory_safety_annotation() {
        let checker = RustBorrowChecker::default();
        let annotation = checker.memory_safety_annotation(&seed_location());

        assert_eq!(annotation.property_proven, PropertyType::MemorySafety);
        assert_eq!(annotation.method, VerificationMethod::BorrowChecker);
        // This used to require High -- "machine-checkable proof" per the enum's
        // own docs -- for a claim produced by a syn walk that only looks at the
        // signature and lists two unverified assumptions. Medium is "sound
        // static analysis with assumptions", which is what this is.
        assert_eq!(annotation.confidence_level, ConfidenceLevel::Medium);
        assert!(
            !annotation.assumptions.is_empty(),
            "a claim that rests on assumptions cannot be a machine-checked proof"
        );

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
        // Was High. The evidence is `type_likely_implements_send_sync`, which
        // matches parameter type names against a hardcoded allowlist -- Low
        // ("heuristic-based") by the enum's own definition.
        assert_eq!(annotation.confidence_level, ConfidenceLevel::Low);
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

    /// Round-5 dogfood: `analyze proof-annotations` reported
    /// "High confidence: N (100.0%)" over this repo and `--high-confidence-only`
    /// removed nothing, because every production construction site wrote
    /// `ConfidenceLevel::High`. Nothing pmat emits here is a machine-checked
    /// proof, so no factory may claim the level reserved for one, and the level
    /// must actually vary with the strength of the evidence.
    #[test]
    fn confidence_is_not_one_hardcoded_level_for_every_annotation() {
        let checker = RustBorrowChecker::default();
        let loc = seed_location();

        let levels = [
            checker.memory_safety_annotation(&loc).confidence_level,
            checker.create_thread_safety_annotation(&loc).confidence_level,
            checker.const_fn_termination(&loc).confidence_level,
        ];

        assert!(
            !levels.contains(&ConfidenceLevel::High),
            "pmat never runs a checker, so it cannot claim a machine-checkable proof: {levels:?}"
        );
        let distinct: std::collections::BTreeSet<_> =
            levels.iter().map(|l| format!("{l:?}")).collect();
        assert!(
            distinct.len() > 1,
            "confidence must follow the evidence, not be one constant: {levels:?}"
        );
    }
}
