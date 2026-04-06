// cargo_mutants_backend_display.rs — included from cargo_mutants_backend.rs
// Display mutation testing statistics

/// Display mutation testing statistics
#[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
pub fn display_statistics(report: &CargoMutantsReport) {
    eprintln!("📊 Mutation Testing Results:");
    eprintln!();

    let total = report.mutants.len();
    let caught = report.count_by_outcome(MutantOutcome::Caught);
    let missed = report.count_by_outcome(MutantOutcome::Missed);
    let timeout = report.count_by_outcome(MutantOutcome::Timeout);
    let unviable = report.count_by_outcome(MutantOutcome::Unviable);

    eprintln!("   Total mutants: {}", total);

    if total > 0 {
        eprintln!(
            "   Caught: {} ({:.1}%)",
            caught,
            (caught as f64 / total as f64) * 100.0
        );
        eprintln!(
            "   Missed: {} ({:.1}%)",
            missed,
            (missed as f64 / total as f64) * 100.0
        );

        if timeout > 0 {
            eprintln!(
                "   Timeout: {} ({:.1}%)",
                timeout,
                (timeout as f64 / total as f64) * 100.0
            );
        }

        if unviable > 0 {
            eprintln!(
                "   Unviable: {} ({:.1}%)",
                unviable,
                (unviable as f64 / total as f64) * 100.0
            );
        }
    }

    eprintln!();

    // Calculate and display mutation score
    let mutation_score = report.mutation_score();
    eprintln!("📈 Mutation Score: {:.1}%", mutation_score);

    // Quality assessment
    if mutation_score >= 90.0 {
        eprintln!("✅ Excellent! Test suite quality is very high");
    } else if mutation_score >= 75.0 {
        eprintln!("👍 Good test coverage, but room for improvement");
    } else if mutation_score >= 50.0 {
        eprintln!("⚠️  Moderate coverage - consider adding more tests");
    } else {
        eprintln!("❌ Low coverage - significant test gaps detected");
    }

    eprintln!();
}
