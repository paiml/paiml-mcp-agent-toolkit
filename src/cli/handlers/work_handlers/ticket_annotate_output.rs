/// Print specification section of text annotations (helper for print_annotations_text)
fn print_text_spec_section(ann: &TicketAnnotations) {
    println!("📋 SPECIFICATION");
    if let Some(ref spec) = ann.spec_path {
        println!("   Path:  {}", spec.display());
        if let Some(score) = ann.spec_score {
            let status = if score >= 95.0 { "✅" } else { "❌" };
            println!("   Score: {:.1}/100 {}", score, status);
        }
    } else {
        println!("   ⚠️  No specification linked");
    }
    println!();
}

/// Print TDG section of text annotations (helper for print_annotations_text)
fn print_text_tdg_section(ann: &TicketAnnotations) {
    println!("📈 TDG (Technical Debt Gradient)");
    if let Some(tdg) = ann.avg_tdg {
        let severity = tdg_severity_label(tdg);
        println!("   Avg Score: {:.2}/5.0 ({})", tdg, severity);
        for ft in &ann.file_tdg_scores {
            println!("     {:.2} [{}] {}", ft.score, ft.severity, ft.file);
        }
    } else {
        println!("   Not calculated (no files)");
    }
    println!();
}

/// Print churn section of text annotations (helper for print_annotations_text)
fn print_text_churn_section(ann: &TicketAnnotations) {
    println!("🔄 CHURN ANALYSIS");
    if let Some(churn) = ann.total_churn {
        println!("   Total Commits: {}", churn);
        for h in &ann.churn_hotspots {
            println!("     ⚠️  {}", h);
        }
    } else {
        println!("   Run with --with-churn to analyze");
    }
    println!();
}

/// Print tarantula and coverage sections (helper for print_annotations_text)
fn print_text_fault_coverage_section(ann: &TicketAnnotations) {
    println!("🔴 TARANTULA FAULT DETECTION");
    if ann.repeated_fixes.is_empty() {
        println!("   ✅ No repeated fix patterns detected");
    } else {
        for fix in &ann.repeated_fixes {
            println!("   ⚠️  {}: {}", fix.file, fix.description);
        }
    }
    println!();

    println!("📊 COVERAGE");
    if let Some(cov) = ann.coverage_percent {
        let status = if cov >= 95.0 { "✅" } else { "❌" };
        println!("   {:.1}% {}", cov, status);
    } else {
        println!("   Not available (run coverage analysis)");
    }
}

fn print_annotations_text(ann: &TicketAnnotations) {
    println!("📊 Quality Annotations for {}\n", ann.ticket_id);
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("Title:    {}", ann.title);
    println!("Status:   {}", ann.status);
    println!("Priority: {}", ann.priority);
    println!();

    print_text_spec_section(ann);

    println!("📁 RELATED FILES ({})", ann.files.len());
    if ann.files.is_empty() {
        println!("   No files detected");
    } else {
        for f in &ann.files {
            println!("   • {}", f.display());
        }
    }
    println!();

    print_text_tdg_section(ann);
    print_text_churn_section(ann);
    print_text_fault_coverage_section(ann);
}

fn print_annotations_json(ann: &TicketAnnotations) -> Result<()> {
    println!("{}", serde_json::to_string_pretty(ann)?);
    Ok(())
}

fn print_annotations_markdown(ann: &TicketAnnotations) {
    println!("# Quality Annotations: {}\n", ann.ticket_id);
    println!("**Title:** {}", ann.title);
    println!("**Status:** {} | **Priority:** {}\n", ann.status, ann.priority);

    println!("## Specification");
    if let Some(ref spec) = ann.spec_path {
        let score_str = ann.spec_score.map(|s| format!("{:.1}/100", s)).unwrap_or_else(|| "N/A".to_string());
        println!("| Metric | Value |");
        println!("|--------|-------|");
        println!("| Path | {} |", spec.display());
        println!("| Score | {} |", score_str);
    } else {
        println!("⚠️ No specification linked\n");
    }

    println!("\n## Metrics Summary");
    println!("| Metric | Value | Status |");
    println!("|--------|-------|--------|");
    println!("| Files | {} | - |", ann.files.len());
    println!("| TDG (avg) | {} | {} |",
        ann.avg_tdg.map(|t| format!("{:.2}/5.0", t)).unwrap_or_else(|| "N/A".to_string()),
        ann.avg_tdg.map(|t| if t <= 2.0 { "✅" } else { "⚠️" }).unwrap_or("⚠️")
    );
    println!("| Coverage | {} | {} |",
        ann.coverage_percent.map(|c| format!("{:.1}%", c)).unwrap_or_else(|| "N/A".to_string()),
        ann.coverage_percent.map(|c| if c >= 95.0 { "✅" } else { "❌" }).unwrap_or("⚠️")
    );
    println!("| Churn | {} | {} |",
        ann.total_churn.map(|c| c.to_string()).unwrap_or_else(|| "N/A".to_string()),
        if ann.total_churn.map(|c| c < 10).unwrap_or(true) { "✅" } else { "⚠️" }
    );
    println!("| Repeated Fixes | {} | {} |",
        ann.repeated_fixes.len(),
        if ann.repeated_fixes.is_empty() { "✅" } else { "🔴" }
    );

    // Per-file TDG breakdown
    if !ann.file_tdg_scores.is_empty() {
        println!("\n## TDG Per-File Breakdown");
        println!("| File | Score | Severity |");
        println!("|------|-------|----------|");
        for ft in &ann.file_tdg_scores {
            println!("| {} | {:.2} | {} |", ft.file, ft.score, ft.severity);
        }
    }
}
