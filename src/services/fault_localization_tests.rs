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
