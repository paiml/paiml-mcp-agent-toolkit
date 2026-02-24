impl SpecFalsificationReport {
    /// Format the report for terminal output
    pub fn display(&self) {
        let file_display = self.target_file.display();
        println!("Falsifying: {}", file_display);
        println!(
            "Extracted: {} falsifiable claims ({} P0, {} P1, {} P2)",
            self.summary.total_claims,
            self.verdicts
                .iter()
                .filter(|v| v.claim.priority == ClaimPriority::P0Critical)
                .count(),
            self.verdicts
                .iter()
                .filter(|v| v.claim.priority == ClaimPriority::P1High)
                .count(),
            self.verdicts
                .iter()
                .filter(|v| v.claim.priority == ClaimPriority::P2Low)
                .count(),
        );
        println!();

        for (i, verdict) in self.verdicts.iter().enumerate() {
            let status_icon = match &verdict.status {
                VerdictStatus::Survived => "\x1b[32mSURVIVED\x1b[0m",
                VerdictStatus::Falsified => "\x1b[31mFALSIFIED\x1b[0m",
                VerdictStatus::Unfalsifiable => "\x1b[33mUNFALSIFIABLE\x1b[0m",
                VerdictStatus::Inconclusive => "\x1b[33mINCONCLUSIVE\x1b[0m",
            };

            println!(
                "[{}/{}] {} {} (line {})",
                i + 1,
                self.summary.total_claims,
                verdict.claim.priority,
                status_icon,
                verdict.claim.source_line,
            );

            // Truncate long claim text
            let text = &verdict.claim.original_text;
            let display_text = if text.len() > 100 {
                format!("{}...", &text[..97])
            } else {
                text.clone()
            };
            println!("       \"{}\"", display_text);
            println!("       Category: {}", verdict.claim.category);

            for ev in &verdict.evidence {
                let icon = if ev.contradiction_score >= 0.8 {
                    "\x1b[31m✗\x1b[0m"
                } else if ev.contradiction_score >= 0.4 {
                    "\x1b[33m?\x1b[0m"
                } else {
                    "\x1b[32m✓\x1b[0m"
                };
                println!("       {} {} → {}", icon, ev.check, ev.finding);
            }
            println!();
        }

        // Summary
        println!("Summary:");
        println!("  Total claims:    {}", self.summary.total_claims);
        println!(
            "  \x1b[32mSurvived\x1b[0m:        {} ({:.1}%)",
            self.summary.survived,
            if self.summary.total_claims > 0 {
                self.summary.survived as f64 / self.summary.total_claims as f64 * 100.0
            } else {
                0.0
            }
        );
        println!(
            "  \x1b[31mFalsified\x1b[0m:       {} ({:.1}%)",
            self.summary.falsified,
            if self.summary.total_claims > 0 {
                self.summary.falsified as f64 / self.summary.total_claims as f64 * 100.0
            } else {
                0.0
            }
        );
        println!("  Unfalsifiable:   {}", self.summary.unfalsifiable);
        println!("  Inconclusive:    {}", self.summary.inconclusive);
        println!();
        println!("  Spec health:     {:.2}", self.summary.health_score);
    }

    /// Format as JSON
    pub fn to_json(&self) -> Result<String> {
        serde_json::to_string_pretty(self).context("Failed to serialize report")
    }
}
