/// Handle analyze subcommand
async fn handle_analyze(path: &PathBuf, config: &CudaTdgCommandConfig) -> Result<()> {
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
    let analyzer = CudaSimdAnalyzer::new();
    let result = analyzer.analyze(path)?;

    // GH-662: no files read ⇒ no score to report (see `format_unmeasured`).
    if report_if_unmeasured(&result, config)? {
        return Ok(());
    }

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
    let analyzer = CudaSimdAnalyzer::new();
    let result = analyzer.analyze(path)?;

    // GH-662: no files read ⇒ no score to report, but the report still has to
    // be delivered wherever the caller asked for it (see `format_unmeasured`).
    let report = if result.files_analyzed == 0 {
        if format == "json" {
            format_unmeasured_json(&result)?
        } else {
            format_unmeasured_text(&result)
        }
    } else {
        match format {
            "html" => format_html_report(&result)?,
            "json" => serde_json::to_string_pretty(&result)?,
            _ => format_markdown_report(&result)?,
        }
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
    let shared_required = tile_kv.saturating_mul(head_dim).saturating_mul(2);
    let overflows = shared_required > shared_memory;
    let undersized = tile_kv < head_dim;
    // A zero dimension is not a tile. `--head-dim 0 --tile-kv 0` satisfied both
    // of the checks above (0 >= 0, and 0 bytes fit in any budget) and was
    // reported "Status: VALID" with exit 0, so a CI job gating on this command
    // passed a kernel configuration that cannot exist.
    let degenerate = head_dim == 0 || tile_kv == 0;

    let output = match config.format {
        CudaTdgOutputFormat::Json => {
            let mut issues: Vec<&str> = Vec::new();
            if degenerate {
                issues.push("Degenerate tile: head_dim and tile_kv must both be > 0");
            }
            if undersized {
                issues.push("PAR-041: tile_kv < head_dim causes shared memory overflow");
            }
            if overflows {
                issues.push("Shared memory overflow: required exceeds the limit");
            }
            let result = serde_json::json!({
                "head_dim": head_dim,
                "tile_kv": tile_kv,
                "shared_memory_limit": shared_memory,
                "valid": issues.is_empty(),
                "shared_memory_required": shared_required,
                "issues": issues
            });
            serde_json::to_string_pretty(&result)?
        }
        _ => format_validate_tiles_text(head_dim, tile_kv, shared_memory),
    };

    write_output(&output, config)?;

    // The printed verdict and the exit status used to be computed from
    // different conditions: the text renderer required `tile_kv >= head_dim`
    // AND the configuration to fit in shared memory, while the only error
    // return was the PAR-041 check. `--head-dim 99999 --tile-kv 99999` printed
    // "Status: INVALID / Issue: Shared memory overflow" and exited 0, so CI
    // gating on this command passed a configuration overflowing shared memory
    // by ~400,000x. One verdict now drives both.
    if degenerate {
        return Err(anyhow!(
            "Degenerate tile: head_dim ({}) and tile_kv ({}) must both be > 0",
            head_dim,
            tile_kv
        ));
    }
    if undersized {
        return Err(anyhow!(
            "PAR-041: tile_kv ({}) < head_dim ({})",
            tile_kv,
            head_dim
        ));
    }
    if overflows {
        return Err(anyhow!(
            "Shared memory overflow: {} bytes required, {} bytes available",
            shared_required,
            shared_memory
        ));
    }

    Ok(())
}

fn format_validate_tiles_text(head_dim: usize, tile_kv: usize, shared_memory: usize) -> String {
    // `head_dim > 0` is part of the verdict: a zero dimension satisfies
    // `tile_kv >= head_dim` and needs 0 bytes, so it used to print VALID.
    let degenerate = head_dim == 0 || tile_kv == 0;
    let valid = !degenerate && tile_kv >= head_dim;
    let shared_required = tile_kv.saturating_mul(head_dim).saturating_mul(2); // FP16

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
        if degenerate {
            output.push_str("Issue: Degenerate tile - head_dim and tile_kv must both be > 0\n");
            output.push_str(&format!(
                "Fix: Set head_dim > 0 (currently {}) and tile_kv > 0 (currently {})\n",
                head_dim, tile_kv
            ));
        }
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

#[cfg(test)]
mod validate_tiles_verdict_tests {
    use super::*;

    fn config(output: Option<PathBuf>) -> CudaTdgCommandConfig {
        CudaTdgCommandConfig {
            path: PathBuf::from("."),
            command: None,
            format: CudaTdgOutputFormat::Terminal,
            min_score: 0.0,
            fail_on_p0: false,
            simd: false,
            wgpu: false,
            output,
            quiet: true,
        }
    }

    fn rendered_status(head_dim: usize, tile_kv: usize, shared_memory: usize) -> String {
        let text = format_validate_tiles_text(head_dim, tile_kv, shared_memory);
        if text.contains("Status: VALID") {
            "VALID".to_string()
        } else {
            "INVALID".to_string()
        }
    }

    async fn exits_ok(head_dim: usize, tile_kv: usize, shared_memory: usize) -> bool {
        let dir = tempfile::tempdir().expect("tempdir");
        let cfg = config(Some(dir.path().join("out.txt")));
        handle_validate_tiles(head_dim, tile_kv, shared_memory, &cfg)
            .await
            .is_ok()
    }

    /// The printed verdict and the exit status came from different conditions:
    /// a shared-memory overflow printed "Status: INVALID" and still returned
    /// Ok(()), so a CI job gating on this command passed the overflowing
    /// configuration. Every INVALID verdict must exit nonzero.
    #[tokio::test]
    async fn every_invalid_verdict_is_an_error() {
        // (head_dim, tile_kv, shared_memory)
        let cases = [
            (64usize, 64usize, 49152usize),   // valid
            (128, 128, 49152),                // valid
            (128, 64, 49152),                 // PAR-041: tile_kv < head_dim
            (99999, 99999, 49152),            // shared memory overflow
            (128, 256, 1024),                 // overflow only
        ];

        for (head_dim, tile_kv, shared_memory) in cases {
            let status = rendered_status(head_dim, tile_kv, shared_memory);
            let ok = exits_ok(head_dim, tile_kv, shared_memory).await;
            assert_eq!(
                status == "VALID",
                ok,
                "head_dim={head_dim} tile_kv={tile_kv} shared_memory={shared_memory}: \
                 printed {status} but exit was {}",
                if ok { "0" } else { "nonzero" }
            );
        }
    }

    /// A zero dimension is not a tile. `--head-dim 0 --tile-kv 0` passed both
    /// existing checks (0 >= 0, 0 bytes fits) and printed "Status: VALID" with
    /// exit 0.
    #[tokio::test]
    async fn degenerate_dimensions_are_invalid() {
        for (head_dim, tile_kv) in [(0usize, 0usize), (0, 64), (64, 0)] {
            let text = format_validate_tiles_text(head_dim, tile_kv, 49152);
            assert!(
                text.contains("Status: INVALID"),
                "head_dim={head_dim} tile_kv={tile_kv} must render INVALID, got:\n{text}"
            );
            assert!(
                !exits_ok(head_dim, tile_kv, 49152).await,
                "head_dim={head_dim} tile_kv={tile_kv} must exit nonzero"
            );
        }
    }

    /// The overflow case specifically: it used to be the silent one.
    #[tokio::test]
    async fn shared_memory_overflow_is_an_error() {
        assert!(
            !exits_ok(99999, 99999, 49152).await,
            "a configuration overflowing shared memory must not exit 0"
        );
    }
}
