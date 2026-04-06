// Core analysis logic: SourceFlags pattern extraction, LightweightProvabilityAnalyzer
// methods (new, analyze_incrementally, analyze_function_fast, read_function_source,
// analyze_source_patterns, compute_impact_set, compute_confidence,
// calculate_provability_factor), AbstractInterpreter impl, and Default impl.

/// Extracted source pattern flags to keep cognitive complexity low.
struct SourceFlags {
    has_statements: bool,
    has_unsafe: bool,
    has_unwrap: bool,
    has_question_mark: bool,
    has_mut_ref: bool,
    has_raw_ptr: bool,
    has_io: bool,
    has_index: bool,
    has_loop: bool,
    has_iterator_chain: bool,
    returns_result_option: bool,
    is_test: bool,
    is_rust: bool,
}

impl SourceFlags {
    fn extract(source: &str, func_id: &FunctionId) -> Self {
        debug_assert!(!source.is_empty(), "source must not be empty");
        let body = source.split('{').nth(1).unwrap_or("");
        // A function body "has statements" if it contains any non-trivial content.
        // This includes explicit statements (with ;) AND expression returns (no ;).
        let body_content = body.trim().trim_end_matches('}').trim();
        let has_statements = !body_content.is_empty() && body_content.len() >= 3;
        let trimmed = source.trim();
        // Check function signature (before first '{') for return type
        let signature = source.split('{').next().unwrap_or("");

        Self {
            has_statements: !trimmed.is_empty() && has_statements,
            has_unsafe: source.contains("unsafe "),
            has_unwrap: source.contains(".unwrap()") || source.contains(".expect("),
            has_question_mark: body.contains('?'),
            has_mut_ref: source.contains("&mut "),
            has_raw_ptr: source.contains("*const ") || source.contains("*mut "),
            has_io: source.contains("println!")
                || source.contains("eprintln!")
                || source.contains("std::fs::")
                || source.contains("std::io::")
                || source.contains("tokio::")
                || source.contains("async fn"),
            has_index: source.contains('[') && source.contains(']'),
            has_loop: source.contains("for ")
                || source.contains("while ")
                || source.contains("loop "),
            has_iterator_chain: body.contains(".iter()")
                || body.contains(".into_iter()")
                || body.contains(".map(")
                || body.contains(".filter(")
                || body.contains(".collect()"),
            returns_result_option: signature.contains("-> Result<")
                || signature.contains("-> Option<")
                || signature.contains("-> anyhow::Result"),
            is_test: func_id.function_name.starts_with("test_")
                || source.contains("#[test]")
                || source.contains("#[tokio::test]"),
            is_rust: func_id.file_path.ends_with(".rs"),
        }
    }

    fn infer_nullability(&self) -> NullabilityLattice {
        if !self.is_rust || self.has_raw_ptr {
            NullabilityLattice::MaybeNull
        } else if self.has_unsafe && !self.returns_result_option {
            // unsafe without Result/Option return — weaker null guarantees
            NullabilityLattice::MaybeNull
        } else {
            // Safe Rust (or unsafe wrapped in Result/Option): references are always valid,
            // Option/Result are explicit null handling via type system
            NullabilityLattice::NotNull
        }
    }

    fn infer_bounds(&self) -> IntervalLattice {
        if self.has_unwrap && !self.has_question_mark {
            // Only unwrap, no ? — panics on failure, no bounds evidence
            IntervalLattice {
                lower: None,
                upper: None,
            }
        } else if self.has_unwrap && self.has_question_mark {
            // Mixed: some unwrap but also proper ? propagation — partial bounds
            IntervalLattice {
                lower: Some(0),
                upper: None,
            }
        } else if self.has_index {
            IntervalLattice {
                lower: Some(0),
                upper: None,
            }
        } else {
            // No unwrap — bounded behavior
            IntervalLattice {
                lower: Some(0),
                upper: Some(i64::MAX),
            }
        }
    }

    fn infer_aliasing(&self) -> AliasLattice {
        if self.has_raw_ptr || self.has_unsafe {
            // Raw pointers and unsafe bypass Rust's borrow checker — may alias
            AliasLattice::MayAlias
        } else {
            // Rust's borrow checker guarantees: &mut is exclusive, & is shared-immutable.
            // Both prevent aliasing. &mut does NOT mean "may alias" — it means exclusive access.
            AliasLattice::NoAlias
        }
    }

    fn infer_purity(&self) -> PurityLattice {
        if !self.has_io && !self.has_mut_ref && !self.has_unsafe && !self.has_loop {
            return PurityLattice::Pure;
        }
        if self.has_io || self.has_unsafe {
            return PurityLattice::WriteGlobal;
        }
        // Iterator chains (.iter().map().filter().collect()) are effectively ReadOnly
        // even if the function takes &mut self, since the chain operates on copies/refs
        if self.has_iterator_chain && !self.has_mut_ref {
            return PurityLattice::ReadOnly;
        }
        if self.has_mut_ref {
            PurityLattice::WriteLocal
        } else {
            PurityLattice::ReadOnly
        }
    }
}

impl LightweightProvabilityAnalyzer {
    /// Creates a new `LightweightProvabilityAnalyzer`
    ///
    /// # Examples
    ///
    /// ```rust
    /// use pmat::services::lightweight_provability_analyzer::LightweightProvabilityAnalyzer;
    ///
    /// let analyzer = LightweightProvabilityAnalyzer::new();
    /// // Analyzer is ready with default configuration
    /// ```
    #[must_use]
    pub fn new() -> Self {
        Self {
            abstract_interpreter: AbstractInterpreter { analysis_depth: 10 },
            proof_cache: Arc::new(DashMap::new()),
            current_version: 1,
        }
    }

    pub async fn analyze_incrementally(
        &self,
        changed_functions: &[FunctionId],
    ) -> Vec<ProofSummary> {
        debug_assert!(!changed_functions.is_empty(), "changed_functions must not be empty");
        let impact_set = self.compute_impact_set(changed_functions);

        impact_set
            .into_iter()
            .map(|func_id| {
                if let Some(cached) = self.proof_cache.get(&func_id) {
                    if cached.version == self.current_version {
                        return cached.clone();
                    }
                }

                let summary = self.analyze_function_fast(&func_id);
                self.proof_cache.insert(func_id, summary.clone());
                summary
            })
            .collect()
    }

    fn analyze_function_fast(&self, func_id: &FunctionId) -> ProofSummary {
        let start = std::time::Instant::now();

        // Read actual source code for evidence-based analysis
        let source_snippet = Self::read_function_source(func_id);
        let state = Self::analyze_source_patterns(&source_snippet, func_id);
        let mut verified_properties = Vec::new();

        // Extract verified properties from concrete analysis
        if state.nullability == NullabilityLattice::NotNull {
            verified_properties.push(VerifiedProperty {
                property_type: PropertyType::NullSafety,
                confidence: 0.9,
                evidence: "Rust type system guarantees non-null (no unsafe)".to_string(),
            });
        }

        if state.bounds.lower.is_some() && state.bounds.upper.is_some() {
            verified_properties.push(VerifiedProperty {
                property_type: PropertyType::BoundsCheck,
                confidence: 0.85,
                evidence: format!(
                    "Bounds: [{:?}, {:?}]",
                    state.bounds.lower, state.bounds.upper
                ),
            });
        }

        if state.aliasing == AliasLattice::NoAlias {
            verified_properties.push(VerifiedProperty {
                property_type: PropertyType::NoAliasing,
                confidence: 0.8,
                evidence: "No mutable aliasing detected".to_string(),
            });
        }

        if state.purity == PurityLattice::Pure {
            verified_properties.push(VerifiedProperty {
                property_type: PropertyType::PureFunction,
                confidence: 0.95,
                evidence: "Function has no side effects".to_string(),
            });
        } else if state.purity == PurityLattice::ReadOnly {
            verified_properties.push(VerifiedProperty {
                property_type: PropertyType::PureFunction,
                confidence: 0.6,
                evidence: "Function only reads state".to_string(),
            });
        }

        // Memory safety for safe Rust functions
        if state.nullability == NullabilityLattice::NotNull
            && state.aliasing != AliasLattice::MayAlias
        {
            verified_properties.push(VerifiedProperty {
                property_type: PropertyType::MemorySafety,
                confidence: 0.85,
                evidence: "Safe Rust guarantees memory safety".to_string(),
            });
        }

        let provability_score = self.compute_confidence(&state);

        ProofSummary {
            provability_score,
            verified_properties,
            analysis_time_us: start.elapsed().as_micros(),
            version: self.current_version,
        }
    }

    /// Read function source code for analysis.
    /// Returns up to 80 lines from the function start.
    fn read_function_source(func_id: &FunctionId) -> String {
        let Ok(content) = std::fs::read_to_string(&func_id.file_path) else {
            return String::new();
        };
        let lines: Vec<&str> = content.lines().collect();
        let start = func_id.line_number.saturating_sub(1).min(lines.len());
        let end = (start + 80).min(lines.len());
        // Extract until matching closing brace or limit
        let mut brace_depth = 0i32;
        let mut seen_opening_brace = false;
        let mut result = String::new();
        for line in &lines[start..end] {
            for ch in line.chars() {
                if ch == '{' {
                    brace_depth += 1;
                    seen_opening_brace = true;
                }
                if ch == '}' {
                    brace_depth -= 1;
                }
            }
            result.push_str(line);
            result.push('\n');
            if seen_opening_brace && brace_depth <= 0 && result.len() > 10 {
                break;
            }
        }
        result
    }

    /// Analyze concrete source patterns to produce differentiated property domains.
    fn analyze_source_patterns(source: &str, func_id: &FunctionId) -> PropertyDomain {
        debug_assert!(!source.is_empty(), "source must not be empty");
        let flags = SourceFlags::extract(source, func_id);
        if !flags.has_statements {
            return PropertyDomain {
                nullability: NullabilityLattice::MaybeNull,
                bounds: IntervalLattice {
                    lower: None,
                    upper: None,
                },
                aliasing: AliasLattice::MayAlias,
                purity: PurityLattice::Top,
            };
        }

        let purity = flags.infer_purity();

        if flags.is_test {
            return PropertyDomain {
                nullability: NullabilityLattice::NotNull,
                bounds: IntervalLattice {
                    lower: Some(0),
                    upper: Some(i64::MAX),
                },
                aliasing: AliasLattice::NoAlias,
                purity,
            };
        }

        PropertyDomain {
            nullability: flags.infer_nullability(),
            bounds: flags.infer_bounds(),
            aliasing: flags.infer_aliasing(),
            purity,
        }
    }

    fn compute_impact_set(&self, changed_functions: &[FunctionId]) -> Vec<FunctionId> {
        debug_assert!(!changed_functions.is_empty(), "changed_functions must not be empty");
        // In a real implementation, this would use call graph analysis
        // For now, just return the changed functions
        changed_functions.to_vec()
    }

    fn compute_confidence(&self, state: &PropertyDomain) -> f64 {
        let mut score = 0.0;
        let mut max_score = 0.0;

        // Nullability confidence
        max_score += 1.0;
        match state.nullability {
            NullabilityLattice::NotNull => score += 1.0,
            NullabilityLattice::MaybeNull => score += 0.5,
            _ => {}
        }

        // Bounds confidence
        max_score += 1.0;
        if state.bounds.lower.is_some() && state.bounds.upper.is_some() {
            score += 1.0;
        } else if state.bounds.lower.is_some() || state.bounds.upper.is_some() {
            score += 0.5;
        }

        // Aliasing confidence
        max_score += 1.0;
        match state.aliasing {
            AliasLattice::NoAlias => score += 1.0,
            AliasLattice::MayAlias => score += 0.3,
            _ => {}
        }

        // Purity confidence
        max_score += 1.0;
        match state.purity {
            PurityLattice::Pure => score += 1.0,
            PurityLattice::ReadOnly => score += 0.7,
            PurityLattice::WriteLocal => score += 0.3,
            _ => {}
        }

        if max_score > 0.0 {
            score / max_score
        } else {
            0.0
        }
    }

    /// Calculate provability factor for TDG integration
    ///
    /// # Examples
    ///
    /// ```
    /// use pmat::services::lightweight_provability_analyzer::{
    ///     LightweightProvabilityAnalyzer, ProofSummary, VerifiedProperty, PropertyType
    /// };
    ///
    /// let analyzer = LightweightProvabilityAnalyzer::new();
    /// let summary = ProofSummary {
    ///     provability_score: 0.8,
    ///     verified_properties: vec![
    ///         VerifiedProperty {
    ///             property_type: PropertyType::NullSafety,
    ///             confidence: 0.9,
    ///             evidence: "No null dereferences".to_string(),
    ///         }
    ///     ],
    ///     analysis_time_us: 1000,
    ///     version: 1,
    /// };
    ///
    /// let factor = analyzer.calculate_provability_factor(&summary);
    /// assert!(factor >= 0.0 && factor <= 5.0);
    /// ```
    #[must_use]
    pub fn calculate_provability_factor(&self, summary: &ProofSummary) -> f64 {
        debug_assert!((0.0..=1.0).contains(&summary.provability_score),
            "provability_score must be 0-1: {}", summary.provability_score);
        // Convert provability score (0-1) to factor (0-5) for TDG
        // Higher provability = lower TDG score
        let base_factor = 5.0 * (1.0 - summary.provability_score);

        // Adjust based on critical properties
        let critical_properties = summary
            .verified_properties
            .iter()
            .filter(|p| {
                matches!(
                    p.property_type,
                    PropertyType::MemorySafety | PropertyType::ThreadSafety
                )
            })
            .count();

        if critical_properties > 0 {
            base_factor * 0.7 // Reduce debt if critical properties are verified
        } else {
            base_factor
        }
    }
}

impl AbstractInterpreter {
    #[allow(dead_code)]
    fn analyze_iteration(&self, state: &PropertyDomain) -> PropertyDomain {
        // Lattice narrowing: move from Top toward concrete values.
        // Used as fallback when source analysis is unavailable.
        let mut new_state = state.clone();

        if state.nullability == NullabilityLattice::Top {
            new_state.nullability = NullabilityLattice::MaybeNull;
        }

        if state.bounds.lower.is_none() {
            new_state.bounds.lower = Some(0);
        }

        if state.purity == PurityLattice::Top {
            new_state.purity = PurityLattice::ReadOnly;
        }

        new_state
    }
}

impl Default for LightweightProvabilityAnalyzer {
    fn default() -> Self {
        Self::new()
    }
}
