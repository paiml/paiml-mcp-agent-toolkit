#![cfg_attr(coverage_nightly, coverage(off))]
// Utility helper functions for work command handlers
// (parsing, spec template, override name mapping, validation)

use crate::cli::handlers::work_falsification::ClaimResult;
use crate::models::roadmap::RoadmapItem;
use anyhow::Result;
use std::path::PathBuf;

use super::types::CLAIM_PATTERNS;

/// Parse acceptance criteria from GitHub issue body
///
/// Looks for markdown checklists in the body and extracts them as criteria.
pub(super) fn parse_acceptance_criteria(body: &str) -> Vec<String> {
    let mut criteria = Vec::new();

    for line in body.lines() {
        let trimmed = line.trim();
        // Match markdown checkboxes: - [ ] or - [x]
        if trimmed.starts_with("- [ ]") || trimmed.starts_with("- [x]") {
            let criterion = trimmed
                .trim_start_matches("- [ ]")
                .trim_start_matches("- [x]")
                .trim()
                .to_string();
            if !criterion.is_empty() {
                criteria.push(criterion);
            }
        }
    }

    criteria
}

/// Create specification template
pub(super) fn create_specification_template(spec_path: &PathBuf, item: &RoadmapItem) -> Result<()> {
    use std::fs;

    if let Some(parent) = spec_path.parent() {
        fs::create_dir_all(parent)?;
    }

    let github_link = if let Some(issue) = item.github_issue {
        format!(
            "**GitHub Issue**: [#{}](https://github.com/YOUR_ORG/YOUR_REPO/issues/{})",
            issue, issue
        )
    } else {
        format!("**Ticket ID**: {}", item.id)
    };

    let template = format!(
        r#"---
title: {}
issue: {}
status: In Progress
created: {}
updated: {}
---

# {} Specification

{}
**Status**: In Progress

## Summary

[Brief 2-3 sentence overview of what this work accomplishes]

## Requirements

### Functional Requirements
- [ ] Requirement 1
- [ ] Requirement 2

### Non-Functional Requirements
- [ ] Performance: [specific target]
- [ ] Test coverage: ≥85%

## Architecture

### Design Overview

[Describe the high-level design approach]

### API Design

```rust
// Example API design
pub struct Example {{
    // ...
}}
```

## Implementation Plan

### Phase 1: Foundation
- [ ] Task 1
- [ ] Task 2

### Phase 2: Core Implementation
- [ ] Task 3
- [ ] Task 4

## Testing Strategy

### Unit Tests
- [ ] Test case 1
- [ ] Test case 2

### Integration Tests
- [ ] Integration test 1

## Success Criteria

- ✅ All acceptance criteria met
- ✅ Test coverage ≥85%
- ✅ Zero clippy warnings
- ✅ Documentation complete

## References

- [Related documentation]
"#,
        item.title, item.id, item.created, item.updated, item.title, github_link
    );

    fs::write(spec_path, template)?;
    Ok(())
}

/// Convert hypothesis text to CLI-friendly override name
///
/// Maps the verbose hypothesis strings from FalsifiableClaim to short,
/// CLI-friendly names that users can specify with --override-claims.
pub(super) fn claim_to_override_name(hypothesis: &str) -> String {
    let hypothesis_lower = hypothesis.to_lowercase();

    // Pattern-based lookup (reduces cyclomatic complexity vs if-else chain)
    for (patterns, name) in CLAIM_PATTERNS {
        if patterns.iter().any(|p| hypothesis_lower.contains(p)) {
            return name.to_string();
        }
    }

    // Unknown claim - use a sanitized version of the hypothesis
    hypothesis_lower
        .chars()
        .filter(|c| c.is_alphanumeric() || *c == '-' || *c == '_')
        .take(30)
        .collect()
}

/// Known aliases for override claim IDs (#249).
/// Maps common user-friendly names to canonical claim names.
const CLAIM_ALIASES: &[(&str, &str)] = &[
    ("baseline-files", "manifest"),
    ("files", "manifest"),
    ("file-manifest", "manifest"),
    ("tests", "coverage"),
    ("test-coverage", "coverage"),
    ("deps", "supply-chain"),
    ("dependencies", "supply-chain"),
    ("vulns", "supply-chain"),
    ("cyclomatic", "complexity"),
    ("loc", "file-size"),
    ("line-count", "file-size"),
    ("todos", "satd"),
    ("fixmes", "satd"),
    ("linting", "lint"),
];

/// Resolve a user-provided override claim to its canonical name (#249).
///
/// First checks known aliases, then checks if it already matches a known claim name.
/// Returns `None` if the override doesn't match any known claim, triggering a warning.
fn resolve_override_claim(user_claim: &str) -> Option<String> {
    let lower = user_claim.to_lowercase();

    // Check aliases first
    for (alias, canonical) in CLAIM_ALIASES {
        if lower == *alias {
            return Some(canonical.to_string());
        }
    }

    // Check if it matches an existing canonical name from CLAIM_PATTERNS
    let known_names: Vec<&str> = CLAIM_PATTERNS.iter().map(|(_, name)| *name).collect();
    if known_names.iter().any(|n| n.to_lowercase() == lower) {
        return Some(lower);
    }

    None
}

/// Filter failures not covered by overrides
pub(super) fn filter_unoverriden_failures<'a>(
    failures: &[&'a ClaimResult],
    override_claims: Option<&Vec<String>>,
) -> Vec<&'a ClaimResult> {
    // Warn about unrecognized override claims (#249)
    if let Some(overrides) = override_claims {
        let known_names: Vec<&str> = CLAIM_PATTERNS.iter().map(|(_, name)| *name).collect();
        for claim in overrides {
            if resolve_override_claim(claim).is_none() {
                eprintln!(
                    "warning: --override-claims '{}' does not match any known claim ID.",
                    claim
                );
                eprintln!("  Known claim IDs: {}", known_names.join(", "));
                eprintln!(
                    "  Hint: use one of the known IDs above, or check `pmat work falsify --help`."
                );
            }
        }
    }

    failures
        .iter()
        .filter(|failure| {
            let claim_name = claim_to_override_name(&failure.hypothesis);
            if let Some(overrides) = override_claims {
                !overrides.iter().any(|o| {
                    let resolved = resolve_override_claim(o).unwrap_or_else(|| o.to_lowercase());
                    resolved == claim_name.to_lowercase()
                })
            } else {
                true
            }
        })
        .copied()
        .collect()
}

/// Validate that override claims have an associated ticket (Popperian accountability)
pub(super) fn validate_override_accountability(
    override_claims: &Option<Vec<String>>,
    ticket: &Option<String>,
    id: &str,
) -> Result<()> {
    if override_claims.is_some() && ticket.is_none() {
        anyhow::bail!(
            "Error: --ticket is mandatory for overrides.\n\n\
             Popperian Principle: Every override must be accountable.\n\
             Create a debt ticket first:\n\
             1. pmat comply upgrade --target popperian\n\
             2. Or manually create .pmat-tickets/DEBT-XXX.yaml\n\n\
             Then retry with: pmat work complete {} --override-claims <claims> --ticket <TICKET-ID>",
            id
        );
    }
    Ok(())
}
