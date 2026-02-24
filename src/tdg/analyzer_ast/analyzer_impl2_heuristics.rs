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

    // ── YAML heuristic analysis ─────────────────────────────────────────

    #[allow(clippy::cast_possible_truncation)]
    fn analyze_yaml_heuristic(
        &self,
        source: &str,
        score: &mut TdgScore,
        tracker: &mut PenaltyTracker,
    ) -> Result<()> {
        score.confidence *= 0.75;

        let lines: Vec<&str> = source.lines().collect();
        let total_lines = lines.len().max(1);

        // Structural: nesting depth via indentation
        let mut max_indent = 0usize;
        let mut indent_sizes = Vec::new();

        for line in &lines {
            if line.trim().is_empty() || line.trim().starts_with('#') {
                continue;
            }
            let indent = line.len() - line.trim_start().len();
            if indent > 0 {
                indent_sizes.push(indent);
                max_indent = max_indent.max(indent);
            }
        }

        // Estimate indent unit (usually 2 or 4)
        let indent_unit = if indent_sizes.len() > 2 {
            let mut diffs: Vec<usize> = indent_sizes
                .windows(2)
                .filter_map(|w| {
                    if w[1] > w[0] {
                        Some(w[1] - w[0])
                    } else {
                        None
                    }
                })
                .collect();
            diffs.sort_unstable();
            diffs.first().copied().unwrap_or(2).max(1)
        } else {
            2
        };

        let nesting_depth = max_indent / indent_unit;

        // Count anchors, aliases, multi-doc markers
        let anchor_count = source.matches(" &").count() as u32;
        let alias_count = source.matches(" *").count() as u32;
        let multi_doc = source.matches("\n---").count() as u32;
        let key_count = lines
            .iter()
            .filter(|l| {
                let t = l.trim();
                !t.is_empty() && !t.starts_with('#') && !t.starts_with('-') && t.contains(':')
            })
            .count() as u32;

        let cyclomatic = 1 + multi_doc + (anchor_count / 2);
        score.structural_complexity = self.score_structural_complexity(
            cyclomatic,
            nesting_depth as u32,
            nesting_depth,
            total_lines,
            tracker,
        );

        // Semantic: type tags, complex values
        let tag_count = source.matches("!!").count() as u32;
        let multiline_count =
            (source.matches(" |").count() + source.matches(" >").count()) as u32;
        score.semantic_complexity = self.score_semantic_complexity(
            key_count as usize,
            tag_count + multiline_count,
            anchor_count + alias_count,
            tracker,
        );

        // Duplication (YAML has lots of repeated keys)
        score.duplication_ratio = self.analyze_duplication_ast(source, score.language, tracker);

        // Coupling: anchors/aliases = internal references
        score.coupling_score =
            self.score_coupling(anchor_count + alias_count, multi_doc, 0, tracker);

        // Documentation: comment ratio
        let comment_lines = lines
            .iter()
            .filter(|l| l.trim().starts_with('#'))
            .count() as u32;
        score.doc_coverage = self.score_documentation(
            comment_lines,
            key_count.max(1),
            comment_lines,
            total_lines as u32,
            tracker,
        );

        // Consistency: indentation consistency
        if indent_sizes.len() > 3 {
            let consistent_indents = indent_sizes
                .iter()
                .filter(|&&s| s % indent_unit == 0)
                .count();
            let ratio = consistent_indents as f32 / indent_sizes.len() as f32;
            score.consistency_score = ratio * self.config.weights.consistency;
        } else {
            score.consistency_score = self.config.weights.consistency;
        }

        // Entropy
        score.entropy_score = self.score_entropy_analysis(source, score.language, tracker);

        Ok(())
    }

    // ── Markdown heuristic analysis ─────────────────────────────────────

    #[allow(clippy::cast_possible_truncation)]
    fn analyze_markdown_heuristic(
        &self,
        source: &str,
        score: &mut TdgScore,
        tracker: &mut PenaltyTracker,
    ) -> Result<()> {
        score.confidence *= 0.70;

        let lines: Vec<&str> = source.lines().collect();
        let total_lines = lines.len().max(1);

        // Structural: heading hierarchy, section count, code block count
        let mut heading_levels = Vec::new();
        let mut code_block_count = 0u32;
        let mut in_code_block = false;
        let mut max_list_depth = 0usize;

        for line in &lines {
            let trimmed = line.trim();

            if trimmed.starts_with("```") {
                in_code_block = !in_code_block;
                if !in_code_block {
                    code_block_count += 1;
                }
                continue;
            }
            if in_code_block {
                continue;
            }

            // Track heading levels
            if trimmed.starts_with('#') {
                let level = trimmed.chars().take_while(|&c| c == '#').count();
                heading_levels.push(level);
            }

            // Track list nesting
            let indent = line.len() - line.trim_start().len();
            if trimmed.starts_with("- ")
                || trimmed.starts_with("* ")
                || trimmed.starts_with("+ ")
                || trimmed.chars().next().is_some_and(|c| c.is_ascii_digit())
                    && trimmed.contains(". ")
            {
                max_list_depth = max_list_depth.max(indent / 2 + 1);
            }
        }

        let section_count = heading_levels.len() as u32;
        score.structural_complexity = self.score_structural_complexity(
            1 + section_count / 5,
            max_list_depth as u32,
            max_list_depth,
            total_lines,
            tracker,
        );

        // Semantic: link density, image count, table count
        let link_count = source.matches("](").count() as u32;
        let image_count = source.matches("![").count() as u32;
        let table_rows = lines
            .iter()
            .filter(|l| l.trim().starts_with('|') && l.trim().ends_with('|'))
            .count() as u32;
        score.semantic_complexity = self.score_semantic_complexity(
            link_count as usize,
            image_count + table_rows,
            code_block_count,
            tracker,
        );

        // Duplication
        score.duplication_ratio = self.analyze_duplication_ast(source, score.language, tracker);

        // Coupling: external links and cross-references
        let external_links = source.matches("](http").count() as u32;
        let internal_links = link_count.saturating_sub(external_links);
        score.coupling_score =
            self.score_coupling(external_links, internal_links, 0, tracker);

        // Documentation: markdown IS documentation, so base on structure quality
        let has_toc = source.contains("## Table of Contents")
            || source.contains("## TOC")
            || source.contains("<!-- toc");
        let has_intro = !heading_levels.is_empty() && heading_levels[0] <= 2;
        let well_structured = (has_toc as u32) + (has_intro as u32) + u32::from(section_count > 2);
        score.doc_coverage = self.score_documentation(
            well_structured,
            3,
            section_count,
            total_lines as u32,
            tracker,
        );

        // Consistency: heading hierarchy (no skipped levels), list marker consistency
        let mut hierarchy_violations = 0u32;
        for window in heading_levels.windows(2) {
            if window[1] > window[0] + 1 {
                hierarchy_violations += 1;
            }
        }

        // List marker consistency
        let dash_lists = lines
            .iter()
            .filter(|l| l.trim().starts_with("- "))
            .count();
        let star_lists = lines
            .iter()
            .filter(|l| l.trim().starts_with("* "))
            .count();
        let total_list_items = dash_lists + star_lists;
        let list_consistency = if total_list_items > 2 {
            dash_lists.max(star_lists) as f32 / total_list_items as f32
        } else {
            1.0
        };

        let heading_penalty = (hierarchy_violations as f32 * 1.5).min(5.0);
        score.consistency_score =
            (list_consistency * self.config.weights.consistency - heading_penalty).max(0.0);

        // Entropy
        score.entropy_score = self.score_entropy_analysis(source, score.language, tracker);

        Ok(())
    }

}
