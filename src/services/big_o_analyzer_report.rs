// Report building and formatting for BigOAnalyzer
// This file is include!()'d into big_o_analyzer.rs scope.
// NO use imports or #! inner attributes allowed.

impl BigOAnalyzer {
    /// Build analysis report
    fn build_report(
        &self,
        functions: Vec<FunctionComplexity>,
        pattern_counts: rustc_hash::FxHashMap<String, usize>,
    ) -> BigOAnalysisReport {
        let mut distribution = ComplexityDistribution {
            constant: 0,
            logarithmic: 0,
            linear: 0,
            linearithmic: 0,
            quadratic: 0,
            cubic: 0,
            exponential: 0,
            unknown: 0,
        };

        let total_functions = functions.len();

        // Count complexity distribution
        for func in &functions {
            Self::increment_distribution(&mut distribution, &func.time_complexity.class);
        }

        // Find high complexity functions
        let mut high_complexity: Vec<_> = functions
            .into_iter()
            .filter(|f| {
                matches!(
                    f.time_complexity.class,
                    BigOClass::Quadratic
                        | BigOClass::Cubic
                        | BigOClass::Exponential
                        | BigOClass::Factorial
                )
            })
            .collect();

        high_complexity.sort_by_key(|f| f.time_complexity.class as u8);

        // Generate pattern matches
        let pattern_matches: Vec<_> = pattern_counts
            .into_iter()
            .map(|(name, count)| PatternMatch {
                pattern_name: name,
                occurrences: count,
                typical_complexity: BigOClass::Linear, // Default
            })
            .collect();

        // Generate recommendations
        let recommendations =
            Self::generate_recommendations(&distribution, total_functions);

        BigOAnalysisReport {
            analyzed_functions: total_functions,
            complexity_distribution: distribution,
            high_complexity_functions: high_complexity,
            pattern_matches,
            recommendations,
        }
    }

    /// Increment the appropriate distribution counter for a complexity class
    fn increment_distribution(distribution: &mut ComplexityDistribution, class: &BigOClass) {
        match class {
            BigOClass::Constant => distribution.constant += 1,
            BigOClass::Logarithmic => distribution.logarithmic += 1,
            BigOClass::Linear => distribution.linear += 1,
            BigOClass::Linearithmic => distribution.linearithmic += 1,
            BigOClass::Quadratic => distribution.quadratic += 1,
            BigOClass::Cubic => distribution.cubic += 1,
            BigOClass::Exponential => distribution.exponential += 1,
            BigOClass::Factorial => distribution.exponential += 1,
            BigOClass::Unknown => distribution.unknown += 1,
        }
    }

    /// Generate recommendations based on complexity distribution
    fn generate_recommendations(
        distribution: &ComplexityDistribution,
        total_functions: usize,
    ) -> Vec<String> {
        let mut recommendations = Vec::new();

        if distribution.quadratic > 0 {
            recommendations.push(format!(
                "Found {} functions with O(n^2) complexity. Consider optimization.",
                distribution.quadratic
            ));
        }

        if distribution.exponential > 0 {
            recommendations.push(format!(
                "Found {} functions with exponential complexity! These need immediate attention.",
                distribution.exponential
            ));
        }

        if distribution.unknown > total_functions / 4 {
            recommendations.push(
                "Many functions have unknown complexity. Consider adding more explicit patterns."
                    .to_string(),
            );
        }

        recommendations
    }

    /// Format report as JSON
    ///
    /// # Examples
    ///
    /// ```
    /// use pmat::services::big_o_analyzer::{BigOAnalyzer, BigOAnalysisReport, ComplexityDistribution};
    ///
    /// let analyzer = BigOAnalyzer::new();
    /// let report = BigOAnalysisReport {
    ///     analyzed_functions: 10,
    ///     complexity_distribution: ComplexityDistribution {
    ///         constant: 3,
    ///         logarithmic: 1,
    ///         linear: 4,
    ///         linearithmic: 1,
    ///         quadratic: 1,
    ///         cubic: 0,
    ///         exponential: 0,
    ///         unknown: 0,
    ///     },
    ///     high_complexity_functions: vec![],
    ///     pattern_matches: vec![],
    ///     recommendations: vec![],
    /// };
    ///
    /// let json = analyzer.format_as_json(&report).unwrap();
    /// assert!(json.contains("\"analyzed_functions\": 10"));
    /// ```
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn format_as_json(&self, report: &BigOAnalysisReport) -> Result<String> {
        let json = serde_json::json!({
            "summary": {
                "analyzed_functions": report.analyzed_functions,
                "high_complexity_count": report.high_complexity_functions.len(),
            },
            "distribution": {
                "O(1)": report.complexity_distribution.constant,
                "O(log n)": report.complexity_distribution.logarithmic,
                "O(n)": report.complexity_distribution.linear,
                "O(n log n)": report.complexity_distribution.linearithmic,
                "O(n^2)": report.complexity_distribution.quadratic,
                "O(n^3)": report.complexity_distribution.cubic,
                "O(2^n)": report.complexity_distribution.exponential,
                "O(?)": report.complexity_distribution.unknown,
            },
            "high_complexity_functions": report.high_complexity_functions.iter().map(|f| {
                serde_json::json!({
                    "file": f.file_path.display().to_string(),
                    "function": f.function_name,
                    "line": f.line_number,
                    "time_complexity": f.time_complexity.notation(),
                    "space_complexity": f.space_complexity.notation(),
                    "confidence": f.confidence,
                })
            }).collect::<Vec<_>>(),
            "pattern_matches": report.pattern_matches.iter().map(|p| {
                serde_json::json!({
                    "pattern": p.pattern_name,
                    "occurrences": p.occurrences,
                })
            }).collect::<Vec<_>>(),
            "recommendations": report.recommendations,
        });

        Ok(serde_json::to_string_pretty(&json)?)
    }

    /// Format report as Markdown
    #[must_use]
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn format_as_markdown(&self, report: &BigOAnalysisReport) -> String {
        let mut md = String::with_capacity(1024);

        md.push_str("# Big-O Complexity Analysis Report\n\n");

        md.push_str("## Summary\n\n");
        md.push_str(&format!(
            "- **Total Functions Analyzed**: {}\n",
            report.analyzed_functions
        ));
        md.push_str(&format!(
            "- **High Complexity Functions**: {}\n\n",
            report.high_complexity_functions.len()
        ));

        Self::format_distribution_table(&mut md, report);

        if !report.high_complexity_functions.is_empty() {
            Self::format_high_complexity_table(&mut md, report);
        }

        if !report.recommendations.is_empty() {
            md.push_str("## Recommendations\n\n");
            for rec in &report.recommendations {
                md.push_str(&format!("- {rec}\n"));
            }
        }

        md
    }

    /// Format the complexity distribution table in markdown
    fn format_distribution_table(md: &mut String, report: &BigOAnalysisReport) {
        md.push_str("## Complexity Distribution\n\n");
        md.push_str("| Complexity | Count | Percentage |\n");
        md.push_str("|------------|-------|------------|\n");

        let total = report.analyzed_functions as f64;
        let dist = &report.complexity_distribution;

        let rows: &[(&str, usize)] = &[
            ("O(1)", dist.constant),
            ("O(log n)", dist.logarithmic),
            ("O(n)", dist.linear),
            ("O(n log n)", dist.linearithmic),
            ("O(n^2)", dist.quadratic),
            ("O(n^3)", dist.cubic),
            ("O(2^n)", dist.exponential),
            ("Unknown", dist.unknown),
        ];

        for (label, count) in rows {
            let suffix = if *label == "Unknown" { "\n" } else { "" };
            md.push_str(&format!(
                "| {} | {} | {:.1}% |{}\n",
                label,
                count,
                (*count as f64 / total) * 100.0,
                suffix
            ));
        }
    }

    /// Format the high-complexity functions table in markdown
    fn format_high_complexity_table(md: &mut String, report: &BigOAnalysisReport) {
        md.push_str("## High Complexity Functions\n\n");
        md.push_str(
            "| File | Function | Line | Time Complexity | Space Complexity | Confidence |\n",
        );
        md.push_str(
            "|------|----------|------|-----------------|------------------|------------|\n",
        );

        for func in &report.high_complexity_functions {
            md.push_str(&format!(
                "| {} | {} | {} | {} | {} | {}% |\n",
                func.file_path
                    .file_name()
                    .unwrap_or_default()
                    .to_string_lossy(),
                func.function_name,
                func.line_number,
                func.time_complexity.notation(),
                func.space_complexity.notation(),
                func.confidence
            ));
        }
        md.push('\n');
    }
}
