// Lean 4 heuristic analysis for TDG scoring
// Analyzes: theorems, lemmas, defs, sorry usage, tactic complexity, imports
//
// The scoring body below is six independent phases, one per TdgScore
// component. Each phase's line-classification rules live in a named predicate
// rather than inline, so the body reads as the list of components it produces
// and each rule can be checked on its own. This mirrors the Scala heuristic in
// `analyzer_impl2_heuristics_scala.rs`, which is factored the same way.

impl TdgAnalyzerAst {
    #[allow(clippy::cast_possible_truncation)]
    fn analyze_lean_heuristic(
        &self,
        source: &str,
        score: &mut TdgScore,
        tracker: &mut PenaltyTracker,
    ) -> Result<()> {
        score.confidence *= 0.9; // Pattern-based but well-defined

        let lines: Vec<&str> = source.lines().collect();
        let total_lines = u32::try_from(lines.len().max(1)).unwrap_or(u32::MAX);

        // ── Structural complexity: tactic nesting, pattern matches, branching ──
        let (cyclomatic, max_nesting, max_proof_length) = Self::lean_structural_metrics(&lines);
        score.structural_complexity = self.score_structural_complexity(
            cyclomatic,
            max_nesting.min(20),
            max_nesting as usize,
            max_proof_length,
            tracker,
        );

        // ── Semantic complexity: type universe levels, dependent types, tactics ──
        let (universe_count, tactic_count) = Self::lean_semantic_metrics(&lines);
        score.semantic_complexity = self.score_semantic_complexity(
            universe_count as usize,
            tactic_count,
            max_nesting.min(10),
            tracker,
        );

        // ── Duplication ──
        score.duplication_ratio = self.analyze_duplication_ast(source, Language::Lean, tracker);

        // ── Coupling: import count + open count ──
        let import_count = Self::lean_count(&lines, Self::is_lean_import);
        let namespace_count = Self::lean_count(&lines, |t| t.starts_with("namespace "));
        score.coupling_score = self.score_coupling(import_count, namespace_count, 0, tracker);

        // ── Documentation: doc comments (/--, /-!, --) ──
        let doc_lines = Self::lean_count(&lines, Self::is_lean_doc_line);
        let def_count = Self::lean_count(&lines, Self::is_lean_declaration);
        score.doc_coverage = self.score_documentation(
            doc_lines,
            def_count.max(1),
            doc_lines,
            total_lines,
            tracker,
        );

        // ── Consistency: indentation style ──
        score.consistency_score =
            Self::lean_indent_consistency(&lines) * self.config.weights.consistency;

        // ── Entropy ──
        score.entropy_score = self.score_entropy_analysis(source, Language::Lean, tracker);

        Ok(())
    }

    /// Count the lines whose trimmed text satisfies `predicate`.
    fn lean_count(lines: &[&str], predicate: impl Fn(&str) -> bool) -> u32 {
        u32::try_from(lines.iter().filter(|l| predicate(l.trim())).count()).unwrap_or(u32::MAX)
    }

    /// Structural metrics for Lean source: (cyclomatic, `max_nesting`, `max_proof_length`).
    ///
    /// One pass, because the three are not independent: proof-block length is
    /// bounded by the declaration that ends the block, and Lean expresses
    /// nesting as indentation on the same lines that carry the branches.
    fn lean_structural_metrics(lines: &[&str]) -> (u32, u32, usize) {
        let mut cyclomatic = 1u32;
        let mut max_nesting = 0u32;
        let mut max_proof_length = 0usize;
        let mut current_proof_lines = 0usize;
        let mut in_proof = false;

        for line in lines {
            let trimmed = line.trim();

            // Track proof blocks (started by := by, ended by next top-level decl)
            if trimmed.contains(":= by") || trimmed == "by" {
                in_proof = true;
                current_proof_lines = 0;
            } else if in_proof && Self::ends_lean_proof_block(trimmed) {
                max_proof_length = max_proof_length.max(current_proof_lines);
                in_proof = false;
            }

            if in_proof {
                current_proof_lines += 1;
            }

            // Branching and control flow in tactics
            if Self::is_lean_branch(trimmed) {
                cyclomatic += 1;
            }

            // Tactic complexity
            if Self::opens_lean_subproof(trimmed) {
                cyclomatic += 1;
            }

            // Indentation-based nesting (Lean uses indentation)
            let indent = line.len() - line.trim_start().len();
            max_nesting = max_nesting.max(u32::try_from(indent / 2).unwrap_or(u32::MAX));
        }
        max_proof_length = max_proof_length.max(current_proof_lines);

        (cyclomatic, max_nesting, max_proof_length)
    }

    /// Semantic metrics for Lean source: (`universe_count`, `tactic_count`).
    ///
    /// Line comments are skipped entirely, so prose mentioning `Prop` or
    /// starting with a tactic name does not inflate either count.
    fn lean_semantic_metrics(lines: &[&str]) -> (u32, u32) {
        let mut universe_count = 0u32;
        let mut tactic_count = 0u32;

        for line in lines {
            let trimmed = line.trim();
            if trimmed.starts_with("--") {
                continue;
            }
            if trimmed.contains("Type") || trimmed.contains("Prop") || trimmed.contains("Sort") {
                universe_count += 1;
            }
            if Self::is_lean_tactic(trimmed) {
                tactic_count += 1;
            }
        }

        (universe_count, tactic_count)
    }

    /// Share of indented lines using the dominant indent width, `1.0` when the
    /// source has no indentation at all (nothing to be inconsistent about).
    fn lean_indent_consistency(lines: &[&str]) -> f32 {
        let mut spaces_2 = 0u32;
        let mut spaces_4 = 0u32;
        for line in lines {
            if line.starts_with("  ") && !line.starts_with("    ") {
                spaces_2 += 1;
            }
            if line.starts_with("    ") {
                spaces_4 += 1;
            }
        }
        let total_indented = spaces_2 + spaces_4;
        if total_indented == 0 {
            return 1.0;
        }
        (f64::from(spaces_2.max(spaces_4)) / f64::from(total_indented)) as f32
    }

    /// A non-comment top-level declaration, which closes any open proof block.
    fn ends_lean_proof_block(trimmed: &str) -> bool {
        !trimmed.is_empty()
            && !trimmed.starts_with("--")
            && (trimmed.starts_with("def ")
                || trimmed.starts_with("theorem ")
                || trimmed.starts_with("lemma ")
                || trimmed.starts_with("structure ")
                || trimmed.starts_with("class ")
                || trimmed.starts_with("namespace ")
                || trimmed.starts_with("end ")
                || trimmed.starts_with("section "))
    }

    /// A line that introduces a branch: `if`, `match`, a match arm or a case.
    fn is_lean_branch(trimmed: &str) -> bool {
        trimmed.starts_with("if ")
            || trimmed.contains(" if ")
            || trimmed.starts_with("match ")
            || trimmed.contains(" match ")
            || trimmed.starts_with("| ")
            || trimmed.starts_with("case ")
    }

    /// A line that opens a structured sub-proof rather than closing a goal.
    fn opens_lean_subproof(trimmed: &str) -> bool {
        trimmed.starts_with("by ")
            || trimmed.starts_with("calc")
            || trimmed.starts_with("have ")
            || trimmed.starts_with("show ")
            || trimmed.starts_with("suffices ")
            || trimmed.starts_with("obtain ")
    }

    /// A line invoking one of the common Lean/Mathlib tactics.
    fn is_lean_tactic(trimmed: &str) -> bool {
        trimmed.starts_with("simp")
            || trimmed.starts_with("rw")
            || trimmed.starts_with("exact")
            || trimmed.starts_with("apply")
            || trimmed.starts_with("intro")
            || trimmed.starts_with("omega")
            || trimmed.starts_with("ring")
            || trimmed.starts_with("norm_num")
            || trimmed.starts_with("decide")
            || trimmed.starts_with("aesop")
    }

    /// An `import` or `open` line — both bring names in, so both are coupling.
    fn is_lean_import(trimmed: &str) -> bool {
        trimmed.starts_with("import ") || trimmed.starts_with("open ")
    }

    /// A documentation or comment line: `/--`, `/-!` or `--`.
    fn is_lean_doc_line(trimmed: &str) -> bool {
        trimmed.starts_with("/--") || trimmed.starts_with("/-!") || trimmed.starts_with("--")
    }

    /// A declaration that documentation coverage is measured against.
    ///
    /// Deliberately a different list from [`Self::ends_lean_proof_block`]:
    /// `inductive`/`axiom`/`instance` are documentable items but do not end a
    /// proof block, and `namespace`/`end`/`section` end a block but are not
    /// items anyone documents.
    fn is_lean_declaration(trimmed: &str) -> bool {
        trimmed.starts_with("def ")
            || trimmed.starts_with("theorem ")
            || trimmed.starts_with("lemma ")
            || trimmed.starts_with("structure ")
            || trimmed.starts_with("class ")
            || trimmed.starts_with("inductive ")
            || trimmed.starts_with("axiom ")
            || trimmed.starts_with("instance ")
    }
}
// `count_lean_sorry_ast` and its two helpers were a byte-for-byte copy of the
// counter in `analyzer_simple_helpers.rs`: one rule, two implementations, and
// only one of them reachable from the MCP surface. Both copies are gone; the
// counter is `crate::tdg::critical_defect_gate::count_lean_sorry`, called by the
// shared gate that both `analyze_source` implementations run.
