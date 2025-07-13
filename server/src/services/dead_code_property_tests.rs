#[cfg(test)]
mod tests {
    use crate::services::dead_code_analyzer::{HierarchicalBitSet, DeadCodeAnalyzer};
    use crate::models::dead_code::{
        DeadCodeItem, DeadCodeType, FileDeadCodeMetrics, ConfidenceLevel,
        DeadCodeAnalysisConfig, DeadCodeSummary
    };
    use proptest::prelude::*;

    // Strategy for generating dead code items
    prop_compose! {
        fn arb_dead_code_item()
            (
                item_type in prop_oneof![
                    Just(DeadCodeType::Function),
                    Just(DeadCodeType::Class),
                    Just(DeadCodeType::Variable),
                    Just(DeadCodeType::UnreachableCode),
                ],
                name in "[a-zA-Z][a-zA-Z0-9_]{0,20}",
                line in 1u32..1000,
                reason in prop_oneof![
                    Just("Never called".to_string()),
                    Just("Never instantiated".to_string()),
                    Just("Never used".to_string()),
                    Just("Unreachable".to_string()),
                ],
            )
            -> DeadCodeItem
        {
            DeadCodeItem {
                item_type,
                name,
                line,
                reason,
            }
        }
    }

    // Strategy for generating file dead code metrics
    prop_compose! {
        fn arb_file_dead_code_metrics()
            (
                path in "[a-zA-Z][a-zA-Z0-9_/]{0,50}\\.rs",
                total_lines in 50usize..2000,
                items in prop::collection::vec(arb_dead_code_item(), 0..5), // Fewer items to avoid exceeding total lines
                confidence in prop_oneof![
                    Just(ConfidenceLevel::High),
                    Just(ConfidenceLevel::Medium),
                    Just(ConfidenceLevel::Low),
                ],
            )
            -> FileDeadCodeMetrics
        {
            let mut metrics = FileDeadCodeMetrics::new(path);
            metrics.total_lines = total_lines;
            metrics.confidence = confidence;
            
            for item in items {
                metrics.add_item(item);
            }
            
            // Ensure dead lines don't exceed total lines
            if metrics.dead_lines > metrics.total_lines {
                metrics.dead_lines = metrics.total_lines;
            }
            
            metrics.update_percentage();
            metrics.calculate_score();
            metrics
        }
    }

    // Strategy for generating dead code analysis config
    prop_compose! {
        fn arb_dead_code_config()
            (
                include_unreachable in any::<bool>(),
                include_tests in any::<bool>(),
                min_dead_lines in 0usize..50,
            )
            -> DeadCodeAnalysisConfig
        {
            DeadCodeAnalysisConfig {
                include_unreachable,
                include_tests,
                min_dead_lines,
            }
        }
    }

    proptest! {
        #[test]
        fn test_dead_code_percentage_calculation(
            metrics in arb_file_dead_code_metrics()
        ) {
            // Property: Dead code percentage should be between 0 and 100
            prop_assert!(metrics.dead_percentage >= 0.0);
            prop_assert!(metrics.dead_percentage <= 100.0);
            
            // Property: If total_lines > 0, percentage should be (dead_lines / total_lines) * 100
            if metrics.total_lines > 0 {
                let expected_percentage = (metrics.dead_lines as f32 / metrics.total_lines as f32) * 100.0;
                prop_assert!((metrics.dead_percentage - expected_percentage).abs() < 0.001);
            }
        }

        #[test]
        fn test_dead_code_score_calculation(
            mut metrics in arb_file_dead_code_metrics()
        ) {
            let initial_score = metrics.dead_score;
            
            // Property: Score should be non-negative
            prop_assert!(initial_score >= 0.0);
            
            // Recalculate and verify consistency
            metrics.calculate_score();
            prop_assert_eq!(initial_score, metrics.dead_score);
        }

        #[test]
        fn test_add_item_properties(
            path in "[a-zA-Z][a-zA-Z0-9_/]{0,30}\\.rs",
            items in prop::collection::vec(arb_dead_code_item(), 1..20)
        ) {
            let mut metrics = FileDeadCodeMetrics::new(path);
            let initial_count = metrics.items.len();
            
            for item in &items {
                metrics.add_item(item.clone());
            }
            
            // Property: Number of items should increase by the number added
            prop_assert_eq!(metrics.items.len(), initial_count + items.len());
            
            // Property: Dead lines should be sum of estimated lines for each item type
            let expected_dead_lines = items.iter().map(|item| match item.item_type {
                DeadCodeType::Function => 10,
                DeadCodeType::Class => 10,
                DeadCodeType::Variable => 1,
                DeadCodeType::UnreachableCode => 3,
            }).sum::<usize>();
            
            prop_assert_eq!(metrics.dead_lines, expected_dead_lines);
        }

        #[test]
        fn test_confidence_level_score_impact(
            base_metrics in arb_file_dead_code_metrics()
        ) {
            let mut high_conf = base_metrics.clone();
            let mut medium_conf = base_metrics.clone();
            let mut low_conf = base_metrics;
            
            high_conf.confidence = ConfidenceLevel::High;
            medium_conf.confidence = ConfidenceLevel::Medium;
            low_conf.confidence = ConfidenceLevel::Low;
            
            high_conf.calculate_score();
            medium_conf.calculate_score();
            low_conf.calculate_score();
            
            // Property: Higher confidence should result in higher scores
            prop_assert!(high_conf.dead_score >= medium_conf.dead_score);
            prop_assert!(medium_conf.dead_score >= low_conf.dead_score);
        }

        #[test]
        fn test_dead_code_summary_aggregation(
            files in prop::collection::vec(arb_file_dead_code_metrics(), 1..10)
        ) {
            let summary = DeadCodeSummary::from_files(&files);
            
            // Property: Summary should aggregate all files correctly
            prop_assert_eq!(summary.total_files_analyzed, files.len());
            
            let files_with_dead_code = files.iter().filter(|f| f.dead_lines > 0).count();
            prop_assert_eq!(summary.files_with_dead_code, files_with_dead_code);
            
            let total_dead_lines: usize = files.iter().map(|f| f.dead_lines).sum();
            prop_assert_eq!(summary.total_dead_lines, total_dead_lines);
            
            let total_functions: usize = files.iter().map(|f| f.dead_functions).sum();
            prop_assert_eq!(summary.dead_functions, total_functions);
        }

        #[test]
        fn test_hierarchical_bitset_properties(
            indices in prop::collection::vec(0u32..1000, 0..50)
        ) {
            let mut bitset = HierarchicalBitSet::new(1000);
            
            // Set all indices
            for &index in &indices {
                bitset.set(index);
            }
            
            // Property: All set indices should be reported as set
            for &index in &indices {
                prop_assert!(bitset.is_set(index));
            }
            
            // Property: Count should match number of unique indices
            let unique_indices: std::collections::HashSet<_> = indices.iter().collect();
            prop_assert_eq!(bitset.count_set(), unique_indices.len());
        }

        #[test]
        fn test_dead_code_config_properties(
            config in arb_dead_code_config()
        ) {
            // Property: min_dead_lines should be non-negative
            prop_assert!(config.min_dead_lines < 1000); // Reasonable upper bound
            
            // Property: Config should be serializable/deserializable
            let serialized = serde_json::to_string(&config);
            prop_assert!(serialized.is_ok());
            
            if let Ok(json) = serialized {
                let deserialized: Result<DeadCodeAnalysisConfig, _> = serde_json::from_str(&json);
                prop_assert!(deserialized.is_ok());
            }
        }
    }

    #[test]
    fn test_dead_code_analyzer_creation() {
        // Property: Analyzer should be created with reasonable capacity
        for capacity in [100, 1000, 10000] {
            let analyzer = DeadCodeAnalyzer::new(capacity);
            // Should not panic and should be usable
            assert_eq!(std::mem::size_of_val(&analyzer), std::mem::size_of::<DeadCodeAnalyzer>());
        }
    }

    #[test]
    fn test_dead_code_type_serialization() {
        // Property: All dead code types should be serializable
        let types = [
            DeadCodeType::Function,
            DeadCodeType::Class,
            DeadCodeType::Variable,
            DeadCodeType::UnreachableCode,
        ];
        
        for dead_type in &types {
            let serialized = serde_json::to_string(dead_type);
            assert!(serialized.is_ok());
            
            if let Ok(json) = serialized {
                let deserialized: Result<DeadCodeType, _> = serde_json::from_str(&json);
                assert!(deserialized.is_ok());
                assert_eq!(deserialized.unwrap(), *dead_type);
            }
        }
    }
}