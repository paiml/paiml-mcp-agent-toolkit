// =============================================================================
// COMPLY-005: SATD MANIFESTATION TYPE
// =============================================================================
//
// Per [SATD-002] Maldonado & Shihab (2015): Code SATD (todo!(), unimplemented!())
// correlates 2.3x more strongly with defects than comment SATD (// TODO).
// This distinction is fundamental to proper severity modeling.

/// SATD manifestation type affects severity scoring
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SATDManifestationType {
    /// Comment-based: // TODO, // FIXME, /* HACK */ - advisory only
    Comment,
    /// Code-based: todo!(), unimplemented!(), raise NotImplementedError - crashes at runtime
    Code,
}

/// Severity levels for SATD
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Severity {
    Low,
    Medium,
    High,
    Critical,
}

impl SATDManifestationType {
    /// Escalate severity for Code manifestations
    /// Per [SATD-003]: Design debt (stubs) costs 2.3x more to fix
    pub fn escalate_severity(&self, base: Severity) -> Severity {
        match self {
            SATDManifestationType::Comment => base, // No escalation
            SATDManifestationType::Code => match base {
                Severity::Low => Severity::Medium,        // Low -> Medium
                Severity::Medium => Severity::High,       // Medium -> High
                Severity::High => Severity::Critical,     // High -> Critical
                Severity::Critical => Severity::Critical, // Already max
            },
        }
    }
}

/// Classify SATD content into Code or Comment manifestation type
pub fn classify_satd_manifestation(content: &str) -> SATDManifestationType {
    debug_assert!(!content.is_empty(), "content must not be empty");
    // Code patterns: deterministic runtime failures
    let code_patterns = [
        "todo!",
        "unimplemented!",
        "panic!(\"not implemented",
        "panic!(\"Not implemented",
        "raise NotImplementedError",
        "fn ", // Empty function bodies (detected separately, but classified here)
    ];

    // Check if content matches any code pattern
    for pattern in &code_patterns {
        if content.contains(pattern) {
            return SATDManifestationType::Code;
        }
    }

    // Check for empty function body pattern
    if content.trim().ends_with("{}") || content.contains("{ }") {
        return SATDManifestationType::Code;
    }

    // Default: Comment manifestation (e.g. `TODO`, `FIXME`, `HACK` markers)
    SATDManifestationType::Comment
}

/// Classify based on pattern ID from CB-050 detection
pub fn classify_satd_by_pattern_id(pattern_id: &str) -> SATDManifestationType {
    match pattern_id {
        // Code patterns - will crash at runtime
        "CB-050-A" => SATDManifestationType::Code, // todo!()
        "CB-050-B" => SATDManifestationType::Code, // unimplemented!()
        "CB-050-C" => SATDManifestationType::Code, // panic!("not implemented")
        "CB-050-D" => SATDManifestationType::Code, // Empty function body
        "CB-050-E" => SATDManifestationType::Code, // Python NotImplementedError
        "CB-050-F" => SATDManifestationType::Comment, // Python pass # stub (advisory)
        // Default to Comment for unknown patterns
        _ => SATDManifestationType::Comment,
    }
}

