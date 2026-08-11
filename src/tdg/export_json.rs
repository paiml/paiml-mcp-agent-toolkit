// JSON export implementations for TdgExporter
// Included by export.rs - shares parent module scope

impl TdgExporter {
    fn score_to_json(score: &TdgScore, options: &ExportOptions) -> Result<String> {
        let output = if options.include_metadata {
            json!({
                "score": score,
                "metadata": {
                    "exported_at": SystemTime::now(),
                    "version": env!("CARGO_PKG_VERSION"),
                    "options": {
                        "include_recommendations": options.include_recommendations,
                        "include_metrics": options.include_metrics,
                    }
                }
            })
        } else {
            json!(score)
        };

        if options.pretty_print {
            Ok(serde_json::to_string_pretty(&output)?)
        } else {
            Ok(serde_json::to_string(&output)?)
        }
    }

    fn project_to_json(project: &ProjectScore, options: &ExportOptions) -> Result<String> {
        let output = json!({
            "project": {
                "total_files": project.total_files,
                // GH #704: null, not 0.0/"F", when no file was analysed —
                // `not_measured` names the fields that are null for that reason.
                "average_score": project.average_score,
                "average_grade": project.average_grade.map(|g| g.to_string()),
                "not_measured": project.not_measured,
                "language_distribution": project.language_distribution,
                "files": if options.include_metrics { Some(&project.files) } else { None },
            },
            "metadata": if options.include_metadata {
                Some(json!({
                    "exported_at": SystemTime::now(),
                    "version": env!("CARGO_PKG_VERSION"),
                }))
            } else {
                None
            }
        });

        if options.pretty_print {
            Ok(serde_json::to_string_pretty(&output)?)
        } else {
            Ok(serde_json::to_string(&output)?)
        }
    }

    fn comparison_to_json(comparison: &Comparison, options: &ExportOptions) -> Result<String> {
        let output = json!({
            "comparison": {
                "delta": comparison.delta,
                "improvement_percentage": comparison.improvement_percentage,
                "winner": comparison.winner,
                "improvements": comparison.improvements,
                "regressions": comparison.regressions,
            },
            "source1": if options.include_metrics { Some(&comparison.source1) } else { None },
            "source2": if options.include_metrics { Some(&comparison.source2) } else { None },
        });

        if options.pretty_print {
            Ok(serde_json::to_string_pretty(&output)?)
        } else {
            Ok(serde_json::to_string(&output)?)
        }
    }
}
