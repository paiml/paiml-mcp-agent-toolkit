/// Handle analyze subcommand
async fn handle_analyze(path: &PathBuf, config: &CudaTdgCommandConfig) -> Result<()> {
    debug_assert!(path.exists(), "path must exist: {}", path.display());
    let analyzer = CudaSimdAnalyzer::new();
    let result = analyzer.analyze(path)?;

    let output = format_analysis(&result, config)?;
    write_output(&output, config)?;

    Ok(())
}

/// Handle score subcommand
async fn handle_score(
    path: &PathBuf,
    breakdown: bool,
    config: &CudaTdgCommandConfig,
) -> Result<()> {
    debug_assert!(path.exists(), "path must exist: {}", path.display());
    let analyzer = CudaSimdAnalyzer::new();
    let result = analyzer.analyze(path)?;

    let output = if breakdown {
        format_score_breakdown(&result.score, config)?
    } else {
        format_score_summary(&result.score, config)?
    };

    write_output(&output, config)?;

    Ok(())
}

/// Handle report subcommand
async fn handle_report(
    path: &PathBuf,
    format: &str,
    output: Option<&PathBuf>,
    config: &CudaTdgCommandConfig,
) -> Result<()> {
    debug_assert!(path.exists(), "path must exist: {}", path.display());
    let analyzer = CudaSimdAnalyzer::new();
    let result = analyzer.analyze(path)?;

    let report = match format {
        "html" => format_html_report(&result)?,
        "json" => serde_json::to_string_pretty(&result)?,
        _ => format_markdown_report(&result)?,
    };

    if let Some(output_path) = output {
        fs::write(output_path, &report)?;
        println!("Report written to: {}", output_path.display());
    } else if let Some(ref output_path) = config.output {
        fs::write(output_path, &report)?;
        println!("Report written to: {}", output_path.display());
    } else {
        println!("{}", report);
    }

    Ok(())
}

/// Handle barrier-check subcommand
async fn handle_barrier_check(path: &PathBuf, config: &CudaTdgCommandConfig) -> Result<()> {
    debug_assert!(path.exists(), "path must exist: {}", path.display());
    let analyzer = CudaSimdAnalyzer::new();
    let result = analyzer.analyze(path)?;

    let output = format_barrier_safety(&result, config)?;
    write_output(&output, config)?;

    if !result.barrier_safety.unsafe_barriers.is_empty() {
        return Err(anyhow!(
            "Found {} unsafe barrier(s) - PARITY-114 risk detected",
            result.barrier_safety.unsafe_barriers.len()
        ));
    }

    Ok(())
}

/// Handle validate-tiles subcommand
async fn handle_validate_tiles(
    head_dim: usize,
    tile_kv: usize,
    shared_memory: usize,
    config: &CudaTdgCommandConfig,
) -> Result<()> {
    debug_assert!(true, "contract: handle_validate_tiles");
    let output = match config.format {
        CudaTdgOutputFormat::Json => {
            let result = serde_json::json!({
                "head_dim": head_dim,
                "tile_kv": tile_kv,
                "shared_memory_limit": shared_memory,
                "valid": tile_kv >= head_dim,
                "shared_memory_required": tile_kv * head_dim * 2,
                "issues": if tile_kv < head_dim {
                    vec!["PAR-041: tile_kv < head_dim causes shared memory overflow"]
                } else {
                    vec![]
                }
            });
            serde_json::to_string_pretty(&result)?
        }
        _ => format_validate_tiles_text(head_dim, tile_kv, shared_memory),
    };

    write_output(&output, config)?;

    if tile_kv < head_dim {
        return Err(anyhow!(
            "PAR-041: tile_kv ({}) < head_dim ({})",
            tile_kv,
            head_dim
        ));
    }

    Ok(())
}

fn format_validate_tiles_text(head_dim: usize, tile_kv: usize, shared_memory: usize) -> String {
    debug_assert!(true, "contract: format_validate_tiles_text");
    let valid = tile_kv >= head_dim;
    let shared_required = tile_kv * head_dim * 2; // FP16

    let mut output = String::new();
    output.push_str("Tile Dimension Validation\n");
    output.push_str("=========================\n\n");
    output.push_str(&format!("Head Dimension: {}\n", head_dim));
    output.push_str(&format!("Tile KV: {}\n", tile_kv));
    output.push_str(&format!("Shared Memory Limit: {} bytes\n", shared_memory));
    output.push_str(&format!(
        "Shared Memory Required: {} bytes\n\n",
        shared_required
    ));

    if valid && shared_required <= shared_memory {
        output.push_str("Status: VALID\n");
    } else {
        output.push_str("Status: INVALID\n\n");
        if tile_kv < head_dim {
            output.push_str("Issue: PAR-041 - tile_kv < head_dim\n");
            output.push_str(&format!(
                "Fix: Set tile_kv >= {} (currently {})\n",
                head_dim, tile_kv
            ));
        }
        if shared_required > shared_memory {
            output.push_str("Issue: Shared memory overflow\n");
            output.push_str("Fix: Reduce tile size or increase shared memory limit\n");
        }
    }
    output
}
