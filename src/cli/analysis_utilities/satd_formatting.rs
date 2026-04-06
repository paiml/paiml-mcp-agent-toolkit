// SATD formatting - extracted for file health (CB-040)
/// Format SATD items as JSON
fn format_satd_json(
    items: &[crate::services::satd_detector::TechnicalDebt],
    metrics: bool,
    evolution: bool,
) -> String {
    let mut json_obj = serde_json::Map::new();
    json_obj.insert(
        "total_items".to_string(),
        serde_json::Value::Number(items.len().into()),
    );
    json_obj.insert(
        "items".to_string(),
        serde_json::to_value(items).unwrap_or_default(),
    );

    if metrics {
        let severity_counts: std::collections::HashMap<String, usize> =
            items
                .iter()
                .fold(std::collections::HashMap::new(), |mut acc, item| {
                    let sev_str = format!("{:?}", item.severity);
                    *acc.entry(sev_str).or_insert(0) += 1;
                    acc
                });
        json_obj.insert(
            "metrics".to_string(),
            serde_json::to_value(severity_counts).unwrap_or_default(),
        );
    }

    if evolution {
        json_obj.insert(
            "evolution".to_string(),
            serde_json::Value::String("Evolution data would be included".to_string()),
        );
    }

    serde_json::to_string_pretty(&json_obj).unwrap_or_default()
}

/// Format SATD items as SARIF
fn format_satd_sarif(items: &[crate::services::satd_detector::TechnicalDebt]) -> String {
    let mut sarif = serde_json::json!({
        "version": "2.1.0",
        "runs": [{
            "tool": {
                "driver": {
                    "name": "pmat-satd",
                    "version": "0.29.0"
                }
            },
            "results": []
        }]
    });

    let results = items
        .iter()
        .map(|item| {
            serde_json::json!({
                "ruleId": format!("{:?}", item.category),
                "level": match item.severity {
                    crate::services::satd_detector::Severity::Critical => "error",
                    crate::services::satd_detector::Severity::High => "error",
                    crate::services::satd_detector::Severity::Medium => "warning",
                    crate::services::satd_detector::Severity::Low => "note"
                },
                "message": {
                    "text": item.text
                },
                "locations": [{
                    "physicalLocation": {
                        "artifactLocation": {
                            "uri": item.file.to_string_lossy()
                        },
                        "region": {
                            "startLine": item.line
                        }
                    }
                }]
            })
        })
        .collect::<Vec<_>>();

    sarif["runs"][0]["results"] = serde_json::Value::Array(results);
    serde_json::to_string_pretty(&sarif).unwrap_or_default()
}

/// Format SATD items as Markdown
fn format_satd_markdown(
    items: &[crate::services::satd_detector::TechnicalDebt],
    evolution: bool,
    days: u32,
) -> String {
    let mut output = String::from("# SATD Analysis Report\n\n");

    if items.is_empty() {
        output.push_str("✅ **No SATD items found.** Excellent technical debt management!\n");
        return output;
    }

    output.push_str(&format!("📊 **Total SATD items:** {}\n\n", items.len()));

    output.push_str("## Items by Severity\n\n");
    let mut severity_groups = std::collections::HashMap::new();
    for item in items {
        severity_groups
            .entry(format!("{:?}", item.severity))
            .or_insert_with(Vec::new)
            .push(item);
    }

    for (severity, group_items) in severity_groups {
        output.push_str(&format!(
            "### {} ({} items)\n\n",
            severity,
            group_items.len()
        ));
        for item in group_items {
            let category_str = format!("{:?}", item.category);
            output.push_str(&format!(
                "- **{}** (line {}): {} - _{}_\n",
                item.file.file_name().unwrap_or_default().to_string_lossy(),
                item.line,
                category_str,
                item.text
            ));
        }
        output.push('\n');
    }

    if evolution {
        output.push_str(&format!(
            "## Evolution Analysis\n\nEvolution tracking over {days} days would be displayed here.\n"
        ));
    }

    output
}

/// Format SATD items as summary
fn format_satd_summary(items: &[crate::services::satd_detector::TechnicalDebt]) -> String {
    if items.is_empty() {
        return "✅ No SATD items found. Excellent technical debt management!\n".to_string();
    }

    let mut severity_counts = std::collections::HashMap::new();
    let mut type_counts = std::collections::HashMap::new();

    for item in items {
        let sev_str = format!("{:?}", item.severity);
        let cat_str = format!("{:?}", item.category);
        *severity_counts.entry(sev_str).or_insert(0) += 1;
        *type_counts.entry(cat_str).or_insert(0) += 1;
    }

    let mut output = format!("📊 SATD Summary: {} total items\n\n", items.len());

    output.push_str("By Severity:\n");
    for (severity, count) in severity_counts {
        output.push_str(&format!("  {severity}: {count}\n"));
    }

    output.push_str("\nBy Type:\n");
    for (debt_type, count) in type_counts {
        output.push_str(&format!("  {debt_type}: {count}\n"));
    }

    output
}

/// Print SATD metrics
fn print_satd_metrics(items: &[crate::services::satd_detector::TechnicalDebt]) {
    eprintln!("\n📈 SATD Metrics:");
    eprintln!("  Total items: {}", items.len());

    let high_severity_count = items
        .iter()
        .filter(|item| {
            matches!(
                item.severity,
                crate::services::satd_detector::Severity::High
            )
        })
        .count();
    eprintln!("  High severity: {high_severity_count}");

    let files_with_satd: std::collections::HashSet<_> =
        items.iter().map(|item| &item.file).collect();
    eprintln!("  Files affected: {}", files_with_satd.len());
}

