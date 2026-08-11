// Included from rust_project_score_handlers.rs — NO use imports, NO #! attributes

/// Build the serializable body shared by `--format json` and `--format yaml`.
///
/// #687: this used to fold `score.categories.values()` (a `HashMap`, whose
/// iteration order is randomised per process) and to emit the categories in
/// that same random order. Float addition is not associative, so the totals and
/// the mean percentage wobbled by one ULP between runs — `--format json` on an
/// unchanged project produced two distinct md5 sums over five runs, differing
/// only in `"percentage": 28.001373626373628` vs `28.001373626373624`. Every
/// figure below now comes from `rust_project_score::aggregation`, which folds
/// in name-sorted order and rounds, and the category array is emitted in that
/// same sorted order — the order `format_text` and `format_markdown` already
/// used, so the four renderers cannot disagree.
fn build_score_document(
    score: &crate::services::rust_project_score::orchestrator::ProjectScore,
    recommendations: &[String],
) -> serde_json::Value {
    use crate::services::rust_project_score::aggregation;

    // Totals exclude N/A categories (#237: GPU N/A pollutes totals).
    let applicable_earned = aggregation::applicable_earned(&score.categories);
    let applicable_possible = aggregation::applicable_possible(&score.categories);

    let categories: Vec<serde_json::Value> = aggregation::sorted_categories(&score.categories)
        .into_iter()
        .map(|(name, cat)| {
            serde_json::json!({
                "name": name,
                "earned": aggregation::round_score(cat.earned),
                "max": aggregation::round_score(cat.max),
                "percentage": aggregation::round_score(cat.percentage()),
                "applicable": cat.applicable,
            })
        })
        .collect();

    // ARITHMETIC SANITY (round-3 sweep): `percentage` is the unweighted MEAN of
    // the per-category percentages, not `total_earned / total_possible`. json
    // and yaml printed all three side by side with no label, so the document
    // contradicted itself on its face: `236.9 / 289 * 100 = 81.97`, not the
    // `87.22669` shown next to it (the mean of the 11 category percentages,
    // 959.49359/11). Every category was `applicable: true`, so the documented
    // "excludes categories that do not apply" caveat did not explain the gap
    // either. Both figures are now emitted and both are named.
    //
    // #717: and the grade is no longer taken from `percentage` — it is derived
    // from `points_percentage`, the ratio that actually follows from
    // `total_earned / total_possible` printed beside it. `grade_basis` names it
    // so a consumer never has to guess which of the two was graded.
    let points_percentage =
        crate::services::rust_project_score::orchestrator::points_percentage(&score.categories);

    serde_json::json!({
        "version": "1.2",
        "total_earned": applicable_earned,
        "total_possible": applicable_possible,
        "percentage": aggregation::round_score(score.percentage),
        "percentage_basis": "mean of applicable category percentages",
        "points_percentage": points_percentage,
        "points_percentage_basis": "applicable points earned / applicable points possible",
        "grade": score.grade.to_string(),
        "grade_basis": "points_percentage",
        "categories": categories,
        "recommendations": recommendations,
    })
}

/// Format score as JSON
fn format_json(
    score: &crate::services::rust_project_score::orchestrator::ProjectScore,
    recommendations: &[String],
) -> Result<String> {
    let json = build_score_document(score, recommendations);
    serde_json::to_string_pretty(&json).context("Failed to serialize to JSON")
}

/// Format score as YAML
fn format_yaml(
    score: &crate::services::rust_project_score::orchestrator::ProjectScore,
    recommendations: &[String],
) -> Result<String> {
    let doc = build_score_document(score, recommendations);
    serde_yaml_ng::to_string(&doc).context("Failed to serialize to YAML")
}
