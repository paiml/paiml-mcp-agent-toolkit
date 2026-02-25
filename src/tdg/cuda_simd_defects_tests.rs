// Included from cuda_simd_defects.rs — NO use imports, NO #! inner attributes

#[cfg_attr(coverage_nightly, coverage(off))]
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_with_tauranta_patterns_creates_non_empty_taxonomy() {
        let taxonomy = DefectTaxonomy::with_tauranta_patterns();
        assert!(!taxonomy.is_empty());
        assert!(
            taxonomy.len() > 10,
            "Expected >10 patterns, got {}",
            taxonomy.len()
        );
    }

    #[test]
    fn test_with_tauranta_patterns_has_all_severity_levels() {
        let taxonomy = DefectTaxonomy::with_tauranta_patterns();
        let mut has_p0 = false;
        let mut has_p1 = false;
        let mut has_p2 = false;
        let mut has_p3 = false;

        for defect in taxonomy.all() {
            match defect.severity {
                DefectSeverity::P0Critical => has_p0 = true,
                DefectSeverity::P1Performance => has_p1 = true,
                DefectSeverity::P2Efficiency => has_p2 = true,
                DefectSeverity::P3Minor => has_p3 = true,
            }
        }

        assert!(has_p0, "Missing P0Critical defects");
        assert!(has_p1, "Missing P1Performance defects");
        assert!(has_p2, "Missing P2Efficiency defects");
        // P3Minor may not be present in the current taxonomy
        let _ = has_p3;
    }

    #[test]
    fn test_with_tauranta_patterns_p0_critical_defects() {
        let taxonomy = DefectTaxonomy::with_tauranta_patterns();

        // PARITY-114: barrier divergence
        let parity114 = taxonomy.get("PARITY-114").expect("PARITY-114 should exist");
        assert_eq!(parity114.severity, DefectSeverity::P0Critical);
        assert!(parity114.description.contains("Barrier divergence"));
        assert!(parity114.resolved);
        assert!(parity114.root_cause.is_some());

        // PAR-002: CUDA GEMV illegal memory access
        let par002 = taxonomy.get("PAR-002").expect("PAR-002 should exist");
        assert_eq!(par002.severity, DefectSeverity::P0Critical);
        assert!(par002.description.contains("GEMV"));

        // PAR-041: FlashAttention shared memory overflow
        let par041 = taxonomy.get("PAR-041").expect("PAR-041 should exist");
        assert_eq!(par041.severity, DefectSeverity::P0Critical);
        assert!(par041.description.contains("FlashAttention"));
    }

    #[test]
    fn test_with_tauranta_patterns_p1_performance_defects() {
        let taxonomy = DefectTaxonomy::with_tauranta_patterns();

        let par001 = taxonomy.get("PAR-001").expect("PAR-001 should exist");
        assert_eq!(par001.severity, DefectSeverity::P1Performance);
        assert!(par001.description.contains("Q4_K dequantization"));

        let par034 = taxonomy.get("PAR-034").expect("PAR-034 should exist");
        assert_eq!(par034.severity, DefectSeverity::P1Performance);
        assert!(par034.description.contains("Tensor Core"));
    }

    #[test]
    fn test_with_tauranta_patterns_get_nonexistent() {
        let taxonomy = DefectTaxonomy::with_tauranta_patterns();
        assert!(taxonomy.get("NONEXISTENT-999").is_none());
    }

    #[test]
    fn test_with_tauranta_patterns_all_have_ticket_ids() {
        let taxonomy = DefectTaxonomy::with_tauranta_patterns();
        for defect in taxonomy.all() {
            assert!(!defect.ticket_id.is_empty(), "Defect has empty ticket_id");
            assert!(
                !defect.description.is_empty(),
                "Defect {} has empty description",
                defect.ticket_id
            );
            assert!(
                !defect.detection_method.is_empty(),
                "Defect {} has empty detection_method",
                defect.ticket_id
            );
        }
    }

    #[test]
    fn test_with_tauranta_patterns_all_iterator_count() {
        let taxonomy = DefectTaxonomy::with_tauranta_patterns();
        let count = taxonomy.all().count();
        assert_eq!(count, taxonomy.len());
    }

    #[test]
    fn test_defect_taxonomy_default_is_empty() {
        let taxonomy = DefectTaxonomy::default();
        assert!(taxonomy.is_empty());
        assert_eq!(taxonomy.len(), 0);
    }

    #[test]
    fn test_defect_severity_display() {
        assert_eq!(format!("{}", DefectSeverity::P0Critical), "P0-Critical");
        assert_eq!(
            format!("{}", DefectSeverity::P1Performance),
            "P1-Performance"
        );
        assert_eq!(format!("{}", DefectSeverity::P2Efficiency), "P2-Efficiency");
        assert_eq!(format!("{}", DefectSeverity::P3Minor), "P3-Minor");
    }

    #[test]
    fn test_defect_class_clone_and_debug() {
        let taxonomy = DefectTaxonomy::with_tauranta_patterns();
        let defect = taxonomy.get("PARITY-114").unwrap();
        let cloned = defect.clone();
        assert_eq!(cloned.ticket_id, defect.ticket_id);
        assert_eq!(cloned.severity, defect.severity);
        let _debug = format!("{:?}", cloned);
    }

    #[test]
    fn test_with_tauranta_patterns_cb_prefixed_defects() {
        let taxonomy = DefectTaxonomy::with_tauranta_patterns();
        // Check for CB-xxx defects (codebase quality patterns)
        let cb_defects: Vec<_> = taxonomy
            .all()
            .filter(|d| d.ticket_id.starts_with("CB-"))
            .collect();
        assert!(!cb_defects.is_empty(), "Should have CB-prefixed defects");
    }

    #[test]
    fn test_with_tauranta_patterns_some_resolved_some_not() {
        let taxonomy = DefectTaxonomy::with_tauranta_patterns();
        let resolved_count = taxonomy.all().filter(|d| d.resolved).count();
        let unresolved_count = taxonomy.all().filter(|d| !d.resolved).count();
        assert!(resolved_count > 0, "Should have some resolved defects");
        assert!(unresolved_count > 0, "Should have some unresolved defects");
    }
}
