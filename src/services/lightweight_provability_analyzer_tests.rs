// Unit tests and property tests for the lightweight provability analyzer.

#[cfg_attr(coverage_nightly, coverage(off))]
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_nullability_lattice() {
        use NullabilityLattice::*;

        assert_eq!(NotNull.join(&NotNull), NotNull);
        assert_eq!(NotNull.join(&MaybeNull), MaybeNull);
        assert_eq!(NotNull.join(&Null), Bottom);
        assert_eq!(Top.join(&NotNull), Top);
    }

    #[test]
    fn test_property_domain_join() {
        let domain1 = PropertyDomain::top();
        let domain2 = PropertyDomain {
            nullability: NullabilityLattice::NotNull,
            bounds: IntervalLattice {
                lower: Some(0),
                upper: Some(100),
            },
            aliasing: AliasLattice::NoAlias,
            purity: PurityLattice::Pure,
        };

        let joined = domain1.join(&domain2);
        assert_eq!(joined.nullability, NullabilityLattice::Top);
    }

    #[tokio::test]
    async fn test_incremental_analysis() {
        let analyzer = LightweightProvabilityAnalyzer::new();

        let functions = vec![FunctionId {
            file_path: "src/main.rs".to_string(),
            function_name: "main".to_string(),
            line_number: 10,
        }];

        let summaries = analyzer.analyze_incrementally(&functions).await;
        assert!(!summaries.is_empty());
        // Timing is not asserted here — speed belongs in a benchmark, not a unit test.
        // Self-hosted CI runners can exceed 1ms under load, which blocked the PR queue.
    }

    /// A relative `FunctionId::file_path` is resolved against the project root,
    /// not against the process working directory.
    ///
    /// `discover_project_functions` strips the project prefix so reports print
    /// `src/lib.rs`; the analyzer then read that string back with
    /// `std::fs::read_to_string`, i.e. relative to wherever the shell happened
    /// to be. Run from outside the analyzed project it read nothing and every
    /// function got the 20% no-evidence baseline.
    ///
    /// The probe directory name cannot exist relative to any plausible cwd, so
    /// the pre-fix behaviour is the empty-source baseline deterministically.
    #[tokio::test]
    async fn relative_paths_resolve_against_the_project_root_not_the_cwd() {
        let dir = tempfile::tempdir().unwrap();
        let probe = dir.path().join("provability_cwd_probe");
        std::fs::create_dir_all(&probe).unwrap();
        std::fs::write(
            probe.join("pure.rs"),
            "pub fn add(a: i32, b: i32) -> i32 {\n    a + b\n}\n",
        )
        .unwrap();

        let relative = FunctionId {
            file_path: "provability_cwd_probe/pure.rs".to_string(),
            function_name: "add".to_string(),
            line_number: 1,
        };
        let absolute = FunctionId {
            file_path: probe.join("pure.rs").to_string_lossy().to_string(),
            ..relative.clone()
        };

        let rooted = LightweightProvabilityAnalyzer::new().with_project_root(dir.path());
        let from_relative = &rooted.analyze_incrementally(&[relative]).await[0];
        let from_absolute = &LightweightProvabilityAnalyzer::new()
            .analyze_incrementally(&[absolute])
            .await[0];

        assert!(
            (from_relative.provability_score - from_absolute.provability_score).abs()
                < f64::EPSILON,
            "a display path plus its root must score exactly like the absolute path: \
             {} vs {}",
            from_relative.provability_score,
            from_absolute.provability_score
        );
        assert!(
            !from_relative.verified_properties.is_empty(),
            "the source was read, so there must be evidence; scoring 20% with zero \
             properties means the file was never opened"
        );
    }

    /// The resolution rule itself: absolute paths are untouched, relative ones
    /// hang off the root, and with no root nothing changes.
    #[test]
    fn resolve_source_path_is_the_one_rule() {
        use std::path::Path;

        let rooted = LightweightProvabilityAnalyzer::new().with_project_root("/proj");
        assert_eq!(
            rooted.resolve_source_path("src/lib.rs"),
            Path::new("/proj/src/lib.rs")
        );
        assert_eq!(
            rooted.resolve_source_path("/elsewhere/lib.rs"),
            Path::new("/elsewhere/lib.rs")
        );
        assert_eq!(
            LightweightProvabilityAnalyzer::new().resolve_source_path("src/lib.rs"),
            Path::new("src/lib.rs")
        );
    }
}

