/// Format `AssemblyScript` analysis results
fn format_assemblyscript_results(
    results: &[(PathBuf, WasmComplexity)],
    format: &ComplexityOutputFormat,
    perf: bool,
    elapsed: std::time::Duration,
) -> Result<String> {
    debug_assert!(!results.is_empty(), "results must not be empty");
    if format == &ComplexityOutputFormat::Json {
        let output = serde_json::json!({
            "analysis_type": "assemblyscript",
            "files_analyzed": results.len(),
            "results": results.iter().map(|(path, complexity)| {
                serde_json::json!({
                    "file": path.display().to_string(),
                    "complexity": complexity
                })
            }).collect::<Vec<_>>(),
            "performance": if perf {
                Some(serde_json::json!({
                    "total_time_ms": elapsed.as_millis(),
                    "avg_time_per_file_ms": elapsed.as_millis() / (results.len() as u128).max(1)
                }))
            } else { None }
        });
        Ok(serde_json::to_string_pretty(&output)?)
    } else {
        let mut output = String::from("# AssemblyScript Analysis Report\n\n");
        output.push_str(&format!("📁 **Files analyzed**: {}\n", results.len()));
        output.push_str(&format!(
            "⏱️  **Analysis time**: {:.2}s\n\n",
            elapsed.as_secs_f64()
        ));

        if !results.is_empty() {
            output.push_str("## Results\n\n");
            for (path, complexity) in results {
                output.push_str(&format!("### {}\n", path.display()));
                output.push_str(&format!(
                    "- **Cyclomatic complexity**: {}\n",
                    complexity.cyclomatic
                ));
                output.push_str(&format!(
                    "- **Cognitive complexity**: {}\n",
                    complexity.cognitive
                ));
                output.push_str(&format!(
                    "- **Memory pressure**: {:.2}\n\n",
                    complexity.memory_pressure
                ));
            }
        }

        Ok(output)
    }
}

/// Format WebAssembly analysis results
fn format_webassembly_results(
    results: &[(PathBuf, WasmMetrics)],
    format: &ComplexityOutputFormat,
    perf: bool,
    elapsed: std::time::Duration,
) -> Result<String> {
    debug_assert!(!results.is_empty(), "results must not be empty");
    if format == &ComplexityOutputFormat::Json {
        let output = serde_json::json!({
            "analysis_type": "webassembly",
            "files_analyzed": results.len(),
            "results": results.iter().map(|(path, metrics)| {
                serde_json::json!({
                    "file": path.display().to_string(),
                    "metrics": metrics
                })
            }).collect::<Vec<_>>(),
            "performance": if perf {
                Some(serde_json::json!({
                    "total_time_ms": elapsed.as_millis(),
                    "avg_time_per_file_ms": elapsed.as_millis() / (results.len() as u128).max(1)
                }))
            } else { None }
        });
        Ok(serde_json::to_string_pretty(&output)?)
    } else {
        let mut output = String::from("# WebAssembly Analysis Report\n\n");
        output.push_str(&format!("📁 **Files analyzed**: {}\n", results.len()));
        output.push_str(&format!(
            "⏱️  **Analysis time**: {:.2}s\n\n",
            elapsed.as_secs_f64()
        ));

        if !results.is_empty() {
            output.push_str("## Results\n\n");
            for (path, metrics) in results {
                output.push_str(&format!("### {}\n", path.display()));
                output.push_str(&format!("- **Functions**: {}\n", metrics.function_count));
                output.push_str(&format!("- **Imports**: {}\n", metrics.import_count));
                output.push_str(&format!("- **Exports**: {}\n", metrics.export_count));
                output.push_str(&format!(
                    "- **Memory pages**: {}\n\n",
                    metrics.linear_memory_pages
                ));
            }
        }

        Ok(output)
    }
}
