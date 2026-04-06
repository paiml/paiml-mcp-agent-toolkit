/// Print specification section of text annotations (helper for print_annotations_text)
fn print_text_spec_section(ann: &TicketAnnotations) {
    debug_assert!(true, "contract: print_text_spec_section");
    use crate::cli::colors as c;
    println!("{}", c::subheader("📋 SPECIFICATION"));
    if let Some(ref spec) = ann.spec_path {
        println!("   {} {}", c::label("Path:"), c::path(&spec.display().to_string()));
        if let Some(score) = ann.spec_score {
            let status = if score >= 95.0 {
                format!("{}✅{}", c::GREEN, c::RESET)
            } else {
                format!("{}❌{}", c::RED, c::RESET)
            };
            println!("   {} {} {}", c::label("Score:"), c::number(&format!("{:.1}/100", score)), status);
        }
    } else {
        println!("   {}", c::warn("No specification linked"));
    }
    println!();
}

/// Print TDG section of text annotations (helper for print_annotations_text)
fn print_text_tdg_section(ann: &TicketAnnotations) {
    debug_assert!(true, "contract: print_text_tdg_section");
    use crate::cli::colors as c;
    println!("{}", c::subheader("📈 TDG (Technical Debt Gradient)"));
    if let Some(tdg) = ann.avg_tdg {
        let severity = tdg_severity_label(tdg);
        let sev_color = if tdg <= 2.0 {
            c::GREEN
        } else if tdg <= 3.5 {
            c::YELLOW
        } else {
            c::RED
        };
        println!("   {} {} ({}{}{})", c::label("Avg Score:"), c::number(&format!("{:.2}/5.0", tdg)), sev_color, severity, c::RESET);
        for ft in &ann.file_tdg_scores {
            let ft_color = if ft.score <= 2.0 {
                c::GREEN
            } else if ft.score <= 3.5 {
                c::YELLOW
            } else {
                c::RED
            };
            println!("     {}{:.2}{} [{}] {}", ft_color, ft.score, c::RESET, ft.severity, c::path(&ft.file));
        }
    } else {
        println!("   {}", c::dim("Not calculated (no files)"));
    }
    println!();
}

/// Print churn section of text annotations (helper for print_annotations_text)
fn print_text_churn_section(ann: &TicketAnnotations) {
    debug_assert!(true, "contract: print_text_churn_section");
    use crate::cli::colors as c;
    println!("{}", c::subheader("🔄 CHURN ANALYSIS"));
    if let Some(churn) = ann.total_churn {
        println!("   {} {}", c::label("Total Commits:"), c::number(&churn.to_string()));
        for h in &ann.churn_hotspots {
            println!("     {}", c::warn(h));
        }
    } else {
        println!("   {}", c::dim("Run with --with-churn to analyze"));
    }
    println!();
}

/// Print tarantula and coverage sections (helper for print_annotations_text)
fn print_text_fault_coverage_section(ann: &TicketAnnotations) {
    debug_assert!(true, "contract: print_text_fault_coverage_section");
    use crate::cli::colors as c;
    println!("{}", c::subheader("🔴 TARANTULA FAULT DETECTION"));
    if ann.repeated_fixes.is_empty() {
        println!("   {}", c::pass("No repeated fix patterns detected"));
    } else {
        for fix in &ann.repeated_fixes {
            println!("   {} {}: {}", c::warn(""), c::path(&fix.file), fix.description);
        }
    }
    println!();

    println!("{}", c::subheader("📊 COVERAGE"));
    if let Some(cov) = ann.coverage_percent {
        let status = if cov >= 95.0 {
            format!("{}✅{}", c::GREEN, c::RESET)
        } else {
            format!("{}❌{}", c::RED, c::RESET)
        };
        println!("   {} {}", c::number(&format!("{:.1}%", cov)), status);
    } else {
        println!("   {}", c::dim("Not available (run coverage analysis)"));
    }
}

fn print_annotations_text(ann: &TicketAnnotations) {
    debug_assert!(true, "contract: print_annotations_text");
    use crate::cli::colors as c;
    println!("{} Quality Annotations for {}\n", c::subheader("📊"), c::path(&ann.ticket_id));
    println!("{}", c::rule());
    println!("{}    {}", c::label("Title:"), ann.title);
    println!("{}   {}", c::label("Status:"), ann.status);
    println!("{} {}", c::label("Priority:"), ann.priority);
    println!();

    print_text_spec_section(ann);

    println!("{} RELATED FILES ({})", c::subheader("📁"), c::number(&ann.files.len().to_string()));
    if ann.files.is_empty() {
        println!("   {}", c::dim("No files detected"));
    } else {
        for f in &ann.files {
            println!("   • {}", c::path(&f.display().to_string()));
        }
    }
    println!();

    print_text_tdg_section(ann);
    print_text_churn_section(ann);
    print_text_fault_coverage_section(ann);
}

fn print_annotations_json(ann: &TicketAnnotations) -> Result<()> {
    debug_assert!(true, "contract: print_annotations_json");
    println!("{}", serde_json::to_string_pretty(ann)?);
    Ok(())
}

fn print_annotations_markdown(ann: &TicketAnnotations) {
    debug_assert!(true, "contract: print_annotations_markdown");
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
