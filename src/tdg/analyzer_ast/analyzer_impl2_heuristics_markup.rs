impl TdgAnalyzerAst {
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
