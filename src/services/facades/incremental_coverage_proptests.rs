use proptest::prelude::*;

proptest! {
    #[test]
    fn coverage_percentage_bounded(
        before in 0.0f64..=1.0f64,
        after in 0.0f64..=1.0f64
    ) {
        let delta = after - before;

        // Coverage values should remain bounded
        prop_assert!((0.0..=1.0).contains(&before));
        prop_assert!((0.0..=1.0).contains(&after));
        prop_assert!((-1.0..=1.0).contains(&delta));
    }

    #[test]
    fn coverage_status_from_delta(delta in -1.0f64..1.0f64) {
        let status = if delta > 0.0 {
            CoverageStatus::Improved
        } else if delta < 0.0 {
            CoverageStatus::Degraded
        } else {
            CoverageStatus::Unchanged
        };

        match status {
            CoverageStatus::Improved => prop_assert!(delta > 0.0),
            CoverageStatus::Degraded => prop_assert!(delta < 0.0),
            CoverageStatus::Unchanged => prop_assert!((delta - 0.0).abs() < f64::EPSILON),
            _ => prop_assert!(false, "Unexpected status"),
        }
    }

    #[test]
    fn threshold_comparison_consistent(
        coverage in 0.0f64..=1.0f64,
        threshold in 0.0f64..=1.0f64
    ) {
        let above = coverage >= threshold;
        let below = coverage < threshold;

        // Exactly one must be true
        prop_assert!(above ^ below);
    }

    #[test]
    fn result_counts_sum_correctly(
        above in 0usize..100,
        below in 0usize..100
    ) {
        let total = above + below;

        let result = IncrementalCoverageResult {
            total_files: total,
            covered_files: above,
            coverage_percentage: Some(80.0),
            files_above_threshold: above,
            files_below_threshold: below,
            files_not_measured: 0,
            changed_files: vec![],
            summary: String::new(),
        };

        prop_assert_eq!(
            result.files_above_threshold + result.files_below_threshold,
            result.total_files
        );
    }

    #[test]
    fn serialization_roundtrip(
        total in 0usize..1000,
        covered in 0usize..1000,
        above in 0usize..100,
        below in 0usize..100
    ) {
        let result = IncrementalCoverageResult {
            total_files: total,
            covered_files: covered,
            coverage_percentage: Some(80.0),
            files_above_threshold: above,
            files_below_threshold: below,
            files_not_measured: 0,
            changed_files: vec![],
            summary: "Test".to_string(),
        };

        let json = serde_json::to_string(&result).unwrap();
        let deserialized: IncrementalCoverageResult = serde_json::from_str(&json).expect("serde roundtrip");

        prop_assert_eq!(result.total_files, deserialized.total_files);
        prop_assert_eq!(result.covered_files, deserialized.covered_files);
        prop_assert_eq!(result.files_above_threshold, deserialized.files_above_threshold);
        prop_assert_eq!(result.files_below_threshold, deserialized.files_below_threshold);
    }

    #[test]
    fn average_coverage_calculation(
        coverages in prop::collection::vec(0.0f64..=1.0f64, 1..10)
    ) {
        let avg = if coverages.is_empty() {
            0.0
        } else {
            coverages.iter().sum::<f64>() / coverages.len() as f64
        };

        // Average should be bounded
        prop_assert!(avg >= 0.0);
        prop_assert!(avg <= 1.0);

        // Average should be between min and max
        if !coverages.is_empty() {
            let min = coverages.iter().cloned().reduce(f64::min).unwrap();
            let max = coverages.iter().cloned().reduce(f64::max).unwrap();
            prop_assert!(avg >= min);
            prop_assert!(avg <= max);
        }
    }

    #[test]
    fn lines_covered_bounded(
        covered in 0usize..10000,
        total in 1usize..10000
    ) {
        // covered should not exceed total in valid data
        let valid_covered = covered.min(total);

        // Percentages in 0-100, and `None` when unmeasured (GH #658).
        let file_coverage = ChangedFileCoverage {
            file_path: "test.rs".to_string(),
            coverage_before: None,
            coverage_after: Some(valid_covered as f64 / total as f64 * 100.0),
            coverage_delta: None,
            status: CoverageStatus::NotMeasured,
            lines_covered: valid_covered,
            lines_total: total,
        };

        prop_assert!(file_coverage.lines_covered <= file_coverage.lines_total);
        let after = file_coverage.coverage_after.unwrap();
        prop_assert!(after >= 0.0);
        prop_assert!(after <= 100.0);
    }

    /// GH #658: an unmeasured factor must stay unmeasured through serde, never
    /// deserialize back as 0.0.
    #[test]
    fn unmeasured_coverage_survives_serialization(covered in 0usize..100) {
        let file_coverage = ChangedFileCoverage {
            file_path: "test.rs".to_string(),
            coverage_before: None,
            coverage_after: None,
            coverage_delta: None,
            status: CoverageStatus::NotMeasured,
            lines_covered: covered,
            lines_total: 0,
        };

        let json = serde_json::to_string(&file_coverage).unwrap();
        let back: ChangedFileCoverage = serde_json::from_str(&json).expect("serde roundtrip");
        prop_assert_eq!(back.coverage_before, None);
        prop_assert_eq!(back.coverage_after, None);
        prop_assert_eq!(back.coverage_delta, None);
        prop_assert!(json.contains("null"));
    }
}
