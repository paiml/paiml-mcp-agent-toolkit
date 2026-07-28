#![cfg_attr(coverage_nightly, coverage(off))]

use super::markdown_format::calculate_component_score;
use super::*;
use crate::models::tdg::{TDGHotspot, TDGSummary};
use proptest::prelude::*;

prop_compose! {
    fn arb_hotspot()(
        path in "[a-z]{1,20}/[a-z]{1,20}\\.rs",
        score in 0.0..10.0f64,
        factor in "[a-z]{3,15}",
        hours in 0.0..100.0f64
    ) -> TDGHotspot {
        TDGHotspot {
            path,
            tdg_score: score,
            primary_factor: factor,
            estimated_hours: hours,
        }
    }
}

prop_compose! {
    fn arb_summary()(
        total_files in 0usize..1000,
        critical_files in 0usize..100,
        warning_files in 0usize..200,
        average_tdg in 0.0..5.0f64,
        p95_tdg in 0.0..8.0f64,
        p99_tdg in 0.0..10.0f64,
        estimated_debt_hours in 0.0..1000.0f64
    ) -> TDGSummary {
        TDGSummary {
            total_files,
            critical_files,
            warning_files,
            average_tdg,
            p95_tdg,
            p99_tdg,
            estimated_debt_hours,
            hotspots: vec![],
        }
    }
}

proptest! {
    #[test]
    fn filter_never_increases_count(
        hotspots in prop::collection::vec(arb_hotspot(), 0..50),
        threshold in 0.0..10.0f64,
        top in 0usize..100,
        critical_only in proptest::bool::ANY
    ) {
        let original_len = hotspots.len();
        let filtered = filter_tdg_hotspots(hotspots, threshold, top, critical_only);
        prop_assert!(filtered.len() <= original_len);
    }

    #[test]
    fn filter_threshold_respected(
        hotspots in prop::collection::vec(arb_hotspot(), 0..50),
        threshold in 0.1..10.0f64
    ) {
        let filtered = filter_tdg_hotspots(hotspots, threshold, 0, false);
        prop_assert!(filtered.iter().all(|h| h.tdg_score >= threshold));
    }

    #[test]
    fn filter_critical_only_respected(
        hotspots in prop::collection::vec(arb_hotspot(), 0..50)
    ) {
        let filtered = filter_tdg_hotspots(hotspots, 0.0, 0, true);
        prop_assert!(filtered.iter().all(|h| h.tdg_score > 2.5));
    }

    #[test]
    fn filter_top_limit_respected(
        hotspots in prop::collection::vec(arb_hotspot(), 0..50),
        top in 1usize..100
    ) {
        let filtered = filter_tdg_hotspots(hotspots, 0.0, top, false);
        prop_assert!(filtered.len() <= top);
    }

    #[test]
    fn json_format_always_valid(
        summary in arb_summary(),
        hotspots in prop::collection::vec(arb_hotspot(), 0..10),
        include_components in proptest::bool::ANY
    ) {
        let result = format_tdg_json(&summary, &hotspots, include_components);
        prop_assert!(result.is_ok());
        let json = result.unwrap();
        prop_assert!(serde_json::from_str::<serde_json::Value>(&json).is_ok());
    }

    #[test]
    fn table_format_always_valid(
        hotspots in prop::collection::vec(arb_hotspot(), 0..10),
        verbose in proptest::bool::ANY
    ) {
        let result = format_tdg_table(&hotspots, verbose);
        prop_assert!(result.is_ok());
        let table = result.unwrap();
        prop_assert!(table.contains("| File | TDG Score |"));
    }

    #[test]
    fn markdown_format_always_valid(
        summary in arb_summary(),
        hotspots in prop::collection::vec(arb_hotspot(), 0..10),
        include_components in proptest::bool::ANY
    ) {
        let result = format_tdg_markdown(&summary, &hotspots, include_components);
        prop_assert!(result.is_ok());
        let md = result.unwrap();
        prop_assert!(md.contains("# Technical Debt Gradient Analysis"));
    }

    #[test]
    fn sarif_format_always_valid(
        hotspots in prop::collection::vec(arb_hotspot(), 0..10)
    ) {
        let result = format_tdg_sarif(&hotspots, &std::path::PathBuf::from("/test"));
        prop_assert!(result.is_ok());
        let sarif = result.unwrap();
        prop_assert!(serde_json::from_str::<serde_json::Value>(&sarif).is_ok());
    }

    #[test]
    fn calculate_component_score_commutative_multiplicative(
        score in 0.0..100.0f64,
        weight in 0.0..1.0f64
    ) {
        let result1 = calculate_component_score(score, weight);
        let result2 = score * weight;
        prop_assert!((result1 - result2).abs() < f64::EPSILON);
    }

    #[test]
    fn sarif_severity_matches_score(
        hotspots in prop::collection::vec(arb_hotspot(), 1..5)
    ) {
        let result = format_tdg_sarif(&hotspots, &std::path::PathBuf::from("/")).unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&result).expect("serde roundtrip");
        let results = parsed["runs"][0]["results"].as_array().unwrap();

        for (i, result) in results.iter().enumerate() {
            let level = result["level"].as_str().unwrap();
            let score = hotspots[i].tdg_score;

            match level {
                "error" => prop_assert!(score > 2.5),
                "warning" => prop_assert!(score > 1.5 && score <= 2.5),
                "note" => prop_assert!(score <= 1.5),
                _ => prop_assert!(false, "unexpected level: {}", level),
            }
        }
    }
}
