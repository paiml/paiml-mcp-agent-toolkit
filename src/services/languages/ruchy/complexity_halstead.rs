impl RuchyComplexityAnalyzer {
    /// Reset Halstead tracking for a new function
    pub(super) fn reset_halstead(&mut self) {
        self.operators.clear();
        self.operands.clear();
        self.operator_count = 0;
        self.operand_count = 0;
    }

    /// Track an operator for Halstead metrics
    pub(super) fn track_operator(&mut self, op: &str) {
        self.operators.insert(op.to_string());
        self.operator_count += 1;
    }

    /// Track an operand for Halstead metrics
    pub(super) fn track_operand(&mut self, operand: &str) {
        self.operands.insert(operand.to_string());
        self.operand_count += 1;
    }

    /// Calculate Halstead metrics for current function
    pub(super) fn calculate_halstead(&self) -> HalsteadMetrics {
        let operators_unique = self.operators.len() as u32;
        let operands_unique = self.operands.len() as u32;
        let operators_total = self.operator_count;
        let operands_total = self.operand_count;

        let n = f64::from(operators_unique + operands_unique);
        let n_total = f64::from(operators_total + operands_total);

        let volume = if n > 0.0 { n_total * n.log2() } else { 0.0 };
        let difficulty = if operands_unique > 0 {
            (f64::from(operators_unique) / 2.0)
                * (f64::from(operands_total) / f64::from(operands_unique))
        } else {
            0.0
        };
        let effort = volume * difficulty;
        let time = effort / 18.0; // Stroud number
        let bugs = volume / 3000.0; // Industry average

        HalsteadMetrics {
            operators_unique,
            operands_unique,
            operators_total,
            operands_total,
            volume,
            difficulty,
            effort,
            time,
            bugs,
        }
    }
}
