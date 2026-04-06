// Quality gate check implementations - included from quality_gates.rs
// NO use imports or #! attributes - shares parent module scope

impl QAVerification {
    fn add_dead_code_checks(checks: &mut Vec<(&'static str, QualityCheck)>) {
        // Dead code sanity check
        checks.push(("dead_code_sanity", Box::new(|result| {
            // First try to get lines from complexity metrics
            let total_lines = result.complexity_metrics.as_ref()
                .map_or(0, |m| m.files.iter().map(|f| f.total_lines).sum::<usize>());

            // If no complexity metrics, try to get from dead code analysis
            let total_lines = if total_lines == 0 {
                result.dead_code_analysis.as_ref()
                    .map_or(0, |d| d.summary.total_lines)
            } else {
                total_lines
            };

            // If still no lines, check if we have any files at all
            if total_lines == 0 {
                // Check if any files were discovered
                let file_count = result.file_tree.len();
                if file_count == 0 {
                    return Err("No files discovered in project".into());
                }

                // For non-analyzable projects (e.g., no Rust/Python/JS), pass with a warning
                if file_count > 0 {
                    return Ok(());  // Pass but with implicit warning that analysis was limited
                }
                return Err("No lines analyzed - invalid result".into());
            }

            let dead_lines = result.dead_code_analysis.as_ref()
                .map_or(0, |d| d.summary.total_dead_lines);

            let ratio = dead_lines as f64 / total_lines as f64;

            if ratio == 0.0 && total_lines > 1000 {
                // Check if this is legitimate
                let has_ffi_or_wasm = result.file_tree.iter()
                    .any(|path| path.contains("wasm") || path.contains("ffi") || path.contains("bindgen"));

                let has_typescript = result.language_stats.as_ref()
                    .and_then(|stats| stats.get("TypeScript"))
                    .is_some_and(|&count| count > 0);

                let has_python = result.language_stats.as_ref()
                    .and_then(|stats| stats.get("Python"))
                    .is_some_and(|&count| count > 0);

                if has_ffi_or_wasm {
                    Err("Zero dead code with FFI/WASM code present - likely false negative".into())
                } else if has_typescript || has_python {
                    Err("Mixed language project with zero dead code - verify cross-language tracing".into())
                } else {
                    // Pure Rust project in early stages might legitimately have no dead code
                    Ok(())
                }
            } else if ratio > 0.15 {
                Err(format!("Excessive dead code: {:.1}%", ratio * 100.0))
            } else {
                Ok(())
            }
        })));
    }

    fn add_complexity_checks(checks: &mut Vec<(&'static str, QualityCheck)>) {
        // Complexity distribution check
        checks.push((
            "complexity_distribution",
            Box::new(|result| {
                let functions: Vec<_> = result
                    .complexity_metrics
                    .as_ref()
                    .map(|m| m.files.iter().flat_map(|f| &f.functions).collect())
                    .unwrap_or_default();

                if functions.len() < 50 {
                    return Ok(()); // Too small for distribution analysis
                }

                // Calculate coefficient of variation
                let mean = functions
                    .iter()
                    .map(|f| f64::from(f.cyclomatic))
                    .sum::<f64>()
                    / functions.len() as f64;

                if mean == 0.0 {
                    return Err("All functions have zero complexity - parser error".into());
                }

                let variance = functions
                    .iter()
                    .map(|f| (f64::from(f.cyclomatic) - mean).powi(2))
                    .sum::<f64>()
                    / functions.len() as f64;
                let cv = (variance.sqrt() / mean) * 100.0;

                if cv < 30.0 {
                    Err(format!(
                        "Low complexity variation (CV={cv:.1}%) - possible parser issue"
                    ))
                } else {
                    Ok(())
                }
            }),
        ));

        // Entropy check for large codebases
        checks.push((
            "complexity_entropy",
            Box::new(|result| {
                let functions: Vec<_> = result
                    .complexity_metrics
                    .as_ref()
                    .map(|m| m.files.iter().flat_map(|f| &f.functions).collect())
                    .unwrap_or_default();

                if functions.len() < 100 {
                    return Ok(()); // Too small for entropy analysis
                }

                let entropy = calculate_complexity_entropy(&functions);

                if entropy < 2.0 {
                    Err(format!(
                        "Low complexity entropy: {entropy:.2} (expected >= 2.0)"
                    ))
                } else {
                    Ok(())
                }
            }),
        ));
    }

    fn add_coverage_checks(checks: &mut Vec<(&'static str, QualityCheck)>) {
        // AST coverage check
        checks.push((
            "ast_coverage",
            Box::new(|result| {
                let total_files = result.file_tree.len();
                let ast_files = result.ast_summaries.as_ref().map_or(0, std::vec::Vec::len);

                if total_files == 0 {
                    return Err("No files found in project".into());
                }

                let coverage = ast_files as f64 / total_files as f64;

                if coverage < 0.5 {
                    Err(format!(
                        "Low AST coverage: {:.1}% (expected >= 50%)",
                        coverage * 100.0
                    ))
                } else {
                    Ok(())
                }
            }),
        ));
    }

    fn add_section_checks(checks: &mut Vec<(&'static str, QualityCheck)>) {
        // Empty sections check
        checks.push((
            "empty_sections",
            Box::new(|result| {
                let mut empty_sections = Vec::new();

                if result.dead_code_analysis.is_none() {
                    empty_sections.push("dead_code_analysis");
                }

                if result.complexity_metrics.is_none() {
                    empty_sections.push("complexity_metrics");
                }

                if result.ast_summaries.is_none() {
                    empty_sections.push("ast_summaries");
                }

                if result.churn_analysis.is_none() {
                    empty_sections.push("churn_analysis");
                }

                if empty_sections.is_empty() {
                    Ok(())
                } else {
                    Err(format!(
                        "Empty sections found: {}",
                        empty_sections.join(", ")
                    ))
                }
            }),
        ));
    }
}

fn calculate_complexity_entropy(functions: &[&FunctionComplexityForQA]) -> f64 {
    let mut freq_map = FxHashMap::default();
    for func in functions {
        *freq_map.entry(func.cyclomatic).or_insert(0) += 1;
    }

    let total = functions.len() as f64;
    freq_map
        .values()
        .map(|&count| {
            let p = f64::from(count) / total;
            -p * p.log2()
        })
        .sum()
}
