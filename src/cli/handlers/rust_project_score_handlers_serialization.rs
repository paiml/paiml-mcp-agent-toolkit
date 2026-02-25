// Included from rust_project_score_handlers.rs — NO use imports, NO #! attributes

/// Format score as JSON
fn format_json(
    score: &crate::services::rust_project_score::orchestrator::ProjectScore,
    recommendations: &[String],
) -> Result<String> {
    let json = serde_json::json!({
        "version": "1.1",
        "total_earned": score.total_earned,
        "total_possible": score.total_possible,
        "percentage": score.percentage,
        "grade": score.grade.to_string(),
        "categories": score.categories.iter().map(|(name, cat)| {
            serde_json::json!({
                "name": name,
                "earned": cat.earned,
                "max": cat.max,
                "percentage": cat.percentage(),
            })
        }).collect::<Vec<_>>(),
        "recommendations": recommendations,
    });

    serde_json::to_string_pretty(&json).context("Failed to serialize to JSON")
}

/// Format score as YAML
fn format_yaml(
    score: &crate::services::rust_project_score::orchestrator::ProjectScore,
    recommendations: &[String],
) -> Result<String> {
    let yaml = serde_yaml_ng::to_string(&serde_json::json!({
        "version": "1.1",
        "total_earned": score.total_earned,
        "total_possible": score.total_possible,
        "percentage": score.percentage,
        "grade": score.grade.to_string(),
        "categories": score.categories.iter().map(|(name, cat)| {
            serde_json::json!({
                "name": name,
                "earned": cat.earned,
                "max": cat.max,
                "percentage": cat.percentage(),
            })
        }).collect::<Vec<_>>(),
        "recommendations": recommendations,
    }))
    .context("Failed to serialize to YAML")?;

    Ok(yaml)
}
