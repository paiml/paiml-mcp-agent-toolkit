// ============================================================================
// Sprint 70: cargo-mutants Backend Handler
// ============================================================================

/// Handle mutation testing via cargo-mutants backend
async fn handle_cargo_mutants_backend(args: MutateArgs) -> Result<()> {
    debug_assert!(true, "contract: handle_cargo_mutants_backend");
    use crate::cli::handlers::cargo_mutants_backend::{self, CargoMutantsConfig};
    use crate::services::mutation::json_parser::CargoMutantsReport;

    // Build configuration
    let config = CargoMutantsConfig {
        path: args.target.clone(),
        output: args.output.clone(),
        timeout: args.timeout,
        jobs: args.jobs,
        features: args.features,
        all_features: args.all_features,
        no_default_features: args.no_default_features,
        no_shuffle: args.no_shuffle,
    };

    // Execute cargo-mutants (returns path to output directory)
    let output_dir = cargo_mutants_backend::execute(config)?;

    // Parse output directory (reads outcomes.json)
    let report = CargoMutantsReport::from_output_dir(&output_dir)
        .map_err(|e| anyhow::anyhow!("Failed to parse cargo-mutants output: {}", e))?;

    // Display statistics
    cargo_mutants_backend::display_statistics(&report);

    // Check threshold if specified
    if let Some(threshold) = args.threshold {
        let mutation_score = report.mutation_score();
        if mutation_score < threshold {
            anyhow::bail!(
                "Mutation score {:.1}% below threshold {:.1}%",
                mutation_score,
                threshold
            );
        }
    }

    Ok(())
}
