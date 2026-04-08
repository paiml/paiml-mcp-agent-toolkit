// Pattern detection and vulnerability matching logic for WASM security analysis.
// Included from security.rs -- shares parent module scope (no `use` imports here).

impl Default for PatternDetector {
    fn default() -> Self {
        Self::new()
    }
}

impl PatternDetector {
    #[must_use]
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    /// Create a new instance.
    pub fn new() -> Self {
        Self {
            patterns: Self::default_patterns(),
            found: Vec::new(),
        }
    }

    /// Default vulnerability patterns to detect
    fn default_patterns() -> Vec<VulnerabilityPattern> {
        vec![
            // Integer overflow in loop counter
            VulnerabilityPattern {
                name: "potential-integer-overflow",
                opcodes: vec![OpcodePattern::Sequence(vec![
                    OperatorMatcher::I32Add,
                    OperatorMatcher::BrIf,
                ])],
                severity: Severity::Medium,
            },
            // Potential timing side-channel
            VulnerabilityPattern {
                name: "timing-side-channel",
                opcodes: vec![OpcodePattern::Within {
                    distance: 5,
                    operators: vec![OperatorMatcher::I32Load, OperatorMatcher::BrIf],
                }],
                severity: Severity::Low,
            },
            // Unvalidated indirect call
            VulnerabilityPattern {
                name: "unvalidated-indirect-call",
                opcodes: vec![OpcodePattern::NotPrecededBy {
                    target: OperatorMatcher::CallIndirect,
                    guards: vec![OperatorMatcher::I32RemU, OperatorMatcher::I32And],
                }],
                severity: Severity::High,
            },
            // Unchecked memory growth
            VulnerabilityPattern {
                name: "unchecked-memory-growth",
                opcodes: vec![OpcodePattern::NotPrecededBy {
                    target: OperatorMatcher::MemoryGrow,
                    guards: vec![OperatorMatcher::I32LtU, OperatorMatcher::BrIf],
                }],
                severity: Severity::Medium,
            },
            // Potential buffer overflow
            VulnerabilityPattern {
                name: "potential-buffer-overflow",
                opcodes: vec![OpcodePattern::Sequence(vec![
                    OperatorMatcher::I32Add,
                    OperatorMatcher::I32Store,
                ])],
                severity: Severity::High,
            },
        ]
    }

    /// Scan WASM payload for vulnerability patterns
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn scan(&mut self, payload: &Payload) -> Result<()> {
        if let Payload::CodeSectionEntry(body) = payload {
            let reader = body.get_operators_reader()?;
            let operators: Vec<_> = reader.into_iter().collect::<Result<Vec<_>, _>>()?;

            // Check each pattern against the operators
            for pattern in &self.patterns {
                if let Some(location) = pattern.matches(&operators) {
                    self.found.push(VulnerabilityMatch {
                        pattern: pattern.name.to_string(),
                        location: body.range().clone(),
                        severity: pattern.severity.clone(),
                        operator_index: location,
                    });
                }
            }
        }
        Ok(())
    }

    /// Get all found vulnerabilities
    #[must_use]
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn finalize(&self) -> Vec<VulnerabilityMatch> {
        self.found.clone()
    }
}

impl VulnerabilityPattern {
    /// Check if pattern matches operators
    fn matches(&self, operators: &[Operator]) -> Option<usize> {
        for pattern in &self.opcodes {
            if let Some(idx) = pattern.find_in(operators) {
                return Some(idx);
            }
        }
        None
    }
}

fn find_sequence(seq: &[OperatorMatcher], operators: &[Operator]) -> Option<usize> {
    'outer: for i in 0..operators.len().saturating_sub(seq.len() - 1) {
        for (j, matcher) in seq.iter().enumerate() {
            if !matcher.matches(&operators[i + j]) {
                continue 'outer;
            }
        }
        return Some(i);
    }
    None
}

fn find_within(distance: usize, op_list: &[OperatorMatcher], operators: &[Operator]) -> Option<usize> {
    for i in 0..operators.len() {
        if op_list[0].matches(&operators[i]) {
            for j in (i + 1)..=(i + distance).min(operators.len() - 1) {
                if op_list.len() > 1 && op_list[1].matches(&operators[j]) {
                    return Some(i);
                }
            }
        }
    }
    None
}

fn find_not_preceded_by(target: &OperatorMatcher, guards: &[OperatorMatcher], operators: &[Operator]) -> Option<usize> {
    for i in 0..operators.len() {
        if target.matches(&operators[i]) {
            let has_guard = operators[i.saturating_sub(10)..i]
                .iter()
                .any(|op| guards.iter().any(|g| g.matches(op)));
            if !has_guard {
                return Some(i);
            }
        }
    }
    None
}

impl OpcodePattern {
    /// Find pattern in operator sequence
    fn find_in(&self, operators: &[Operator]) -> Option<usize> {
        match self {
            OpcodePattern::Sequence(seq) => find_sequence(seq, operators),
            OpcodePattern::Within { distance, operators: op_list } => {
                find_within(*distance, op_list, operators)
            }
            OpcodePattern::NotPrecededBy { target, guards } => {
                find_not_preceded_by(target, guards, operators)
            }
        }
    }
}

impl VulnerabilityMatch {
    /// Get risk score (0-100)
    #[must_use]
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "score_range")]
    pub fn risk_score(&self) -> u32 {
        match self.severity {
            Severity::Low => 25,
            Severity::Medium => 50,
            Severity::High => 75,
            Severity::Critical => 100,
        }
    }
}
