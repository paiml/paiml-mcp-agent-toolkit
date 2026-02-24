// ── Document search helpers ─────────────────────────────────────────────────

/// Handle `--docs-only` mode: search only documents, skip code index.
pub(super) fn handle_docs_search(
    query: &str,
    limit: usize,
    project_path: &PathBuf,
    format: &QueryOutputFormat,
    quiet: bool,
) -> anyhow::Result<()> {
    let doc_results = run_document_query(query, limit, project_path, quiet)?;

    match format {
        QueryOutputFormat::Json => {
            let json = serde_json::to_string_pretty(&doc_results)
                .map_err(|e| anyhow::anyhow!("JSON serialize: {e}"))?;
            println!("{json}");
        }
        _ => {
            print_document_results(&doc_results, false);
        }
    }
    Ok(())
}

/// Emit a document results section appended after code results (for `--docs`).
pub(super) fn emit_docs_section(
    query: &str,
    limit: usize,
    project_path: &PathBuf,
    format: &QueryOutputFormat,
    quiet: bool,
) -> anyhow::Result<()> {
    let doc_results = run_document_query(query, limit, project_path, quiet)?;

    if doc_results.is_empty() {
        return Ok(());
    }

    match format {
        QueryOutputFormat::Json => {
            // For JSON, print a separate documents array
            let json = serde_json::json!({ "documents": doc_results });
            println!(
                "{}",
                serde_json::to_string_pretty(&json)
                    .map_err(|e| anyhow::anyhow!("JSON serialize: {e}"))?
            );
        }
        _ => {
            print_document_results(&doc_results, true);
        }
    }
    Ok(())
}

/// Execute the document query: build index if needed, then FTS5 search.
fn run_document_query(
    query: &str,
    limit: usize,
    project_path: &PathBuf,
    quiet: bool,
) -> anyhow::Result<Vec<crate::services::agent_context::DocumentResult>> {
    use crate::services::agent_context::document_index::{build_document_index, query_documents};
    use crate::services::agent_context::function_index::sqlite_backend::open_db;

    let db_path = project_path.join(".pmat").join("context.db");
    if !db_path.exists() {
        // Need to create DB with schema first
        std::fs::create_dir_all(project_path.join(".pmat"))
            .map_err(|e| anyhow::anyhow!("Failed to create .pmat dir: {e}"))?;
    }

    let conn = open_db(&db_path).map_err(|e| anyhow::anyhow!("{e}"))?;

    // Ensure documents schema exists (may be missing on pre-existing DBs)
    crate::services::agent_context::document_index::create_documents_schema(&conn)
        .map_err(|e| anyhow::anyhow!("{e}"))?;

    // Lazy-build document index
    if !quiet {
        eprint!("{DIM}Building document index...{RESET}");
    }
    let build_result =
        build_document_index(&conn, project_path).map_err(|e| anyhow::anyhow!("{e}"))?;
    if !quiet {
        eprintln!(
            "\r{DIM}Documents: {} scanned, {} indexed, {} cached{RESET}",
            build_result.files_scanned, build_result.files_indexed, build_result.files_skipped
        );
        for err in &build_result.errors {
            eprintln!("{DIM}{YELLOW}  warn: {err}{RESET}");
        }
    }

    let results = query_documents(&conn, query, limit).map_err(|e| anyhow::anyhow!("{e}"))?;
    Ok(results)
}

/// Print document results to terminal with colors.
pub(super) fn print_document_results(
    results: &[crate::services::agent_context::DocumentResult],
    show_separator: bool,
) {
    if results.is_empty() {
        eprintln!("{DIM}No document matches found.{RESET}");
        return;
    }

    if show_separator {
        println!("\n{BOLD}-- Document Results --{RESET}\n");
    }

    for (i, r) in results.iter().enumerate() {
        let doc_type_badge = match r.doc_type.as_str() {
            "pdf" => format!("{RED}PDF{RESET}"),
            "svg" => format!("{GREEN}SVG{RESET}"),
            "image" => format!("{YELLOW}IMG{RESET}"),
            "markdown" => format!("{CYAN}MD{RESET}"),
            "plaintext" => format!("{DIM}TXT{RESET}"),
            other => other.to_string(),
        };

        let location = if let Some(page) = r.page_number {
            format!(" p.{page}")
        } else if let Some(ref heading) = r.section_heading {
            format!(" \u{00a7} {heading}")
        } else {
            String::new()
        };

        let quality_bar = if r.extraction_quality >= 0.8 {
            format!("{GREEN}\u{25cf}{RESET}")
        } else if r.extraction_quality >= 0.5 {
            format!("{YELLOW}\u{25cf}{RESET}")
        } else {
            format!("{RED}\u{25cb}{RESET}")
        };

        println!(
            "{DIM}{:>3}.{RESET} [{doc_type_badge}] {quality_bar} {BOLD}{}{RESET}{DIM}{location}{RESET}",
            i + 1,
            r.file_path,
        );

        // Print snippet (first 200 chars)
        let snippet = if r.snippet.len() > 200 {
            format!("{}...", &r.snippet[..200])
        } else {
            r.snippet.clone()
        };
        println!("     {DIM}{snippet}{RESET}");
    }

    println!(
        "\n{DIM}Found {} document match{}{RESET}",
        results.len(),
        if results.len() == 1 { "" } else { "es" }
    );
}

/// Apply coverage diff enrichment from a baseline file
pub(super) fn apply_coverage_diff(
    results: &mut [QueryResult],
    project_path: &std::path::Path,
    diff_path: &std::path::Path,
    quiet: bool,
) {
    match std::fs::read_to_string(diff_path) {
        Ok(json) => match build_coverage_map(&json, project_path) {
            Ok(baseline) => {
                enrich_with_coverage_diff(results, &baseline);
            }
            Err(e) => {
                if !quiet {
                    eprintln!("Warning: Could not parse coverage baseline: {}", e);
                }
            }
        },
        Err(e) => {
            if !quiet {
                eprintln!(
                    "Warning: Could not read coverage baseline {}: {}",
                    diff_path.display(),
                    e
                );
            }
        }
    }
}
