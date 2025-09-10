//! Pattern-based security analysis for WASM bytecode

use anyhow::Result;
use wasmparser::{Operator, Payload};
use std::ops::Range;
use serde::{Serialize, Deserialize};

/// Pattern detector for vulnerability scanning
#[derive(Debug, Clone)]
pub struct PatternDetector {
    patterns: Vec<VulnerabilityPattern>,
    found: Vec<VulnerabilityMatch>,
}

impl Default for PatternDetector {
    fn default() -> Self {
        Self::new()
    }
}

impl PatternDetector {
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
                opcodes: vec![
                    OpcodePattern::Sequence(vec![
                        OperatorMatcher::I32Add,
                        OperatorMatcher::BrIf,
                    ]),
                ],
                severity: Severity::Medium,
            },
            
            // Potential timing side-channel
            VulnerabilityPattern {
                name: "timing-side-channel",
                opcodes: vec![
                    OpcodePattern::Within {
                        distance: 5,
                        operators: vec![
                            OperatorMatcher::I32Load,
                            OperatorMatcher::BrIf,
                        ],
                    },
                ],
                severity: Severity::Low,
            },
            
            // Unvalidated indirect call
            VulnerabilityPattern {
                name: "unvalidated-indirect-call",
                opcodes: vec![
                    OpcodePattern::NotPrecededBy {
                        target: OperatorMatcher::CallIndirect,
                        guards: vec![
                            OperatorMatcher::I32RemU,
                            OperatorMatcher::I32And,
                        ],
                    },
                ],
                severity: Severity::High,
            },
            
            // Unchecked memory growth
            VulnerabilityPattern {
                name: "unchecked-memory-growth",
                opcodes: vec![
                    OpcodePattern::NotPrecededBy {
                        target: OperatorMatcher::MemoryGrow,
                        guards: vec![
                            OperatorMatcher::I32LtU,
                            OperatorMatcher::BrIf,
                        ],
                    },
                ],
                severity: Severity::Medium,
            },
            
            // Potential buffer overflow
            VulnerabilityPattern {
                name: "potential-buffer-overflow",
                opcodes: vec![
                    OpcodePattern::Sequence(vec![
                        OperatorMatcher::I32Add,
                        OperatorMatcher::I32Store,
                    ]),
                ],
                severity: Severity::High,
            },
        ]
    }

    /// Scan WASM payload for vulnerability patterns
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
    pub fn finalize(&self) -> Vec<VulnerabilityMatch> {
        self.found.clone()
    }
}

/// Vulnerability pattern definition
#[derive(Debug, Clone)]
pub struct VulnerabilityPattern {
    pub name: &'static str,
    pub opcodes: Vec<OpcodePattern>,
    pub severity: Severity,
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

/// Pattern matching strategies
#[derive(Debug, Clone)]
pub enum OpcodePattern {
    /// Exact sequence of operators
    Sequence(Vec<OperatorMatcher>),
    
    /// Operators within specified distance
    Within {
        distance: usize,
        operators: Vec<OperatorMatcher>,
    },
    
    /// Target not preceded by guards
    NotPrecededBy {
        target: OperatorMatcher,
        guards: Vec<OperatorMatcher>,
    },
}

impl OpcodePattern {
    /// Find pattern in operator sequence
    fn find_in(&self, operators: &[Operator]) -> Option<usize> {
        match self {
            OpcodePattern::Sequence(seq) => {
                // Find exact sequence
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
            
            OpcodePattern::Within { distance, operators: op_list } => {
                // Find operators within distance
                for i in 0..operators.len() {
                    if op_list[0].matches(&operators[i]) {
                        // Check if second operator is within distance
                        for j in (i + 1)..=(i + distance).min(operators.len() - 1) {
                            if op_list.len() > 1 && op_list[1].matches(&operators[j]) {
                                return Some(i);
                            }
                        }
                    }
                }
                None
            }
            
            OpcodePattern::NotPrecededBy { target, guards } => {
                // Find target not preceded by guards
                for i in 0..operators.len() {
                    if target.matches(&operators[i]) {
                        // Check if guards are missing before target
                        let mut has_guard = false;
                        for j in i.saturating_sub(10)..i {
                            for guard in guards {
                                if guard.matches(&operators[j]) {
                                    has_guard = true;
                                    break;
                                }
                            }
                        }
                        if !has_guard {
                            return Some(i);
                        }
                    }
                }
                None
            }
        }
    }
}

/// Operator matcher for pattern matching
#[derive(Debug, Clone)]
pub enum OperatorMatcher {
    I32Add,
    I32Sub,
    I32Mul,
    I32DivS,
    I32DivU,
    I32RemU,
    I32And,
    I32Or,
    I32Xor,
    I32Eqz,
    I32Eq,
    I32Ne,
    I32LtS,
    I32LtU,
    I32GtS,
    I32GtU,
    I32Load,
    I32Store,
    I64Load,
    I64Store,
    BrIf,
    Br,
    Call,
    CallIndirect,
    MemoryGrow,
    MemorySize,
    Any,
}

impl OperatorMatcher {
    /// Check if operator matches pattern
    fn matches(&self, op: &Operator) -> bool {
        use Operator::*;
        use OperatorMatcher as M;
        
        match (self, op) {
            (M::I32Add, I32Add) => true,
            (M::I32Sub, I32Sub) => true,
            (M::I32Mul, I32Mul) => true,
            (M::I32DivS, I32DivS) => true,
            (M::I32DivU, I32DivU) => true,
            (M::I32RemU, I32RemU) => true,
            (M::I32And, I32And) => true,
            (M::I32Or, I32Or) => true,
            (M::I32Xor, I32Xor) => true,
            (M::I32Eqz, I32Eqz) => true,
            (M::I32Eq, I32Eq) => true,
            (M::I32Ne, I32Ne) => true,
            (M::I32LtS, I32LtS) => true,
            (M::I32LtU, I32LtU) => true,
            (M::I32GtS, I32GtS) => true,
            (M::I32GtU, I32GtU) => true,
            (M::I32Load, I32Load { .. }) => true,
            (M::I32Store, I32Store { .. }) => true,
            (M::I64Load, I64Load { .. }) => true,
            (M::I64Store, I64Store { .. }) => true,
            (M::BrIf, BrIf { .. }) => true,
            (M::Br, Br { .. }) => true,
            (M::Call, Call { .. }) => true,
            (M::CallIndirect, CallIndirect { .. }) => true,
            (M::MemoryGrow, MemoryGrow { .. }) => true,
            (M::MemorySize, MemorySize { .. }) => true,
            (M::Any, _) => true,
            _ => false,
        }
    }
}

/// Vulnerability severity levels
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum Severity {
    Low,
    Medium,
    High,
    Critical,
}

/// Matched vulnerability instance
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VulnerabilityMatch {
    pub pattern: String,
    pub location: Range<usize>,
    pub severity: Severity,
    pub operator_index: usize,
}

impl VulnerabilityMatch {
    /// Get risk score (0-100)
    pub fn risk_score(&self) -> u32 {
        match self.severity {
            Severity::Low => 25,
            Severity::Medium => 50,
            Severity::High => 75,
            Severity::Critical => 100,
        }
    }
}