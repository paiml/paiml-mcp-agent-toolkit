impl BaselineComparison {
    /// Check if there are any regressions
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn has_regressions(&self) -> bool {
        !self.regressed.is_empty()
    }

    /// Get total number of changes
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn total_changes(&self) -> usize {
        self.improved.len() + self.regressed.len() + self.added.len() + self.removed.len()
    }

    /// Format comparison as human-readable text
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn format_text(&self) -> String {
        let mut output = String::new();

        if !self.improved.is_empty() {
            output.push_str(&format!("✅ Improved: {} files\n", self.improved.len()));
            for cmp in &self.improved {
                output.push_str(&format!(
                    "   - {}: {:?} ({:.1}) → {:?} ({:.1}) [+{:.1}]\n",
                    cmp.path.display(),
                    cmp.grade_change.0,
                    cmp.old_score.total,
                    cmp.grade_change.1,
                    cmp.new_score.total,
                    cmp.delta
                ));
            }
        }

        if !self.regressed.is_empty() {
            output.push_str(&format!("⚠️  Regressed: {} files\n", self.regressed.len()));
            for cmp in &self.regressed {
                output.push_str(&format!(
                    "   - {}: {:?} ({:.1}) → {:?} ({:.1}) [{:.1}]\n",
                    cmp.path.display(),
                    cmp.grade_change.0,
                    cmp.old_score.total,
                    cmp.grade_change.1,
                    cmp.new_score.total,
                    cmp.delta
                ));
            }
        }

        if !self.unchanged.is_empty() {
            output.push_str(&format!("➡️  Unchanged: {} files\n", self.unchanged.len()));
        }

        if !self.added.is_empty() {
            output.push_str(&format!("➕ Added: {} files\n", self.added.len()));
            for path in &self.added {
                output.push_str(&format!("   - {}\n", path.display()));
            }
        }

        if !self.removed.is_empty() {
            output.push_str(&format!("➖ Removed: {} files\n", self.removed.len()));
            for path in &self.removed {
                output.push_str(&format!("   - {}\n", path.display()));
            }
        }

        output
    }
}
