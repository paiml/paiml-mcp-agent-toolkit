// cargo_mutants_backend_runner.rs — included from cargo_mutants_backend.rs
// Execute cargo-mutants and return path to output directory

/// Execute cargo-mutants and return path to output directory
pub fn execute(config: CargoMutantsConfig) -> Result<PathBuf> {
    // 1. Detect and validate cargo-mutants installation
    eprintln!("🧪 cargo-mutants Backend");
    eprintln!();

    let wrapper = CargoMutantsWrapper::new().map_err(|e| {
        anyhow::anyhow!(
            "cargo-mutants not found. Install: cargo install cargo-mutants. Error: {}",
            e
        )
    })?;

    wrapper.validate_version()
        .map_err(|e| anyhow::anyhow!("cargo-mutants version too old. Minimum v24.7.0 required. Upgrade: cargo install --force cargo-mutants. Error: {}", e))?;

    let version = wrapper
        .version()
        .map_err(|e| anyhow::anyhow!("Failed to get cargo-mutants version: {}", e))?;
    eprintln!("✅ Detected: {}", version);
    eprintln!();

    // Determine output directory
    let output_dir = if let Some(ref output) = config.output {
        output.clone()
    } else {
        config.path.join("mutants.out")
    };

    // 2. Build cargo mutants command
    let mut cmd = Command::new("cargo");
    cmd.arg("mutants");
    cmd.arg("--output").arg(&output_dir);

    // Set working directory
    cmd.current_dir(&config.path);

    // Add timeout
    cmd.arg("--timeout").arg(config.timeout.to_string());

    // Add parallel jobs
    if let Some(j) = config.jobs {
        cmd.arg("--jobs").arg(j.to_string());
    }

    // Add features
    if config.all_features {
        cmd.arg("--all-features");
    } else if config.no_default_features {
        cmd.arg("--no-default-features");
        if let Some(ref feats) = config.features {
            cmd.arg("--features").arg(feats.join(","));
        }
    } else if let Some(ref feats) = config.features {
        cmd.arg("--features").arg(feats.join(","));
    }

    // Add no-shuffle flag
    if config.no_shuffle {
        cmd.arg("--no-shuffle");
    }

    // Display command being executed
    eprintln!(
        "🔧 Executing: cargo mutants --output {} --timeout {} {}",
        output_dir.display(),
        config.timeout,
        if let Some(j) = config.jobs {
            format!("--jobs {}", j)
        } else {
            String::new()
        }
    );
    eprintln!();

    // 3. Execute cargo-mutants
    eprintln!("⏳ Running mutation tests... (this may take several minutes)");
    eprintln!();

    let output_result = cmd
        .output()
        .context("Failed to execute cargo mutants command")?;

    // cargo-mutants exit codes:
    // 0 - Success (all mutants caught)
    // 2 - Success with missed mutants (this is expected!)
    // Other - Actual failure
    let exit_code = output_result.status.code().unwrap_or(-1);
    if exit_code != 0 && exit_code != 2 {
        let stderr = String::from_utf8_lossy(&output_result.stderr);
        anyhow::bail!(
            "cargo-mutants execution failed with exit code {}:\n{}",
            exit_code,
            stderr
        );
    }

    eprintln!("✅ Mutation testing complete");
    eprintln!();

    // cargo-mutants may create a nested directory structure
    // Check if outcomes.json exists, if not check nested location
    let actual_output = if output_dir.join("outcomes.json").exists() {
        output_dir
    } else if output_dir
        .join("mutants.out")
        .join("outcomes.json")
        .exists()
    {
        output_dir.join("mutants.out")
    } else {
        output_dir
    };

    Ok(actual_output)
}
