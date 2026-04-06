// DemoScorer G2: Error pattern analysis of demo files

impl DemoScorer {
    async fn g2_analyze_demo_files(
        &self,
        demo_files: &[PathBuf],
        max_score: f64,
    ) -> Result<SubcategoryScore> {
        debug_assert!(!demo_files.is_empty(), "demo_files must not be empty");
        let mut score: f64 = max_score;
        let mut findings = vec![];

        let counts = count_error_patterns(demo_files).await;

        // Deduct for raw unwraps (0.5 points per 5 unwraps, max -1.5)
        let unwrap_penalty = (counts.raw_unwrap as f64 / 5.0 * 0.5).min(1.5);
        if counts.raw_unwrap > 0 {
            score -= unwrap_penalty;
            findings.push(Finding {
                severity: if counts.raw_unwrap > 10 { Severity::Error } else { Severity::Warning },
                category: "Demo Quality".to_string(),
                message: format!(
                    "{} raw .unwrap() calls in user-facing demo code (use .expect() with message or proper error handling)",
                    counts.raw_unwrap
                ),
                location: None,
                impact_points: -unwrap_penalty,
            });
        }

        if counts.contextual_unwrap > 0 {
            findings.push(Finding {
                severity: Severity::Info,
                category: "Demo Quality".to_string(),
                message: format!("{} .unwrap() calls in test/setup functions (acceptable)", counts.contextual_unwrap),
                location: None,
                impact_points: 0.0,
            });
        }

        // Deduct for raw panics (0.5 points per panic, max -1.0)
        let panic_penalty = (counts.raw_panic as f64 * 0.5).min(1.0);
        if counts.raw_panic > 0 {
            score -= panic_penalty;
            findings.push(Finding {
                severity: Severity::Error,
                category: "Demo Quality".to_string(),
                message: format!("{} panic!() calls in demo code (prefer graceful error messages)", counts.raw_panic),
                location: None,
                impact_points: -panic_penalty,
            });
        }

        if counts.proper_error_handling > 5 && counts.raw_unwrap < 5 {
            findings.push(Finding {
                severity: Severity::Success,
                category: "Demo Quality".to_string(),
                message: "Good error handling patterns detected in demo code".to_string(),
                location: None,
                impact_points: 0.0,
            });
        }

        if counts.expect_with_message > 0 {
            findings.push(Finding {
                severity: Severity::Info,
                category: "Demo Quality".to_string(),
                message: format!("{} .expect() calls with messages (acceptable for demos)", counts.expect_with_message),
                location: None,
                impact_points: 0.0,
            });
        }

        score = score.max(0.0);

        if findings.is_empty() {
            findings.push(Finding {
                severity: Severity::Success,
                category: "Demo Quality".to_string(),
                message: "Demo code has graceful error handling".to_string(),
                location: None,
                impact_points: 0.0,
            });
        }

        Ok(SubcategoryScore {
            id: "G2".to_string(),
            name: "Error Gracefulness".to_string(),
            score,
            max_score,
            findings,
        })
    }
}

struct ErrorPatternCounts {
    raw_unwrap: usize,
    contextual_unwrap: usize,
    raw_panic: usize,
    expect_with_message: usize,
    proper_error_handling: usize,
}

async fn count_error_patterns(demo_files: &[PathBuf]) -> ErrorPatternCounts {
    debug_assert!(!demo_files.is_empty(), "demo_files must not be empty");
    let contextual_fn_pattern = regex::Regex::new(
        r"(?s)fn\s+(test_|setup|init|proof_of_concept|example_)[^{]*\{[^}]*\.unwrap\(\)",
    )
    .expect("internal error");
    let unwrap_pattern = regex::Regex::new(r"\.unwrap\(\)").expect("internal error");
    let panic_pattern = regex::Regex::new(r"panic!\(").expect("internal error");
    let expect_pattern = regex::Regex::new(r#"\.expect\("[^"]+"\)"#).expect("internal error");

    let mut counts = ErrorPatternCounts {
        raw_unwrap: 0,
        contextual_unwrap: 0,
        raw_panic: 0,
        expect_with_message: 0,
        proper_error_handling: 0,
    };

    for file_path in demo_files {
        let Ok(content) = tokio::fs::read_to_string(file_path).await else {
            continue;
        };

        counts.contextual_unwrap += contextual_fn_pattern.find_iter(&content).count();
        let total_unwraps = unwrap_pattern.find_iter(&content).count();
        counts.raw_unwrap += total_unwraps.saturating_sub(counts.contextual_unwrap);
        counts.raw_panic += panic_pattern.find_iter(&content).count();
        counts.expect_with_message += expect_pattern.find_iter(&content).count();

        let error_handling_patterns = [
            r"\?;",
            r"match\s+.*\{[^}]*Err\(",
            r"if\s+let\s+Err\(",
            r"\.map_err\(",
            r"anyhow::|thiserror::|eyre::",
        ];
        for pattern in error_handling_patterns {
            if let Ok(re) = regex::Regex::new(pattern) {
                counts.proper_error_handling += re.find_iter(&content).count();
            }
        }
    }

    counts
}
