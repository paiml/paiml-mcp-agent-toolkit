/// Handle mutation testing command
pub async fn handle(args: MutateArgs, _server: Arc<StatelessTemplateServer>) -> Result<()> {
    info!("Starting mutation testing on {:?}", args.target);

    // Sprint 70: Route to cargo-mutants backend if requested
    if args.use_cargo_mutants {
        return handle_cargo_mutants_backend(args).await;
    }

    // 1. Validate target
    let target = args
        .target
        .canonicalize()
        .context("Target file not found")?;

    // 2. Create engine
    let config = MutationConfig {
        strategy: MutationStrategy::Selective,
        max_mutants: 0,
        parallel_threads: args.jobs.unwrap_or_else(num_cpus::get),
    };
    let engine = MutationEngine::default_rust();

    // 3. Generate mutants
    let mutants = engine.generate_mutants_from_file(&target).await?;
    let total_mutants = mutants.len();
    eprintln!("Generated {} mutants", total_mutants);

    // 4. Execute mutants with progress indicators
    eprintln!("\nExecuting mutants...");
    let start_time = Instant::now();

    let results = if config.parallel_threads > 1 {
        execute_with_progress(engine, mutants, total_mutants).await?
    } else {
        execute_sequential_with_progress(engine, mutants, total_mutants).await?
    };

    let elapsed = start_time.elapsed();
    eprintln!("\nCompleted in {:.1}s\n", elapsed.as_secs_f64());

    // 5. Calculate score
    let score = MutationScore::from_results(&results);

    // 6. Output
    match args.output_format.as_str() {
        "json" => output_json(&score, &results, args.failures_only)?,
        "markdown" => output_markdown(&score, &results, args.failures_only)?,
        _ => output_text(&score, &results, args.failures_only)?,
    }

    // 7. Check threshold
    if let Some(threshold) = args.threshold {
        if score.score < threshold / 100.0 {
            anyhow::bail!(
                "Mutation score {:.1}% below threshold {:.1}%",
                score.score * 100.0,
                threshold
            );
        }
    }

    Ok(())
}

/// Execute mutants in parallel with progress indicators
async fn execute_with_progress(
    engine: MutationEngine,
    mutants: Vec<crate::services::mutation::types::Mutant>,
    total: usize,
) -> Result<Vec<MutationResult>> {
    use tokio::time::sleep;

    // Start execution in background
    let exec_handle = tokio::spawn(async move { engine.execute_mutants_parallel(mutants).await });

    // Progress reporting loop
    let mut completed = 0;
    while !exec_handle.is_finished() {
        sleep(Duration::from_millis(500)).await;

        // Simple progress indicator (will be enhanced with actual progress tracking)
        completed = (completed + 1) % (total + 1);
        print_progress(completed.min(total), total);
    }

    // Get final results
    let results = exec_handle.await??;
    print_progress(total, total);
    eprintln!(); // New line after progress

    Ok(results)
}

/// Execute mutants sequentially with progress indicators
async fn execute_sequential_with_progress(
    engine: MutationEngine,
    mutants: Vec<crate::services::mutation::types::Mutant>,
    total: usize,
) -> Result<Vec<MutationResult>> {
    let mut results = Vec::new();

    for (i, mutant) in mutants.into_iter().enumerate() {
        print_progress(i, total);

        // Execute single mutant (we need to expose this method)
        // For now, use the batch method with single item
        let single_result = engine.execute_mutants(vec![mutant]).await?;
        results.extend(single_result);
    }

    print_progress(total, total);
    eprintln!(); // New line after progress

    Ok(results)
}

/// Print progress indicator
fn print_progress(completed: usize, total: usize) {
    if total == 0 {
        return;
    }

    let percentage = (completed as f64 / total as f64) * 100.0;
    let bar_width = 40;
    let filled = (bar_width as f64 * completed as f64 / total as f64) as usize;
    let empty = bar_width - filled;

    eprint!(
        "\r[{}{}] {}/{} ({:.1}%)",
        "=".repeat(filled),
        " ".repeat(empty),
        completed,
        total,
        percentage
    );

    use std::io::Write;
    let _ = std::io::stderr().flush();
}
