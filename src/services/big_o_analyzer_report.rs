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

        // DETERMINISM: sorting on the class alone leaves every same-class
        // function in input order, which is only accidentally stable. The
        // (file, line, name) suffix makes the order a function of the code.
        high_complexity.sort_by(|a, b| {
            (a.time_complexity.class as u8)
                .cmp(&(b.time_complexity.class as u8))
                .then_with(|| a.file_path.cmp(&b.file_path))
                .then_with(|| a.line_number.cmp(&b.line_number))
                .then_with(|| a.function_name.cmp(&b.function_name))
        });

        // Generate pattern matches
        //
        // DETERMINISM: `pattern_counts` is an `FxHashMap`, so collecting it
        // straight into a `Vec` put the same patterns in a different array
        // order on every run. Sorted by descending occurrences then by name.
        let mut pattern_matches: Vec<_> = pattern_counts
            .into_iter()
            .map(|(name, count)| PatternMatch {
                pattern_name: name,
                occurrences: count,
                typical_complexity: BigOClass::Linear, // Default
            })
            .collect();
        pattern_matches.sort_by(|a, b| {
            b.occurrences
                .cmp(&a.occurrences)
                .then_with(|| a.pattern_name.cmp(&b.pattern_name))
        });

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

    /// How many high-complexity functions are LISTED vs how many were FOUND.
    ///
    /// The distribution is computed over every analysed function and is never
    /// touched by the `--top-files` / `--high-complexity-only` filters, so
    /// `quadratic + cubic + exponential` (Factorial is folded into
    /// `exponential` by `increment_distribution`) is exactly the number of
    /// functions that qualified before truncation. `.max(listed)` keeps a part
    /// from ever exceeding its whole if a caller hands us a report whose
    /// distribution and list disagree.
    fn high_complexity_listed_and_found(report: &BigOAnalysisReport) -> (usize, usize) {
        let listed = report.high_complexity_functions.len();
        let dist = &report.complexity_distribution;
        let found = (dist.quadratic + dist.cubic + dist.exponential).max(listed);
        (listed, found)
    }

    /// The distribution rows, in report order, as `(label, count, is_high)`.
    ///
    /// ONE AUTHORITY FOR "HIGH": `--high-complexity-only` used to be applied by
    /// `retain`ing `high_complexity_functions` on the same predicate
    /// `build_report` had already used to build that list, so the flag could not
    /// remove an element for any input and every format was byte-identical with
    /// and without it. The list was already high-only; the DISTRIBUTION was not,
    /// and it is what the flag narrows. `is_high` marks exactly the classes
    /// `is_high_complexity_class` accepts (Quadratic | Cubic | Exponential |
    /// Factorial — Factorial is folded into `exponential` by
    /// `increment_distribution`), so no renderer can drift from the list.
    /// Renderers keep their own display labels (`O(n^2)` in markdown, `O(n²)` on
    /// the terminal, `O(?)` in JSON); the key here is the JSON one and the order
    /// is fixed, so a renderer can `zip` its own label table against these rows.
    #[must_use]
    pub fn distribution_rows(report: &BigOAnalysisReport) -> [(&'static str, usize, bool); 8] {
        let dist = &report.complexity_distribution;
        [
            ("O(1)", dist.constant, false),
            ("O(log n)", dist.logarithmic, false),
            ("O(n)", dist.linear, false),
            ("O(n log n)", dist.linearithmic, false),
            ("O(n^2)", dist.quadratic, true),
            ("O(n^3)", dist.cubic, true),
            ("O(2^n)", dist.exponential, true),
            ("O(?)", dist.unknown, false),
        ]
    }

    /// True when a distribution row survives `--high-complexity-only`.
    #[must_use]
    pub fn distribution_row_kept(is_high: bool, high_complexity_only: bool) -> bool {
        is_high || !high_complexity_only
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
        self.format_as_json_scoped(report, false)
    }

    /// `format_as_json`, with the `--high-complexity-only` scope applied.
    ///
    /// With the flag on, `distribution` carries only the O(n^2)-or-worse rows
    /// and `high_complexity_only: true` says so, so a consumer can tell a
    /// narrowed document from a full one instead of receiving the same bytes
    /// either way.
    ///
    /// # Errors
    /// Returns an error if the report cannot be serialised.
    pub fn format_as_json_scoped(
        &self,
        report: &BigOAnalysisReport,
        high_complexity_only: bool,
    ) -> Result<String> {
        let (listed, found) = Self::high_complexity_listed_and_found(report);
        let distribution: serde_json::Map<String, serde_json::Value> =
            Self::distribution_rows(report)
                .into_iter()
                .filter(|(_, _, is_high)| {
                    Self::distribution_row_kept(*is_high, high_complexity_only)
                })
                .map(|(key, count, _)| (key.to_string(), serde_json::json!(count)))
                .collect();
        let json = serde_json::json!({
            "summary": {
                "analyzed_functions": report.analyzed_functions,
                // A TOTAL THAT IS SECRETLY A CAP is a fabrication. This field
                // is the length of `high_complexity_functions`, and the default
                // `--top-files 10` truncates that list: on pmat's own tree the
                // distribution said O(n^2)+O(n^3)+O(2^n) = 106 while
                // `high_complexity_count` said 24, with nothing saying so. Both
                // numbers are now named, and the flag is explicit.
                "high_complexity_count": listed,
                "high_complexity_found": found,
                "high_complexity_truncated": listed < found,
                "high_complexity_only": high_complexity_only,
            },
            "distribution": distribution,
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
        self.format_as_markdown_scoped(report, false)
    }

    /// `format_as_markdown`, with the `--high-complexity-only` scope applied.
    #[must_use]
    pub fn format_as_markdown_scoped(
        &self,
        report: &BigOAnalysisReport,
        high_complexity_only: bool,
    ) -> String {
        let mut md = String::with_capacity(1024);

        md.push_str("# Big-O Complexity Analysis Report\n\n");

        md.push_str("## Summary\n\n");
        md.push_str(&format!(
            "- **Total Functions Analyzed**: {}\n",
            report.analyzed_functions
        ));
        // Same truncation disclosure as `format_as_json`: the listed count is
        // capped by `--top-files`, the found count is not.
        let (listed, found) = Self::high_complexity_listed_and_found(report);
        if listed < found {
            md.push_str(&format!(
                "- **High Complexity Functions**: {listed} listed of {found} found (truncated by --top-files)\n\n"
            ));
        } else {
            md.push_str(&format!("- **High Complexity Functions**: {listed}\n\n"));
        }

        Self::format_distribution_table(&mut md, report, high_complexity_only);

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
    fn format_distribution_table(
        md: &mut String,
        report: &BigOAnalysisReport,
        high_complexity_only: bool,
    ) {
        if high_complexity_only {
            md.push_str("## Complexity Distribution (--high-complexity-only)\n\n");
        } else {
            md.push_str("## Complexity Distribution\n\n");
        }
        md.push_str("| Complexity | Count | Percentage |\n");
        md.push_str("|------------|-------|------------|\n");

        let total = report.analyzed_functions as f64;
        // Markdown spells the unknown row "Unknown" where JSON keys it "O(?)";
        // the counts and the is-high predicate come from the shared rows so the
        // two cannot disagree about WHICH rows `--high-complexity-only` keeps.
        const LABELS: [&str; 8] = [
            "O(1)",
            "O(log n)",
            "O(n)",
            "O(n log n)",
            "O(n^2)",
            "O(n^3)",
            "O(2^n)",
            "Unknown",
        ];

        for ((_, count, is_high), label) in Self::distribution_rows(report).into_iter().zip(LABELS)
        {
            if !Self::distribution_row_kept(is_high, high_complexity_only) {
                continue;
            }
            let suffix = if label == "Unknown" { "\n" } else { "" };
            md.push_str(&format!(
                "| {} | {} | {:.1}% |{}\n",
                label,
                count,
                (count as f64 / total) * 100.0,
                suffix
            ));
        }
        if high_complexity_only {
            md.push('\n');
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
