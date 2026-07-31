// Included from rust_project_score_handlers.rs — NO use imports, NO #! attributes

/// Categories as a stably ordered array.
///
/// `ProjectScore::categories` is a `HashMap`, so serialising it with `.iter()`
/// produced a different array order on every invocation — three runs over an
/// unchanged project gave three different orders while every number was
/// identical, making the JSON undiffable and unusable as a CI baseline
/// (GH #687). The text renderer already sorted by name; JSON and YAML now do
/// the same, so all three agree.
fn categories_sorted_by_name(
    score: &crate::services::rust_project_score::orchestrator::ProjectScore,
) -> Vec<serde_json::Value> {
    let mut categories: Vec<_> = score.categories.iter().collect();
    categories.sort_by_key(|(name, _)| *name);
    categories
        .into_iter()
        .map(|(name, cat)| {
            serde_json::json!({
                "name": name,
                "earned": cat.earned,
                "max": cat.max,
                "percentage": cat.percentage(),
                "applicable": cat.applicable,
            })
        })
        .collect()
}

/// Format score as JSON
fn format_json(
    score: &crate::services::rust_project_score::orchestrator::ProjectScore,
    recommendations: &[String],
) -> Result<String> {
    // Calculate totals excluding N/A categories (#237: GPU N/A pollutes totals)
    let applicable_earned: f64 = score
        .categories
        .values()
        .filter(|cat| cat.applicable)
        .map(|cat| cat.earned)
        .sum();
    let applicable_possible: f64 = score
        .categories
        .values()
        .filter(|cat| cat.applicable)
        .map(|cat| cat.max)
        .sum();

    let json = serde_json::json!({
        "version": "1.1",
        "total_earned": applicable_earned,
        "total_possible": applicable_possible,
        "percentage": score.percentage,
        "grade": score.grade.to_string(),
        "categories": categories_sorted_by_name(score),
        "recommendations": recommendations,
    });

    serde_json::to_string_pretty(&json).context("Failed to serialize to JSON")
}

/// Format score as YAML
fn format_yaml(
    score: &crate::services::rust_project_score::orchestrator::ProjectScore,
    recommendations: &[String],
) -> Result<String> {
    // Calculate totals excluding N/A categories (#237: GPU N/A pollutes totals)
    let applicable_earned: f64 = score
        .categories
        .values()
        .filter(|cat| cat.applicable)
        .map(|cat| cat.earned)
        .sum();
    let applicable_possible: f64 = score
        .categories
        .values()
        .filter(|cat| cat.applicable)
        .map(|cat| cat.max)
        .sum();

    let yaml = serde_yaml_ng::to_string(&serde_json::json!({
        "version": "1.1",
        "total_earned": applicable_earned,
        "total_possible": applicable_possible,
        "percentage": score.percentage,
        "grade": score.grade.to_string(),
        "categories": categories_sorted_by_name(score),
        "recommendations": recommendations,
    }))
    .context("Failed to serialize to YAML")?;

    Ok(yaml)
}
