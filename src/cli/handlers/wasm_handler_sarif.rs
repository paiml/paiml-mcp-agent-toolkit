/// Format SARIF output for security results (Complexity: 4)
fn format_sarif(vulnerabilities: &[VulnerabilityMatch]) -> Result<String> {
    let sarif_output = create_sarif_output(vulnerabilities);
    Ok(serde_json::to_string_pretty(&sarif_output)?)
}

/// Create SARIF output structure (Complexity: 6)
fn create_sarif_output(vulnerabilities: &[VulnerabilityMatch]) -> serde_json::Value {
    let rules = create_sarif_rules(vulnerabilities);
    let results = create_sarif_results(vulnerabilities);

    serde_json::json!({
        "$schema": "https://raw.githubusercontent.com/oasis-tcs/sarif-spec/master/Schemata/sarif-schema-2.1.0.json",
        "version": "2.1.0",
        "runs": [{
            "tool": {
                "driver": {
                    "name": "pmat-wasm-analyzer",
                    "version": env!("CARGO_PKG_VERSION"),
                    "informationUri": "https://github.com/paiml/pmat",
                    "rules": rules
                }
            },
            "results": results
        }]
    })
}

/// Create SARIF rules from vulnerabilities (Complexity: 4)
fn create_sarif_rules(vulnerabilities: &[VulnerabilityMatch]) -> Vec<serde_json::Value> {
    let unique_patterns: std::collections::HashSet<_> =
        vulnerabilities.iter().map(|v| &v.pattern).collect();

    unique_patterns
        .into_iter()
        .map(|pattern| create_sarif_rule(pattern))
        .collect()
}

/// Create single SARIF rule (Complexity: 1)
fn create_sarif_rule(pattern: &str) -> serde_json::Value {
    serde_json::json!({
        "id": pattern,
        "name": pattern,
        "shortDescription": {
            "text": format!("WASM vulnerability: {}", pattern)
        },
        "fullDescription": {
            "text": format!("Security vulnerability pattern detected in WebAssembly module: {}", pattern)
        },
        "defaultConfiguration": {
            "level": "warning"
        }
    })
}

/// Create SARIF results from vulnerabilities (Complexity: 3)
fn create_sarif_results(vulnerabilities: &[VulnerabilityMatch]) -> Vec<serde_json::Value> {
    vulnerabilities.iter().map(create_sarif_result).collect()
}

/// Create single SARIF result (Complexity: 3)
fn create_sarif_result(vuln: &VulnerabilityMatch) -> serde_json::Value {
    use crate::wasm::security::Severity;

    let level = match vuln.severity {
        Severity::Critical | Severity::High => "error",
        Severity::Medium => "warning",
        Severity::Low => "note",
    };

    serde_json::json!({
        "ruleId": vuln.pattern,
        "level": level,
        "message": {
            "text": format!("Found {} vulnerability at instruction {}", vuln.pattern, vuln.operator_index)
        },
        "locations": [{
            "physicalLocation": {
                "artifactLocation": {
                    "uri": "module.wasm"
                },
                "region": {
                    "startLine": vuln.operator_index,
                    "startColumn": 1
                }
            }
        }]
    })
}
