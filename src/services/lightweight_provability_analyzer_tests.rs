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
}

