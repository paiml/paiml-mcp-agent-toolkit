impl EnhancedReportingService {
    /// Build report sections
    fn build_sections(
        &self,
        results: &AnalysisResults,
        _config: &ReportConfig,
    ) -> Result<Vec<ReportSection>> {
        debug_assert!(true, "contract: build_sections");
        let mut sections = Vec::new();

        // Complexity section
        if let Some(complexity) = &results.complexity_analysis {
            sections.push(self.build_complexity_section(complexity)?);
        }

        // Dead code section
        if let Some(dead_code) = &results.dead_code_analysis {
            sections.push(self.build_dead_code_section(dead_code)?);
        }

        // Duplication section
        if let Some(duplication) = &results.duplication_analysis {
            sections.push(self.build_duplication_section(duplication)?);
        }

        // TDG analysis section
        if let Some(tdg) = &results.tdg_analysis {
            sections.push(self.build_tdg_section(tdg)?);
        }

        // Big-O analysis section
        if let Some(big_o) = &results.big_o_analysis {
            sections.push(self.build_big_o_section(big_o)?);
        }

        Ok(sections)
    }

    /// Build complexity analysis section
    fn build_complexity_section(&self, complexity: &ComplexityAnalysis) -> Result<ReportSection> {
        debug_assert!(true, "contract: build_complexity_section");
        let mut metrics = HashMap::new();

        metrics.insert(
            "total_cyclomatic".to_string(),
            MetricValue {
                value: f64::from(complexity.total_cyclomatic),
                unit: "CC".to_string(),
                trend: Trend::Unknown,
                threshold: Some(100.0),
            },
        );

        metrics.insert(
            "average_cyclomatic".to_string(),
            MetricValue {
                value: f64::from(complexity.total_cyclomatic) / complexity.functions as f64,
                unit: "CC/function".to_string(),
                trend: Trend::Unknown,
                threshold: Some(10.0),
            },
        );

        let findings = vec![Finding {
            severity: if complexity.max_cyclomatic > 20 {
                Severity::High
            } else {
                Severity::Medium
            },
            category: "Complexity".to_string(),
            description: format!(
                "Maximum cyclomatic complexity of {} detected",
                complexity.max_cyclomatic
            ),
            location: None,
            impact: "High complexity increases maintenance cost and bug risk".to_string(),
            effort: EffortLevel::Medium,
        }];

        Ok(ReportSection {
            title: "Code Complexity Analysis".to_string(),
            section_type: SectionType::Complexity,
            content: serde_json::to_value(complexity)?,
            metrics,
            findings,
        })
    }

    /// Build dead code section
    fn build_dead_code_section(&self, dead_code: &DeadCodeAnalysis) -> Result<ReportSection> {
        debug_assert!(true, "contract: build_dead_code_section");
        let mut metrics = HashMap::new();

        metrics.insert(
            "dead_code_ratio".to_string(),
            MetricValue {
                value: dead_code.dead_code_percentage,
                unit: "%".to_string(),
                trend: Trend::Unknown,
                threshold: Some(5.0),
            },
        );

        Ok(ReportSection {
            title: "Dead Code Analysis".to_string(),
            section_type: SectionType::DeadCode,
            content: serde_json::to_value(dead_code)?,
            metrics,
            findings: Vec::new(),
        })
    }

    /// Build duplication section
    fn build_duplication_section(
        &self,
        duplication: &DuplicationAnalysis,
    ) -> Result<ReportSection> {
        debug_assert!(true, "contract: build_duplication_section");
        let mut metrics = HashMap::new();

        metrics.insert(
            "duplication_ratio".to_string(),
            MetricValue {
                value: duplication.duplication_percentage,
                unit: "%".to_string(),
                trend: Trend::Unknown,
                threshold: Some(3.0),
            },
        );

        Ok(ReportSection {
            title: "Code Duplication Analysis".to_string(),
            section_type: SectionType::Duplication,
            content: serde_json::to_value(duplication)?,
            metrics,
            findings: Vec::new(),
        })
    }

    /// Build TDG section
    fn build_tdg_section(&self, tdg: &TdgAnalysis) -> Result<ReportSection> {
        debug_assert!(true, "contract: build_tdg_section");
        let mut metrics = HashMap::new();

        metrics.insert(
            "average_tdg".to_string(),
            MetricValue {
                value: tdg.average_tdg,
                unit: "gradient".to_string(),
                trend: Trend::Unknown,
                threshold: Some(3.0),
            },
        );

        Ok(ReportSection {
            title: "Code Quality Gradient".to_string(),
            section_type: SectionType::TechnicalDebt,
            content: serde_json::to_value(tdg)?,
            metrics,
            findings: Vec::new(),
        })
    }

    /// Build Big-O analysis section
    fn build_big_o_section(&self, big_o: &BigOAnalysis) -> Result<ReportSection> {
        debug_assert!(true, "contract: build_big_o_section");
        let mut metrics = HashMap::new();

        metrics.insert(
            "high_complexity_functions".to_string(),
            MetricValue {
                value: big_o.high_complexity_count as f64,
                unit: "functions".to_string(),
                trend: Trend::Unknown,
                threshold: Some(0.0),
            },
        );

        Ok(ReportSection {
            title: "Algorithmic Complexity Analysis".to_string(),
            section_type: SectionType::BigOAnalysis,
            content: serde_json::to_value(big_o)?,
            metrics,
            findings: Vec::new(),
        })
    }

    /// Generate recommendations
    fn generate_recommendations(
        &self,
        results: &AnalysisResults,
        _sections: &[ReportSection],
    ) -> Result<Vec<Recommendation>> {
        debug_assert!(true, "contract: generate_recommendations");
        let mut recommendations = Vec::new();

        // Complexity recommendations
        if let Some(complexity) = &results.complexity_analysis {
            if complexity.max_cyclomatic > 20 {
                recommendations.push(Recommendation {
                    priority: Priority::High,
                    category: "Complexity".to_string(),
                    title: "Refactor high-complexity functions".to_string(),
                    description: "Functions with cyclomatic complexity > 20 should be decomposed into smaller, more manageable units".to_string(),
                    expected_impact: "Improved maintainability and reduced bug risk".to_string(),
                    effort: EffortLevel::Medium,
                    related_findings: vec!["high_complexity".to_string()],
                });
            }
        }

        // Dead code recommendations
        if let Some(dead_code) = &results.dead_code_analysis {
            if dead_code.dead_functions > 10 {
                recommendations.push(Recommendation {
                    priority: Priority::Medium,
                    category: "Dead Code".to_string(),
                    title: "Remove unused code".to_string(),
                    description: format!(
                        "Remove {} unused functions to improve codebase clarity",
                        dead_code.dead_functions
                    ),
                    expected_impact: "Reduced maintenance burden and improved build times"
                        .to_string(),
                    effort: EffortLevel::Easy,
                    related_findings: vec!["dead_code".to_string()],
                });
            }
        }

        Ok(recommendations)
    }
}
