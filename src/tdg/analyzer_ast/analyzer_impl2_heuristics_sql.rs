impl TdgAnalyzerAst {
    // ── SQL heuristic analysis ──────────────────────────────────────────

    #[allow(clippy::cast_possible_truncation)]
    fn analyze_sql_heuristic(
        &self,
        source: &str,
        score: &mut TdgScore,
        tracker: &mut PenaltyTracker,
    ) -> Result<()> {
        score.confidence *= 0.8;

        let lines: Vec<&str> = source.lines().collect();
        let total_lines = lines.len().max(1);

        // Structural: subquery nesting + JOIN count + statement length
        let mut max_nesting = 0u32;
        let mut current_nesting = 0u32;
        let mut join_count = 0u32;
        let mut longest_stmt = 0usize;
        let mut current_stmt_lines = 0usize;

        let upper = source.to_uppercase();
        for line in upper.lines() {
            let trimmed = line.trim();
            if trimmed.is_empty() {
                continue;
            }
            current_nesting += trimmed.matches('(').count() as u32;
            current_nesting = current_nesting.saturating_sub(trimmed.matches(')').count() as u32);
            max_nesting = max_nesting.max(current_nesting);

            if trimmed.contains("JOIN") {
                join_count += 1;
            }
            current_stmt_lines += 1;
            if trimmed.ends_with(';') {
                longest_stmt = longest_stmt.max(current_stmt_lines);
                current_stmt_lines = 0;
            }
        }
        longest_stmt = longest_stmt.max(current_stmt_lines);

        let cyclomatic = 1 + join_count + (max_nesting / 2);
        score.structural_complexity = self.score_structural_complexity(
            cyclomatic,
            max_nesting,
            max_nesting as usize,
            longest_stmt,
            tracker,
        );

        // Semantic: column count, CASE expressions, function calls
        let case_count = upper.matches("CASE ").count() as u32;
        let coalesce_count = upper.matches("COALESCE").count() as u32;
        let cast_count = upper.matches("CAST(").count() as u32;
        let type_complexity = case_count + coalesce_count + cast_count;
        score.semantic_complexity =
            self.score_semantic_complexity(join_count as usize, type_complexity, max_nesting, tracker);

        // Duplication
        score.duplication_ratio = self.analyze_duplication_ast(source, score.language, tracker);

        // Coupling: table references
        let from_count = upper.matches(" FROM ").count() as u32;
        let into_count = upper.matches(" INTO ").count() as u32;
        score.coupling_score =
            self.score_coupling(from_count + into_count + join_count, 0, 0, tracker);

        // Documentation: comment ratio
        let comment_lines = lines
            .iter()
            .filter(|l| {
                let t = l.trim();
                t.starts_with("--") || t.starts_with("/*")
            })
            .count() as u32;
        score.doc_coverage =
            self.score_documentation(comment_lines, total_lines as u32, comment_lines, total_lines as u32, tracker);

        // Consistency: keyword casing (all-upper vs mixed)
        let sql_keywords = ["SELECT", "FROM", "WHERE", "INSERT", "UPDATE", "DELETE", "JOIN", "GROUP", "ORDER", "HAVING"];
        let mut upper_kw = 0u32;
        let mut lower_kw = 0u32;
        for line in source.lines() {
            for kw in &sql_keywords {
                if line.contains(kw) {
                    upper_kw += 1;
                }
                if line.contains(&kw.to_lowercase()) && !line.contains(kw) {
                    lower_kw += 1;
                }
            }
        }
        let total_kw = upper_kw + lower_kw;
        if total_kw > 0 {
            let dominant = upper_kw.max(lower_kw) as f32 / total_kw as f32;
            score.consistency_score = dominant * self.config.weights.consistency;
        } else {
            score.consistency_score = self.config.weights.consistency;
        }

        // Entropy
        score.entropy_score = self.score_entropy_analysis(source, score.language, tracker);

        Ok(())
    }
}
