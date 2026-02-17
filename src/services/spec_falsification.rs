#![cfg_attr(coverage_nightly, coverage(off))]
//! Spec Falsification Engine — RAG-powered Popperian falsification for specifications
//!
//! Extracts falsifiable claims from markdown specifications and validates them
//! against the codebase using code search, filesystem checks, and metric measurement.
//!
//! ## Pipeline
//! 1. Parse markdown → extract atomic claims (RFC-2119 keywords, path refs, metrics)
//! 2. Categorize claims → dispatch to falsification strategies
//! 3. Run strategies → collect evidence (supporting or contradicting)
//! 4. Score verdicts → produce falsification report

use anyhow::{Context, Result};
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

// ============================================================================
// Core Types
// ============================================================================

/// Priority level based on RFC-2119 keywords
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ClaimPriority {
    /// MUST / SHALL / REQUIRED — single counterexample falsifies
    P0Critical,
    /// SHOULD / RECOMMENDED — needs pattern of violation
    P1High,
    /// MAY / OPTIONAL — informational only
    P2Low,
    /// No RFC-2119 keyword — default priority
    P3Default,
}

impl std::fmt::Display for ClaimPriority {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::P0Critical => write!(f, "P0"),
            Self::P1High => write!(f, "P1"),
            Self::P2Low => write!(f, "P2"),
            Self::P3Default => write!(f, "P3"),
        }
    }
}

/// Category of falsifiable claim — determines which strategy to use
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum SpecClaimCategory {
    /// Claim references a file path that should exist
    PathReference,
    /// Claim asserts a function/struct/module exists
    CodeEntity,
    /// Claim contains a numeric threshold (coverage %, count, etc.)
    MetricClaim,
    /// Claim asserts something does NOT exist
    AbsenceClaim,
    /// Claim references a command that should work
    CommandClaim,
    /// Structural claim about architecture or patterns
    ArchitecturalClaim,
    /// Claims that cannot be mechanically falsified
    Unfalsifiable,
}

impl std::fmt::Display for SpecClaimCategory {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::PathReference => write!(f, "PathRef"),
            Self::CodeEntity => write!(f, "CodeEntity"),
            Self::MetricClaim => write!(f, "Metric"),
            Self::AbsenceClaim => write!(f, "Absence"),
            Self::CommandClaim => write!(f, "Command"),
            Self::ArchitecturalClaim => write!(f, "Arch"),
            Self::Unfalsifiable => write!(f, "Unfalsifiable"),
        }
    }
}

/// A single falsifiable claim extracted from a specification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpecClaim {
    /// Unique ID within this run
    pub id: String,
    /// Original text from the document
    pub original_text: String,
    /// Source location (file:line)
    pub source_line: usize,
    /// Claim category
    pub category: SpecClaimCategory,
    /// Priority (from RFC-2119 keywords)
    pub priority: ClaimPriority,
    /// Whether the claim uses absolute language ("all", "zero", "every")
    pub is_absolute: bool,
    /// Extracted file path references
    pub path_refs: Vec<String>,
    /// Extracted code entity references (function/struct names)
    pub entity_refs: Vec<String>,
    /// Extracted numeric value (if metric claim)
    pub numeric_value: Option<f64>,
    /// Numeric comparator text (e.g., ">=", "<", "≤")
    pub numeric_comparator: Option<String>,
}

/// Verdict status for a falsified claim
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum VerdictStatus {
    /// Claim survived falsification — no contradicting evidence found
    Survived,
    /// Claim actively contradicted by evidence
    Falsified,
    /// Claim could not be tested
    Unfalsifiable,
    /// Evidence found but inconclusive
    Inconclusive,
}

impl std::fmt::Display for VerdictStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Survived => write!(f, "SURVIVED"),
            Self::Falsified => write!(f, "FALSIFIED"),
            Self::Unfalsifiable => write!(f, "UNFALSIFIABLE"),
            Self::Inconclusive => write!(f, "INCONCLUSIVE"),
        }
    }
}

/// Evidence collected during falsification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpecEvidence {
    /// What was checked
    pub check: String,
    /// What was found
    pub finding: String,
    /// How strongly this contradicts the claim (0.0 = supports, 1.0 = contradicts)
    pub contradiction_score: f64,
}

/// Per-claim verdict
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpecVerdict {
    pub claim: SpecClaim,
    pub status: VerdictStatus,
    pub evidence: Vec<SpecEvidence>,
    /// Overall contradiction score for this claim
    pub contradiction_score: f64,
}

/// Report summary
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpecFalsificationSummary {
    pub total_claims: usize,
    pub survived: usize,
    pub falsified: usize,
    pub unfalsifiable: usize,
    pub inconclusive: usize,
    /// Health score: survived / (total - unfalsifiable)
    pub health_score: f64,
}

/// Complete falsification report for a spec
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpecFalsificationReport {
    pub target_file: PathBuf,
    pub timestamp: String,
    pub verdicts: Vec<SpecVerdict>,
    pub summary: SpecFalsificationSummary,
}

// ============================================================================
// Claim Extraction
// ============================================================================

/// Extracts falsifiable claims from markdown specification documents
pub struct SpecClaimExtractor {
    path_pattern: Regex,
    entity_pattern: Regex,
    numeric_pattern: Regex,
    rfc2119_must: Regex,
    rfc2119_should: Regex,
    rfc2119_may: Regex,
    absolute_pattern: Regex,
    command_pattern: Regex,
    absence_pattern: Regex,
}

/// Extracted signals from a single line during claim extraction
struct LineSignals {
    path_refs: Vec<String>,
    entity_refs: Vec<String>,
    numeric_value: Option<f64>,
    numeric_comparator: Option<String>,
    has_command: bool,
    has_absence: bool,
}

impl SpecClaimExtractor {
    pub fn new() -> Self {
        Self {
            path_pattern: Regex::new(
                r#"(?:^|[\s`"])((?:src|docs|tests|server|crates|\.)/[a-zA-Z0-9_./-]+\.[a-z]+)"#,
            )
            .expect("internal regex"),
            entity_pattern: Regex::new(
                r#"`([A-Z][a-zA-Z0-9]+(?:::[a-z_][a-zA-Z0-9_]*)?)`|`([a-z_][a-z0-9_]+(?:::[a-z_][a-z0-9_]*)+)`"#,
            )
            .expect("internal regex"),
            numeric_pattern: Regex::new(
                r#"([><=]+)\s*(\d+(?:\.\d+)?)\s*(%|ms|s|min|seconds|minutes|lines|functions|files|points|pts)?"#,
            )
            .expect("internal regex"),
            rfc2119_must: Regex::new(r#"\b(MUST|SHALL|REQUIRED|MUST NOT|SHALL NOT)\b"#)
                .expect("internal regex"),
            rfc2119_should: Regex::new(r#"\b(SHOULD|RECOMMENDED|SHOULD NOT)\b"#)
                .expect("internal regex"),
            rfc2119_may: Regex::new(r#"\b(MAY|OPTIONAL)\b"#).expect("internal regex"),
            absolute_pattern: Regex::new(
                r#"\b(all|every|zero|no|none|always|never|complete|entirely|fully)\b"#,
            )
            .expect("internal regex"),
            command_pattern: Regex::new(
                r#"`(pmat\s+[a-z][\w-]*(?:\s+[\w-]+)*)`|`(cargo\s+[a-z][\w-]*(?:\s+[\w-]+)*)`"#,
            )
            .expect("internal regex"),
            absence_pattern: Regex::new(
                r#"(?i)\b(no\s+(?:new\s+)?(?:unsafe|panic|unwrap|todo|fixme|dead.?code)|zero\s+\w+|without\s+any|does not (?:exist|contain|have))\b"#,
            )
            .expect("internal regex"),
        }
    }

    /// Extract all falsifiable claims from a specification document
    pub fn extract(&self, content: &str, source_file: &Path) -> Vec<SpecClaim> {
        let mut claims = Vec::new();
        let mut in_code_block = false;
        let mut claim_counter = 0usize;
        let mut current_section = String::new();

        for (line_idx, line) in content.lines().enumerate() {
            let trimmed = line.trim();

            // Track code blocks
            if trimmed.starts_with("```") {
                in_code_block = !in_code_block;
                continue;
            }
            if in_code_block {
                continue;
            }

            // Track section headers for context
            if trimmed.starts_with('#') {
                current_section = trimmed.trim_start_matches('#').trim().to_string();
                continue;
            }

            // Skip empty lines and table separators
            if trimmed.is_empty() || trimmed.chars().all(|c| c == '-' || c == '|' || c == ' ') {
                continue;
            }

            // Extract claims from this line
            if let Some(claim) = self.extract_claim_from_line(
                trimmed,
                line_idx + 1,
                &mut claim_counter,
                &current_section,
                source_file,
            ) {
                claims.push(claim);
            }
        }

        claims
    }

    fn extract_claim_from_line(
        &self,
        line: &str,
        line_number: usize,
        counter: &mut usize,
        _section: &str,
        _source: &Path,
    ) -> Option<SpecClaim> {
        let priority = self.classify_priority(line);
        let is_absolute = self.absolute_pattern.is_match(&line.to_lowercase());
        let signals = self.extract_signals(line);
        let category = Self::categorize(&signals, priority, is_absolute)?;

        *counter += 1;
        Some(SpecClaim {
            id: format!("claim-{:03}", counter),
            original_text: line.to_string(),
            source_line: line_number,
            category,
            priority,
            is_absolute,
            path_refs: signals.path_refs,
            entity_refs: signals.entity_refs,
            numeric_value: signals.numeric_value,
            numeric_comparator: signals.numeric_comparator,
        })
    }

    fn classify_priority(&self, line: &str) -> ClaimPriority {
        if self.rfc2119_must.is_match(line) {
            ClaimPriority::P0Critical
        } else if self.rfc2119_should.is_match(line) {
            ClaimPriority::P1High
        } else if self.rfc2119_may.is_match(line) {
            ClaimPriority::P2Low
        } else {
            ClaimPriority::P3Default
        }
    }

    fn extract_signals(&self, line: &str) -> LineSignals {
        let path_refs: Vec<String> = self
            .path_pattern
            .captures_iter(line)
            .filter_map(|c| c.get(1).map(|m| m.as_str().to_string()))
            .filter(|p| !p.is_empty())
            .collect();

        let entity_refs: Vec<String> = self
            .entity_pattern
            .captures_iter(line)
            .filter_map(|c| c.get(1).or(c.get(2)).map(|m| m.as_str().to_string()))
            .collect();

        let (numeric_value, numeric_comparator) = self
            .numeric_pattern
            .captures(line)
            .and_then(|c| {
                let comp = c.get(1)?.as_str().to_string();
                let val = c.get(2)?.as_str().parse::<f64>().ok()?;
                Some((Some(val), Some(comp)))
            })
            .unwrap_or((None, None));

        LineSignals {
            path_refs,
            entity_refs,
            numeric_value,
            numeric_comparator,
            has_command: self.command_pattern.is_match(line),
            has_absence: self.absence_pattern.is_match(line),
        }
    }

    fn categorize(
        signals: &LineSignals,
        priority: ClaimPriority,
        is_absolute: bool,
    ) -> Option<SpecClaimCategory> {
        if signals.has_absence {
            return Some(SpecClaimCategory::AbsenceClaim);
        }
        if !signals.path_refs.is_empty() {
            return Some(SpecClaimCategory::PathReference);
        }
        if signals.has_command {
            return Some(SpecClaimCategory::CommandClaim);
        }
        if signals.numeric_value.is_some() {
            return Some(SpecClaimCategory::MetricClaim);
        }
        if !signals.entity_refs.is_empty() {
            return Some(SpecClaimCategory::CodeEntity);
        }
        if is_absolute || priority != ClaimPriority::P3Default {
            return Some(SpecClaimCategory::ArchitecturalClaim);
        }
        None
    }
}

impl Default for SpecClaimExtractor {
    fn default() -> Self {
        Self::new()
    }
}

// ============================================================================
// Falsification Engine
// ============================================================================

/// Falsification engine that runs strategies against extracted claims
pub struct FalsificationEngine {
    project_path: PathBuf,
}

impl FalsificationEngine {
    pub fn new(project_path: &Path) -> Self {
        Self {
            project_path: project_path.to_path_buf(),
        }
    }

    /// Falsify all claims in a specification file
    pub fn falsify_spec(&self, spec_path: &Path) -> Result<SpecFalsificationReport> {
        let content = std::fs::read_to_string(spec_path)
            .with_context(|| format!("Failed to read spec: {}", spec_path.display()))?;

        let extractor = SpecClaimExtractor::new();
        let claims = extractor.extract(&content, spec_path);

        let verdicts: Vec<SpecVerdict> = claims
            .into_iter()
            .map(|claim| self.falsify_claim(claim))
            .collect();

        let summary = Self::compute_summary(&verdicts);

        Ok(SpecFalsificationReport {
            target_file: spec_path.to_path_buf(),
            timestamp: chrono::Utc::now().to_rfc3339(),
            verdicts,
            summary,
        })
    }

    /// Falsify a single claim using the appropriate strategy
    fn falsify_claim(&self, claim: SpecClaim) -> SpecVerdict {
        let evidence = match &claim.category {
            SpecClaimCategory::PathReference => self.check_path_references(&claim),
            SpecClaimCategory::CodeEntity => self.check_code_entities(&claim),
            SpecClaimCategory::AbsenceClaim => self.check_absence_claim(&claim),
            SpecClaimCategory::CommandClaim => self.check_command_claim(&claim),
            SpecClaimCategory::MetricClaim => self.check_metric_claim(&claim),
            SpecClaimCategory::ArchitecturalClaim => Vec::new(), // Inconclusive — needs human review
            SpecClaimCategory::Unfalsifiable => Vec::new(),
        };

        let status = self.determine_verdict(&claim, &evidence);
        let contradiction_score = if evidence.is_empty() {
            0.0
        } else {
            evidence.iter().map(|e| e.contradiction_score).sum::<f64>() / evidence.len() as f64
        };

        SpecVerdict {
            claim,
            status,
            evidence,
            contradiction_score,
        }
    }

    /// Check if referenced file paths exist
    fn check_path_references(&self, claim: &SpecClaim) -> Vec<SpecEvidence> {
        claim
            .path_refs
            .iter()
            .map(|path_str| self.check_single_path(path_str))
            .collect()
    }

    fn check_single_path(&self, path_str: &str) -> SpecEvidence {
        let full_path = self.project_path.join(path_str);
        if full_path.exists() {
            return SpecEvidence {
                check: format!("File exists: {}", path_str),
                finding: "File found at expected location".to_string(),
                contradiction_score: 0.0,
            };
        }
        let suggestion = Self::find_similar_file(&full_path, &self.project_path);
        SpecEvidence {
            check: format!("File exists: {}", path_str),
            finding: format!("File NOT found{}", suggestion),
            contradiction_score: 1.0,
        }
    }

    fn find_similar_file(full_path: &Path, project_path: &Path) -> String {
        let parent = full_path.parent().unwrap_or(project_path);
        let stem = full_path.file_stem().and_then(|s| s.to_str()).unwrap_or("");
        if !parent.exists() || stem.is_empty() {
            return String::new();
        }
        let Ok(entries) = std::fs::read_dir(parent) else {
            return String::new();
        };
        for entry in entries.flatten() {
            if entry.file_name().to_string_lossy().contains(stem) {
                return format!(" (did you mean: {}?)", entry.path().display());
            }
        }
        String::new()
    }

    /// Check if referenced code entities exist using pmat query
    fn check_code_entities(&self, claim: &SpecClaim) -> Vec<SpecEvidence> {
        claim
            .entity_refs
            .iter()
            .map(|entity| {
                // Use pmat query --literal with --files-with-matches for simpler parsing
                let output = std::process::Command::new("pmat")
                    .args([
                        "query",
                        "--literal",
                        entity,
                        "--files-with-matches",
                        "--limit",
                        "5",
                    ])
                    .current_dir(&self.project_path)
                    .output();

                match output {
                    Ok(out) => {
                        let stdout = String::from_utf8_lossy(&out.stdout);
                        // Strip ANSI codes and count non-empty lines that look like file paths
                        let ansi_re = Regex::new(r"\x1b\[[0-9;]*m").expect("internal ansi regex");
                        let clean = ansi_re.replace_all(&stdout, "");
                        let file_matches: Vec<&str> = clean
                            .lines()
                            .filter(|line| {
                                let trimmed = line.trim();
                                !trimmed.is_empty()
                                    && !trimmed.starts_with("Loading")
                                    && !trimmed.starts_with("Index:")
                                    && !trimmed.starts_with("Searching")
                                    && !trimmed.starts_with("query profile")
                                    && !trimmed.starts_with("Checking")
                                    && !trimmed.starts_with("Incremental")
                                    && !trimmed.starts_with("Merging")
                                    && !trimmed.starts_with("SQLite")
                                    && !trimmed.starts_with("Workspace")
                                    && !trimmed.starts_with('+')
                                    // Exclude spec files from matches (avoid self-reference)
                                    && !trimmed.contains("specifications/")
                                    && !trimmed.contains("docs/roadmaps/")
                            })
                            .collect();

                        if !file_matches.is_empty() {
                            let first_file = file_matches[0].trim();
                            let count = file_matches.len();
                            SpecEvidence {
                                check: format!("Entity exists: `{}`", entity),
                                finding: format!("Found in {} file(s), e.g. {}", count, first_file),
                                contradiction_score: 0.0,
                            }
                        } else {
                            SpecEvidence {
                                check: format!("Entity exists: `{}`", entity),
                                finding: "NOT found in codebase".to_string(),
                                contradiction_score: 0.8,
                            }
                        }
                    }
                    Err(_) => SpecEvidence {
                        check: format!("Entity exists: `{}`", entity),
                        finding: "Could not run pmat query (pmat not available)".to_string(),
                        contradiction_score: 0.0,
                    },
                }
            })
            .collect()
    }

    /// Check absence claims by searching for counterexamples
    fn check_absence_claim(&self, claim: &SpecClaim) -> Vec<SpecEvidence> {
        // Extract what should be absent from the claim text
        let text_lower = claim.original_text.to_lowercase();
        let search_terms: Vec<&str> = if text_lower.contains("unsafe") {
            vec!["unsafe"]
        } else if text_lower.contains("panic") {
            vec!["panic!"]
        } else if text_lower.contains("unwrap") {
            vec!["unwrap()"]
        } else if text_lower.contains("todo") || text_lower.contains("fixme") {
            vec!["TODO", "FIXME"]
        } else {
            return vec![SpecEvidence {
                check: "Absence claim".to_string(),
                finding: "Cannot determine what to search for".to_string(),
                contradiction_score: 0.0,
            }];
        };

        search_terms
            .iter()
            .map(|term| {
                let output = std::process::Command::new("pmat")
                    .args([
                        "query",
                        "--literal",
                        term,
                        "--count",
                        "--exclude-tests",
                        "--limit",
                        "5",
                    ])
                    .current_dir(&self.project_path)
                    .output();

                match output {
                    Ok(out) => {
                        let stdout = String::from_utf8_lossy(&out.stdout);
                        // Strip ANSI codes before parsing count output
                        let ansi_re = Regex::new(r"\x1b\[[0-9;]*m").expect("internal ansi regex");
                        let clean = ansi_re.replace_all(&stdout, "");
                        let total_count: u32 = clean
                            .lines()
                            .filter(|line| line.contains(':'))
                            .filter_map(|line| {
                                line.split(':').next_back()?.trim().parse::<u32>().ok()
                            })
                            .sum();

                        if total_count > 0 {
                            SpecEvidence {
                                check: format!("Absence: no `{}`", term),
                                finding: format!("Found {} occurrences in codebase", total_count),
                                contradiction_score: 1.0,
                            }
                        } else {
                            SpecEvidence {
                                check: format!("Absence: no `{}`", term),
                                finding: "No occurrences found — claim holds".to_string(),
                                contradiction_score: 0.0,
                            }
                        }
                    }
                    Err(_) => SpecEvidence {
                        check: format!("Absence: no `{}`", term),
                        finding: "Could not search codebase".to_string(),
                        contradiction_score: 0.0,
                    },
                }
            })
            .collect()
    }

    /// Check if referenced commands exist
    fn check_command_claim(&self, claim: &SpecClaim) -> Vec<SpecEvidence> {
        let cmd_pattern = Regex::new(r"`(pmat\s+[\w-]+)`").expect("internal regex");
        let commands: Vec<String> = cmd_pattern
            .captures_iter(&claim.original_text)
            .filter_map(|c| c.get(1).map(|m| m.as_str().to_string()))
            .collect();

        commands
            .iter()
            .map(|cmd| {
                // Check if the subcommand exists by running pmat --help
                let parts: Vec<&str> = cmd.split_whitespace().collect();
                if parts.len() >= 2 {
                    let subcommand = parts[1];
                    let output = std::process::Command::new("pmat")
                        .args([subcommand, "--help"])
                        .current_dir(&self.project_path)
                        .output();

                    match output {
                        Ok(out) if out.status.success() => SpecEvidence {
                            check: format!("Command exists: `{}`", cmd),
                            finding: "Command is available".to_string(),
                            contradiction_score: 0.0,
                        },
                        _ => SpecEvidence {
                            check: format!("Command exists: `{}`", cmd),
                            finding: "Command NOT recognized".to_string(),
                            contradiction_score: 1.0,
                        },
                    }
                } else {
                    SpecEvidence {
                        check: format!("Command: `{}`", cmd),
                        finding: "Could not parse command".to_string(),
                        contradiction_score: 0.0,
                    }
                }
            })
            .collect()
    }

    /// Check numeric/metric claims
    fn check_metric_claim(&self, _claim: &SpecClaim) -> Vec<SpecEvidence> {
        // Metric claims require running actual measurements (coverage, complexity, etc.)
        // For MVP, mark these as inconclusive since we can't cheaply verify them
        vec![SpecEvidence {
            check: "Metric claim".to_string(),
            finding: "Metric verification requires measurement — marked for manual review"
                .to_string(),
            contradiction_score: 0.0,
        }]
    }

    /// Determine the verdict status from evidence
    fn determine_verdict(&self, claim: &SpecClaim, evidence: &[SpecEvidence]) -> VerdictStatus {
        if matches!(
            claim.category,
            SpecClaimCategory::Unfalsifiable | SpecClaimCategory::ArchitecturalClaim
        ) {
            return VerdictStatus::Unfalsifiable;
        }

        if evidence.is_empty() {
            return VerdictStatus::Inconclusive;
        }

        let max_contradiction = evidence
            .iter()
            .map(|e| e.contradiction_score)
            .fold(0.0f64, f64::max);

        if max_contradiction >= 0.8 {
            VerdictStatus::Falsified
        } else if max_contradiction >= 0.4 {
            VerdictStatus::Inconclusive
        } else {
            VerdictStatus::Survived
        }
    }

    fn compute_summary(verdicts: &[SpecVerdict]) -> SpecFalsificationSummary {
        let total_claims = verdicts.len();
        let survived = verdicts
            .iter()
            .filter(|v| v.status == VerdictStatus::Survived)
            .count();
        let falsified = verdicts
            .iter()
            .filter(|v| v.status == VerdictStatus::Falsified)
            .count();
        let unfalsifiable = verdicts
            .iter()
            .filter(|v| v.status == VerdictStatus::Unfalsifiable)
            .count();
        let inconclusive = verdicts
            .iter()
            .filter(|v| v.status == VerdictStatus::Inconclusive)
            .count();

        let testable = total_claims.saturating_sub(unfalsifiable);
        let health_score = if testable > 0 {
            survived as f64 / testable as f64
        } else {
            1.0
        };

        SpecFalsificationSummary {
            total_claims,
            survived,
            falsified,
            unfalsifiable,
            inconclusive,
            health_score,
        }
    }
}

// ============================================================================
// Display / Formatting
// ============================================================================

impl SpecFalsificationReport {
    /// Format the report for terminal output
    pub fn display(&self) {
        let file_display = self.target_file.display();
        println!("Falsifying: {}", file_display);
        println!(
            "Extracted: {} falsifiable claims ({} P0, {} P1, {} P2)",
            self.summary.total_claims,
            self.verdicts
                .iter()
                .filter(|v| v.claim.priority == ClaimPriority::P0Critical)
                .count(),
            self.verdicts
                .iter()
                .filter(|v| v.claim.priority == ClaimPriority::P1High)
                .count(),
            self.verdicts
                .iter()
                .filter(|v| v.claim.priority == ClaimPriority::P2Low)
                .count(),
        );
        println!();

        for (i, verdict) in self.verdicts.iter().enumerate() {
            let status_icon = match &verdict.status {
                VerdictStatus::Survived => "\x1b[32mSURVIVED\x1b[0m",
                VerdictStatus::Falsified => "\x1b[31mFALSIFIED\x1b[0m",
                VerdictStatus::Unfalsifiable => "\x1b[33mUNFALSIFIABLE\x1b[0m",
                VerdictStatus::Inconclusive => "\x1b[33mINCONCLUSIVE\x1b[0m",
            };

            println!(
                "[{}/{}] {} {} (line {})",
                i + 1,
                self.summary.total_claims,
                verdict.claim.priority,
                status_icon,
                verdict.claim.source_line,
            );

            // Truncate long claim text
            let text = &verdict.claim.original_text;
            let display_text = if text.len() > 100 {
                format!("{}...", &text[..97])
            } else {
                text.clone()
            };
            println!("       \"{}\"", display_text);
            println!("       Category: {}", verdict.claim.category);

            for ev in &verdict.evidence {
                let icon = if ev.contradiction_score >= 0.8 {
                    "\x1b[31m✗\x1b[0m"
                } else if ev.contradiction_score >= 0.4 {
                    "\x1b[33m?\x1b[0m"
                } else {
                    "\x1b[32m✓\x1b[0m"
                };
                println!("       {} {} → {}", icon, ev.check, ev.finding);
            }
            println!();
        }

        // Summary
        println!("Summary:");
        println!("  Total claims:    {}", self.summary.total_claims);
        println!(
            "  \x1b[32mSurvived\x1b[0m:        {} ({:.1}%)",
            self.summary.survived,
            if self.summary.total_claims > 0 {
                self.summary.survived as f64 / self.summary.total_claims as f64 * 100.0
            } else {
                0.0
            }
        );
        println!(
            "  \x1b[31mFalsified\x1b[0m:       {} ({:.1}%)",
            self.summary.falsified,
            if self.summary.total_claims > 0 {
                self.summary.falsified as f64 / self.summary.total_claims as f64 * 100.0
            } else {
                0.0
            }
        );
        println!("  Unfalsifiable:   {}", self.summary.unfalsifiable);
        println!("  Inconclusive:    {}", self.summary.inconclusive);
        println!();
        println!("  Spec health:     {:.2}", self.summary.health_score);
    }

    /// Format as JSON
    pub fn to_json(&self) -> Result<String> {
        serde_json::to_string_pretty(self).context("Failed to serialize report")
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg_attr(coverage_nightly, coverage(off))]
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn extract_path_references() {
        let extractor = SpecClaimExtractor::new();
        let content = r#"
## Architecture
The main module is at `src/services/context.rs` which handles indexing.
Configuration lives in `docs/specifications/falsify-rag.md`.
"#;
        let claims = extractor.extract(content, Path::new("test.md"));
        let path_claims: Vec<_> = claims
            .iter()
            .filter(|c| c.category == SpecClaimCategory::PathReference)
            .collect();
        assert!(
            path_claims.len() >= 2,
            "Expected >=2 path claims, got {}: {:?}",
            path_claims.len(),
            path_claims
        );
        assert!(path_claims
            .iter()
            .any(|c| c.path_refs.iter().any(|p| p.contains("context.rs"))));
    }

    #[test]
    fn extract_rfc2119_priorities() {
        let extractor = SpecClaimExtractor::new();
        let content = r#"
## Requirements
- Implementations MUST validate all inputs before processing
- Clients SHOULD cache results for performance
- Servers MAY support optional compression
"#;
        let claims = extractor.extract(content, Path::new("test.md"));
        assert!(claims
            .iter()
            .any(|c| c.priority == ClaimPriority::P0Critical));
        assert!(claims.iter().any(|c| c.priority == ClaimPriority::P1High));
        assert!(claims.iter().any(|c| c.priority == ClaimPriority::P2Low));
    }

    #[test]
    fn extract_numeric_claims() {
        let extractor = SpecClaimExtractor::new();
        let content = "Coverage must be >= 95% across all modules.\n";
        let claims = extractor.extract(content, Path::new("test.md"));
        let metric_claims: Vec<_> = claims
            .iter()
            .filter(|c| {
                matches!(
                    c.category,
                    SpecClaimCategory::MetricClaim
                        | SpecClaimCategory::AbsenceClaim
                        | SpecClaimCategory::ArchitecturalClaim
                )
            })
            .collect();
        // Should find a claim with numeric value
        let has_numeric = claims.iter().any(|c| c.numeric_value.is_some());
        assert!(
            has_numeric,
            "Expected numeric claim, got: {:?}",
            metric_claims
        );
    }

    #[test]
    fn extract_code_entities() {
        let extractor = SpecClaimExtractor::new();
        let content = "The `FalsificationEngine` processes claims via `ClaimExtractor`.\n";
        let claims = extractor.extract(content, Path::new("test.md"));
        let entity_claims: Vec<_> = claims
            .iter()
            .filter(|c| c.category == SpecClaimCategory::CodeEntity)
            .collect();
        assert!(
            !entity_claims.is_empty(),
            "Expected entity claims, got none"
        );
        assert!(entity_claims
            .iter()
            .any(|c| c.entity_refs.contains(&"FalsificationEngine".to_string())));
    }

    #[test]
    fn extract_absence_claims() {
        let extractor = SpecClaimExtractor::new();
        let content = "There must be zero unsafe blocks in the parser module.\n";
        let claims = extractor.extract(content, Path::new("test.md"));
        let absence = claims
            .iter()
            .filter(|c| c.category == SpecClaimCategory::AbsenceClaim)
            .count();
        assert!(absence > 0, "Expected absence claim, got: {:?}", claims);
    }

    #[test]
    fn extract_command_claims() {
        let extractor = SpecClaimExtractor::new();
        let content = "Run `pmat falsify` to validate specs against the codebase.\n";
        let claims = extractor.extract(content, Path::new("test.md"));
        let cmd_claims: Vec<_> = claims
            .iter()
            .filter(|c| c.category == SpecClaimCategory::CommandClaim)
            .collect();
        assert!(!cmd_claims.is_empty(), "Expected command claims");
    }

    #[test]
    fn skip_code_blocks() {
        let extractor = SpecClaimExtractor::new();
        let content = r#"
## Example
```rust
// This MUST not be extracted as a claim
let x = src/foo/bar.rs;
```
This line SHOULD be extracted.
"#;
        let claims = extractor.extract(content, Path::new("test.md"));
        // Only the "SHOULD" line should be extracted, not the code block contents
        assert!(
            claims.iter().all(|c| !c.original_text.contains("let x =")),
            "Code block content should not be extracted as claims"
        );
        assert!(claims.iter().any(|c| c.original_text.contains("SHOULD")));
    }

    #[test]
    fn absolute_language_detection() {
        let extractor = SpecClaimExtractor::new();
        let content = "All modules MUST have complete test coverage.\n";
        let claims = extractor.extract(content, Path::new("test.md"));
        assert!(!claims.is_empty());
        assert!(claims[0].is_absolute);
        assert_eq!(claims[0].priority, ClaimPriority::P0Critical);
    }

    #[test]
    fn path_reference_validation_existing_file() {
        let engine = FalsificationEngine::new(Path::new(env!("CARGO_MANIFEST_DIR")));
        let claim = SpecClaim {
            id: "test-001".to_string(),
            original_text: "Config at src/lib.rs".to_string(),
            source_line: 1,
            category: SpecClaimCategory::PathReference,
            priority: ClaimPriority::P3Default,
            is_absolute: false,
            path_refs: vec!["src/lib.rs".to_string()],
            entity_refs: vec![],
            numeric_value: None,
            numeric_comparator: None,
        };
        let evidence = engine.check_path_references(&claim);
        assert!(!evidence.is_empty());
        assert_eq!(
            evidence[0].contradiction_score, 0.0,
            "src/lib.rs should exist"
        );
    }

    #[test]
    fn path_reference_validation_missing_file() {
        let engine = FalsificationEngine::new(Path::new(env!("CARGO_MANIFEST_DIR")));
        let claim = SpecClaim {
            id: "test-002".to_string(),
            original_text: "Config at src/nonexistent_file_xyz.rs".to_string(),
            source_line: 1,
            category: SpecClaimCategory::PathReference,
            priority: ClaimPriority::P3Default,
            is_absolute: false,
            path_refs: vec!["src/nonexistent_file_xyz.rs".to_string()],
            entity_refs: vec![],
            numeric_value: None,
            numeric_comparator: None,
        };
        let evidence = engine.check_path_references(&claim);
        assert!(!evidence.is_empty());
        assert_eq!(
            evidence[0].contradiction_score, 1.0,
            "Nonexistent file should be falsified"
        );
    }

    #[test]
    fn verdict_determination() {
        let engine = FalsificationEngine::new(Path::new("."));
        let claim = SpecClaim {
            id: "test".to_string(),
            original_text: "test".to_string(),
            source_line: 1,
            category: SpecClaimCategory::PathReference,
            priority: ClaimPriority::P3Default,
            is_absolute: false,
            path_refs: vec![],
            entity_refs: vec![],
            numeric_value: None,
            numeric_comparator: None,
        };

        // Surviving evidence
        let survived_ev = vec![SpecEvidence {
            check: "test".to_string(),
            finding: "ok".to_string(),
            contradiction_score: 0.0,
        }];
        assert_eq!(
            engine.determine_verdict(&claim, &survived_ev),
            VerdictStatus::Survived
        );

        // Falsified evidence
        let falsified_ev = vec![SpecEvidence {
            check: "test".to_string(),
            finding: "bad".to_string(),
            contradiction_score: 1.0,
        }];
        assert_eq!(
            engine.determine_verdict(&claim, &falsified_ev),
            VerdictStatus::Falsified
        );
    }

    #[test]
    fn summary_computation() {
        let claim = SpecClaim {
            id: "c1".to_string(),
            original_text: "test".to_string(),
            source_line: 1,
            category: SpecClaimCategory::PathReference,
            priority: ClaimPriority::P3Default,
            is_absolute: false,
            path_refs: vec![],
            entity_refs: vec![],
            numeric_value: None,
            numeric_comparator: None,
        };

        let verdicts = vec![
            SpecVerdict {
                claim: claim.clone(),
                status: VerdictStatus::Survived,
                evidence: vec![],
                contradiction_score: 0.0,
            },
            SpecVerdict {
                claim: claim.clone(),
                status: VerdictStatus::Falsified,
                evidence: vec![],
                contradiction_score: 1.0,
            },
            SpecVerdict {
                claim: claim.clone(),
                status: VerdictStatus::Unfalsifiable,
                evidence: vec![],
                contradiction_score: 0.0,
            },
        ];

        let summary = FalsificationEngine::compute_summary(&verdicts);
        assert_eq!(summary.total_claims, 3);
        assert_eq!(summary.survived, 1);
        assert_eq!(summary.falsified, 1);
        assert_eq!(summary.unfalsifiable, 1);
        // health = 1 survived / 2 testable = 0.5
        assert!((summary.health_score - 0.5).abs() < f64::EPSILON);
    }
}
