impl SpecFalsificationReport {
    /// Format the report for terminal output
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn display(&self) {
        use crate::cli::colors as c;
        let file_display = self.target_file.display();
        println!("{}", c::header(&format!("Falsifying: {}", file_display)));
        let p0 = self.verdicts.iter().filter(|v| v.claim.priority == ClaimPriority::P0Critical).count();
        let p1 = self.verdicts.iter().filter(|v| v.claim.priority == ClaimPriority::P1High).count();
        let p2 = self.verdicts.iter().filter(|v| v.claim.priority == ClaimPriority::P2Low).count();
        println!(
            "Extracted: {} falsifiable claims ({}{} P0{}, {}{} P1{}, {}{} P2{})",
            c::number(&self.summary.total_claims.to_string()),
            c::RED, p0, c::RESET,
            c::YELLOW, p1, c::RESET,
            c::DIM, p2, c::RESET,
        );
        println!();

        for (i, verdict) in self.verdicts.iter().enumerate() {
            let status_icon = match &verdict.status {
                VerdictStatus::Survived => &format!("{}SURVIVED{}", c::GREEN, c::RESET),
                VerdictStatus::Falsified => &format!("{}FALSIFIED{}", c::RED, c::RESET),
                VerdictStatus::Unfalsifiable => &format!("{}UNFALSIFIABLE{}", c::YELLOW, c::RESET),
                VerdictStatus::Inconclusive => &format!("{}INCONCLUSIVE{}", c::YELLOW, c::RESET),
            };

            println!(
                "{}[{}/{}]{} {} {} (line {}{}{})",
                c::DIM, i + 1, self.summary.total_claims, c::RESET,
                verdict.claim.priority,
                status_icon,
                c::CYAN, verdict.claim.source_line, c::RESET,
            );

            // Truncate long claim text (GH-291 safe for multi-byte UTF-8)
            let display_text = crate::utils::string_truncate::truncate_with_ellipsis(
                &verdict.claim.original_text,
                97,
            );
            println!("       {}\"{}\"{}",c::DIM, display_text, c::RESET);
            println!("       Category: {}", c::label(&verdict.claim.category.to_string()));

            for ev in &verdict.evidence {
                let icon = if ev.contradicts() {
                    format!("{}✗{}", c::RED, c::RESET)
                } else if ev.is_ambiguous() || !ev.measured {
                    format!("{}?{}", c::YELLOW, c::RESET)
                } else {
                    format!("{}✓{}", c::GREEN, c::RESET)
                };
                println!("       {} {} → {}", icon, c::dim(&ev.check), ev.finding);
            }
            println!();
        }

        // Summary
        println!("{}", c::subheader("Summary:"));
        println!("  Total claims:    {}", c::number(&self.summary.total_claims.to_string()));
        let survived_pct = if self.summary.total_claims > 0 {
            self.summary.survived as f64 / self.summary.total_claims as f64 * 100.0
        } else { 0.0 };
        println!(
            "  {}Survived{}:        {} ({})",
            c::GREEN, c::RESET,
            c::number(&self.summary.survived.to_string()),
            c::pct(survived_pct, 80.0, 50.0),
        );
        let falsified_pct = if self.summary.total_claims > 0 {
            self.summary.falsified as f64 / self.summary.total_claims as f64 * 100.0
        } else { 0.0 };
        println!(
            "  {}Falsified{}:       {} ({:.1}%)",
            c::RED, c::RESET,
            c::number(&self.summary.falsified.to_string()),
            falsified_pct,
        );
        println!("  {}Unfalsifiable{}:   {}", c::YELLOW, c::RESET, c::number(&self.summary.unfalsifiable.to_string()));
        println!("  {}Inconclusive{}:    {}", c::DIM, c::RESET, c::number(&self.summary.inconclusive.to_string()));
        println!();
        let health_color = if self.summary.health_score >= 0.8 { c::GREEN } else if self.summary.health_score >= 0.5 { c::YELLOW } else { c::RED };
        println!("  Spec health:     {}{:.2}{}", health_color, self.summary.health_score, c::RESET);
    }

    /// Format as JSON
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn to_json(&self) -> Result<String> {
        serde_json::to_string_pretty(self).context("Failed to serialize report")
    }
}
