//! Kaizen prioritization: ranking, composite priority, tarantula suspiciousness enrichment.

use super::{FindingSeverity, KaizenFinding};
use std::path::Path;

/// Compute composite priority: severity_weight * (1.0 + suspiciousness).
/// Critical=4, High=3, Medium=2, Low=1.
#[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
pub(crate) fn composite_priority(finding: &KaizenFinding) -> f32 {
    let severity_weight = match finding.severity {
        FindingSeverity::Critical => 4.0f32,
        FindingSeverity::High => 3.0,
        FindingSeverity::Medium => 2.0,
        FindingSeverity::Low => 1.0,
    };
    let suspiciousness = finding.suspiciousness_score.unwrap_or(0.0);
    severity_weight * (1.0 + suspiciousness)
}

#[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
pub(crate) fn sort_findings(findings: &mut [KaizenFinding]) {
    findings.sort_by(|a, b| {
        composite_priority(b)
            .partial_cmp(&composite_priority(a))
            .unwrap_or(std::cmp::Ordering::Equal)
    });
}

/// Enrich findings with tarantula suspiciousness scores from LCOV coverage data.
/// Gracefully does nothing if no coverage data is available.
#[provable_contracts_macros::contract("pmat-core.yaml", equation = "path_exists")]
pub(crate) fn enrich_with_tarantula(path: &Path, findings: &mut [KaizenFinding]) {
    let lcov_candidates = [
        path.join("target/coverage/lcov.info"),
        path.join("target/llvm-cov/lcov.info"),
        path.join("coverage/lcov.info"),
        path.join("lcov.info"),
    ];

    let lcov_path = match lcov_candidates.iter().find(|p| p.exists()) {
        Some(p) => p,
        None => return, // No coverage data, skip enrichment
    };

    let content = match std::fs::read_to_string(lcov_path) {
        Ok(c) => c,
        Err(_) => return,
    };

    // Parse LCOV into file -> line -> hit_count map
    let line_hits = parse_lcov_line_hits(&content);
    if line_hits.is_empty() {
        return;
    }

    // Build per-file suspiciousness: ratio of uncovered lines
    // Higher ratio = more suspicious (more untested code)
    for finding in findings.iter_mut() {
        if let Some(ref file) = finding.file {
            // Normalize path: strip leading "./" or project prefix
            let normalized = file.trim_start_matches("./");
            if let Some(hits) = line_hits.get(normalized) {
                let total = hits.len() as f32;
                if total > 0.0 {
                    let uncovered = hits.values().filter(|&&h| h == 0).count() as f32;
                    finding.suspiciousness_score = Some(uncovered / total);
                }
            }
        }
    }
}

/// Parse LCOV format into file -> (line -> hit_count) map.
fn parse_lcov_line_hits(
    content: &str,
) -> std::collections::HashMap<String, std::collections::HashMap<usize, u64>> {
    let mut result: std::collections::HashMap<String, std::collections::HashMap<usize, u64>> =
        std::collections::HashMap::new();
    let mut current_file = String::new();

    for line in content.lines() {
        if let Some(sf) = line.strip_prefix("SF:") {
            current_file = sf.trim().to_string();
        } else if let Some(da) = line.strip_prefix("DA:") {
            if current_file.is_empty() {
                continue;
            }
            let parts: Vec<&str> = da.split(',').collect();
            if parts.len() >= 2 {
                if let (Ok(line_no), Ok(hits)) =
                    (parts[0].parse::<usize>(), parts[1].parse::<u64>())
                {
                    result
                        .entry(current_file.clone())
                        .or_default()
                        .insert(line_no, hits);
                }
            }
        } else if line == "end_of_record" {
            current_file.clear();
        }
    }

    result
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cli::handlers::kaizen_handler::FindingSource;

    #[test]
    fn test_finding_severity_ordering() {
        assert!(FindingSeverity::Critical > FindingSeverity::High);
        assert!(FindingSeverity::High > FindingSeverity::Medium);
        assert!(FindingSeverity::Medium > FindingSeverity::Low);
    }

    #[test]
    fn test_composite_priority_severity_only() {
        let finding = KaizenFinding {
            source: FindingSource::Clippy,
            severity: FindingSeverity::Critical,
            category: "test".to_string(),
            message: "test".to_string(),
            file: None,
            auto_fixable: false,
            agent_fixable: false,
            fix_applied: false,
            agent_prompt: None,
            suspiciousness_score: None,
            crate_name: None,
        };
        // Critical=4.0 * (1.0 + 0.0) = 4.0
        assert!((composite_priority(&finding) - 4.0).abs() < 0.001);
    }

    #[test]
    fn test_composite_priority_with_suspiciousness() {
        let finding = KaizenFinding {
            source: FindingSource::Clippy,
            severity: FindingSeverity::Medium,
            category: "test".to_string(),
            message: "test".to_string(),
            file: None,
            auto_fixable: false,
            agent_fixable: false,
            fix_applied: false,
            agent_prompt: None,
            suspiciousness_score: Some(0.8),
            crate_name: None,
        };
        // Medium=2.0 * (1.0 + 0.8) = 3.6
        assert!((composite_priority(&finding) - 3.6).abs() < 0.001);
    }

    #[test]
    fn test_composite_priority_ordering() {
        // High severity, no suspiciousness: 3.0 * 1.0 = 3.0
        let high_no_sus = KaizenFinding {
            source: FindingSource::Clippy,
            severity: FindingSeverity::High,
            category: "a".to_string(),
            message: "a".to_string(),
            file: None,
            auto_fixable: false,
            agent_fixable: false,
            fix_applied: false,
            agent_prompt: None,
            suspiciousness_score: None,
            crate_name: None,
        };
        // Medium severity, high suspiciousness: 2.0 * (1.0 + 0.9) = 3.8
        let med_high_sus = KaizenFinding {
            source: FindingSource::Clippy,
            severity: FindingSeverity::Medium,
            category: "b".to_string(),
            message: "b".to_string(),
            file: None,
            auto_fixable: false,
            agent_fixable: false,
            fix_applied: false,
            agent_prompt: None,
            suspiciousness_score: Some(0.9),
            crate_name: None,
        };
        // Medium with high suspiciousness should rank higher
        assert!(composite_priority(&med_high_sus) > composite_priority(&high_no_sus));
    }

    #[test]
    fn test_parse_lcov_line_hits() {
        let lcov = "\
SF:src/main.rs\n\
DA:1,5\n\
DA:2,0\n\
DA:3,10\n\
end_of_record\n\
SF:src/lib.rs\n\
DA:10,0\n\
DA:20,3\n\
end_of_record\n";

        let hits = parse_lcov_line_hits(lcov);
        assert_eq!(hits.len(), 2);

        let main_hits = hits.get("src/main.rs").unwrap();
        assert_eq!(main_hits.get(&1), Some(&5));
        assert_eq!(main_hits.get(&2), Some(&0));
        assert_eq!(main_hits.get(&3), Some(&10));

        let lib_hits = hits.get("src/lib.rs").unwrap();
        assert_eq!(lib_hits.get(&10), Some(&0));
        assert_eq!(lib_hits.get(&20), Some(&3));
    }

    #[test]
    fn test_parse_lcov_empty() {
        let hits = parse_lcov_line_hits("");
        assert!(hits.is_empty());
    }

    #[test]
    fn test_sort_findings_orders_by_priority() {
        let mut findings = vec![
            KaizenFinding {
                source: FindingSource::Clippy,
                severity: FindingSeverity::Low,
                category: "low".to_string(),
                message: "low".to_string(),
                file: None,
                auto_fixable: false,
                agent_fixable: false,
                fix_applied: false,
                agent_prompt: None,
                suspiciousness_score: None,
                crate_name: None,
            },
            KaizenFinding {
                source: FindingSource::Clippy,
                severity: FindingSeverity::Critical,
                category: "crit".to_string(),
                message: "crit".to_string(),
                file: None,
                auto_fixable: false,
                agent_fixable: false,
                fix_applied: false,
                agent_prompt: None,
                suspiciousness_score: None,
                crate_name: None,
            },
        ];
        sort_findings(&mut findings);
        assert_eq!(findings[0].category, "crit");
        assert_eq!(findings[1].category, "low");
    }
}
