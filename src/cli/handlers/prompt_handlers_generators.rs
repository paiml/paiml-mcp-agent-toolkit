/// Handle defect-aware prompt generation
async fn handle_generate_prompt(
    task: &str,
    context: &str,
    summary_path: &PathBuf,
    output: &Option<PathBuf>,
) -> Result<()> {
    let generator = DefectAwarePromptGenerator::from_file(summary_path)
        .context("Failed to load organizational intelligence summary")?;

    let prompt = generator.generate_prompt(task, context);

    if let Some(output_path) = output {
        std::fs::write(output_path, &prompt)
            .context(format!("Failed to write prompt to {:?}", output_path))?;
        println!("✅ Defect-aware prompt written to {:?}", output_path);
    } else {
        println!("{}", prompt);
    }

    Ok(())
}

/// Handle EXTREME TDD ticket prompt generation
async fn handle_ticket_prompt(
    ticket: &str,
    summary_path: Option<&PathBuf>,
    output: &Option<PathBuf>,
) -> Result<()> {
    let mut prompt = format!(
        "# EXTREME TDD: Fix Ticket\n\n\
         ## Ticket\n{}\n\n\
         ## Workflow\n\
         1. **RED**: Write failing test that reproduces the issue\n\
         2. **GREEN**: Implement minimal fix to make test pass\n\
         3. **REFACTOR**: Clean up code while keeping tests green\n\
         4. **VERIFY**: Run full test suite and quality gates\n\
         5. **COMMIT**: Only commit if all gates pass\n\n\
         ## Quality Gates (Before Commit)\n\
         ```bash\n\
         pmat analyze tdg --threshold 85\n\
         cargo test --all-features\n\
         cargo llvm-cov report --summary-only\n\
         ```\n\n",
        ticket
    );

    // Add organizational intelligence if available
    if let Some(summary) = summary_path {
        enrich_ticket_prompt_with_intelligence(&mut prompt, summary)?;
    }

    write_prompt_output(&prompt, output, "Ticket")
}

/// Enrich ticket prompt with organizational intelligence data
fn enrich_ticket_prompt_with_intelligence(
    prompt: &mut String,
    summary: &PathBuf,
) -> Result<()> {
    if summary.exists() {
        let generator = DefectAwarePromptGenerator::from_file(summary)?;
        prompt.push_str(&format!(
            "## Organizational Intelligence\n\
             Based on analysis of {} repositories with {} commits:\n\n\
             ### Common Defect Patterns to Avoid\n",
            generator.metadata.repositories_analyzed, generator.metadata.commits_analyzed
        ));

        for pattern in generator.defect_patterns.iter().take(5) {
            let tdg = pattern
                .quality_signals
                .avg_tdg_score
                .map(|s| format!("{:.1}", s))
                .unwrap_or_else(|| "N/A".to_string());
            prompt.push_str(&format!(
                "- {} ({} occurrences, TDG: {})\n",
                pattern.category, pattern.frequency, tdg
            ));
        }
        prompt.push('\n');
    }
    Ok(())
}

/// Handle specification-based implementation prompt
async fn handle_implement_prompt(
    spec_path: &PathBuf,
    summary_path: Option<&PathBuf>,
    output: &Option<PathBuf>,
) -> Result<()> {
    debug_assert!(spec_path.exists(), "spec_path must exist: {}", spec_path.display());
    let spec_content = std::fs::read_to_string(spec_path).context(format!(
        "Failed to read specification file: {:?}",
        spec_path
    ))?;

    let mut prompt = format!(
        "# Implementation from Specification\n\n\
         ## Specification\n{}\n\n\
         ## Implementation Strategy\n\
         1. **Analyze**: Break down spec into testable components\n\
         2. **RED**: Write tests for each component (EXTREME TDD)\n\
         3. **GREEN**: Implement to pass tests\n\
         4. **REFACTOR**: Optimize while maintaining test coverage\n\
         5. **VERIFY**: All quality gates must pass\n\n\
         ## Quality Requirements\n\
         - TDG Score: 85+\n\
         - Test Coverage: 85%+\n\
         - Max Complexity: 10\n\
         - Zero SATD (Self-Admitted Technical Debt)\n\n",
        spec_content
    );

    // Add organizational intelligence if available
    if let Some(summary) = summary_path {
        enrich_implement_prompt_with_intelligence(&mut prompt, summary)?;
    }

    write_prompt_output(&prompt, output, "Implementation")
}

/// Enrich implementation prompt with organizational intelligence data
fn enrich_implement_prompt_with_intelligence(
    prompt: &mut String,
    summary: &PathBuf,
) -> Result<()> {
    if summary.exists() {
        let generator = DefectAwarePromptGenerator::from_file(summary)?;
        prompt.push_str(&format!(
            "## Organizational Quality Standards\n\
             Based on {} repositories, {} commits:\n\n",
            generator.metadata.repositories_analyzed, generator.metadata.commits_analyzed
        ));

        for pattern in generator.defect_patterns.iter().take(3) {
            if let Some(tdg) = pattern.quality_signals.avg_tdg_score {
                prompt.push_str(&format!(
                    "⚠️  Avoid {}: {} historical occurrences (TDG: {:.1})\n",
                    pattern.category, pattern.frequency, tdg
                ));
            }
        }
        prompt.push('\n');
    }
    Ok(())
}

/// Handle new repository scaffolding prompt
async fn handle_scaffold_repo_prompt(
    spec_path: &PathBuf,
    include_pmat: bool,
    include_bashrs: bool,
    include_roadmap: bool,
    output: &Option<PathBuf>,
) -> Result<()> {
    debug_assert!(spec_path.exists(), "spec_path must exist: {}", spec_path.display());
    let spec_content = std::fs::read_to_string(spec_path).context(format!(
        "Failed to read specification file: {:?}",
        spec_path
    ))?;

    let mut prompt = format!(
        "# Scaffold New Repository\n\n\
         ## Repository Specification\n{}\n\n\
         ## Setup Checklist\n\n",
        spec_content
    );

    append_scaffold_sections(&mut prompt, include_pmat, include_bashrs, include_roadmap);

    write_prompt_output(&prompt, output, "Scaffold")
}

/// Append optional scaffold sections based on flags
fn append_scaffold_sections(
    prompt: &mut String,
    include_pmat: bool,
    include_bashrs: bool,
    include_roadmap: bool,
) {
    if include_pmat {
        prompt.push_str(
            "### PMAT Tools Integration\n\
             - [ ] Add `pmat` as dev dependency\n\
             - [ ] Configure `.git/hooks/pre-commit` with `pmat hooks install --tdg-enforcement`\n\
             - [ ] Add quality gates to CI/CD: `pmat quality-gate --fail-on-violation`\n\
             - [ ] Configure TDG thresholds in `.pmat/config.toml`\n\
             - [ ] Set up `pmat context` for AI-assisted development\n\n",
        );
    }

    if include_bashrs {
        prompt.push_str(
            "### bashrs Integration\n\
             - [ ] Install bashrs: `cargo install bashrs`\n\
             - [ ] Add bashrs linting to pre-commit hooks\n\
             - [ ] Configure bashrs in `.bashrsrc` (if needed)\n\
             - [ ] Lint all bash scripts: `find . -name '*.sh' -exec bashrs lint {} \\;`\n\n",
        );
    }

    if include_roadmap {
        prompt.push_str(
            "### Roadmapping Tools\n\
             - [ ] Initialize roadmap: `pmat roadmap init`\n\
             - [ ] Define milestones in `docs/roadmap/`\n\
             - [ ] Set up milestone tracking in project board\n\
             - [ ] Configure roadmap visualization\n\n",
        );
    }

    prompt.push_str(
        "## Repository Structure\n\
         ```\n\
         repo-name/\n\
         ├── .git/hooks/          # Pre-commit, pre-push hooks\n\
         ├── .pmat/               # PMAT configuration\n\
         ├── docs/\n\
         │   ├── specifications/  # Markdown specs\n\
         │   └── roadmap/         # Milestone tracking\n\
         ├── src/                 # Source code\n\
         ├── tests/               # Test suites (>85% coverage)\n\
         ├── scripts/             # Bash scripts (bashrs-validated)\n\
         ├── Cargo.toml           # Rust manifest (or equivalent)\n\
         └── README.md            # Project documentation\n\
         ```\n\n\
         ## EXTREME TDD Workflow\n\
         1. Write specification in `docs/specifications/`\n\
         2. Generate prompt: `pmat prompt implement --spec <spec.md>`\n\
         3. RED → GREEN → REFACTOR cycle\n\
         4. Quality gates before commit (enforced by hooks)\n\
         5. Continuous roadmap updates\n\n",
    );
}

/// Write prompt output to file or stdout
fn write_prompt_output(
    prompt: &str,
    output: &Option<PathBuf>,
    label: &str,
) -> Result<()> {
    if let Some(output_path) = output {
        std::fs::write(output_path, prompt)?;
        println!("✅ {} prompt written to {:?}", label, output_path);
    } else {
        println!("{}", prompt);
    }
    Ok(())
}
