fn format_summary_report(report: &ComprehensiveReport) -> Result<String> {
    use std::fmt::Write;
    let mut output = String::new();

    writeln!(&mut output, "# Code Similarity Analysis Summary\n")?;

    format_summary_metrics(&mut output, &report.metrics)?;
    format_summary_clone_types(&mut output, report)?;
    format_summary_refactoring_opportunities(&mut output, &report.refactoring_opportunities)?;

    Ok(output)
}

fn format_summary_metrics(output: &mut String, metrics: &Metrics) -> Result<()> {
    use std::fmt::Write;

    writeln!(output, "## Metrics")?;
    writeln!(
        output,
        "- Duplication: {:.1}%",
        metrics.duplication_percentage
    )?;
    writeln!(output, "- Average Entropy: {:.2}", metrics.average_entropy)?;
    writeln!(output, "- Total Clones: {}", metrics.total_clones)?;
    writeln!(output)?;

    Ok(())
}

fn format_summary_clone_types(output: &mut String, report: &ComprehensiveReport) -> Result<()> {
    use std::fmt::Write;

    writeln!(output, "## Clone Types")?;
    writeln!(
        output,
        "- Exact Duplicates: {}",
        report.exact_duplicates.len()
    )?;
    writeln!(
        output,
        "- Structural Similarities: {}",
        report.structural_similarities.len()
    )?;
    writeln!(
        output,
        "- Semantic Similarities: {}",
        report.semantic_similarities.len()
    )?;
    writeln!(output)?;

    Ok(())
}

fn format_summary_refactoring_opportunities(
    output: &mut String,
    opportunities: &[RefactoringHint],
) -> Result<()> {
    use std::fmt::Write;

    if !opportunities.is_empty() {
        writeln!(output, "## Top Refactoring Opportunities")?;
        for (i, hint) in opportunities.iter().take(5).enumerate() {
            writeln!(output, "{}. {}: {}", i + 1, hint.pattern, hint.suggestion)?;
        }
    }

    Ok(())
}

fn format_detailed_report(report: &ComprehensiveReport) -> Result<String> {
    use std::fmt::Write;
    let mut output = String::new();

    writeln!(&mut output, "# Comprehensive Code Similarity Report\n")?;

    format_metrics_section(&mut output, &report.metrics)?;
    format_exact_duplicates_section(&mut output, &report.exact_duplicates)?;

    format_structural_similarities_section(&mut output, &report.structural_similarities)?;

    format_entropy_analysis_section(&mut output, &report.entropy_analysis)?;

    format_refactoring_opportunities_section(&mut output, &report.refactoring_opportunities)?;

    Ok(output)
}

/// Format metrics section (cognitive complexity ≤5)
fn format_metrics_section(output: &mut String, metrics: &Metrics) -> Result<()> {
    use std::fmt::Write;

    writeln!(output, "## Overall Metrics")?;
    writeln!(
        output,
        "- Duplication Percentage: {:.1}%",
        metrics.duplication_percentage
    )?;
    writeln!(output, "- Average Entropy: {:.2}", metrics.average_entropy)?;
    writeln!(output, "- Total Clones Found: {}", metrics.total_clones)?;
    writeln!(output)?;

    Ok(())
}

/// Format exact duplicates section (cognitive complexity ≤8)
fn format_exact_duplicates_section(
    output: &mut String,
    exact_duplicates: &[SimilarBlock],
) -> Result<()> {
    use std::fmt::Write;

    if exact_duplicates.is_empty() {
        return Ok(());
    }

    writeln!(output, "## Exact Duplicates (Type-1 Clones)")?;
    for block in exact_duplicates {
        format_single_duplicate_block(output, block)?;
    }

    Ok(())
}

/// Format a single duplicate block (cognitive complexity ≤6)
fn format_single_duplicate_block(output: &mut String, block: &SimilarBlock) -> Result<()> {
    use std::fmt::Write;

    writeln!(output, "\n### Block {}", block.id)?;
    writeln!(output, "- Lines: {}", block.lines)?;
    writeln!(output, "- Tokens: {}", block.tokens)?;
    writeln!(output, "- Locations:")?;

    for loc in &block.locations {
        writeln!(
            output,
            "  - {}:{}-{}",
            loc.file.display(),
            loc.start_line,
            loc.end_line
        )?;
    }

    writeln!(output, "- Preview:\n```\n{}\n```", block.content_preview)?;

    Ok(())
}

/// Format structural similarities section (cognitive complexity ≤8)
fn format_structural_similarities_section(
    output: &mut String,
    structural_similarities: &[SimilarBlock],
) -> Result<()> {
    use std::fmt::Write;

    if structural_similarities.is_empty() {
        return Ok(());
    }

    writeln!(output, "\n## Structural Similarities (Type-2/3 Clones)")?;

    for block in structural_similarities.iter().take(10) {
        format_single_structural_block(output, block)?;
    }

    Ok(())
}

/// Format a single structural similarity block (cognitive complexity ≤6)
fn format_single_structural_block(output: &mut String, block: &SimilarBlock) -> Result<()> {
    use std::fmt::Write;

    writeln!(output, "\n### Similarity {}", block.id)?;
    writeln!(output, "- Similarity: {:.1}%", block.similarity * 100.0)?;
    writeln!(output, "- Type: {:?}", block.clone_type)?;
    writeln!(output, "- Locations:")?;

    for loc in &block.locations {
        writeln!(
            output,
            "  - {}:{}-{}",
            loc.file.display(),
            loc.start_line,
            loc.end_line
        )?;
    }

    Ok(())
}

/// Format entropy analysis section (cognitive complexity ≤8)
fn format_entropy_analysis_section(
    output: &mut String,
    entropy_analysis: &Option<EntropyReport>,
) -> Result<()> {
    use std::fmt::Write;

    let Some(entropy) = entropy_analysis else {
        return Ok(());
    };

    writeln!(output, "\n## Entropy Analysis")?;
    writeln!(output, "- Average Entropy: {:.2}", entropy.average_entropy)?;

    format_high_entropy_blocks(output, entropy)?;
    format_low_entropy_patterns(output, entropy)?;

    Ok(())
}

/// Format high entropy blocks (cognitive complexity ≤7)
fn format_high_entropy_blocks(output: &mut String, entropy: &EntropyReport) -> Result<()> {
    use std::fmt::Write;

    if entropy.high_entropy_blocks.is_empty() {
        return Ok(());
    }

    writeln!(output, "\n### High Complexity Code (High Entropy)")?;

    for block in entropy.high_entropy_blocks.iter().take(5) {
        format_entropy_block_item(output, block)?;
    }

    Ok(())
}

/// Format low entropy patterns (cognitive complexity ≤7)
fn format_low_entropy_patterns(output: &mut String, entropy: &EntropyReport) -> Result<()> {
    use std::fmt::Write;

    if entropy.low_entropy_patterns.is_empty() {
        return Ok(());
    }

    writeln!(output, "\n### Repetitive Patterns (Low Entropy)")?;

    for block in entropy.low_entropy_patterns.iter().take(5) {
        format_entropy_block_item(output, block)?;
    }

    Ok(())
}

/// Format single entropy block item (cognitive complexity ≤4)
fn format_entropy_block_item(output: &mut String, block: &EntropyBlock) -> Result<()> {
    use std::fmt::Write;

    writeln!(
        output,
        "- {}:{} (entropy: {:.2})",
        block.location.file.display(),
        block.location.start_line,
        block.entropy
    )?;
    writeln!(output, "  Suggestion: {}", block.suggestion)?;

    Ok(())
}

/// Format refactoring opportunities section (cognitive complexity ≤8)
fn format_refactoring_opportunities_section(
    output: &mut String,
    refactoring_opportunities: &[RefactoringHint],
) -> Result<()> {
    use std::fmt::Write;

    if refactoring_opportunities.is_empty() {
        return Ok(());
    }

    writeln!(output, "\n## Refactoring Opportunities")?;

    for hint in refactoring_opportunities {
        format_single_refactoring_hint(output, hint)?;
    }

    Ok(())
}

/// Format single refactoring hint (cognitive complexity ≤7)
fn format_single_refactoring_hint(output: &mut String, hint: &RefactoringHint) -> Result<()> {
    use std::fmt::Write;

    writeln!(output, "\n### {}", hint.pattern)?;
    writeln!(output, "- Priority: {:?}", hint.priority)?;
    writeln!(output, "- Suggestion: {}", hint.suggestion)?;
    writeln!(output, "- Affected locations:")?;

    for loc in &hint.locations {
        writeln!(
            output,
            "  - {}:{}-{}",
            loc.file.display(),
            loc.start_line,
            loc.end_line
        )?;
    }

    Ok(())
}
