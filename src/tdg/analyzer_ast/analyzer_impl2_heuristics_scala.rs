impl TdgAnalyzerAst {
    // ── Scala heuristic analysis ────────────────────────────────────────

    #[allow(clippy::cast_possible_truncation)]
    fn analyze_scala_heuristic(
        &self,
        source: &str,
        score: &mut TdgScore,
        tracker: &mut PenaltyTracker,
    ) -> Result<()> {
        score.confidence *= 0.85;

        let lines: Vec<&str> = source.lines().collect();
        let total_lines = lines.len().max(1);

        // Structural metrics
        let (cyclomatic, cognitive, max_nesting, longest_method) =
            Self::scala_structural_metrics(&lines);
        score.structural_complexity = self.score_structural_complexity(
            cyclomatic, cognitive, max_nesting, longest_method, tracker,
        );

        // Semantic: implicit chains, type params, higher-kinded types
        let implicit_count = source.matches("implicit ").count() as u32;
        let type_param_count = source.matches("[_").count() as u32
            + source.matches("[A-Z]").count().min(20) as u32;
        let hkt_count = source.matches("[F[_]]").count() as u32
            + source.matches("[M[_]]").count() as u32;
        let param_count = source.matches("def ").count();
        score.semantic_complexity = self.score_semantic_complexity(
            param_count, implicit_count + hkt_count, type_param_count.min(10), tracker,
        );

        // Duplication
        score.duplication_ratio = self.analyze_duplication_ast(source, score.language, tracker);

        // Coupling: imports
        let import_count = source.matches("import ").count() as u32;
        let extends_count = source.matches(" extends ").count() as u32;
        let with_count = source.matches(" with ").count() as u32;
        score.coupling_score =
            self.score_coupling(import_count, extends_count + with_count, 0, tracker);

        // Documentation: scaladoc + comments
        let doc_comment_lines = lines
            .iter()
            .filter(|l| {
                let t = l.trim();
                t.starts_with("/**") || t.starts_with("*") || t.starts_with("//")
            })
            .count() as u32;
        let public_items = source.matches("def ").count() as u32
            + source.matches("val ").count() as u32
            + source.matches("class ").count() as u32
            + source.matches("object ").count() as u32
            + source.matches("trait ").count() as u32;
        score.doc_coverage = self.score_documentation(
            doc_comment_lines.min(public_items),
            public_items.max(1),
            doc_comment_lines,
            total_lines as u32,
            tracker,
        );

        // Consistency: camelCase naming
        score.consistency_score =
            Self::scala_naming_consistency(&lines) * self.config.weights.consistency;

        // Entropy
        score.entropy_score = self.score_entropy_analysis(source, score.language, tracker);

        Ok(())
    }

    /// Extract structural metrics from Scala source: (cyclomatic, cognitive, max_nesting, longest_method)
    fn scala_structural_metrics(lines: &[&str]) -> (u32, u32, usize, usize) {
        let mut max_nesting = 0usize;
        let mut current_nesting = 0usize;
        let mut match_arms = 0u32;
        let mut longest_method = 0usize;
        let mut current_method_lines = 0usize;
        let mut in_method = false;
        let mut cyclomatic = 1u32;

        for line in lines {
            let trimmed = line.trim();
            current_nesting += trimmed.matches('{').count();
            current_nesting = current_nesting.saturating_sub(trimmed.matches('}').count());
            max_nesting = max_nesting.max(current_nesting);

            if Self::is_scala_control_flow(trimmed) {
                cyclomatic += 1;
            }
            if trimmed.starts_with("case ") && trimmed.contains("=>") {
                match_arms += 1;
            }

            if trimmed.starts_with("def ") || trimmed.starts_with("override def ") {
                if in_method {
                    longest_method = longest_method.max(current_method_lines);
                }
                current_method_lines = 0;
                in_method = true;
            }
            if in_method {
                current_method_lines += 1;
            }
        }
        if in_method {
            longest_method = longest_method.max(current_method_lines);
        }

        let cognitive = match_arms + (max_nesting as u32).saturating_sub(2);
        (cyclomatic, cognitive, max_nesting, longest_method)
    }

    fn is_scala_control_flow(trimmed: &str) -> bool {
        trimmed.starts_with("if ")
            || trimmed.starts_with("if(")
            || trimmed.contains(" if ")
            || trimmed.starts_with("case ")
            || trimmed.starts_with("while ")
            || trimmed.starts_with("for ")
            || trimmed.starts_with("for(")
            || trimmed.contains("catch ")
    }

    /// Calculate Scala naming consistency ratio (0.0-1.0)
    fn scala_naming_consistency(lines: &[&str]) -> f32 {
        let mut camel_defs = 0u32;
        let mut non_camel_defs = 0u32;
        for line in lines {
            let trimmed = line.trim();
            let rest = trimmed.strip_prefix("def ")
                .or_else(|| trimmed.strip_prefix("val "))
                .or_else(|| trimmed.strip_prefix("var "));
            let Some(rest) = rest else { continue };
            let name = rest.split(|c: char| !c.is_alphanumeric() && c != '_').next().unwrap_or("");
            if name.is_empty() {
                continue;
            }
            if name.chars().next().is_some_and(|c| c.is_lowercase()) && !name.contains('_') {
                camel_defs += 1;
            } else if name.contains('_') {
                non_camel_defs += 1;
            }
        }
        let total_defs = camel_defs + non_camel_defs;
        if total_defs > 2 {
            camel_defs.max(non_camel_defs) as f32 / total_defs as f32
        } else {
            1.0
        }
    }
}
