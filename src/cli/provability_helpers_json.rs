/// Format provability results as JSON
pub fn format_provability_json(
    function_ids: &[FunctionId],
    summaries: &[ProofSummary],
    include_evidence: bool,
) -> Result<String> {
    debug_assert!(!function_ids.is_empty(), "function_ids must not be empty");
    let results: Vec<_> = function_ids
        .iter()
        .zip(summaries.iter())
        .map(|(func_id, summary)| {
            let mut result = serde_json::json!({
                "function": {
                    "name": func_id.function_name,
                    "file": func_id.file_path,
                    "line": func_id.line_number,
                },
                "provability_score": summary.provability_score,
                "analysis_time_us": summary.analysis_time_us,
                "verified_properties": summary.verified_properties.len(),
            });

            if include_evidence {
                result["properties"] = serde_json::json!(summary
                    .verified_properties
                    .iter()
                    .map(|prop| {
                        serde_json::json!({
                            "type": format!("{:?}", prop.property_type),
                            "confidence": prop.confidence,
                            "evidence": prop.evidence,
                        })
                    })
                    .collect::<Vec<_>>());
            }

            result
        })
        .collect();

    let output = serde_json::json!({
        "provability_analysis": {
            "total_functions": function_ids.len(),
            "results": results,
        }
    });

    serde_json::to_string_pretty(&output).map_err(Into::into)
}
