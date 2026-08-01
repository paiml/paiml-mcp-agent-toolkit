/// Format provability results as JSON
#[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
pub fn format_provability_json(
    function_ids: &[FunctionId],
    summaries: &[ProofSummary],
    include_evidence: bool,
) -> Result<String> {
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
                // DETERMINISM (round-3 sweep): `analysis_time_us` was the only
                // thing that moved between runs of
                // `analyze provability --format json` on an unchanged tree —
                // the same four functions came back with 28/1/1/1 µs on one run
                // and 12/2/2/2 µs on the next, so every run diffed. A wall-clock
                // duration measures the machine and its current load, not the
                // code under analysis, so it does not belong in the analysis
                // document; the human renderer (`--format detailed`) still
                // prints it. See contracts/pmat-no-fabrication-v1.yaml, the
                // determinism criterion.
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
