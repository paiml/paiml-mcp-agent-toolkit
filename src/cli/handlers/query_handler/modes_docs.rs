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
            // Documents go to STDERR, not stdout.
            //
            // `--docs` is on by default, and this used to `println!` a second
            // top-level JSON document after the code-results array. Two
            // concatenated documents are not valid JSON, so the default output
            // of `pmat query --format json` — the search command CLAUDE.md
            // mandates over grep — could not be piped to `jq` at all; only
            // `--no-docs` parsed. stdout now carries exactly one document.
            //
            // stderr is where the sibling `raw_matches` section already goes
            // (`print_raw_results`), so this keeps the two supplementary
            // sections consistent and loses no data. For documents as
            // machine-readable stdout, use `--docs-only`.
            let json = serde_json::json!({ "documents": doc_results });
            eprintln!(
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
        // Need to create DB with schema first. The directory carries its own
        // ignore rule (#1070); `open_db` below reports if it could not be made.
        crate::utils::pmat_cache_dir::ensure_cache_dir(project_path);
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
    // Every sequence below goes through `colors::seq`, the one place that
    // decides whether this binary emits colour. These were the raw constants
    // interpolated directly, so this printer coloured unconditionally:
    // `--color never` did not silence it, and redirecting the output wrote
    // escapes into the file. That is the same defect `tint` had, in a third
    // place — the constants stay importable, so the guard belongs at the use
    // site, and pointing at the shared rule keeps a fourth copy from appearing.
    use crate::cli::colors::seq;
    let (bold, dim, reset) = (seq(BOLD), seq(DIM), seq(RESET));
    let (red, green, yellow, cyan) = (seq(RED), seq(GREEN), seq(YELLOW), seq(CYAN));
    if results.is_empty() {
        eprintln!("{dim}No document matches found.{reset}");
        return;
    }

    if show_separator {
        println!("\n{bold}-- Document Results --{reset}\n");
    }

    for (i, r) in results.iter().enumerate() {
        let doc_type_badge = match r.doc_type.as_str() {
            "pdf" => format!("{red}PDF{reset}"),
            "svg" => format!("{green}SVG{reset}"),
            "image" => format!("{yellow}IMG{reset}"),
            "markdown" => format!("{cyan}MD{reset}"),
            "plaintext" => format!("{dim}TXT{reset}"),
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
            format!("{green}\u{25cf}{reset}")
        } else if r.extraction_quality >= 0.5 {
            format!("{yellow}\u{25cf}{reset}")
        } else {
            format!("{red}\u{25cb}{reset}")
        };

        println!(
            "{dim}{:>3}.{reset} [{doc_type_badge}] {quality_bar} {bold}{}{reset}{dim}{location}{reset}",
            i + 1,
            r.file_path,
        );

        // Print snippet (first 200 chars)
        let snippet = if r.snippet.len() > 200 {
            format!("{}...", &r.snippet[..200])
        } else {
            r.snippet.clone()
        };
        println!("     {dim}{snippet}{reset}");
    }

    println!(
        "\n{dim}Found {} document match{}{reset}",
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
