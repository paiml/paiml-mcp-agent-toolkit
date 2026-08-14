// Unit tests for fault localization formulas, localizer, LCOV parser, and formula parsing.

#[cfg_attr(coverage_nightly, coverage(off))]
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tarantula_perfect_fault() {
        // Statement executed by all failing tests, no passing tests
        let score = tarantula(10, 0, 10, 100);
        assert!((score - 1.0).abs() < 0.001);
    }

    #[test]
    fn test_tarantula_perfect_clean() {
        // Statement executed by all passing tests, no failing tests
        let score = tarantula(0, 100, 10, 100);
        assert!(score.abs() < 0.001);
    }

    #[test]
    fn test_tarantula_mixed() {
        let score = tarantula(5, 50, 10, 100);
        assert!(score > 0.0 && score < 1.0);
    }

    #[test]
    fn test_ochiai_perfect_fault() {
        let score = ochiai(10, 0, 10);
        assert!((score - 1.0).abs() < 0.001);
    }

    #[test]
    fn test_dstar_mixed() {
        let score = dstar(5, 50, 10, 2);
        // 25 / (50 + 5) = 0.4545...
        assert!((score - 0.4545).abs() < 0.01);
    }

    #[test]
    fn test_localizer_basic() {
        let localizer = SbflLocalizer::new();

        let coverage = vec![
            StatementCoverage::new(StatementId::new("file.rs", 10), 0, 10), // All failing
            StatementCoverage::new(StatementId::new("file.rs", 20), 100, 0), // All passing
            StatementCoverage::new(StatementId::new("file.rs", 30), 50, 5), // Mixed
        ];

        let result = localizer.localize(&coverage, 100, 10);

        assert_eq!(result.rankings.len(), 3);
        assert_eq!(result.rankings[0].statement.line, 10); // Most suspicious first
    }

    #[test]
    fn test_lcov_parser() {
        let lcov = r#"
SF:src/main.rs
DA:1,10
DA:2,5
DA:3,0
end_of_record
SF:src/lib.rs
DA:10,1
end_of_record
"#;

        let result = LcovParser::parse(lcov).unwrap();
        assert_eq!(result.len(), 4);
        assert_eq!(result[0].0.file.to_str().unwrap(), "src/main.rs");
        assert_eq!(result[0].0.line, 1);
        assert_eq!(result[0].1, 10);
    }

    // ── #949: the aggregate LCOV shape, and a ranking that survives a rerun ──

    /// 100 statements, one aggregate record per file — the shape
    /// `cargo llvm-cov --lcov` emits and the parser's own doc comment names.
    /// Lines 51..=100 are hit by all 8 failing tests and 1 passing test;
    /// lines 1..=50 by all 10 passing tests and 1 failing test.
    ///
    /// Hand-computed Tarantula: 51..=100 → (8/8) / ((1/10) + (8/8)) = 0.909;
    /// 1..=50 → (1/8) / ((10/10) + (1/8)) = 0.111.
    fn aggregate_fixture() -> (String, String) {
        let mut passed = String::from("SF:src/lib.rs\n");
        let mut failed = String::from("SF:src/lib.rs\n");
        for i in 1..=100 {
            let (p, f) = if i <= 50 { (10, 1) } else { (1, 8) };
            passed.push_str(&format!("DA:{i},{p}\n"));
            failed.push_str(&format!("DA:{i},{f}\n"));
        }
        passed.push_str("end_of_record\n");
        failed.push_str("end_of_record\n");
        (passed, failed)
    }

    #[test]
    fn aggregate_lcov_hit_counts_are_test_counts_not_booleans() {
        let (passed, failed) = aggregate_fixture();
        let combined = LcovParser::combine_coverage(
            &LcovParser::parse(&passed).unwrap(),
            &LcovParser::parse(&failed).unwrap(),
            10,
            8,
        );
        assert_eq!(combined.len(), 100);

        let by_line: HashMap<usize, &StatementCoverage> =
            combined.iter().map(|c| (c.id.line, c)).collect();

        // The 8 in `DA:51,8` used to be read as a boolean and tallied as 1.
        let hot = by_line[&51];
        assert_eq!(hot.executed_by_failed, 8, "aggregate hit count discarded");
        assert_eq!(hot.executed_by_passed, 1);
        let cold = by_line[&1];
        assert_eq!(cold.executed_by_failed, 1);
        assert_eq!(cold.executed_by_passed, 10);

        let hot_score = tarantula(8, 1, 8, 10);
        let cold_score = tarantula(1, 10, 8, 10);
        assert!(
            (hot_score - 0.909).abs() < 0.001,
            "expected 0.909, got {hot_score}"
        );
        assert!(
            (cold_score - 0.111).abs() < 0.001,
            "expected 0.111, got {cold_score}"
        );
        assert!(
            hot_score > cold_score,
            "the whole point: the failing half must outrank the passing half"
        );
    }

    #[test]
    fn aggregate_lcov_top_n_is_the_failing_half_and_is_reproducible() {
        let (passed, failed) = aggregate_fixture();
        let p = LcovParser::parse(&passed).unwrap();
        let f = LcovParser::parse(&failed).unwrap();

        let run = || {
            let result =
                FaultLocalizer::run_localization(&p, &f, 10, 8, SbflFormula::Tarantula, 10);
            result
                .rankings
                .iter()
                .map(|r| r.statement.line)
                .collect::<Vec<_>>()
        };

        let first = run();
        assert_eq!(first.len(), 10);
        assert!(
            first.iter().all(|l| *l > 50),
            "top-10 must be the failing half (51..=100), got {first:?}"
        );
        // Six identical invocations returned six disjoint top-10s before the
        // ranking had a tie-break.
        for _ in 0..5 {
            assert_eq!(run(), first, "identical inputs must rank identically");
        }
    }

    #[test]
    fn concatenated_per_test_lcov_still_counts_one_record_as_one_test() {
        // The shape that already worked must keep working: one record per test
        // (as cargo-llvm-cov emits per test, listing every instrumented line
        // including the ones that test did not reach), so 1 of 10 passing tests
        // and 8 of 8 failing tests executed line 51 — and an in-test loop count
        // of 99 must not inflate that to 99 tests.
        let mut passed = String::new();
        for i in 0..10 {
            let hits = if i == 0 { 99 } else { 0 };
            passed.push_str(&format!("SF:src/lib.rs\nDA:51,{hits}\nend_of_record\n"));
        }
        let mut failed = String::new();
        for _ in 0..8 {
            failed.push_str("SF:src/lib.rs\nDA:51,99\nend_of_record\n");
        }
        let combined = LcovParser::combine_coverage(
            &LcovParser::parse(&passed).unwrap(),
            &LcovParser::parse(&failed).unwrap(),
            10,
            8,
        );
        assert_eq!(combined.len(), 1);
        assert_eq!(combined[0].executed_by_failed, 8);
        assert_eq!(combined[0].executed_by_passed, 1);
    }

    #[test]
    fn an_aggregate_hit_count_cannot_exceed_the_declared_test_total() {
        // A loop body hit 1000 times by at most 10 passing tests is executed by
        // at most 10 passing tests.
        let combined = LcovParser::combine_coverage(
            &LcovParser::parse("SF:a.rs\nDA:1,1000\nend_of_record\n").unwrap(),
            &LcovParser::parse("SF:a.rs\nDA:1,7\nend_of_record\n").unwrap(),
            10,
            3,
        );
        assert_eq!(combined[0].executed_by_passed, 10);
        assert_eq!(combined[0].executed_by_failed, 3);
    }

    #[test]
    fn test_formula_from_str() {
        assert!(matches!(
            "tarantula".parse::<SbflFormula>().unwrap(),
            SbflFormula::Tarantula
        ));
        assert!(matches!(
            "ochiai".parse::<SbflFormula>().unwrap(),
            SbflFormula::Ochiai
        ));
        assert!(matches!(
            "dstar2".parse::<SbflFormula>().unwrap(),
            SbflFormula::DStar { exponent: 2 }
        ));
    }
}
